use crate::tool_registry::{
    ToolInstallation, ToolRegistry, ToolSpecification, ValidationResult, ValidationStatus,
    ValidationType,
};
use async_trait::async_trait;
use regex::Regex;
use std::path::PathBuf;
use std::process::Command;

#[async_trait]
pub trait ValidationEngine: Send + Sync {
    async fn validate_tool(
        &self,
        tool: &ToolSpecification,
    ) -> Result<Vec<ValidationResult>, String>;

    async fn validate_installation(
        &self,
        tool: &ToolSpecification,
    ) -> Result<ValidationResult, String>;

    async fn validate_configuration(
        &self,
        tool: &ToolSpecification,
    ) -> Result<ValidationResult, String>;

    async fn validate_functional(
        &self,
        tool: &ToolSpecification,
    ) -> Result<ValidationResult, String>;

    async fn validate_network(&self, tool: &ToolSpecification) -> Result<ValidationResult, String>;

    async fn validate_permissions(
        &self,
        tool: &ToolSpecification,
    ) -> Result<ValidationResult, String>;
}

pub struct ValidationEngineImpl;

impl ValidationEngineImpl {
    pub fn new() -> Self {
        Self
    }

    fn expand_path(path: &str) -> String {
        if path.starts_with("~/") {
            if let Ok(home) = std::env::var("HOME") {
                return path.replacen("~", &home, 1);
            }
        }
        path.to_string()
    }

    fn check_binary_exists(&self,
        binary_names: &Option<Vec<String>>,
        standard_paths: &Option<Vec<String>>,
    ) -> Option<String> {
        if let Some(names) = binary_names {
            for name in names {
                if let Ok(output) = Command::new("which").arg(name).output() {
                    if output.status.success() {
                        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                        if !path.is_empty() {
                            return Some(path);
                        }
                    }
                }
            }
        }

        if let Some(paths) = standard_paths {
            for path in paths {
                let expanded = Self::expand_path(path);
                if PathBuf::from(&expanded).exists() {
                    return Some(expanded);
                }
            }
        }

        None
    }

    fn extract_version(&self,
        command: &str,
        version_args: &Option<Vec<String>>,
        version_pattern: &Option<String>,
    ) -> Option<String> {
        let args = version_args.as_ref().map(|v| v.clone()).unwrap_or_else(|| vec!["--version".to_string()]);

        let output = Command::new(command)
            .args(&args)
            .output()
            .ok()?;

        let output_str = if output.status.success() {
            String::from_utf8_lossy(&output.stdout).to_string()
        } else {
            String::from_utf8_lossy(&output.stderr).to_string()
        };

        let pattern_str = version_pattern.as_ref()
            .map(|p| p.as_str())
            .unwrap_or(r"(\d+\.\d+\.\d+)");

        let regex = Regex::new(pattern_str).ok()?;

        regex.captures(&output_str)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }

    fn check_config_files(&self,
        base_dir: &str,
        files: &Option<Vec<crate::tool_registry::ConfigFileSpec>>,
    ) -> Vec<(String, bool)> {
        let expanded_base = Self::expand_path(base_dir);
        let base = PathBuf::from(&expanded_base);

        let mut results = Vec::new();

        if let Some(config_files) = files {
            for file in config_files {
                let file_path = base.join(&file.path);
                results.push((file.path.clone(), file_path.exists()));
            }
        }

        results
    }
}

#[async_trait]
impl ValidationEngine for ValidationEngineImpl {
    async fn validate_tool(
        &self,
        tool: &ToolSpecification,
    ) -> Result<Vec<ValidationResult>, String> {
        let mut results = Vec::new();

        results.push(self.validate_installation(tool).await?);

        if tool.capabilities.settings || tool.capabilities.files {
            results.push(self.validate_configuration(tool).await?);
        }

        if tool.capabilities.commands {
            results.push(self.validate_functional(tool).await?);
        }

        if tool.capabilities.mcp_servers {
            results.push(self.validate_network(tool).await?);
        }

        results.push(self.validate_permissions(tool).await?);

        Ok(results)
    }

