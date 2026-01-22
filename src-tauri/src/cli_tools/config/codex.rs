//! Codex CLI configuration management
//!
//! Handles configuration for OpenAI's Codex CLI tool.
//! Config locations:
//! - User: ~/.codex/config.toml (TOML format)
//!
//! MCP Server configuration:
//! - Stored in config.toml under `[mcp_servers.server-name]` sections
//! - Only supports STDIO transport (not SSE or HTTP yet)
//! - Format:
//!   ```toml
//!   [mcp_servers.context7]
//!   command = "npx"
//!   args = ["-y", "@upstash/context7-mcp@latest"]
//!   env = { "API_KEY" = "value" }
//!   ```

use super::parsers::ConfigParser;
use super::traits::*;
use crate::cli_tools::CLIToolType;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tokio::process::Command;

/// Codex CLI configuration handler
pub struct CodexConfig {
    config_dir: PathBuf,
}

impl CodexConfig {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_default();
        Self {
            config_dir: home.join(".codex"),
        }
    }
}

impl Default for CodexConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl CodexConfig {
    /// Parse MCP servers from the config (parsed as JSON after TOML conversion)
    /// Codex only supports STDIO transport
    fn parse_mcp_servers(config: &serde_json::Value) -> Vec<MCPServerConfig> {
        let mut servers = Vec::new();

        if let Some(mcp_servers) = config.get("mcp_servers").and_then(|v| v.as_object()) {
            for (name, server_config) in mcp_servers {
                // Codex only supports STDIO transport (command + args)
                if let Some(command) = server_config.get("command").and_then(|v| v.as_str()) {
                    let args = server_config
                        .get("args")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                .collect()
                        })
                        .unwrap_or_default();

                    // Parse environment variables
                    let env: HashMap<String, String> = server_config
                        .get("env")
                        .and_then(|v| v.as_object())
                        .map(|obj| {
                            obj.iter()
                                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                                .collect()
                        })
                        .unwrap_or_default();

                    // Check for disabled flag
                    let disabled = server_config
                        .get("disabled")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);

                    servers.push(MCPServerConfig {
                        name: name.clone(),
                        transport: MCPTransport::Stdio {
                            command: command.to_string(),
                            args,
                        },
                        enabled: !disabled,
                        env,
                        description: None,
                    });
                }
            }
        }

        servers
    }

    /// Read and parse the config.toml file
    async fn read_config(&self) -> Result<serde_json::Value, ConfigError> {
        let config_path = self.config_dir.join("config.toml");

        if !config_path.exists() {
            return Ok(serde_json::json!({}));
        }

        let content = fs::read_to_string(&config_path)
            .await
            .map_err(|e| ConfigError::IoError(e.to_string()))?;

        ConfigParser::parse_toml(&content).map_err(|e| ConfigError::ParseError(e.to_string()))
    }

    /// Write config to the config.toml file
    async fn write_config(&self, config: &serde_json::Value) -> Result<(), ConfigError> {
        let config_path = self.config_dir.join("config.toml");

        // Ensure config directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| ConfigError::IoError(e.to_string()))?;
        }

        let content = ConfigParser::serialize(config, ConfigFileType::Toml)
            .map_err(|e| ConfigError::WriteError(e.to_string()))?;

        fs::write(&config_path, content)
            .await
            .map_err(|e| ConfigError::WriteError(e.to_string()))
    }
}

#[async_trait::async_trait]
impl CLIToolConfig for CodexConfig {
    fn tool_type(&self) -> CLIToolType {
        CLIToolType::Codex
    }

    fn config_dir(&self) -> PathBuf {
        self.config_dir.clone()
    }

