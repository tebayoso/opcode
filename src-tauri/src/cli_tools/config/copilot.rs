//! GitHub Copilot CLI configuration management
//!
//! Handles configuration for GitHub's Copilot CLI tool.
//! Config locations:
//! - User: ~/.copilot/config (YAML format)
//! - Agents: ~/.copilot/agents/*.md

use super::parsers::ConfigParser;
use super::traits::*;
use crate::cli_tools::CLIToolType;
use chrono::{DateTime, Utc};
use std::path::PathBuf;
use tokio::fs;
use tokio::process::Command;

/// GitHub Copilot CLI configuration handler
pub struct CopilotConfig {
    config_dir: PathBuf,
}

impl CopilotConfig {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_default();
        Self {
            config_dir: home.join(".copilot"),
        }
    }
}

impl Default for CopilotConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl CLIToolConfig for CopilotConfig {
    fn tool_type(&self) -> CLIToolType {
        CLIToolType::GitHubCopilot
    }

    fn config_dir(&self) -> PathBuf {
        self.config_dir.clone()
    }

    async fn list_config_files(&self) -> Result<Vec<ConfigFileInfo>, ConfigError> {
        let mut files = Vec::new();

        let paths = vec![
            (
                self.config_dir.join("config"),
                ConfigFileType::Yaml,
                "Main configuration",
            ),
            (
                self.config_dir.join("config.yaml"),
                ConfigFileType::Yaml,
                "YAML configuration",
            ),
            (
                self.config_dir.join("config.yml"),
                ConfigFileType::Yaml,
                "YML configuration",
            ),
        ];

        for (path, file_type, desc) in paths {
            if path.exists() {
                if let Ok(metadata) = fs::metadata(&path).await {
                    let modified = metadata
                        .modified()
                        .map(|t| DateTime::<Utc>::from(t))
                        .unwrap_or_else(|_| Utc::now());

                    files.push(ConfigFileInfo {
                        path: path.to_string_lossy().to_string(),
                        name: path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("config")
                            .to_string(),
                        file_type,
                        size_bytes: metadata.len(),
                        modified,
                        scope: ConfigScope::User,
                        description: Some(desc.to_string()),
                    });
                }
            }
        }

        // Scan agents directory
        let agents_dir = self.config_dir.join("agents");
        if agents_dir.exists() {
            if let Ok(mut entries) = fs::read_dir(&agents_dir).await {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    let path = entry.path();
                    if path.extension().map(|e| e == "md").unwrap_or(false) {
                        if let Ok(metadata) = fs::metadata(&path).await {
                            let modified = metadata
                                .modified()
                                .map(|t| DateTime::<Utc>::from(t))
                                .unwrap_or_else(|_| Utc::now());

                            files.push(ConfigFileInfo {
                                path: path.to_string_lossy().to_string(),
                                name: path
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("agent")
                                    .to_string(),
                                file_type: ConfigFileType::Markdown,
                                size_bytes: metadata.len(),
                                modified,
                                scope: ConfigScope::User,
                                description: Some("Agent definition".to_string()),
                            });
                        }
                    }
                }
            }
        }