    async fn validate_installation(
        &self,
        tool: &ToolSpecification,
    ) -> Result<ValidationResult, String> {
        let config = &tool.installation;

        let binary_path = self.check_binary_exists(
            &config.binary_names,
            &config.standard_paths,
        );

        let (status, message, details) = match binary_path {
            Some(path) => {
                let version = self.extract_version(
                    &path,
                    &config.version_args,
                    &config.version_pattern,
                );

                let details = serde_json::json!({
                    "path": path,
                    "version": version,
                });

                if version.is_some() {
                    (
                        ValidationStatus::Valid,
                        format!("Tool installed at {} with version {}", path, version.as_ref().unwrap()),
                        details,
                    )
                } else {
                    (
                        ValidationStatus::Warning,
                        format!("Tool installed at {} but version could not be determined", path),
                        details,
                    )
                }
            }
            None => {
                let details = serde_json::json!({
                    "checked_paths": config.standard_paths,
                    "checked_binaries": config.binary_names,
                });

                (
                    ValidationStatus::Invalid,
                    format!("Tool '{}' not found in PATH or standard locations", tool.name),
                    details,
                )
            }
        };

        Ok(ValidationResult {
            id: format!("{}_installation", tool.id),
            tool_id: tool.id.clone(),
            validation_type: ValidationType::Installation,
            status,
            details: Some(details),
            checked_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    async fn validate_configuration(
        &self,
        tool: &ToolSpecification,
    ) -> Result<ValidationResult, String> {
        let config = &tool.config;
        let file_results = self.check_config_files(&config.base_dir, &config.files);

        let all_exist = file_results.iter().all(|(_, exists)| *exists);
        let required_exist = file_results
            .iter()
            .filter(|(path, _)| {
                tool.config.files.as_ref().map(|files| {
                    files.iter().any(|f| &f.path == path && f.required == Some(true))
                }).unwrap_or(false)
            })
            .all(|(_, exists)| *exists);

        let (status, message) = if all_exist {
            (ValidationStatus::Valid, "All configuration files present".to_string())
        } else if required_exist {
            (ValidationStatus::Warning, "Required config files present, optional files missing".to_string())
        } else {
            (ValidationStatus::Invalid, "Required configuration files missing".to_string())
        };

        let details = serde_json::json!({
            "base_dir": config.base_dir,
            "files": file_results,
        });

        Ok(ValidationResult {
            id: format!("{}_config", tool.id),
            tool_id: tool.id.clone(),
            validation_type: ValidationType::Configuration,
            status,
            details: Some(details),
            checked_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    async fn validate_functional(
        &self,
        tool: &ToolSpecification,
    ) -> Result<ValidationResult, String> {
        let config = &tool.installation;

        let binary_path = self.check_binary_exists(
            &config.binary_names,
            &config.standard_paths,
        );

        let (status, message) = match binary_path {
            Some(path) => {
                let help_result = Command::new(&path)
                    .arg("--help")
                    .output();

                match help_result {
                    Ok(output) if output.status.success() || !output.stdout.is_empty() => {
                        (ValidationStatus::Valid, "Tool executes successfully".to_string())
                    }
                    Ok(_) => {
                        (ValidationStatus::Warning, "Tool executes but may have issues".to_string())
                    }
                    Err(e) => {
                        (ValidationStatus::Invalid, format!("Tool execution failed: {}", e))
                    }
                }
            }
            None => {
                (ValidationStatus::Skipped, "Tool not installed, skipping functional test".to_string())
            }
        };

        Ok(ValidationResult {
            id: format!("{}_functional", tool.id),
            tool_id: tool.id.clone(),
            validation_type: ValidationType::Functional,
            status,
            details: None,
            checked_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    async fn validate_network(&self, tool: &ToolSpecification) -> Result<ValidationResult, String> {
        Ok(ValidationResult {
            id: format!("{}_network", tool.id),
            tool_id: tool.id.clone(),
            validation_type: ValidationType::Network,
            status: ValidationStatus::Skipped,
            details: Some(serde_json::json!({
                "message": "Network validation not yet implemented"
            })),
            checked_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    async fn validate_permissions(
        &self,
        tool: &ToolSpecification,
    ) -> Result<ValidationResult, String> {
        let config = &tool.installation;

        let binary_path = self.check_binary_exists(
            &config.binary_names,
            &config.standard_paths,
        );

        let (status, message, details) = match binary_path {
            Some(path) => {
                let path_buf = PathBuf::from(&path);
                let metadata = std::fs::metadata(&path_buf);

                match metadata {
                    Ok(meta) => {
                        let permissions = meta.permissions();
                        let is_executable = permissions.mode() & 0o111 != 0;

                        if is_executable {
                            (
                                ValidationStatus::Valid,
                                "Tool has executable permissions".to_string(),
                                serde_json::json!({"path": path, "executable": true}),
                            )
                        } else {
                            (
                                ValidationStatus::Invalid,
                                "Tool exists but is not executable".to_string(),
                                serde_json::json!({"path": path, "executable": false}),
                            )
                        }
                    }
                    Err(e) => (
                        ValidationStatus::Warning,
                        format!("Could not check permissions: {}", e),
                        serde_json::json!({"path": path, "error": e.to_string()}),
                    ),
                }
            }
            None => (
                ValidationStatus::Skipped,
                "Tool not installed, skipping permission check".to_string(),
                serde_json::json!({"message": "Binary not found"}),
            ),
        };

        Ok(ValidationResult {
            id: format!("{}_permissions", tool.id),
            tool_id: tool.id.clone(),
            validation_type: ValidationType::Permissions,
            status,
            details: Some(details),
            checked_at: chrono::Utc::now().to_rfc3339(),
        })
    }
}
