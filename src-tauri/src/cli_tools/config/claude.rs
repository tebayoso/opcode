//! Claude Code configuration management
//!
//! Handles configuration for Anthropic's Claude Code CLI tool.
//! Config locations:
//! - User: ~/.claude/settings.json, ~/.claude.json
//! - Project: .claude/settings.json, CLAUDE.md

use super::parsers::ConfigParser;
use super::traits::*;
use crate::cli_tools::CLIToolType;
use chrono::{DateTime, Utc};
use std::path::PathBuf;
use tokio::fs;
use tokio::process::Command;

/// Claude Code configuration handler
pub struct ClaudeConfig {
    config_dir: PathBuf,
}

impl ClaudeConfig {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_default();
        Self {
            config_dir: home.join(".claude"),
        }
    }

    /// Get paths to check for config files
    fn config_paths(&self) -> Vec<(PathBuf, ConfigScope)> {
        let home = dirs::home_dir().unwrap_or_default();
        vec![
            (self.config_dir.join("settings.json"), ConfigScope::User),
            (home.join(".claude.json"), ConfigScope::User),
            (self.config_dir.join("mcp.json"), ConfigScope::User),
            (self.config_dir.join("CLAUDE.md"), ConfigScope::User),
        ]
    }

    /// Parse MCP servers from Claude's settings format
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

    /// Parse MCP transport from config
    fn parse_mcp_transport(config: &serde_json::Value) -> Option<MCPTransport> {
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

    /// Build MCP server JSON for Claude's format
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

impl Default for ClaudeConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl CLIToolConfig for ClaudeConfig {
    fn tool_type(&self) -> CLIToolType {
        CLIToolType::Claude
    }

    fn config_dir(&self) -> PathBuf {
        self.config_dir.clone()
    }

    async fn list_config_files(&self) -> Result<Vec<ConfigFileInfo>, ConfigError> {
        let mut files = Vec::new();

        for (path, scope) in self.config_paths() {
            if path.exists() {
                if let Ok(metadata) = fs::metadata(&path).await {
                    let modified = metadata
                        .modified()
                        .map(|t| DateTime::<Utc>::from(t))
                        .unwrap_or_else(|_| Utc::now());

                    let ext = path
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("json");
                    let file_type =
                        ConfigFileType::from_extension(ext).unwrap_or(ConfigFileType::Json);

                    let name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("config")
                        .to_string();

                    let description = match name.as_str() {
                        "settings.json" => Some("Claude Code settings".to_string()),
                        ".claude.json" => Some("Legacy Claude configuration".to_string()),
                        "mcp.json" => Some("MCP server configuration".to_string()),
                        "CLAUDE.md" => Some("Claude instructions markdown".to_string()),
                        _ => None,
                    };

                    files.push(ConfigFileInfo {
                        path: path.to_string_lossy().to_string(),
                        name,
                        file_type,
                        size_bytes: metadata.len(),
                        modified,
                        scope,
                        description,
                    });
                }
            }
        }

        // Also scan for agent files in ~/.claude/agents/
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
                                description: Some("Custom agent definition".to_string()),
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

        // Ensure parent directory exists
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

        // Read main settings if exists
        if settings_path.exists() {
            let content = fs::read_to_string(&settings_path)
                .await
                .map_err(|e| ConfigError::IoError(e.to_string()))?;

            if let Ok(config) = ConfigParser::parse_json(&content) {
                let mut general_settings = Vec::new();
                let mut permission_settings = Vec::new();

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

                        let setting = SettingDefinition {
                            key: key.clone(),
                            value: value.clone(),
                            value_type: setting_type,
                            description: Self::get_setting_description(key),
                            default: None,
                            options: None,
                            readonly: false,
                        };

                        // Categorize settings
                        if key.contains("permission") || key.contains("allow") || key.contains("deny")
                        {
                            permission_settings.push(setting);
                        } else {
                            general_settings.push(setting);
                        }
                    }
                }

                if !general_settings.is_empty() {
                    categories.push(SettingsCategory {
                        name: "General".to_string(),
                        description: Some("General Claude Code settings".to_string()),
                        settings: general_settings,
                    });
                }

                if !permission_settings.is_empty() {
                    categories.push(SettingsCategory {
                        name: "Permissions".to_string(),
                        description: Some("File and command permissions".to_string()),
                        settings: permission_settings,
                    });
                }
            }
        }

        Ok(ToolSettings {
            tool_type: CLIToolType::Claude,
            categories,
        })
    }

    async fn set_setting(&self, key: &str, value: serde_json::Value) -> Result<(), ConfigError> {
        let settings_path = self.config_dir.join("settings.json");

        // Read existing settings or create new
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

        // Update setting
        config.insert(key.to_string(), value);

        // Write back
        let content = serde_json::to_string_pretty(&config)
            .map_err(|e| ConfigError::WriteError(e.to_string()))?;

        self.write_config_file(&settings_path.to_string_lossy(), &content)
            .await
    }

    async fn list_mcp_servers(&self) -> Result<Vec<MCPServerConfig>, ConfigError> {
        // Check multiple possible locations for MCP config
        // IMPORTANT: ~/.claude.json is the PRIMARY location for global MCP servers
        let home = dirs::home_dir().unwrap_or_default();
        let mcp_paths = vec![
            home.join(".claude.json"), // Global MCP servers (primary location)
            self.config_dir.join("mcp.json"),
            self.config_dir.join("settings.json"),
        ];

        for path in mcp_paths {
            if path.exists() {
                let content = fs::read_to_string(&path)
                    .await
                    .map_err(|e| ConfigError::IoError(e.to_string()))?;

                if let Ok(config) = ConfigParser::parse_json(&content) {
                    let servers = Self::parse_mcp_servers(&config);
                    if !servers.is_empty() {
                        return Ok(servers);
                    }
                }
            }
        }

        Ok(Vec::new())
    }

    async fn add_mcp_server(&self, config: MCPServerConfig) -> Result<(), ConfigError> {
        let mcp_path = self.config_dir.join("mcp.json");

        // Read existing or create new
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

        // Get or create mcpServers object
        let mcp_servers = root
            .entry("mcpServers".to_string())
            .or_insert_with(|| serde_json::json!({}));

        if let Some(servers) = mcp_servers.as_object_mut() {
            servers.insert(config.name.clone(), Self::build_mcp_server_json(&config));
        }

        // Write back
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
        let agents_dir = self.config_dir.join("agents");
        let mut agents = Vec::new();

        if !agents_dir.exists() {
            return Ok(agents);
        }

        if let Ok(mut entries) = fs::read_dir(&agents_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if path.extension().map(|e| e == "md").unwrap_or(false) {
                    if let Ok(agent) = self.get_agent_from_path(&path).await {
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

        self.get_agent_from_path(&agent_path).await
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

impl ClaudeConfig {
    /// Get description for known settings
    fn get_setting_description(key: &str) -> Option<String> {
        match key {
            "model" => Some("Default model to use".to_string()),
            "apiKey" => Some("Anthropic API key".to_string()),
            "maxTokens" => Some("Maximum tokens per response".to_string()),
            "temperature" => Some("Response temperature (0-1)".to_string()),
            "mcpServers" => Some("MCP server configurations".to_string()),
            "allowedTools" => Some("List of allowed tools".to_string()),
            "deniedTools" => Some("List of denied tools".to_string()),
            "permissions" => Some("Permission settings".to_string()),
            _ => None,
        }
    }

    /// Parse agent from markdown file
    async fn get_agent_from_path(
        &self,
        path: &PathBuf,
    ) -> Result<AgentDefinition, ConfigError> {
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

        // Extract from frontmatter
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

        // Body is the system prompt
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_mcp_servers_with_stdio() {
        let config = json!({
            "mcpServers": {
                "supabase": {
                    "command": "npx",
                    "args": ["-y", "mcp-remote@latest", "https://mcp.supabase.com/mcp"]
                },
                "playwright": {
                    "command": "npx",
                    "args": ["@playwright/mcp@latest"]
                }
            }
        });

        let servers = ClaudeConfig::parse_mcp_servers(&config);

        assert_eq!(servers.len(), 2, "Expected 2 MCP servers");

        let supabase = servers.iter().find(|s| s.name == "supabase").expect("Supabase not found");
        assert!(supabase.enabled, "Supabase should be enabled");
        match &supabase.transport {
            MCPTransport::Stdio { command, args } => {
                assert_eq!(command, "npx", "Command should be npx");
                assert_eq!(args.len(), 3, "Should have 3 args");
                assert_eq!(args[0], "-y");
                assert_eq!(args[1], "mcp-remote@latest");
                assert!(args[2].contains("mcp.supabase.com"));
            }
            _ => panic!("Supabase should use Stdio transport"),
        }

        let playwright = servers.iter().find(|s| s.name == "playwright").expect("Playwright not found");
        match &playwright.transport {
            MCPTransport::Stdio { command, args } => {
                assert_eq!(command, "npx");
                assert_eq!(args[0], "@playwright/mcp@latest");
            }
            _ => panic!("Playwright should use Stdio transport"),
        }
    }

    #[test]
    fn test_parse_mcp_servers_with_sse() {
        let config = json!({
            "mcpServers": {
                "remote-service": {
                    "url": "https://api.example.com/mcp",
                    "transport": "sse",
                    "headers": {
                        "Authorization": "Bearer secret_token"
                    }
                }
            }
        });

        let servers = ClaudeConfig::parse_mcp_servers(&config);

        assert_eq!(servers.len(), 1, "Expected 1 MCP server");

        let remote = &servers[0];
        assert_eq!(remote.name, "remote-service");
        assert!(remote.enabled);
        match &remote.transport {
            MCPTransport::Sse { url, headers } => {
                assert_eq!(url, "https://api.example.com/mcp");
                assert_eq!(headers.get("Authorization").unwrap(), "Bearer secret_token");
            }
            _ => panic!("Should use SSE transport"),
        }
    }

    #[test]
    fn test_parse_mcp_servers_with_disabled() {
        let config = json!({
            "mcpServers": {
                "disabled-server": {
                    "command": "npx",
                    "args": ["some-package"],
                    "disabled": true
                }
            }
        });

        let servers = ClaudeConfig::parse_mcp_servers(&config);

        assert_eq!(servers.len(), 1);
        assert!(!servers[0].enabled, "Server should be disabled");
    }

    #[tokio::test]
    async fn test_list_mcp_servers_integration() {
        use tempfile::tempdir;
        use tokio::fs;

        let temp_dir = tempdir().expect("Failed to create temp dir");
        let mcp_path = temp_dir.path().join("mcp.json");

        let config = json!({
            "mcpServers": {
                "supabase": {
                    "command": "npx",
                    "args": ["-y", "mcp-remote@latest", "https://mcp.supabase.com/mcp?project_ref=test123"]
                },
                "posthog": {
                    "url": "https://mcp.posthog.com/sse",
                    "transport": "sse",
                    "headers": {
                        "X-API-Key": "phk_test"
                    }
                }
            }
        });

        fs::write(&mcp_path, serde_json::to_string_pretty(&config).unwrap())
            .await
            .expect("Failed to write config");

        let claude_config = ClaudeConfig {
            config_dir: temp_dir.path().to_path_buf(),
        };

        let servers = claude_config.list_mcp_servers().await.expect("Failed to list MCP servers");

        assert_eq!(servers.len(), 2, "Expected 2 MCP servers");

        // Verify Supabase (Stdio via mcp-remote)
        let supabase = servers.iter().find(|s| s.name == "supabase").expect("Supabase not found");
        match &supabase.transport {
            MCPTransport::Stdio { command, args } => {
                assert_eq!(command, "npx");
                assert!(args.iter().any(|a| a.contains("mcp.supabase.com")));
            }
            _ => panic!("Supabase should use Stdio transport"),
        }

        // Verify PostHog (SSE)
        let posthog = servers.iter().find(|s| s.name == "posthog").expect("PostHog not found");
        match &posthog.transport {
            MCPTransport::Sse { url, headers } => {
                assert!(url.contains("posthog.com"));
                assert!(headers.contains_key("X-API-Key"));
            }
            _ => panic!("PostHog should use SSE transport"),
        }
    }
}