        Ok(files)
    }

    async fn read_config_file(&self, path: &str) -> Result<ConfigFileContent, ConfigError> {
        let path_buf = PathBuf::from(path);

        if !path_buf.exists() {
            return Err(ConfigError::NotFound(path.to_string()));
        }

        let content = fs::read_to_string(&path_buf)
            .await
            .map_err(|e| ConfigError::IoError(e.to_string()))?;

        let ext = path_buf
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("yaml");
        let file_type = ConfigFileType::from_extension(ext).unwrap_or(ConfigFileType::Yaml);

        Ok(ConfigParser::create_file_content(path, &content, file_type))
    }

    async fn write_config_file(&self, path: &str, content: &str) -> Result<(), ConfigError> {
        let path_buf = PathBuf::from(path);

        if let Some(parent) = path_buf.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| ConfigError::IoError(e.to_string()))?;
        }

        fs::write(&path_buf, content)
            .await
            .map_err(|e| ConfigError::WriteError(e.to_string()))?;

        Ok(())
    }

    async fn get_settings(&self) -> Result<ToolSettings, ConfigError> {
        // Try different config file locations
        let config_paths = vec![
            self.config_dir.join("config"),
            self.config_dir.join("config.yaml"),
            self.config_dir.join("config.yml"),
        ];

        let mut categories = Vec::new();

        for config_path in config_paths {
            if config_path.exists() {
                let content = fs::read_to_string(&config_path)
                    .await
                    .map_err(|e| ConfigError::IoError(e.to_string()))?;

                if let Ok(config) = ConfigParser::parse_yaml(&content) {
                    let mut settings = Vec::new();

                    if let Some(obj) = config.as_object() {
                        for (key, value) in obj {
                            let setting_type = match value {
                                serde_json::Value::String(_) => SettingType::String,
                                serde_json::Value::Number(_) => SettingType::Number,
                                serde_json::Value::Bool(_) => SettingType::Boolean,
                                serde_json::Value::Array(_) => SettingType::Array,
                                serde_json::Value::Object(_) => SettingType::Object,
                                serde_json::Value::Null => SettingType::String,
                            };

                            settings.push(SettingDefinition {
                                key: key.clone(),
                                value: value.clone(),
                                value_type: setting_type,
                                description: None,
                                default: None,
                                options: None,
                                readonly: false,
                            });
                        }
                    }

                    if !settings.is_empty() {
                        categories.push(SettingsCategory {
                            name: "General".to_string(),
                            description: Some("GitHub Copilot CLI settings".to_string()),
                            settings,
                        });
                    }

                    break;
                }
            }
        }

        Ok(ToolSettings {
            tool_type: CLIToolType::GitHubCopilot,
            categories,
        })
    }

    async fn set_setting(&self, key: &str, value: serde_json::Value) -> Result<(), ConfigError> {
        let config_path = self.config_dir.join("config.yaml");

        let mut config: serde_json::Map<String, serde_json::Value> = if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .await
                .map_err(|e| ConfigError::IoError(e.to_string()))?;
            ConfigParser::parse_yaml(&content)
                .map_err(|e| ConfigError::ParseError(e.to_string()))?
                .as_object()
                .cloned()
                .unwrap_or_default()
        } else {
            serde_json::Map::new()
        };

        config.insert(key.to_string(), value);

        let content = ConfigParser::serialize(&serde_json::Value::Object(config), ConfigFileType::Yaml)
            .map_err(|e| ConfigError::WriteError(e.to_string()))?;

        self.write_config_file(&config_path.to_string_lossy(), &content)
            .await
    }

    async fn list_agents(&self) -> Result<Vec<AgentDefinition>, ConfigError> {
        let agents_dir = self.config_dir.join("agents");
        let mut agents = Vec::new();

        if !agents_dir.exists() {
            return Ok(agents);
        }

        if let Ok(mut entries) = fs::read_dir(&agents_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if path.extension().map(|e| e == "md").unwrap_or(false) {
                    if let Ok(agent) = self.parse_agent(&path).await {
                        agents.push(agent);
                    }
                }
            }
        }

        Ok(agents)
    }

    async fn get_agent(&self, name: &str) -> Result<AgentDefinition, ConfigError> {
        let agent_path = self.config_dir.join("agents").join(format!("{}.md", name));

        if !agent_path.exists() {
            return Err(ConfigError::NotFound(format!("Agent '{}' not found", name)));
        }

        self.parse_agent(&agent_path).await
    }

    async fn execute_command(
        &self,
        command: &str,
        args: &[&str],
    ) -> Result<CommandOutput, ConfigError> {
        let output = Command::new(command)
            .args(args)
            .output()
            .await
            .map_err(|e| ConfigError::CommandError(e.to_string()))?;

        Ok(CommandOutput {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            success: output.status.success(),
        })
    }
}

impl CopilotConfig {
    async fn parse_agent(&self, path: &PathBuf) -> Result<AgentDefinition, ConfigError> {
        let content = fs::read_to_string(path)
            .await
            .map_err(|e| ConfigError::IoError(e.to_string()))?;

        let parsed = ConfigParser::parse_markdown(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;

        let name = path
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let mut description = None;
        let mut model = None;
        let mut tools = Vec::new();
        let mut system_prompt = None;

        if let Some(fm) = parsed.get("frontmatter").and_then(|v| v.as_object()) {
            description = fm.get("description").and_then(|v| v.as_str()).map(String::from);
            model = fm.get("model").and_then(|v| v.as_str()).map(String::from);
            if let Some(t) = fm.get("tools").and_then(|v| v.as_array()) {
                tools = t
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();
            }
        }

        system_prompt = parsed
            .get("body")
            .and_then(|v| v.as_str())
            .map(String::from);

        Ok(AgentDefinition {
            name,
            description,
            model,
            tools,
            system_prompt,
            source_file: path.to_string_lossy().to_string(),
        })
    }
}
