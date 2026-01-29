//! OpenCode configuration management
//!
//! Handles configuration for the OpenCode CLI tool.
//! Config locations:
//! - User: ~/.config/opencode/opencode.json
//! - Skills: ~/.config/opencode/skill/<skill-name>/SKILL.md

use super::parsers::ConfigParser;
use super::traits::*;
use crate::cli_tools::CLIToolType;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tokio::process::Command;

/// OpenCode configuration handler
pub struct OpenCodeConfig {
    config_dir: PathBuf,
}

impl OpenCodeConfig {
    pub fn new() -> Self {
        let config_base = dirs::config_dir().unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_default()
                .join(".config")
        });
        Self {
            config_dir: config_base.join("opencode"),
        }
    }
}

impl Default for OpenCodeConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl CLIToolConfig for OpenCodeConfig {
    fn tool_type(&self) -> CLIToolType {
        CLIToolType::OpenCode
    }

    fn config_dir(&self) -> PathBuf {
        self.config_dir.clone()
    }

    async fn list_config_files(&self) -> Result<Vec<ConfigFileInfo>, ConfigError> {
        let mut files = Vec::new();

        let paths = vec![
            (
                self.config_dir.join("opencode.json"),
                ConfigFileType::Json,
                "Main configuration",
            ),
            (
                self.config_dir.join("config.json"),
                ConfigFileType::Json,
                "Alternate configuration",
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

        // Scan skill directory - OpenCode uses skill/<skill-name>/SKILL.md structure
        let skill_dir = self.config_dir.join("skill");
        if skill_dir.exists() {
            if let Ok(mut entries) = fs::read_dir(&skill_dir).await {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    let skill_path = entry.path();
                    // Each skill is a directory containing SKILL.md
                    if skill_path.is_dir() {
                        let skill_md = skill_path.join("SKILL.md");
                        if skill_md.exists() {
                            if let Ok(metadata) = fs::metadata(&skill_md).await {
                                let modified = metadata
                                    .modified()
                                    .map(|t| DateTime::<Utc>::from(t))
                                    .unwrap_or_else(|_| Utc::now());

                                let skill_name = skill_path
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("skill")
                                    .to_string();

                                files.push(ConfigFileInfo {
                                    path: skill_md.to_string_lossy().to_string(),
                                    name: format!("{}/SKILL.md", skill_name),
                                    file_type: ConfigFileType::Markdown,
                                    size_bytes: metadata.len(),
                                    modified,
                                    scope: ConfigScope::User,
                                    description: Some(format!("Skill definition: {}", skill_name)),
                                });
                            }
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
        let config_path = self.config_dir.join("opencode.json");
        let mut categories = Vec::new();

        // Load config if exists
        let config: serde_json::Value = if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .await
                .map_err(|e| ConfigError::IoError(e.to_string()))?;
            ConfigParser::parse_json(&content).unwrap_or(serde_json::Value::Object(Default::default()))
        } else {
            serde_json::Value::Object(Default::default())
        };

        let obj = config.as_object();

        // ========== Model Settings ==========
        let mut model_settings = Vec::new();

        model_settings.push(SettingDefinition {
            key: "model".to_string(),
            value: obj.and_then(|o| o.get("model")).cloned().unwrap_or(serde_json::Value::Null),
            value_type: SettingType::String,
            description: Some("Primary model to use (e.g., 'claude-sonnet-4-20250514', 'gpt-4o')".to_string()),
            default: Some(serde_json::Value::String("claude-sonnet-4-20250514".to_string())),
            options: Some(vec![
                "claude-sonnet-4-20250514".to_string(),
                "claude-opus-4-20250514".to_string(),
                "gpt-4o".to_string(),
                "gpt-4.1".to_string(),
                "o3".to_string(),
                "gemini-2.5-pro".to_string(),
            ]),
            readonly: false,
        });

        model_settings.push(SettingDefinition {
            key: "small_model".to_string(),
            value: obj.and_then(|o| o.get("small_model")).cloned().unwrap_or(serde_json::Value::Null),
            value_type: SettingType::String,
            description: Some("Smaller/faster model for simple tasks (e.g., 'claude-haiku-3-5-20241022')".to_string()),
            default: Some(serde_json::Value::String("claude-haiku-3-5-20241022".to_string())),
            options: Some(vec![
                "claude-haiku-3-5-20241022".to_string(),
                "gpt-4o-mini".to_string(),
                "gemini-2.0-flash".to_string(),
            ]),
            readonly: false,
        });

        model_settings.push(SettingDefinition {
            key: "default_agent".to_string(),
            value: obj.and_then(|o| o.get("default_agent")).cloned().unwrap_or(serde_json::Value::Null),
            value_type: SettingType::String,
            description: Some("Default agent to use for new sessions".to_string()),
            default: None,
            options: None,
            readonly: false,
        });

        categories.push(SettingsCategory {
            name: "Model".to_string(),
            description: Some("Model selection and agent configuration".to_string()),
            settings: model_settings,
        });

        // ========== UI Settings ==========
        let mut ui_settings = Vec::new();

        ui_settings.push(SettingDefinition {
            key: "theme".to_string(),
            value: obj.and_then(|o| o.get("theme")).cloned().unwrap_or(serde_json::Value::String("opencode".to_string())),
            value_type: SettingType::String,
            description: Some("Color theme for the TUI interface".to_string()),
            default: Some(serde_json::Value::String("opencode".to_string())),
            options: Some(vec![
                "opencode".to_string(),
                "catppuccin".to_string(),
                "gruvbox".to_string(),
                "tokyonight".to_string(),
            ]),
            readonly: false,
        });

        ui_settings.push(SettingDefinition {
            key: "autoupdate".to_string(),
            value: obj.and_then(|o| o.get("autoupdate")).cloned().unwrap_or(serde_json::Value::Bool(true)),
            value_type: SettingType::String,
            description: Some("Auto-update behavior: true (auto), false (disabled), or 'notify' (prompt only)".to_string()),
            default: Some(serde_json::Value::Bool(true)),
            options: Some(vec!["true".to_string(), "false".to_string(), "notify".to_string()]),
            readonly: false,
        });

        // TUI settings (nested)
        if let Some(tui) = obj.and_then(|o| o.get("tui")).and_then(|v| v.as_object()) {
            ui_settings.push(SettingDefinition {
                key: "tui.scroll_speed".to_string(),
                value: tui.get("scroll_speed").cloned().unwrap_or(serde_json::Value::Number(3.into())),
                value_type: SettingType::Number,
                description: Some("Number of lines to scroll per mouse wheel tick".to_string()),
                default: Some(serde_json::Value::Number(3.into())),
                options: None,
                readonly: false,
            });

            ui_settings.push(SettingDefinition {
                key: "tui.diff_style".to_string(),
                value: tui.get("diff_style").cloned().unwrap_or(serde_json::Value::String("auto".to_string())),
                value_type: SettingType::String,
                description: Some("How to display diffs: 'auto' (side-by-side when space allows) or 'stacked'".to_string()),
                default: Some(serde_json::Value::String("auto".to_string())),
                options: Some(vec!["auto".to_string(), "stacked".to_string()]),
                readonly: false,
            });

            if let Some(scroll_accel) = tui.get("scroll_acceleration").and_then(|v| v.as_object()) {
                ui_settings.push(SettingDefinition {
                    key: "tui.scroll_acceleration.enabled".to_string(),
                    value: scroll_accel.get("enabled").cloned().unwrap_or(serde_json::Value::Bool(false)),
                    value_type: SettingType::Boolean,
                    description: Some("Enable scroll acceleration for faster scrolling".to_string()),
                    default: Some(serde_json::Value::Bool(false)),
                    options: None,
                    readonly: false,
                });
            }
        } else {
            // Add default TUI settings even if not configured
            ui_settings.push(SettingDefinition {
                key: "tui.scroll_speed".to_string(),
                value: serde_json::Value::Number(3.into()),
                value_type: SettingType::Number,
                description: Some("Number of lines to scroll per mouse wheel tick".to_string()),
                default: Some(serde_json::Value::Number(3.into())),
                options: None,
                readonly: false,
            });

            ui_settings.push(SettingDefinition {
                key: "tui.diff_style".to_string(),
                value: serde_json::Value::String("auto".to_string()),
                value_type: SettingType::String,
                description: Some("How to display diffs: 'auto' (side-by-side when space allows) or 'stacked'".to_string()),
                default: Some(serde_json::Value::String("auto".to_string())),
                options: Some(vec!["auto".to_string(), "stacked".to_string()]),
                readonly: false,
            });
        }

        categories.push(SettingsCategory {
            name: "Interface".to_string(),
            description: Some("Theme and TUI display settings".to_string()),
            settings: ui_settings,
        });

        // ========== Provider Settings ==========
        let mut provider_settings = Vec::new();

        if let Some(providers) = obj.and_then(|o| o.get("provider")).and_then(|v| v.as_object()) {
            for (provider_name, provider_config) in providers {
                provider_settings.push(SettingDefinition {
                    key: format!("provider.{}", provider_name),
                    value: provider_config.clone(),
                    value_type: SettingType::Object,
                    description: Some(format!("Configuration for {} provider (apiKey, baseURL, timeout, etc.)", provider_name)),
                    default: None,
                    options: None,
                    readonly: false,
                });
            }
        }

        // Add supported providers info
        provider_settings.push(SettingDefinition {
            key: "disabled_providers".to_string(),
            value: obj.and_then(|o| o.get("disabled_providers")).cloned().unwrap_or(serde_json::Value::Array(vec![])),
            value_type: SettingType::Array,
            description: Some("List of provider names to disable".to_string()),
            default: Some(serde_json::Value::Array(vec![])),
            options: Some(vec![
                "anthropic".to_string(),
                "openai".to_string(),
                "google".to_string(),
                "amazon-bedrock".to_string(),
                "azure".to_string(),
                "openrouter".to_string(),
                "groq".to_string(),
                "xai".to_string(),
                "ollama".to_string(),
            ]),
            readonly: false,
        });

        provider_settings.push(SettingDefinition {
            key: "enabled_providers".to_string(),
            value: obj.and_then(|o| o.get("enabled_providers")).cloned().unwrap_or(serde_json::Value::Array(vec![])),
            value_type: SettingType::Array,
            description: Some("List of provider names to enable (empty = all enabled except disabled)".to_string()),
            default: Some(serde_json::Value::Array(vec![])),
            options: None,
            readonly: false,
        });

        if !provider_settings.is_empty() {
            categories.push(SettingsCategory {
                name: "Providers".to_string(),
                description: Some("AI provider configurations (API keys, endpoints, timeouts)".to_string()),
                settings: provider_settings,
            });
        }

        // ========== Server Settings ==========
        let mut server_settings = Vec::new();

        if let Some(server) = obj.and_then(|o| o.get("server")).and_then(|v| v.as_object()) {
            server_settings.push(SettingDefinition {
                key: "server.port".to_string(),
                value: server.get("port").cloned().unwrap_or(serde_json::Value::Number(42424.into())),
                value_type: SettingType::Number,
                description: Some("Port for the OpenCode server".to_string()),
                default: Some(serde_json::Value::Number(42424.into())),
                options: None,
                readonly: false,
            });

            server_settings.push(SettingDefinition {
                key: "server.hostname".to_string(),
                value: server.get("hostname").cloned().unwrap_or(serde_json::Value::String("localhost".to_string())),
                value_type: SettingType::String,
                description: Some("Hostname for the OpenCode server".to_string()),
                default: Some(serde_json::Value::String("localhost".to_string())),
                options: None,
                readonly: false,
            });

            server_settings.push(SettingDefinition {
                key: "server.mdns".to_string(),
                value: server.get("mdns").cloned().unwrap_or(serde_json::Value::Bool(false)),
                value_type: SettingType::Boolean,
                description: Some("Enable mDNS for local network discovery".to_string()),
                default: Some(serde_json::Value::Bool(false)),
                options: None,
                readonly: false,
            });

            server_settings.push(SettingDefinition {
                key: "server.cors".to_string(),
                value: server.get("cors").cloned().unwrap_or(serde_json::Value::Array(vec![])),
                value_type: SettingType::Array,
                description: Some("Allowed CORS origins for the server".to_string()),
                default: Some(serde_json::Value::Array(vec![])),
                options: None,
                readonly: false,
            });
        } else {
            // Add default server settings
            server_settings.push(SettingDefinition {
                key: "server.port".to_string(),
                value: serde_json::Value::Number(42424.into()),
                value_type: SettingType::Number,
                description: Some("Port for the OpenCode server".to_string()),
                default: Some(serde_json::Value::Number(42424.into())),
                options: None,
                readonly: false,
            });

            server_settings.push(SettingDefinition {
                key: "server.hostname".to_string(),
                value: serde_json::Value::String("localhost".to_string()),
                value_type: SettingType::String,
                description: Some("Hostname for the OpenCode server".to_string()),
                default: Some(serde_json::Value::String("localhost".to_string())),
                options: None,
                readonly: false,
            });
        }

        categories.push(SettingsCategory {
            name: "Server".to_string(),
            description: Some("OpenCode server configuration".to_string()),
            settings: server_settings,
        });

        // ========== Tools Settings ==========
        let mut tools_settings = Vec::new();

        let tools_obj = obj.and_then(|o| o.get("tools")).and_then(|v| v.as_object());

        tools_settings.push(SettingDefinition {
            key: "tools.write".to_string(),
            value: tools_obj.and_then(|t| t.get("write")).cloned().unwrap_or(serde_json::Value::Bool(true)),
            value_type: SettingType::Boolean,
            description: Some("Enable the write tool for file creation".to_string()),
            default: Some(serde_json::Value::Bool(true)),
            options: None,
            readonly: false,
        });

        tools_settings.push(SettingDefinition {
            key: "tools.bash".to_string(),
            value: tools_obj.and_then(|t| t.get("bash")).cloned().unwrap_or(serde_json::Value::Bool(true)),
            value_type: SettingType::Boolean,
            description: Some("Enable the bash tool for command execution".to_string()),
            default: Some(serde_json::Value::Bool(true)),
            options: None,
            readonly: false,
        });

        tools_settings.push(SettingDefinition {
            key: "tools.edit".to_string(),
            value: tools_obj.and_then(|t| t.get("edit")).cloned().unwrap_or(serde_json::Value::Bool(true)),
            value_type: SettingType::Boolean,
            description: Some("Enable the edit tool for file modifications".to_string()),
            default: Some(serde_json::Value::Bool(true)),
            options: None,
            readonly: false,
        });

        categories.push(SettingsCategory {
            name: "Tools".to_string(),
            description: Some("Enable or disable built-in tools".to_string()),
            settings: tools_settings,
        });

        // ========== Permission Settings ==========
        let mut permission_settings = Vec::new();

        if let Some(permissions) = obj.and_then(|o| o.get("permission")).and_then(|v| v.as_object()) {
            for (tool_name, permission) in permissions {
                permission_settings.push(SettingDefinition {
                    key: format!("permission.{}", tool_name),
                    value: permission.clone(),
                    value_type: SettingType::String,
                    description: Some(format!("Permission for {} tool: 'ask', 'allow', or 'deny'", tool_name)),
                    default: Some(serde_json::Value::String("ask".to_string())),
                    options: Some(vec!["ask".to_string(), "allow".to_string(), "deny".to_string()]),
                    readonly: false,
                });
            }
        }

        // Add default permission settings explanation
        if permission_settings.is_empty() {
            permission_settings.push(SettingDefinition {
                key: "permission.Bash".to_string(),
                value: serde_json::Value::String("ask".to_string()),
                value_type: SettingType::String,
                description: Some("Permission for Bash tool: 'ask', 'allow', or 'deny'".to_string()),
                default: Some(serde_json::Value::String("ask".to_string())),
                options: Some(vec!["ask".to_string(), "allow".to_string(), "deny".to_string()]),
                readonly: false,
            });

            permission_settings.push(SettingDefinition {
                key: "permission.Write".to_string(),
                value: serde_json::Value::String("ask".to_string()),
                value_type: SettingType::String,
                description: Some("Permission for Write tool: 'ask', 'allow', or 'deny'".to_string()),
                default: Some(serde_json::Value::String("ask".to_string())),
                options: Some(vec!["ask".to_string(), "allow".to_string(), "deny".to_string()]),
                readonly: false,
            });
        }

        categories.push(SettingsCategory {
            name: "Permissions".to_string(),
            description: Some("Tool permission levels (ask/allow/deny)".to_string()),
            settings: permission_settings,
        });

        // ========== Sharing Settings ==========
        let mut sharing_settings = Vec::new();

        sharing_settings.push(SettingDefinition {
            key: "share".to_string(),
            value: obj.and_then(|o| o.get("share")).cloned().unwrap_or(serde_json::Value::String("manual".to_string())),
            value_type: SettingType::String,
            description: Some("Session sharing mode: 'manual', 'auto', or 'disabled'".to_string()),
            default: Some(serde_json::Value::String("manual".to_string())),
            options: Some(vec!["manual".to_string(), "auto".to_string(), "disabled".to_string()]),
            readonly: false,
        });

        categories.push(SettingsCategory {
            name: "Sharing".to_string(),
            description: Some("Session sharing configuration".to_string()),
            settings: sharing_settings,
        });

        // ========== Compaction Settings ==========
        let mut compaction_settings = Vec::new();

        let compaction_obj = obj.and_then(|o| o.get("compaction")).and_then(|v| v.as_object());

        compaction_settings.push(SettingDefinition {
            key: "compaction.auto".to_string(),
            value: compaction_obj.and_then(|c| c.get("auto")).cloned().unwrap_or(serde_json::Value::Bool(false)),
            value_type: SettingType::Boolean,
            description: Some("Automatically compact conversation context when needed".to_string()),
            default: Some(serde_json::Value::Bool(false)),
            options: None,
            readonly: false,
        });

        compaction_settings.push(SettingDefinition {
            key: "compaction.prune".to_string(),
            value: compaction_obj.and_then(|c| c.get("prune")).cloned().unwrap_or(serde_json::Value::Bool(false)),
            value_type: SettingType::Boolean,
            description: Some("Prune old messages during compaction".to_string()),
            default: Some(serde_json::Value::Bool(false)),
            options: None,
            readonly: false,
        });

        categories.push(SettingsCategory {
            name: "Compaction".to_string(),
            description: Some("Context compaction and pruning settings".to_string()),
            settings: compaction_settings,
        });

        // ========== Watcher Settings ==========
        let mut watcher_settings = Vec::new();

        if let Some(watcher) = obj.and_then(|o| o.get("watcher")).and_then(|v| v.as_object()) {
            watcher_settings.push(SettingDefinition {
                key: "watcher.ignore".to_string(),
                value: watcher.get("ignore").cloned().unwrap_or(serde_json::Value::Array(vec![])),
                value_type: SettingType::Array,
                description: Some("Glob patterns for files/directories to ignore in file watcher".to_string()),
                default: Some(serde_json::Value::Array(vec![
                    serde_json::Value::String("**/node_modules/**".to_string()),
                    serde_json::Value::String("**/.git/**".to_string()),
                ])),
                options: None,
                readonly: false,
            });
        } else {
            watcher_settings.push(SettingDefinition {
                key: "watcher.ignore".to_string(),
                value: serde_json::Value::Array(vec![]),
                value_type: SettingType::Array,
                description: Some("Glob patterns for files/directories to ignore in file watcher".to_string()),
                default: Some(serde_json::Value::Array(vec![
                    serde_json::Value::String("**/node_modules/**".to_string()),
                    serde_json::Value::String("**/.git/**".to_string()),
                ])),
                options: None,
                readonly: false,
            });
        }

        categories.push(SettingsCategory {
            name: "File Watcher".to_string(),
            description: Some("File system watcher configuration".to_string()),
            settings: watcher_settings,
        });

        // ========== Agents (from config) ==========
        let mut agent_settings = Vec::new();

        if let Some(agents) = obj.and_then(|o| o.get("agent")).and_then(|v| v.as_object()) {
            for (agent_name, agent_config) in agents {
                agent_settings.push(SettingDefinition {
                    key: format!("agent.{}", agent_name),
                    value: agent_config.clone(),
                    value_type: SettingType::Object,
                    description: Some(format!("Custom agent definition: {}", agent_name)),
                    default: None,
                    options: None,
                    readonly: false,
                });
            }
        }

        if !agent_settings.is_empty() {
            categories.push(SettingsCategory {
                name: "Custom Agents".to_string(),
                description: Some("Custom agent definitions in config file".to_string()),
                settings: agent_settings,
            });
        }

        // ========== Commands ==========
        let mut command_settings = Vec::new();

        if let Some(commands) = obj.and_then(|o| o.get("command")).and_then(|v| v.as_object()) {
            for (cmd_name, cmd_config) in commands {
                command_settings.push(SettingDefinition {
                    key: format!("command.{}", cmd_name),
                    value: cmd_config.clone(),
                    value_type: SettingType::Object,
                    description: Some(format!("Custom command: {} (template with {{input}} placeholder)", cmd_name)),
                    default: None,
                    options: None,
                    readonly: false,
                });
            }
        }

        if !command_settings.is_empty() {
            categories.push(SettingsCategory {
                name: "Custom Commands".to_string(),
                description: Some("Custom slash commands (use {{input}} for user input)".to_string()),
                settings: command_settings,
            });
        }

        // ========== Keybinds ==========
        let mut keybind_settings = Vec::new();

        if let Some(keybinds) = obj.and_then(|o| o.get("keybinds")).and_then(|v| v.as_object()) {
            for (action, binding) in keybinds {
                keybind_settings.push(SettingDefinition {
                    key: format!("keybinds.{}", action),
                    value: binding.clone(),
                    value_type: SettingType::String,
                    description: Some(format!("Key binding for {} action", action)),
                    default: None,
                    options: None,
                    readonly: false,
                });
            }
        }

        if !keybind_settings.is_empty() {
            categories.push(SettingsCategory {
                name: "Keybinds".to_string(),
                description: Some("Custom keyboard shortcuts".to_string()),
                settings: keybind_settings,
            });
        }

        // ========== Formatters ==========
        let mut formatter_settings = Vec::new();

        if let Some(formatters) = obj.and_then(|o| o.get("formatter")).and_then(|v| v.as_object()) {
            for (fmt_name, fmt_config) in formatters {
                formatter_settings.push(SettingDefinition {
                    key: format!("formatter.{}", fmt_name),
                    value: fmt_config.clone(),
                    value_type: SettingType::Object,
                    description: Some(format!("Formatter configuration: {} (command, extensions, disabled)", fmt_name)),
                    default: None,
                    options: None,
                    readonly: false,
                });
            }
        }

        if !formatter_settings.is_empty() {
            categories.push(SettingsCategory {
                name: "Formatters".to_string(),
                description: Some("Code formatter configurations".to_string()),
                settings: formatter_settings,
            });
        }

        // ========== Plugins & Instructions ==========
        let mut plugin_settings = Vec::new();

        plugin_settings.push(SettingDefinition {
            key: "plugin".to_string(),
            value: obj.and_then(|o| o.get("plugin")).cloned().unwrap_or(serde_json::Value::Array(vec![])),
            value_type: SettingType::Array,
            description: Some("List of plugins to load".to_string()),
            default: Some(serde_json::Value::Array(vec![])),
            options: None,
            readonly: false,
        });

        plugin_settings.push(SettingDefinition {
            key: "instructions".to_string(),
            value: obj.and_then(|o| o.get("instructions")).cloned().unwrap_or(serde_json::Value::Array(vec![])),
            value_type: SettingType::Array,
            description: Some("List of instruction file paths to include in system prompt".to_string()),
            default: Some(serde_json::Value::Array(vec![])),
            options: None,
            readonly: false,
        });

        categories.push(SettingsCategory {
            name: "Plugins & Instructions".to_string(),
            description: Some("Plugin loading and custom instruction files".to_string()),
            settings: plugin_settings,
        });

        // ========== Experimental ==========
        let mut experimental_settings = Vec::new();

        if let Some(experimental) = obj.and_then(|o| o.get("experimental")).and_then(|v| v.as_object()) {
            for (key, value) in experimental {
                experimental_settings.push(SettingDefinition {
                    key: format!("experimental.{}", key),
                    value: value.clone(),
                    value_type: match value {
                        serde_json::Value::Bool(_) => SettingType::Boolean,
                        serde_json::Value::Number(_) => SettingType::Number,
                        serde_json::Value::String(_) => SettingType::String,
                        serde_json::Value::Array(_) => SettingType::Array,
                        serde_json::Value::Object(_) => SettingType::Object,
                        serde_json::Value::Null => SettingType::String,
                    },
                    description: Some(format!("Experimental feature: {}", key)),
                    default: None,
                    options: None,
                    readonly: false,
                });
            }
        }

        if !experimental_settings.is_empty() {
            categories.push(SettingsCategory {
                name: "Experimental".to_string(),
                description: Some("Experimental features (may change or be removed)".to_string()),
                settings: experimental_settings,
            });
        }

        Ok(ToolSettings {
            tool_type: CLIToolType::OpenCode,
            categories,
        })
    }

    async fn set_setting(&self, key: &str, value: serde_json::Value) -> Result<(), ConfigError> {
        let config_path = self.config_dir.join("opencode.json");

        let mut config: serde_json::Map<String, serde_json::Value> = if config_path.exists() {
            let content = fs::read_to_string(&config_path)
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

        self.write_config_file(&config_path.to_string_lossy(), &content)
            .await
    }

    async fn list_agents(&self) -> Result<Vec<AgentDefinition>, ConfigError> {
        // OpenCode uses skill/ directory with nested structure: skill/<skill-name>/SKILL.md
        let skill_dir = self.config_dir.join("skill");
        let mut agents = Vec::new();

        if !skill_dir.exists() {
            return Ok(agents);
        }

        if let Ok(mut entries) = fs::read_dir(&skill_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let skill_path = entry.path();
                // Each skill is a directory containing SKILL.md
                if skill_path.is_dir() {
                    let skill_md = skill_path.join("SKILL.md");
                    if skill_md.exists() {
                        if let Ok(agent) = self.parse_skill(&skill_path, &skill_md).await {
                            agents.push(agent);
                        }
                    }
                }
            }
        }

        Ok(agents)
    }

    async fn get_agent(&self, name: &str) -> Result<AgentDefinition, ConfigError> {
        // OpenCode uses skill/<skill-name>/SKILL.md structure
        let skill_dir = self.config_dir.join("skill").join(name);
        let skill_md = skill_dir.join("SKILL.md");

        if !skill_md.exists() {
            return Err(ConfigError::NotFound(format!("Skill '{}' not found", name)));
        }

        self.parse_skill(&skill_dir, &skill_md).await
    }

    async fn list_mcp_servers(&self) -> Result<Vec<MCPServerConfig>, ConfigError> {
        self.parse_mcp_servers().await
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

impl OpenCodeConfig {
    /// Parse a skill from its directory and SKILL.md file
    /// OpenCode skills use YAML frontmatter with:
    /// - name: skill name (optional, fallback to directory name)
    /// - description: skill description
    /// - allowed-tools: tools the skill can use (string or array)
    /// - metadata: optional metadata object
    async fn parse_skill(
        &self,
        skill_dir: &PathBuf,
        skill_md: &PathBuf,
    ) -> Result<AgentDefinition, ConfigError> {
        let content = fs::read_to_string(skill_md)
            .await
            .map_err(|e| ConfigError::IoError(e.to_string()))?;

        let parsed = ConfigParser::parse_markdown(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;

        // Get skill name from directory name (fallback)
        let dir_name = skill_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let mut name = dir_name.clone();
        let mut description = None;
        let mut model = None;
        let mut tools = Vec::new();

        // Parse frontmatter if present
        // OpenCode uses: name, description, allowed-tools (with hyphen)
        if let Some(fm) = parsed.get("frontmatter").and_then(|v| v.as_object()) {
            // Use frontmatter name if available, otherwise use directory name
            if let Some(fm_name) = fm.get("name").and_then(|v| v.as_str()) {
                name = fm_name.to_string();
            }

            description = fm
                .get("description")
                .and_then(|v| v.as_str())
                .map(String::from);

            model = fm.get("model").and_then(|v| v.as_str()).map(String::from);

            // OpenCode uses "allowed-tools" (with hyphen), can be string or array
            if let Some(allowed_tools) = fm.get("allowed-tools") {
                if let Some(tools_str) = allowed_tools.as_str() {
                    // Single tool as string - split by comma if multiple
                    tools = tools_str
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                } else if let Some(tools_array) = allowed_tools.as_array() {
                    // Array of tools
                    tools = tools_array
                        .iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect();
                }
            }

            // Also check for "tools" field (alternative naming)
            if tools.is_empty() {
                if let Some(t) = fm.get("tools").and_then(|v| v.as_array()) {
                    tools = t
                        .iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect();
                }
            }
        }

        // Body becomes the system prompt
        let system_prompt = parsed
            .get("body")
            .and_then(|v| v.as_str())
            .map(String::from);

        // If no frontmatter description, extract from content
        if description.is_none() {
            // Try to extract description from the first paragraph or heading
            if let Some(body) = &system_prompt {
                let first_line = body.lines().find(|l| !l.trim().is_empty());
                if let Some(line) = first_line {
                    let clean = line.trim_start_matches('#').trim();
                    if clean.len() < 200 {
                        description = Some(clean.to_string());
                    }
                }
            }
        }

        Ok(AgentDefinition {
            name,
            description,
            model,
            tools,
            system_prompt,
            source_file: skill_md.to_string_lossy().to_string(),
        })
    }

    /// Parse MCP servers from opencode.json config
    async fn parse_mcp_servers(&self) -> Result<Vec<MCPServerConfig>, ConfigError> {
        let config_path = self.config_dir.join("opencode.json");
        let mut servers = Vec::new();

        if !config_path.exists() {
            return Ok(servers);
        }

        let content = fs::read_to_string(&config_path)
            .await
            .map_err(|e| ConfigError::IoError(e.to_string()))?;

        let config = ConfigParser::parse_json(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;

        // OpenCode stores MCP servers in the "mcp" object
        // Format uses "type": "remote" or "type": "local" to distinguish transports
        // Local commands are arrays: "command": ["npx", "@playwright/mcp@latest"]
        if let Some(mcp_obj) = config.get("mcp").and_then(|v| v.as_object()) {
            for (name, server_config) in mcp_obj {
                if let Some(server_obj) = server_config.as_object() {
                    let server_type = server_obj
                        .get("type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");

                    let enabled = server_obj
                        .get("enabled")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true);

                    let description = server_obj
                        .get("description")
                        .and_then(|v| v.as_str())
                        .map(String::from);

                    // Check for remote (SSE) transport - uses "type": "remote"
                    if server_type == "remote" {
                        if let Some(url) = server_obj.get("url").and_then(|v| v.as_str()) {
                            let headers = server_obj
                                .get("headers")
                                .and_then(|v| v.as_object())
                                .map(|h| {
                                    h.iter()
                                        .filter_map(|(k, v)| {
                                            v.as_str().map(|s| (k.clone(), s.to_string()))
                                        })
                                        .collect()
                                })
                                .unwrap_or_default();

                            servers.push(MCPServerConfig {
                                name: name.clone(),
                                transport: MCPTransport::Sse {
                                    url: url.to_string(),
                                    headers,
                                },
                                enabled,
                                env: HashMap::new(),
                                description,
                            });
                        }
                    }
                    // Check for local (stdio) transport - uses "type": "local"
                    // Command can be an array: ["npx", "@playwright/mcp@latest"]
                    // or a string: "npx"
                    else if server_type == "local" {
                        let (command, args) = if let Some(cmd_array) =
                            server_obj.get("command").and_then(|v| v.as_array())
                        {
                            // Command is an array - first element is command, rest are args
                            let cmd = cmd_array
                                .first()
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            let args: Vec<String> = cmd_array
                                .iter()
                                .skip(1)
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect();
                            (cmd, args)
                        } else if let Some(cmd_str) =
                            server_obj.get("command").and_then(|v| v.as_str())
                        {
                            // Command is a string - args come from separate "args" field
                            let args = server_obj
                                .get("args")
                                .and_then(|v| v.as_array())
                                .map(|a| {
                                    a.iter()
                                        .filter_map(|v| v.as_str().map(String::from))
                                        .collect()
                                })
                                .unwrap_or_default();
                            (cmd_str.to_string(), args)
                        } else {
                            continue; // Skip if no valid command
                        };

                        if command.is_empty() {
                            continue; // Skip if command is empty
                        }

                        let env = server_obj
                            .get("env")
                            .and_then(|v| v.as_object())
                            .map(|e| {
                                e.iter()
                                    .filter_map(|(k, v)| {
                                        v.as_str().map(|s| (k.clone(), s.to_string()))
                                    })
                                    .collect()
                            })
                            .unwrap_or_default();

                        servers.push(MCPServerConfig {
                            name: name.clone(),
                            transport: MCPTransport::Stdio { command, args },
                            enabled,
                            env,
                            description,
                        });
                    }
                    // Fallback: try to detect type from fields (for backwards compatibility)
                    else if server_obj.contains_key("url") {
                        if let Some(url) = server_obj.get("url").and_then(|v| v.as_str()) {
                            let headers = server_obj
                                .get("headers")
                                .and_then(|v| v.as_object())
                                .map(|h| {
                                    h.iter()
                                        .filter_map(|(k, v)| {
                                            v.as_str().map(|s| (k.clone(), s.to_string()))
                                        })
                                        .collect()
                                })
                                .unwrap_or_default();

                            servers.push(MCPServerConfig {
                                name: name.clone(),
                                transport: MCPTransport::Sse {
                                    url: url.to_string(),
                                    headers,
                                },
                                enabled,
                                env: HashMap::new(),
                                description,
                            });
                        }
                    } else if server_obj.contains_key("command") {
                        let (command, args) = if let Some(cmd_array) =
                            server_obj.get("command").and_then(|v| v.as_array())
                        {
                            let cmd = cmd_array
                                .first()
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            let args: Vec<String> = cmd_array
                                .iter()
                                .skip(1)
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect();
                            (cmd, args)
                        } else if let Some(cmd_str) =
                            server_obj.get("command").and_then(|v| v.as_str())
                        {
                            let args = server_obj
                                .get("args")
                                .and_then(|v| v.as_array())
                                .map(|a| {
                                    a.iter()
                                        .filter_map(|v| v.as_str().map(String::from))
                                        .collect()
                                })
                                .unwrap_or_default();
                            (cmd_str.to_string(), args)
                        } else {
                            continue;
                        };

                        if command.is_empty() {
                            continue;
                        }

                        let env = server_obj
                            .get("env")
                            .and_then(|v| v.as_object())
                            .map(|e| {
                                e.iter()
                                    .filter_map(|(k, v)| {
                                        v.as_str().map(|s| (k.clone(), s.to_string()))
                                    })
                                    .collect()
                            })
                            .unwrap_or_default();

                        servers.push(MCPServerConfig {
                            name: name.clone(),
                            transport: MCPTransport::Stdio { command, args },
                            enabled,
                            env,
                            description,
                        });
                    }
                }
            }
        }

        Ok(servers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_mcp_config_with_local_array_command() {
        // This is the actual OpenCode MCP config format
        let config = json!({
            "mcp": {
                "playwright": {
                    "type": "local",
                    "enabled": true,
                    "command": ["npx", "@playwright/mcp@latest"]
                }
            }
        });

        let mcp_obj = config.get("mcp").unwrap().as_object().unwrap();

        for (name, server_config) in mcp_obj {
            let server_obj = server_config.as_object().unwrap();
            let server_type = server_obj.get("type").and_then(|v| v.as_str()).unwrap_or("");

            assert_eq!(name, "playwright");
            assert_eq!(server_type, "local");

            // Test command array parsing
            if let Some(cmd_array) = server_obj.get("command").and_then(|v| v.as_array()) {
                let cmd = cmd_array.first().and_then(|v| v.as_str()).unwrap();
                let args: Vec<String> = cmd_array
                    .iter()
                    .skip(1)
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();

                assert_eq!(cmd, "npx");
                assert_eq!(args, vec!["@playwright/mcp@latest"]);
            } else {
                panic!("Expected command to be an array");
            }
        }
    }

    #[test]
    fn test_parse_mcp_config_with_remote_sse() {
        let config = json!({
            "mcp": {
                "supabase": {
                    "type": "remote",
                    "enabled": true,
                    "url": "https://mcp.supabase.com/mcp?project_ref=test",
                    "headers": {
                        "Authorization": "Bearer test_token"
                    }
                }
            }
        });

        let mcp_obj = config.get("mcp").unwrap().as_object().unwrap();

        for (name, server_config) in mcp_obj {
            let server_obj = server_config.as_object().unwrap();
            let server_type = server_obj.get("type").and_then(|v| v.as_str()).unwrap_or("");

            assert_eq!(name, "supabase");
            assert_eq!(server_type, "remote");

            let url = server_obj.get("url").and_then(|v| v.as_str()).unwrap();
            assert_eq!(url, "https://mcp.supabase.com/mcp?project_ref=test");

            let headers = server_obj.get("headers").and_then(|v| v.as_object()).unwrap();
            assert_eq!(
                headers.get("Authorization").and_then(|v| v.as_str()).unwrap(),
                "Bearer test_token"
            );
        }
    }

    #[test]
    fn test_parse_skill_allowed_tools_string() {
        let frontmatter = json!({
            "name": "agent-browser",
            "description": "Browser automation skill",
            "allowed-tools": "Bash(agent-browser:*)"
        });

        let fm = frontmatter.as_object().unwrap();

        // Test parsing allowed-tools as string
        let mut tools: Vec<String> = Vec::new();
        if let Some(allowed_tools) = fm.get("allowed-tools") {
            if let Some(tools_str) = allowed_tools.as_str() {
                tools = tools_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }

        assert_eq!(tools, vec!["Bash(agent-browser:*)"]);
    }

    #[test]
    fn test_parse_skill_allowed_tools_array() {
        let frontmatter = json!({
            "name": "multi-tool-skill",
            "description": "Skill with multiple tools",
            "allowed-tools": ["Bash", "Read", "Write"]
        });

        let fm = frontmatter.as_object().unwrap();

        // Test parsing allowed-tools as array
        let mut tools: Vec<String> = Vec::new();
        if let Some(allowed_tools) = fm.get("allowed-tools") {
            if let Some(tools_array) = allowed_tools.as_array() {
                tools = tools_array
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();
            }
        }

        assert_eq!(tools, vec!["Bash", "Read", "Write"]);
    }

    #[tokio::test]
    async fn test_get_settings_creates_proper_categories() {
        use tempfile::tempdir;
        use tokio::fs;

        // Create a temporary directory with a config file
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("opencode.json");

        // Write a comprehensive test config
        let config = json!({
            "$schema": "https://opencode.ai/config.json",
            "model": "claude-sonnet-4-20250514",
            "small_model": "claude-haiku-3-5-20241022",
            "theme": "catppuccin",
            "autoupdate": true,
            "tui": {
                "scroll_speed": 5,
                "diff_style": "stacked",
                "scroll_acceleration": {
                    "enabled": true
                }
            },
            "server": {
                "port": 9000,
                "hostname": "0.0.0.0"
            },
            "tools": {
                "write": true,
                "bash": false,
                "edit": true
            },
            "permission": {
                "Bash": "deny",
                "Write": "allow"
            },
            "share": "auto",
            "compaction": {
                "auto": true,
                "prune": false
            },
            "watcher": {
                "ignore": ["**/node_modules/**", "**/dist/**"]
            },
            "agent": {
                "test-agent": {
                    "description": "Test agent",
                    "model": "gpt-4o"
                }
            },
            "command": {
                "test-cmd": {
                    "template": "Test {{input}}",
                    "description": "A test command"
                }
            },
            "plugin": ["plugin1", "plugin2"],
            "instructions": ["./INSTRUCTIONS.md"]
        });

        fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap())
            .await
            .expect("Failed to write config");

        // Create OpenCodeConfig pointing to temp dir
        let opencode_config = OpenCodeConfig {
            config_dir: temp_dir.path().to_path_buf(),
        };

        // Get settings
        let settings = opencode_config.get_settings().await.expect("Failed to get settings");

        // Verify expected categories exist
        let category_names: Vec<&str> = settings.categories.iter().map(|c| c.name.as_str()).collect();

        assert!(category_names.contains(&"Model"), "Missing Model category");
        assert!(category_names.contains(&"Interface"), "Missing Interface category");
        assert!(category_names.contains(&"Server"), "Missing Server category");
        assert!(category_names.contains(&"Tools"), "Missing Tools category");
        assert!(category_names.contains(&"Permissions"), "Missing Permissions category");
        assert!(category_names.contains(&"Sharing"), "Missing Sharing category");
        assert!(category_names.contains(&"Compaction"), "Missing Compaction category");
        assert!(category_names.contains(&"File Watcher"), "Missing File Watcher category");
        assert!(category_names.contains(&"Custom Agents"), "Missing Custom Agents category");
        assert!(category_names.contains(&"Custom Commands"), "Missing Custom Commands category");
        assert!(category_names.contains(&"Plugins & Instructions"), "Missing Plugins & Instructions category");

        // Verify specific settings values
        let model_category = settings.categories.iter().find(|c| c.name == "Model").unwrap();
        let model_setting = model_category.settings.iter().find(|s| s.key == "model").unwrap();
        assert_eq!(model_setting.value, json!("claude-sonnet-4-20250514"));

        let interface_category = settings.categories.iter().find(|c| c.name == "Interface").unwrap();
        let theme_setting = interface_category.settings.iter().find(|s| s.key == "theme").unwrap();
        assert_eq!(theme_setting.value, json!("catppuccin"));

        let tui_scroll = interface_category.settings.iter().find(|s| s.key == "tui.scroll_speed").unwrap();
        assert_eq!(tui_scroll.value, json!(5));

        let tools_category = settings.categories.iter().find(|c| c.name == "Tools").unwrap();
        let bash_setting = tools_category.settings.iter().find(|s| s.key == "tools.bash").unwrap();
        assert_eq!(bash_setting.value, json!(false));

        let permissions_category = settings.categories.iter().find(|c| c.name == "Permissions").unwrap();
        let bash_permission = permissions_category.settings.iter().find(|s| s.key == "permission.Bash").unwrap();
        assert_eq!(bash_permission.value, json!("deny"));
    }

    #[tokio::test]
    async fn test_get_settings_with_empty_config() {
        use tempfile::tempdir;
        use tokio::fs;

        let temp_dir = tempdir().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("opencode.json");

        // Write an empty config
        fs::write(&config_path, "{}")
            .await
            .expect("Failed to write config");

        let opencode_config = OpenCodeConfig {
            config_dir: temp_dir.path().to_path_buf(),
        };

        let settings = opencode_config.get_settings().await.expect("Failed to get settings");

        // Should still have basic categories with default values
        let category_names: Vec<&str> = settings.categories.iter().map(|c| c.name.as_str()).collect();

        assert!(category_names.contains(&"Model"), "Missing Model category");
        assert!(category_names.contains(&"Interface"), "Missing Interface category");
        assert!(category_names.contains(&"Server"), "Missing Server category");
        assert!(category_names.contains(&"Tools"), "Missing Tools category");
        assert!(category_names.contains(&"Permissions"), "Missing Permissions category");

        // Verify default values are used
        let model_category = settings.categories.iter().find(|c| c.name == "Model").unwrap();
        let model_setting = model_category.settings.iter().find(|s| s.key == "model").unwrap();
        assert!(model_setting.value.is_null(), "Model should be null when not configured");
        assert!(model_setting.default.is_some(), "Model should have a default value");
    }

    #[tokio::test]
    async fn test_list_mcp_servers_integration() {
        use tempfile::tempdir;
        use tokio::fs;

        let temp_dir = tempdir().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("opencode.json");

        // Write a config with multiple MCP servers (Supabase remote, Playwright local)
        let config = json!({
            "mcp": {
                "supabase": {
                    "type": "remote",
                    "enabled": true,
                    "url": "https://mcp.supabase.com/mcp?project_ref=testproject",
                    "headers": {
                        "Authorization": "Bearer test_token_abc123"
                    }
                },
                "playwright": {
                    "type": "local",
                    "enabled": true,
                    "command": ["npx", "@playwright/mcp@latest"]
                },
                "context7": {
                    "type": "local",
                    "enabled": false,
                    "command": "npx",
                    "args": ["-y", "@context7/mcp@latest"]
                }
            }
        });

        fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap())
            .await
            .expect("Failed to write config");

        let opencode_config = OpenCodeConfig {
            config_dir: temp_dir.path().to_path_buf(),
        };

        let servers = opencode_config.list_mcp_servers().await.expect("Failed to list MCP servers");

        // Should have 3 servers
        assert_eq!(servers.len(), 3, "Expected 3 MCP servers");

        // Verify Supabase (remote/SSE)
        let supabase = servers.iter().find(|s| s.name == "supabase").expect("Supabase not found");
        assert!(supabase.enabled, "Supabase should be enabled");
        match &supabase.transport {
            MCPTransport::Sse { url, headers } => {
                assert!(url.contains("mcp.supabase.com"), "URL should contain supabase.com");
                assert!(url.contains("testproject"), "URL should contain project_ref");
                assert!(headers.contains_key("Authorization"), "Should have Authorization header");
                assert!(headers.get("Authorization").unwrap().contains("Bearer"), "Should be Bearer token");
            }
            _ => panic!("Supabase should use SSE transport"),
        }

        // Verify Playwright (local with array command)
        let playwright = servers.iter().find(|s| s.name == "playwright").expect("Playwright not found");
        assert!(playwright.enabled, "Playwright should be enabled");
        match &playwright.transport {
            MCPTransport::Stdio { command, args } => {
                assert_eq!(command, "npx", "Command should be npx");
                assert_eq!(args.len(), 1, "Should have 1 arg");
                assert_eq!(args[0], "@playwright/mcp@latest", "Arg should be @playwright/mcp@latest");
            }
            _ => panic!("Playwright should use Stdio transport"),
        }

        // Verify context7 (local with string command and separate args)
        let context7 = servers.iter().find(|s| s.name == "context7").expect("Context7 not found");
        assert!(!context7.enabled, "Context7 should be disabled");
        match &context7.transport {
            MCPTransport::Stdio { command, args } => {
                assert_eq!(command, "npx", "Command should be npx");
                assert_eq!(args.len(), 2, "Should have 2 args");
                assert_eq!(args[0], "-y", "First arg should be -y");
                assert_eq!(args[1], "@context7/mcp@latest", "Second arg should be @context7/mcp@latest");
            }
            _ => panic!("Context7 should use Stdio transport"),
        }
    }
}