    async fn list_config_files(&self) -> Result<Vec<ConfigFileInfo>, ConfigError> {
        let mut files = Vec::new();

        let paths = vec![
            (
                self.config_dir.join("config.toml"),
                ConfigFileType::Toml,
                "Main configuration",
            ),
            (
                self.config_dir.join("config.json"),
                ConfigFileType::Json,
                "JSON configuration",
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
            .unwrap_or("toml");
        let file_type = ConfigFileType::from_extension(ext).unwrap_or(ConfigFileType::Toml);

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
        let config_path = self.config_dir.join("config.toml");
        let mut categories = Vec::new();

        if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .await
                .map_err(|e| ConfigError::IoError(e.to_string()))?;

            if let Ok(config) = ConfigParser::parse_toml(&content) {
                let mut settings = Vec::new();

                if let Some(obj) = config.as_object() {
                    for (key, value) in obj {
                        // Handle nested TOML sections
                        if value.is_object() {
                            let section_settings: Vec<SettingDefinition> = value
                                .as_object()
                                .map(|section| {
                                    section
                                        .iter()
                                        .map(|(k, v)| {
                                            let setting_type = match v {
                                                serde_json::Value::String(_) => SettingType::String,
                                                serde_json::Value::Number(_) => SettingType::Number,
                                                serde_json::Value::Bool(_) => SettingType::Boolean,
                                                serde_json::Value::Array(_) => SettingType::Array,
                                                serde_json::Value::Object(_) => SettingType::Object,
                                                serde_json::Value::Null => SettingType::String,
                                            };

                                            SettingDefinition {
                                                key: format!("{}.{}", key, k),
                                                value: v.clone(),
                                                value_type: setting_type,
                                                description: None,
                                                default: None,
                                                options: None,
                                                readonly: false,
                                            }
                                        })
                                        .collect()
                                })
                                .unwrap_or_default();

                            if !section_settings.is_empty() {
                                categories.push(SettingsCategory {
                                    name: key.clone(),
                                    description: None,
                                    settings: section_settings,
                                });
                            }
                        } else {
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
                }

                if !settings.is_empty() {
                    categories.insert(
                        0,
                        SettingsCategory {
                            name: "General".to_string(),
                            description: Some("Codex CLI settings".to_string()),
                            settings,
                        },
                    );
                }
            }
        }

        Ok(ToolSettings {
            tool_type: CLIToolType::Codex,
            categories,
        })
    }

    async fn set_setting(&self, key: &str, value: serde_json::Value) -> Result<(), ConfigError> {
        let config_path = self.config_dir.join("config.toml");

        let mut config: serde_json::Map<String, serde_json::Value> = if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .await
                .map_err(|e| ConfigError::IoError(e.to_string()))?;
            ConfigParser::parse_toml(&content)
                .map_err(|e| ConfigError::ParseError(e.to_string()))?
                .as_object()
                .cloned()
                .unwrap_or_default()
        } else {
            serde_json::Map::new()
        };

        // Handle nested keys (e.g., "section.key")
        let parts: Vec<&str> = key.split('.').collect();
        if parts.len() == 2 {
            let section = config
                .entry(parts[0].to_string())
                .or_insert_with(|| serde_json::json!({}));
            if let Some(obj) = section.as_object_mut() {
                obj.insert(parts[1].to_string(), value);
            }
        } else {
            config.insert(key.to_string(), value);
        }

        // Serialize back to TOML
        let content = ConfigParser::serialize(&serde_json::Value::Object(config), ConfigFileType::Toml)
            .map_err(|e| ConfigError::WriteError(e.to_string()))?;

        self.write_config_file(&config_path.to_string_lossy(), &content)
            .await
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

    async fn list_mcp_servers(&self) -> Result<Vec<MCPServerConfig>, ConfigError> {
        let config = self.read_config().await?;
        Ok(Self::parse_mcp_servers(&config))
    }

    async fn add_mcp_server(&self, server: MCPServerConfig) -> Result<(), ConfigError> {
        // Codex only supports STDIO transport
        let (command, args) = match &server.transport {
            MCPTransport::Stdio { command, args } => (command.clone(), args.clone()),
            MCPTransport::Sse { .. } | MCPTransport::Http { .. } => {
                return Err(ConfigError::NotSupported(
                    "Codex CLI only supports STDIO transport. SSE and HTTP are not yet supported."
                        .to_string(),
                ));
            }
        };

        let mut config = self.read_config().await?;

        // Ensure mcp_servers object exists
        if config.get("mcp_servers").is_none() {
            config
                .as_object_mut()
                .unwrap()
                .insert("mcp_servers".to_string(), serde_json::json!({}));
        }

        // Build server configuration
        let mut server_config = serde_json::Map::new();
        server_config.insert("command".to_string(), serde_json::json!(command));

        if !args.is_empty() {
            server_config.insert("args".to_string(), serde_json::json!(args));
        }

        if !server.env.is_empty() {
            server_config.insert("env".to_string(), serde_json::json!(server.env));
        }

        if !server.enabled {
            server_config.insert("disabled".to_string(), serde_json::json!(true));
        }

        // Insert the server config
        config
            .get_mut("mcp_servers")
            .and_then(|v| v.as_object_mut())
            .unwrap()
            .insert(server.name.clone(), serde_json::Value::Object(server_config));

        self.write_config(&config).await
    }

    async fn remove_mcp_server(&self, name: &str) -> Result<(), ConfigError> {
        let mut config = self.read_config().await?;

        if let Some(mcp_servers) = config.get_mut("mcp_servers").and_then(|v| v.as_object_mut()) {
            if mcp_servers.remove(name).is_none() {
                return Err(ConfigError::NotFound(format!(
                    "MCP server '{}' not found",
                    name
                )));
            }
        } else {
            return Err(ConfigError::NotFound(format!(
                "MCP server '{}' not found",
                name
            )));
        }

        self.write_config(&config).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_mcp_servers_stdio() {
        // Simulating parsed TOML as JSON (how ConfigParser::parse_toml returns it)
        let config = json!({
            "mcp_servers": {
                "context7": {
                    "command": "npx",
                    "args": ["-y", "@upstash/context7-mcp@latest"]
                },
                "playwright": {
                    "command": "npx",
                    "args": ["@playwright/mcp@latest"],
                    "env": {
                        "DISPLAY": ":0"
                    }
                }
            }
        });

        let servers = CodexConfig::parse_mcp_servers(&config);
        assert_eq!(servers.len(), 2, "Expected 2 MCP servers");

        let context7 = servers.iter().find(|s| s.name == "context7").unwrap();
        assert!(context7.enabled);
        match &context7.transport {
            MCPTransport::Stdio { command, args } => {
                assert_eq!(command, "npx");
                assert_eq!(args, &["-y", "@upstash/context7-mcp@latest"]);
            }
            _ => panic!("Expected Stdio transport"),
        }

        let playwright = servers.iter().find(|s| s.name == "playwright").unwrap();
        assert_eq!(playwright.env.get("DISPLAY"), Some(&":0".to_string()));
    }

    #[test]
    fn test_parse_mcp_servers_with_disabled() {
        let config = json!({
            "mcp_servers": {
                "disabled-server": {
                    "command": "npx",
                    "args": ["some-mcp"],
                    "disabled": true
                }
            }
        });

        let servers = CodexConfig::parse_mcp_servers(&config);
        assert_eq!(servers.len(), 1);
        assert!(!servers[0].enabled, "Server should be disabled");
    }

    #[test]
    fn test_parse_mcp_servers_ignores_non_stdio() {
        // Codex only supports STDIO, so URL-based servers should be ignored
        let config = json!({
            "mcp_servers": {
                "valid-stdio": {
                    "command": "npx",
                    "args": ["valid-mcp"]
                },
                "invalid-sse": {
                    "url": "https://example.com/mcp"
                }
            }
        });

        let servers = CodexConfig::parse_mcp_servers(&config);
        assert_eq!(servers.len(), 1, "Should only parse STDIO servers");
        assert_eq!(servers[0].name, "valid-stdio");
    }

    #[tokio::test]
    async fn test_list_mcp_servers_integration() {
        use tempfile::tempdir;
        use tokio::fs;

        let temp_dir = tempdir().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("config.toml");

        // Write TOML config directly
        let toml_content = r#"
[mcp_servers.context7]
command = "npx"
args = ["-y", "@upstash/context7-mcp@latest"]

[mcp_servers.playwright]
command = "npx"
args = ["@playwright/mcp@latest"]
env = { DISPLAY = ":0" }

[mcp_servers.disabled-mcp]
command = "npx"
args = ["disabled-server"]
disabled = true
"#;

        fs::write(&config_path, toml_content)
            .await
            .expect("Failed to write test config");

        let codex_config = CodexConfig {
            config_dir: temp_dir.path().to_path_buf(),
        };

        let servers = codex_config
            .list_mcp_servers()
            .await
            .expect("Failed to list MCP servers");

        assert_eq!(servers.len(), 3, "Expected 3 MCP servers");

        // Check Context7 server
        let context7 = servers.iter().find(|s| s.name == "context7").unwrap();
        assert!(context7.enabled);
        match &context7.transport {
            MCPTransport::Stdio { command, args } => {
                assert_eq!(command, "npx");
                assert!(args.contains(&"-y".to_string()));
            }
            _ => panic!("Expected Stdio transport"),
        }

        // Check Playwright with env
        let playwright = servers.iter().find(|s| s.name == "playwright").unwrap();
        assert_eq!(playwright.env.get("DISPLAY"), Some(&":0".to_string()));

        // Check disabled server
        let disabled = servers.iter().find(|s| s.name == "disabled-mcp").unwrap();
        assert!(!disabled.enabled, "Server should be disabled");
    }

    #[tokio::test]
    async fn test_add_mcp_server_rejects_sse() {
        use tempfile::tempdir;

        let temp_dir = tempdir().expect("Failed to create temp dir");

        let codex_config = CodexConfig {
            config_dir: temp_dir.path().to_path_buf(),
        };

        // Attempt to add SSE server should fail
        let sse_server = MCPServerConfig {
            name: "sse-server".to_string(),
            transport: MCPTransport::Sse {
                url: "https://example.com/mcp".to_string(),
                headers: HashMap::new(),
            },
            enabled: true,
            env: HashMap::new(),
            description: None,
        };

        let result = codex_config.add_mcp_server(sse_server).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(ConfigError::NotSupported(_))));
    }
}
