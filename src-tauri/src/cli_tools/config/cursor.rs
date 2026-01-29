//! Cursor CLI configuration management
//!
//! Handles configuration for Cursor's CLI tool.
//! Config locations:
//! - User: ~/.cursor/mcp.json, ~/.cursor/cli-config.json
//! - Rules: ~/.cursor/rules/*.md

use super::parsers::ConfigParser;
use super::traits::*;
use crate::cli_tools::CLIToolType;
use chrono::{DateTime, Utc};
use std::path::PathBuf;
use tokio::fs;
use tokio::process::Command;

/// Cursor CLI configuration handler
pub struct CursorConfig {
    config_dir: PathBuf,
}

impl CursorConfig {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_default();
        Self {
            config_dir: home.join(".cursor"),
        }
    }

    /// Parse MCP servers from Cursor's format
    fn parse_mcp_servers(config: &serde_json::Value) -> Vec<MCPServerConfig> {
        let mut servers = Vec::new();

        if let Some(mcp_servers) = config.get("mcpServers").and_then(|v| v.as_object()) {
            for (name, server_config) in mcp_servers {
                if let Some(transport) = Self::parse_mcp_transport(server_config) {
                    let enabled = server_config
                        .get("disabled")
                        .and_then(|v| v.as_bool())
                        .map(|d| !d)
                        .unwrap_or(true);

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
                        enabled,
                        env,
                        description: None,
                    });
                }
            }
        }

        servers
    }

    fn parse_mcp_transport(config: &serde_json::Value) -> Option<MCPTransport> {
        // Check for command-based (Stdio) transport first
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
            let transport_type = config
                .get("transport")
                .and_then(|v| v.as_str())
                .unwrap_or("sse");

            let headers = config
                .get("headers")
                .and_then(|v| v.as_object())
                .map(|obj| {
                    obj.iter()
                        .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                        .collect()
                })
                .unwrap_or_default();

            return match transport_type {
                "sse" => Some(MCPTransport::Sse {
                    url: url.to_string(),
                    headers,
                }),
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

    fn build_mcp_server_json(config: &MCPServerConfig) -> serde_json::Value {
        let mut obj = serde_json::Map::new();

        match &config.transport {
            MCPTransport::Stdio { command, args } => {
                obj.insert("command".to_string(), serde_json::json!(command));
                obj.insert("args".to_string(), serde_json::json!(args));
            }
            MCPTransport::Sse { url, headers } => {
                obj.insert("url".to_string(), serde_json::json!(url));
                obj.insert("transport".to_string(), serde_json::json!("sse"));
                if !headers.is_empty() {
                    obj.insert("headers".to_string(), serde_json::json!(headers));
                }
            }
            MCPTransport::Http { url, headers } => {
                obj.insert("url".to_string(), serde_json::json!(url));
                obj.insert("transport".to_string(), serde_json::json!("http"));
                if !headers.is_empty() {
                    obj.insert("headers".to_string(), serde_json::json!(headers));
                }
            }
        }

        if !config.enabled {
            obj.insert("disabled".to_string(), serde_json::json!(true));
        }

        if !config.env.is_empty() {
            obj.insert("env".to_string(), serde_json::json!(config.env));
        }

        serde_json::Value::Object(obj)
    }
}

impl Default for CursorConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl CLIToolConfig for CursorConfig {
    fn tool_type(&self) -> CLIToolType {
        CLIToolType::Cursor
    }

    fn config_dir(&self) -> PathBuf {
        self.config_dir.clone()
    }

    async fn list_config_files(&self) -> Result<Vec<ConfigFileInfo>, ConfigError> {
        let mut files = Vec::new();

        let paths = vec![
            (
                self.config_dir.join("mcp.json"),
                ConfigFileType::Json,
                "MCP server configuration",
            ),
            (
                self.config_dir.join("cli-config.json"),
                ConfigFileType::Json,
                "CLI configuration",
            ),
            (
                self.config_dir.join("settings.json"),
                ConfigFileType::Json,
                "General settings",
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

        // Scan rules directory
        let rules_dir = self.config_dir.join("rules");
        if rules_dir.exists() {
            if let Ok(mut entries) = fs::read_dir(&rules_dir).await {
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
                                    .unwrap_or("rule")
                                    .to_string(),
                                file_type: ConfigFileType::Markdown,
                                size_bytes: metadata.len(),
                                modified,
                                scope: ConfigScope::User,
                                description: Some("Cursor rule".to_string()),
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
            .unwrap_or("json");
        let file_type = ConfigFileType::from_extension(ext).unwrap_or(ConfigFileType::Json);

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
        let settings_path = self.config_dir.join("cli-config.json");
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
                        description: Some("Cursor CLI settings".to_string()),
                        settings,
                    });
                }
            }
        }

        Ok(ToolSettings {
            tool_type: CLIToolType::Cursor,
            categories,
        })
    }

    async fn set_setting(&self, key: &str, value: serde_json::Value) -> Result<(), ConfigError> {
        let settings_path = self.config_dir.join("cli-config.json");

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

    async fn list_mcp_servers(&self) -> Result<Vec<MCPServerConfig>, ConfigError> {
        let mcp_path = self.config_dir.join("mcp.json");

        if !mcp_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&mcp_path)
            .await
            .map_err(|e| ConfigError::IoError(e.to_string()))?;

        let config = ConfigParser::parse_json(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;

        Ok(Self::parse_mcp_servers(&config))
    }

    async fn add_mcp_server(&self, config: MCPServerConfig) -> Result<(), ConfigError> {
        let mcp_path = self.config_dir.join("mcp.json");

        let mut root: serde_json::Map<String, serde_json::Value> = if mcp_path.exists() {
            let content = fs::read_to_string(&mcp_path)
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

        let mcp_servers = root
            .entry("mcpServers".to_string())
            .or_insert_with(|| serde_json::json!({}));

        if let Some(servers) = mcp_servers.as_object_mut() {
            servers.insert(config.name.clone(), Self::build_mcp_server_json(&config));
        }

        let content = serde_json::to_string_pretty(&root)
            .map_err(|e| ConfigError::WriteError(e.to_string()))?;

        self.write_config_file(&mcp_path.to_string_lossy(), &content)
            .await
    }

    async fn remove_mcp_server(&self, name: &str) -> Result<(), ConfigError> {
        let mcp_path = self.config_dir.join("mcp.json");

        if !mcp_path.exists() {
            return Err(ConfigError::NotFound("MCP configuration not found".to_string()));
        }

        let content = fs::read_to_string(&mcp_path)
            .await
            .map_err(|e| ConfigError::IoError(e.to_string()))?;

        let mut root: serde_json::Map<String, serde_json::Value> =
            ConfigParser::parse_json(&content)
                .map_err(|e| ConfigError::ParseError(e.to_string()))?
                .as_object()
                .cloned()
                .unwrap_or_default();

        if let Some(mcp_servers) = root.get_mut("mcpServers").and_then(|v| v.as_object_mut()) {
            mcp_servers.remove(name);
        }

        let content = serde_json::to_string_pretty(&root)
            .map_err(|e| ConfigError::WriteError(e.to_string()))?;

        self.write_config_file(&mcp_path.to_string_lossy(), &content)
            .await
    }

    async fn list_agents(&self) -> Result<Vec<AgentDefinition>, ConfigError> {
        // Cursor uses "rules" instead of "agents"
        let rules_dir = self.config_dir.join("rules");
        let mut agents = Vec::new();

        if !rules_dir.exists() {
            return Ok(agents);
        }

        if let Ok(mut entries) = fs::read_dir(&rules_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if path.extension().map(|e| e == "md").unwrap_or(false) {
                    if let Ok(agent) = self.parse_rule(&path).await {
                        agents.push(agent);
                    }
                }
            }
        }

        Ok(agents)
    }

    async fn get_agent(&self, name: &str) -> Result<AgentDefinition, ConfigError> {
        let rule_path = self.config_dir.join("rules").join(format!("{}.md", name));

        if !rule_path.exists() {
            return Err(ConfigError::NotFound(format!("Rule '{}' not found", name)));
        }

        self.parse_rule(&rule_path).await
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

impl CursorConfig {
    async fn parse_rule(&self, path: &PathBuf) -> Result<AgentDefinition, ConfigError> {
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
        let mut tools = Vec::new();

        if let Some(fm) = parsed.get("frontmatter").and_then(|v| v.as_object()) {
            description = fm.get("description").and_then(|v| v.as_str()).map(String::from);
            if let Some(t) = fm.get("tools").and_then(|v| v.as_array()) {
                tools = t
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();
            }
        }

        let system_prompt = parsed
            .get("body")
            .and_then(|v| v.as_str())
            .map(String::from);

        Ok(AgentDefinition {
            name,
            description,
            model: None, // Cursor rules don't specify models
            tools,
            system_prompt,
            source_file: path.to_string_lossy().to_string(),
        })
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
                "posthog": {
                    "command": "npx",
                    "args": ["-y", "@anthropic/posthog-mcp@latest"]
                },
                "sequential": {
                    "command": "npx",
                    "args": ["@modelcontextprotocol/server-sequential-thinking"]
                }
            }
        });

        let servers = CursorConfig::parse_mcp_servers(&config);

        assert_eq!(servers.len(), 2, "Expected 2 MCP servers");

        let posthog = servers.iter().find(|s| s.name == "posthog").expect("PostHog not found");
        assert!(posthog.enabled, "PostHog should be enabled");
        match &posthog.transport {
            MCPTransport::Stdio { command, args } => {
                assert_eq!(command, "npx", "Command should be npx");
                assert!(args.iter().any(|a| a.contains("posthog")));
            }
            _ => panic!("PostHog should use Stdio transport"),
        }
    }

    #[test]
    fn test_parse_mcp_servers_with_url() {
        let config = json!({
            "mcpServers": {
                "supabase": {
                    "url": "https://mcp.supabase.com/mcp"
                }
            }
        });

        let servers = CursorConfig::parse_mcp_servers(&config);

        assert_eq!(servers.len(), 1, "Expected 1 MCP server");

        let supabase = &servers[0];
        assert_eq!(supabase.name, "supabase");
        match &supabase.transport {
            MCPTransport::Sse { url, .. } => {
                assert!(url.contains("mcp.supabase.com"));
            }
            _ => panic!("Supabase should use SSE transport"),
        }
    }

    #[tokio::test]
    async fn test_list_mcp_servers_integration() {
        use tempfile::tempdir;
        use tokio::fs;

        let temp_dir = tempdir().expect("Failed to create temp dir");
        let mcp_path = temp_dir.path().join("mcp.json");

        let config = json!({
            "mcpServers": {
                "posthog": {
                    "command": "npx",
                    "args": ["-y", "@anthropic/posthog-mcp@latest"],
                    "env": {
                        "POSTHOG_API_KEY": "test_key"
                    }
                },
                "supabase": {
                    "url": "https://mcp.supabase.com/mcp?project_ref=testproject"
                }
            }
        });

        fs::write(&mcp_path, serde_json::to_string_pretty(&config).unwrap())
            .await
            .expect("Failed to write config");

        let cursor_config = CursorConfig {
            config_dir: temp_dir.path().to_path_buf(),
        };

        let servers = cursor_config.list_mcp_servers().await.expect("Failed to list MCP servers");

        assert_eq!(servers.len(), 2, "Expected 2 MCP servers");

        // Verify PostHog (Stdio with env)
        let posthog = servers.iter().find(|s| s.name == "posthog").expect("PostHog not found");
        match &posthog.transport {
            MCPTransport::Stdio { command, .. } => {
                assert_eq!(command, "npx");
            }
            _ => panic!("PostHog should use Stdio transport"),
        }
        assert!(posthog.env.contains_key("POSTHOG_API_KEY"));

        // Verify Supabase (URL-based SSE)
        let supabase = servers.iter().find(|s| s.name == "supabase").expect("Supabase not found");
        match &supabase.transport {
            MCPTransport::Sse { url, .. } => {
                assert!(url.contains("mcp.supabase.com"));
                assert!(url.contains("testproject"));
            }
            _ => panic!("Supabase should use SSE transport"),
        }
    }
}
