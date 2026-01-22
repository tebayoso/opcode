//! Gemini CLI configuration management
//!
//! Handles configuration for Google's Gemini CLI tool.
//! Config locations:
//! - User: ~/.gemini/settings.json
//!
//! MCP Server configuration:
//! - Stored in settings.json under `mcpServers` key
//! - Supports Stdio (command/args) and SSE (url/headers) transports

use super::parsers::ConfigParser;
use super::traits::*;
use crate::cli_tools::CLIToolType;
use chrono::{DateTime, Utc};
use std::path::PathBuf;
use tokio::fs;
use tokio::process::Command;

/// Gemini CLI configuration handler
pub struct GeminiConfig {
    config_dir: PathBuf,
}

impl GeminiConfig {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_default();
        Self {
            config_dir: home.join(".gemini"),
        }
    }
}

impl Default for GeminiConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl GeminiConfig {
    /// Parse MCP transport from a server configuration
    fn parse_mcp_transport(config: &serde_json::Value) -> Option<MCPTransport> {
        // Check for command-based (Stdio) transport
        if let Some(command) = config.get("command").and_then(|v| v.as_str()) {
            let args = config
                .get("args")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            return Some(MCPTransport::Stdio {
                command: command.to_string(),
                args,
            });
        }

        // Check for URL-based (SSE/HTTP) transport
        if let Some(url) = config.get("url").and_then(|v| v.as_str()) {
            let headers = config
                .get("headers")
                .and_then(|v| v.as_object())
                .map(|obj| {
                    obj.iter()
                        .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                        .collect()
                })
                .unwrap_or_default();

            // Check for explicit transport type or default to SSE
            let transport_type = config
                .get("transport")
                .and_then(|v| v.as_str())
                .unwrap_or("sse");

            return match transport_type {
                "http" => Some(MCPTransport::Http {
                    url: url.to_string(),
                    headers,
                }),
                _ => Some(MCPTransport::Sse {
                    url: url.to_string(),
                    headers,
                }),
            };
        }

        None
    }

    /// Parse MCP servers from the settings JSON
    fn parse_mcp_servers(config: &serde_json::Value) -> Vec<MCPServerConfig> {
        let mut servers = Vec::new();

        if let Some(mcp_servers) = config.get("mcpServers").and_then(|v| v.as_object()) {
            for (name, server_config) in mcp_servers {
                if let Some(transport) = Self::parse_mcp_transport(server_config) {
                    // Check for disabled flag
                    let disabled = server_config
                        .get("disabled")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);

                    // Parse environment variables
                    let env = server_config
                        .get("env")
                        .and_then(|v| v.as_object())
                        .map(|obj| {
                            obj.iter()
                                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                                .collect()
                        })
                        .unwrap_or_default();

                    servers.push(MCPServerConfig {
                        name: name.clone(),
                        transport,
                        enabled: !disabled,
                        env,
                        description: None,
                    });
                }
            }
        }

        servers
    }

    /// Read and parse the settings.json file
    async fn read_settings(&self) -> Result<serde_json::Value, ConfigError> {
        let settings_path = self.config_dir.join("settings.json");

        if !settings_path.exists() {
            return Ok(serde_json::json!({}));
        }

        let content = fs::read_to_string(&settings_path)
            .await
            .map_err(|e| ConfigError::IoError(e.to_string()))?;

        ConfigParser::parse_json(&content).map_err(|e| ConfigError::ParseError(e.to_string()))
    }

    /// Write settings to the settings.json file
    async fn write_settings(&self, config: &serde_json::Value) -> Result<(), ConfigError> {
        let settings_path = self.config_dir.join("settings.json");

        // Ensure config directory exists
        if let Some(parent) = settings_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| ConfigError::IoError(e.to_string()))?;
        }

        let content = serde_json::to_string_pretty(config)
            .map_err(|e| ConfigError::WriteError(e.to_string()))?;

        fs::write(&settings_path, content)
            .await
            .map_err(|e| ConfigError::WriteError(e.to_string()))
    }
}

#[async_trait::async_trait]
impl CLIToolConfig for GeminiConfig {
    fn tool_type(&self) -> CLIToolType {
        CLIToolType::Gemini
    }

    fn config_dir(&self) -> PathBuf {
        self.config_dir.clone()
    }

    async fn list_config_files(&self) -> Result<Vec<ConfigFileInfo>, ConfigError> {
        let mut files = Vec::new();

        let paths = vec![
            (self.config_dir.join("settings.json"), "Main settings"),
            (self.config_dir.join("config.json"), "Configuration"),
        ];

        for (path, desc) in paths {
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
                        file_type: ConfigFileType::Json,
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

        Ok(ConfigParser::create_file_content(
            path,
            &content,
            ConfigFileType::Json,
        ))
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
        let settings_path = self.config_dir.join("settings.json");
        let mut categories = Vec::new();

        if settings_path.exists() {
            let content = fs::read_to_string(&settings_path)
                .await
                .map_err(|e| ConfigError::IoError(e.to_string()))?;

            if let Ok(config) = ConfigParser::parse_json(&content) {
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
                        description: Some("Gemini CLI settings".to_string()),
                        settings,
                    });
                }
            }
        }

        Ok(ToolSettings {
            tool_type: CLIToolType::Gemini,
            categories,
        })
    }

    async fn set_setting(&self, key: &str, value: serde_json::Value) -> Result<(), ConfigError> {
        let settings_path = self.config_dir.join("settings.json");

        let mut config: serde_json::Map<String, serde_json::Value> = if settings_path.exists() {
            let content = fs::read_to_string(&settings_path)
                .await
                .map_err(|e| ConfigError::IoError(e.to_string()))?;
            ConfigParser::parse_json(&content)
                .map_err(|e| ConfigError::ParseError(e.to_string()))?
                .as_object()
                .cloned()
                .unwrap_or_default()
        } else {
            serde_json::Map::new()
        };

        config.insert(key.to_string(), value);

        let content = serde_json::to_string_pretty(&config)
            .map_err(|e| ConfigError::WriteError(e.to_string()))?;

        self.write_config_file(&settings_path.to_string_lossy(), &content)
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
        let config = self.read_settings().await?;
        Ok(Self::parse_mcp_servers(&config))
    }

    async fn add_mcp_server(&self, server: MCPServerConfig) -> Result<(), ConfigError> {
        let mut config = self.read_settings().await?;

        // Ensure mcpServers object exists
        if config.get("mcpServers").is_none() {
            config
                .as_object_mut()
                .unwrap()
                .insert("mcpServers".to_string(), serde_json::json!({}));
        }

        // Build server configuration based on transport type
        let mut server_config = serde_json::Map::new();

        match &server.transport {
            MCPTransport::Stdio { command, args } => {
                server_config.insert("command".to_string(), serde_json::json!(command));
                if !args.is_empty() {
                    server_config.insert("args".to_string(), serde_json::json!(args));
                }
            }
            MCPTransport::Sse { url, headers } => {
                server_config.insert("url".to_string(), serde_json::json!(url));
                server_config.insert("transport".to_string(), serde_json::json!("sse"));
                if !headers.is_empty() {
                    server_config.insert("headers".to_string(), serde_json::json!(headers));
                }
            }
            MCPTransport::Http { url, headers } => {
                server_config.insert("url".to_string(), serde_json::json!(url));
                server_config.insert("transport".to_string(), serde_json::json!("http"));
                if !headers.is_empty() {
                    server_config.insert("headers".to_string(), serde_json::json!(headers));
                }
            }
        }

        // Add env if present
        if !server.env.is_empty() {
            server_config.insert("env".to_string(), serde_json::json!(server.env));
        }

        // Add disabled flag if not enabled
        if !server.enabled {
            server_config.insert("disabled".to_string(), serde_json::json!(true));
        }

        // Insert the server config
        config
            .get_mut("mcpServers")
            .and_then(|v| v.as_object_mut())
            .unwrap()
            .insert(server.name.clone(), serde_json::Value::Object(server_config));

        self.write_settings(&config).await
    }

    async fn remove_mcp_server(&self, name: &str) -> Result<(), ConfigError> {
        let mut config = self.read_settings().await?;

        if let Some(mcp_servers) = config.get_mut("mcpServers").and_then(|v| v.as_object_mut()) {
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

        self.write_settings(&config).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_mcp_servers_with_stdio() {
        let config = json!({
            "mcpServers": {
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

        let servers = GeminiConfig::parse_mcp_servers(&config);
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
    fn test_parse_mcp_servers_with_sse() {
        let config = json!({
            "mcpServers": {
                "supabase": {
                    "url": "https://mcp.supabase.com/mcp?project_ref=test",
                    "headers": {
                        "Authorization": "Bearer test_token"
                    }
                }
            }
        });

        let servers = GeminiConfig::parse_mcp_servers(&config);
        assert_eq!(servers.len(), 1, "Expected 1 MCP server");

        let supabase = &servers[0];
        assert_eq!(supabase.name, "supabase");
        assert!(supabase.enabled);
        match &supabase.transport {
            MCPTransport::Sse { url, headers } => {
                assert_eq!(url, "https://mcp.supabase.com/mcp?project_ref=test");
                assert_eq!(
                    headers.get("Authorization"),
                    Some(&"Bearer test_token".to_string())
                );
            }
            _ => panic!("Expected SSE transport"),
        }
    }

    #[test]
    fn test_parse_mcp_servers_with_disabled() {
        let config = json!({
            "mcpServers": {
                "disabled-server": {
                    "command": "npx",
                    "args": ["some-mcp"],
                    "disabled": true
                }
            }
        });

        let servers = GeminiConfig::parse_mcp_servers(&config);
        assert_eq!(servers.len(), 1);
        assert!(!servers[0].enabled, "Server should be disabled");
    }

    #[tokio::test]
    async fn test_list_mcp_servers_integration() {
        use tempfile::tempdir;
        use tokio::fs;

        let temp_dir = tempdir().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("settings.json");

        let config = json!({
            "mcpServers": {
                "supabase": {
                    "url": "https://mcp.supabase.com/mcp?project_ref=testproject",
                    "headers": {
                        "Authorization": "Bearer test_token_abc123"
                    }
                },
                "context7": {
                    "command": "npx",
                    "args": ["-y", "@upstash/context7-mcp@latest"]
                },
                "disabled-mcp": {
                    "command": "npx",
                    "args": ["disabled-server"],
                    "disabled": true
                }
            }
        });

        fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap())
            .await
            .expect("Failed to write test config");

        let gemini_config = GeminiConfig {
            config_dir: temp_dir.path().to_path_buf(),
        };

        let servers = gemini_config
            .list_mcp_servers()
            .await
            .expect("Failed to list MCP servers");

        assert_eq!(servers.len(), 3, "Expected 3 MCP servers");

        // Check Supabase SSE server
        let supabase = servers.iter().find(|s| s.name == "supabase").unwrap();
        assert!(supabase.enabled);
        match &supabase.transport {
            MCPTransport::Sse { url, headers } => {
                assert!(url.contains("supabase.com"));
                assert!(headers.contains_key("Authorization"));
            }
            _ => panic!("Expected SSE transport for supabase"),
        }

        // Check Context7 Stdio server
        let context7 = servers.iter().find(|s| s.name == "context7").unwrap();
        assert!(context7.enabled);
        match &context7.transport {
            MCPTransport::Stdio { command, args } => {
                assert_eq!(command, "npx");
                assert!(args.contains(&"-y".to_string()));
            }
            _ => panic!("Expected Stdio transport for context7"),
        }

        // Check disabled server
        let disabled = servers.iter().find(|s| s.name == "disabled-mcp").unwrap();
        assert!(!disabled.enabled, "Server should be disabled");
    }
}
