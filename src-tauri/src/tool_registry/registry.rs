use crate::tool_registry::{
    error::RegistryError,
    types::{
        AsyncJob, InstallationConfig, JobStatus, JobType, MCPServer, MCPToolEnablement,
        SettingSchema, SystemWarning, ToolCapabilities, ToolInstallation, ToolSource,
        ToolSpecification, ToolType, ValidationResult,
    },
};
use async_trait::async_trait;
use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[async_trait]
pub trait ToolRegistry: Send + Sync {
    async fn load_tools(&self) -> Result<Vec<ToolSpecification>, RegistryError>;
    async fn get_tool(&self, tool_id: &str) -> Result<Option<ToolSpecification>, RegistryError>;
    async fn register_tool(&self, spec: ToolSpecification) -> Result<String, RegistryError>;
    async fn update_tool(&self, tool_id: &str, spec: ToolSpecification) -> Result<(), RegistryError>;
    async fn unregister_tool(&self, tool_id: &str) -> Result<(), RegistryError>;
    async fn validate_spec(&self, spec: &ToolSpecification) -> Result<(), RegistryError>;
    async fn get_tool_installation(
        &self,
        tool_id: &str,
    ) -> Result<Option<ToolInstallation>, RegistryError>;
    async fn update_tool_installation(
        &self,
        installation: ToolInstallation,
    ) -> Result<(), RegistryError>;
}

pub struct ToolRegistryImpl {
    db: Arc<Mutex<Connection>>,
    builtin_tools: HashMap<String, ToolSpecification>,
}

impl ToolRegistryImpl {
    pub fn new(db: Arc<Mutex<Connection>>) -> Result<Self, RegistryError> {
        let mut registry = Self {
            db,
            builtin_tools: HashMap::new(),
        };
        registry.load_builtin_tools();
        Ok(registry)
    }

    fn load_builtin_tools(&mut self) {
        let tools = vec![
            Self::create_claude_spec(),
            Self::create_gemini_spec(),
            Self::create_codex_spec(),
            Self::create_opencode_spec(),
            Self::create_cursor_spec(),
            Self::create_copilot_spec(),
            Self::create_eslint_spec(),
            Self::create_vite_spec(),
        ];

        for tool in tools {
            self.builtin_tools.insert(tool.id.clone(), tool);
        }
    }

    fn create_claude_spec() -> ToolSpecification {
        ToolSpecification {
            id: "claude".to_string(),
            name: "Claude Code".to_string(),
            tool_type: ToolType::Llm,
            source: ToolSource::Builtin,
            version: "1.0.0".to_string(),
            description: Some("Anthropic's AI coding assistant".to_string()),
            website: Some("https://claude.ai/code".to_string()),
            icon: None,
            installation: InstallationConfig {
                binary_names: Some(vec!["claude".to_string()]),
                homebrew_formulas: Some(vec!["claude-code".to_string()]),
                npm_packages: Some(vec!["@anthropic-ai/claude-code".to_string()]),
                nvm_packages: None,
                gh_extensions: None,
                standard_paths: Some(vec![
                    "/usr/local/bin/claude".to_string(),
                    "/opt/homebrew/bin/claude".to_string(),
                    "~/.local/bin/claude".to_string(),
                ]),
                version_args: Some(vec!["--version".to_string()]),
                version_pattern: Some(r"(\d+\.\d+\.\d+)".to_string()),
            },
            config: crate::tool_registry::types::ConfigSpec {
                base_dir: "~/.claude".to_string(),
                files: Some(vec![crate::tool_registry::types::ConfigFileSpec {
                    path: "settings.json".to_string(),
                    file_type: "json".to_string(),
                    description: Some("Claude settings".to_string()),
                    required: Some(false),
                }]),
                settings_format: Some("json".to_string()),
                settings_path: Some("settings.json".to_string()),
            },
            capabilities: ToolCapabilities {
                files: true,
                settings: true,
                mcp_servers: true,
                agents: true,
                commands: true,
                usage: true,
            },
            settings_schema: None,
        }
    }

    fn create_gemini_spec() -> ToolSpecification {
        ToolSpecification {
            id: "gemini".to_string(),
            name: "Gemini CLI".to_string(),
            tool_type: ToolType::Llm,
            source: ToolSource::Builtin,
            version: "1.0.0".to_string(),
            description: Some("Google's AI-powered CLI for code assistance".to_string()),
            website: Some("https://github.com/google-gemini/gemini-cli".to_string()),
            icon: None,
            installation: InstallationConfig {
                binary_names: Some(vec!["gemini".to_string()]),
                homebrew_formulas: None,
                npm_packages: Some(vec!["@google/gemini-cli".to_string()]),
                nvm_packages: None,
                gh_extensions: None,
                standard_paths: Some(vec![
                    "/usr/local/bin/gemini".to_string(),
                    "/opt/homebrew/bin/gemini".to_string(),
                    "~/.local/bin/gemini".to_string(),
                ]),
                version_args: Some(vec!["--version".to_string()]),
                version_pattern: Some(r"(\d+\.\d+\.\d+)".to_string()),
            },
            config: crate::tool_registry::types::ConfigSpec {
                base_dir: "~/.gemini".to_string(),
                files: Some(vec![crate::tool_registry::types::ConfigFileSpec {
                    path: "settings.json".to_string(),
                    file_type: "json".to_string(),
                    description: Some("Gemini settings".to_string()),
                    required: Some(false),
                }]),
                settings_format: Some("json".to_string()),
                settings_path: Some("settings.json".to_string()),
            },
            capabilities: ToolCapabilities {
                files: true,
                settings: true,
                mcp_servers: true,
                agents: false,
                commands: true,
                usage: true,
            },
            settings_schema: None,
        }
    }

    fn create_codex_spec() -> ToolSpecification {
        ToolSpecification {
            id: "codex".to_string(),
            name: "Codex CLI".to_string(),
            tool_type: ToolType::Llm,
            source: ToolSource::Builtin,
            version: "1.0.0".to_string(),
            description: Some("OpenAI's command-line coding assistant".to_string()),
            website: Some("https://github.com/openai/codex".to_string()),
            icon: None,
            installation: InstallationConfig {
                binary_names: Some(vec!["codex".to_string()]),
                homebrew_formulas: None,
                npm_packages: Some(vec!["@openai/codex".to_string()]),
                nvm_packages: None,
                gh_extensions: None,
                standard_paths: Some(vec![
                    "/usr/local/bin/codex".to_string(),
                    "/opt/homebrew/bin/codex".to_string(),
                    "~/.local/bin/codex".to_string(),
                ]),
                version_args: Some(vec!["--version".to_string()]),
                version_pattern: Some(r"(\d+\.\d+\.\d+)".to_string()),
            },
            config: crate::tool_registry::types::ConfigSpec {
                base_dir: "~/.config/codex".to_string(),
                files: Some(vec![crate::tool_registry::types::ConfigFileSpec {
                    path: "config.toml".to_string(),
                    file_type: "toml".to_string(),
                    description: Some("Codex configuration".to_string()),
                    required: Some(false),
                }]),
                settings_format: Some("toml".to_string()),
                settings_path: Some("config.toml".to_string()),
            },
            capabilities: ToolCapabilities {
                files: true,
                settings: true,
                mcp_servers: true,
                agents: false,
                commands: true,
                usage: true,
            },
            settings_schema: None,
        }
    }

    fn create_opencode_spec() -> ToolSpecification {
        ToolSpecification {
            id: "opencode".to_string(),
            name: "OpenCode".to_string(),
            tool_type: ToolType::Llm,
            source: ToolSource::Builtin,
            version: "1.0.0".to_string(),
            description: Some("Open-source AI coding assistant".to_string()),
            website: Some("https://github.com/opencode-ai/opencode".to_string()),
            icon: None,
            installation: InstallationConfig {
                binary_names: Some(vec!["opencode".to_string()]),
                homebrew_formulas: None,
                npm_packages: None,
                nvm_packages: None,
                gh_extensions: None,
                standard_paths: Some(vec![
                    "~/.local/bin/opencode".to_string(),
                    "~/.opencode/bin/opencode".to_string(),
                    "/usr/local/bin/opencode".to_string(),
                ]),
                version_args: Some(vec!["--version".to_string()]),
                version_pattern: Some(r"(\d+\.\d+\.\d+)".to_string()),
            },
            config: crate::tool_registry::types::ConfigSpec {
                base_dir: "~/.config/opencode".to_string(),
                files: Some(vec![crate::tool_registry::types::ConfigFileSpec {
                    path: "opencode.json".to_string(),
                    file_type: "json".to_string(),
                    description: Some("OpenCode configuration".to_string()),
                    required: Some(false),
                }]),
                settings_format: Some("json".to_string()),
                settings_path: Some("opencode.json".to_string()),
            },
            capabilities: ToolCapabilities {
                files: true,
                settings: true,
                mcp_servers: true,
                agents: true,
                commands: true,
                usage: true,
            },
            settings_schema: None,
        }
    }

    fn create_cursor_spec() -> ToolSpecification {
        ToolSpecification {
            id: "cursor".to_string(),
            name: "Cursor".to_string(),
            tool_type: ToolType::Llm,
            source: ToolSource::Builtin,
            version: "1.0.0".to_string(),
            description: Some("Cursor's AI coding agent".to_string()),
            website: Some("https://cursor.com".to_string()),
            icon: None,
            installation: InstallationConfig {
                binary_names: Some(vec!["cursor".to_string(), "cursor-agent".to_string()]),
                homebrew_formulas: Some(vec!["cursor".to_string()]),
                npm_packages: None,
                nvm_packages: None,
                gh_extensions: None,
                standard_paths: Some(vec![
                    "~/.local/bin/cursor".to_string(),
                    "/Applications/Cursor.app/Contents/MacOS/Cursor".to_string(),
                    "/usr/local/bin/cursor".to_string(),
                ]),
                version_args: Some(vec!["--version".to_string()]),
                version_pattern: Some(r"(\d+\.\d+\.\d+)".to_string()),
            },
            config: crate::tool_registry::types::ConfigSpec {
                base_dir: "~/.cursor".to_string(),
                files: Some(vec![crate::tool_registry::types::ConfigFileSpec {
                    path: "mcp.json".to_string(),
                    file_type: "json".to_string(),
                    description: Some("Cursor MCP configuration".to_string()),
                    required: Some(false),
                }]),
                settings_format: Some("json".to_string()),
                settings_path: Some("mcp.json".to_string()),
            },
            capabilities: ToolCapabilities {
                files: true,
                settings: true,
                mcp_servers: true,
                agents: false,
                commands: true,
                usage: true,
            },
            settings_schema: None,
        }
    }

    fn create_copilot_spec() -> ToolSpecification {
        ToolSpecification {
            id: "copilot".to_string(),
            name: "GitHub Copilot CLI".to_string(),
            tool_type: ToolType::Llm,
            source: ToolSource::Builtin,
            version: "1.0.0".to_string(),
            description: Some("GitHub's AI pair programmer in the terminal".to_string()),
            website: Some(
                "https://docs.github.com/en/copilot/using-github-copilot".to_string(),
            ),
            icon: None,
            installation: InstallationConfig {
                binary_names: None,
                homebrew_formulas: None,
                npm_packages: None,
                nvm_packages: None,
                gh_extensions: Some(vec!["github/gh-copilot".to_string()]),
                standard_paths: None,
                version_args: Some(vec!["--version".to_string()]),
                version_pattern: Some(r"(\d+\.\d+\.\d+)".to_string()),
            },
            config: crate::tool_registry::types::ConfigSpec {
                base_dir: "~/.config/gh".to_string(),
                files: None,
                settings_format: None,
                settings_path: None,
            },
            capabilities: ToolCapabilities {
                files: false,
                settings: false,
                mcp_servers: false,
                agents: false,
                commands: true,
                usage: true,
            },
            settings_schema: None,
        }
    }

    fn create_eslint_spec() -> ToolSpecification {
        ToolSpecification {
            id: "eslint".to_string(),
            name: "ESLint".to_string(),
            tool_type: ToolType::Codec,
            source: ToolSource::Builtin,
            version: "1.0.0".to_string(),
            description: Some("Pluggable JavaScript/TypeScript linter".to_string()),
            website: Some("https://eslint.org".to_string()),
            icon: None,
            installation: InstallationConfig {
                binary_names: Some(vec!["eslint".to_string()]),
                homebrew_formulas: None,
                npm_packages: Some(vec!["eslint".to_string()]),
                nvm_packages: None,
                gh_extensions: None,
                standard_paths: Some(vec![
                    "./node_modules/.bin/eslint".to_string(),
                    "~/.local/bin/eslint".to_string(),
                ]),
                version_args: Some(vec!["--version".to_string()]),
                version_pattern: Some(r"v(\d+\.\d+\.\d+)".to_string()),
            },
            config: crate::tool_registry::types::ConfigSpec {
                base_dir: ".".to_string(),
                files: Some(vec![
                    crate::tool_registry::types::ConfigFileSpec {
                        path: "eslint.config.js".to_string(),
                        file_type: "javascript".to_string(),
                        description: Some("ESLint flat config".to_string()),
                        required: Some(false),
                    },
                    crate::tool_registry::types::ConfigFileSpec {
                        path: ".eslintrc.json".to_string(),
                        file_type: "json".to_string(),
                        description: Some("Legacy ESLint config".to_string()),
                        required: Some(false),
                    },
                ]),
                settings_format: None,
                settings_path: None,
            },
            capabilities: ToolCapabilities {
                files: true,
                settings: true,
                mcp_servers: false,
                agents: false,
                commands: true,
                usage: true,
            },
            settings_schema: None,
        }
    }

    fn create_vite_spec() -> ToolSpecification {
        ToolSpecification {
            id: "vite".to_string(),
            name: "Vite".to_string(),
            tool_type: ToolType::Codec,
            source: ToolSource::Builtin,
            version: "1.0.0".to_string(),
            description: Some("Next generation frontend tooling".to_string()),
            website: Some("https://vitejs.dev".to_string()),
            icon: None,
            installation: InstallationConfig {
                binary_names: Some(vec!["vite".to_string()]),
                homebrew_formulas: None,
                npm_packages: Some(vec!["vite".to_string()]),
                nvm_packages: None,
                gh_extensions: None,
                standard_paths: Some(vec![
                    "./node_modules/.bin/vite".to_string(),
                    "~/.local/bin/vite".to_string(),
                ]),
                version_args: Some(vec!["--version".to_string()]),
                version_pattern: Some(r"(\d+\.\d+\.\d+)".to_string()),
            },
            config: crate::tool_registry::types::ConfigSpec {
                base_dir: ".".to_string(),
                files: Some(vec![
                    crate::tool_registry::types::ConfigFileSpec {
                        path: "vite.config.js".to_string(),
                        file_type: "javascript".to_string(),
                        description: Some("Vite configuration".to_string()),
                        required: Some(false),
                    },
                    crate::tool_registry::types::ConfigFileSpec {
                        path: "vite.config.ts".to_string(),
                        file_type: "typescript".to_string(),
                        description: Some("Vite TypeScript config".to_string()),
                        required: Some(false),
                    },
                ]),
                settings_format: None,
                settings_path: None,
            },
            capabilities: ToolCapabilities {
                files: true,
                settings: true,
                mcp_servers: false,
                agents: false,
                commands: true,
                usage: true,
            },
            settings_schema: None,
        }
    }
}

#[async_trait]
impl ToolRegistry for ToolRegistryImpl {
    async fn load_tools(&self) -> Result<Vec<ToolSpecification>, RegistryError> {
        let mut tools: Vec<ToolSpecification> = self.builtin_tools.values().cloned().collect();

        let conn = self.db.lock().map_err(|_| RegistryError::LockError)?;
        let mut stmt = conn.prepare(
            "SELECT spec_json FROM tool_specifications WHERE source = 'user_defined' AND is_enabled = 1"
        )?;

        let rows = stmt.query_map([], |row| {
            let json: String = row.get(0)?;
            Ok(json)
        })?;

        for row in rows {
            let json = row?;
            let spec: ToolSpecification = serde_json::from_str(&json)?;
            tools.push(spec);
        }

        Ok(tools)
    }

    async fn get_tool(&self, tool_id: &str) -> Result<Option<ToolSpecification>, RegistryError> {
        if let Some(tool) = self.builtin_tools.get(tool_id) {
            return Ok(Some(tool.clone()));
        }

        let conn = self.db.lock().map_err(|_| RegistryError::LockError)?;
        let mut stmt = conn.prepare("SELECT spec_json FROM tool_specifications WHERE id = ?1")?;

        let result = stmt.query_row([tool_id], |row| {
            let json: String = row.get(0)?;
            Ok(json)
        });

        match result {
            Ok(json) => {
                let spec: ToolSpecification = serde_json::from_str(&json)?;
                Ok(Some(spec))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn register_tool(&self, spec: ToolSpecification) -> Result<String, RegistryError> {
        if self.builtin_tools.contains_key(&spec.id) {
            return Err(RegistryError::ToolAlreadyExists(spec.id.clone()));
        }

        self.validate_spec(&spec).await?;

        let conn = self.db.lock().map_err(|_| RegistryError::LockError)?;
        let json = serde_json::to_string(&spec)?;

        conn.execute(
            "INSERT INTO tool_specifications (id, name, type, source, spec_json, is_enabled)
             VALUES (?1, ?2, ?3, 'user_defined', ?4, 1)
             ON CONFLICT(id) DO UPDATE SET
             name = excluded.name,
             type = excluded.type,
             spec_json = excluded.spec_json,
             updated_at = CURRENT_TIMESTAMP",
            params![
                &spec.id,
                &spec.name,
                format!("{:?}", spec.tool_type).to_lowercase(),
                &json
            ],
        )?;

        Ok(spec.id)
    }

    async fn update_tool(&self, tool_id: &str, spec: ToolSpecification) -> Result<(), RegistryError> {
        if tool_id != spec.id {
            return Err(RegistryError::InvalidSpec(
                "Tool ID mismatch".to_string(),
            ));
        }

        if self.builtin_tools.contains_key(tool_id) {
            return Err(RegistryError::InvalidSpec(
                "Cannot modify builtin tools".to_string(),
            ));
        }

        self.validate_spec(&spec).await?;

        let conn = self.db.lock().map_err(|_| RegistryError::LockError)?;
        let json = serde_json::to_string(&spec)?;

        let rows_affected = conn.execute(
            "UPDATE tool_specifications SET name = ?2, type = ?3, spec_json = ?4, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?1 AND source = 'user_defined'",
            params![
                tool_id,
                &spec.name,
                format!("{:?}", spec.tool_type).to_lowercase(),
                &json
            ],
        )?;

        if rows_affected == 0 {
            return Err(RegistryError::ToolNotFound(tool_id.to_string()));
        }

        Ok(())
    }

    async fn unregister_tool(&self, tool_id: &str) -> Result<(), RegistryError> {
        if self.builtin_tools.contains_key(tool_id) {
            return Err(RegistryError::InvalidSpec(
                "Cannot unregister builtin tools".to_string(),
            ));
        }

        let conn = self.db.lock().map_err(|_| RegistryError::LockError)?;

        let rows_affected = conn.execute(
            "DELETE FROM tool_specifications WHERE id = ?1 AND source = 'user_defined'",
            [tool_id],
        )?;

        if rows_affected == 0 {
            return Err(RegistryError::ToolNotFound(tool_id.to_string()));
        }

        Ok(())
    }

    async fn validate_spec(&self, spec: &ToolSpecification) -> Result<(), RegistryError> {
        if spec.id.is_empty() {
            return Err(RegistryError::InvalidSpec("Tool ID cannot be empty".to_string()));
        }

        if spec.name.is_empty() {
            return Err(RegistryError::InvalidSpec("Tool name cannot be empty".to_string()));
        }

        if !spec
            .id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(RegistryError::InvalidSpec(
                "Tool ID must be alphanumeric with underscores or hyphens".to_string(),
            ));
        }

        Ok(())
    }

    async fn get_tool_installation(
        &self,
        tool_id: &str,
    ) -> Result<Option<ToolInstallation>, RegistryError> {
        let conn = self.db.lock().map_err(|_| RegistryError::LockError)?;
        let mut stmt = conn.prepare(
            "SELECT tool_id, detected_paths, preferred_path, version, is_valid, last_validated, validation_errors
             FROM tool_installations WHERE tool_id = ?1"
        )?;

        let result = stmt.query_row([tool_id], |row| {
            let detected_paths_json: Option<String> = row.get(1)?;
            let validation_errors_json: Option<String> = row.get(6)?;

            let detected_paths = detected_paths_json
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default();

            let validation_errors = validation_errors_json
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default();

            Ok(ToolInstallation {
                tool_id: row.get(0)?,
                detected_paths,
                preferred_path: row.get(2)?,
                version: row.get(3)?,
                is_valid: row.get::<_, i32>(4).unwrap_or(0) != 0,
                last_validated: row.get(5)?,
                validation_errors,
            })
        });

        match result {
            Ok(installation) => Ok(Some(installation)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn update_tool_installation(
        &self,
        installation: ToolInstallation,
    ) -> Result<(), RegistryError> {
        let conn = self.db.lock().map_err(|_| RegistryError::LockError)?;

        let detected_paths_json = serde_json::to_string(&installation.detected_paths)?;
        let validation_errors_json = serde_json::to_string(&installation.validation_errors)?;

        conn.execute(
            "INSERT INTO tool_installations (tool_id, detected_paths, preferred_path, version, is_valid, last_validated, validation_errors)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(tool_id) DO UPDATE SET
             detected_paths = excluded.detected_paths,
             preferred_path = excluded.preferred_path,
             version = excluded.version,
             is_valid = excluded.is_valid,
             last_validated = excluded.last_validated,
             validation_errors = excluded.validation_errors",
            params![
                &installation.tool_id,
                &detected_paths_json,
                &installation.preferred_path,
                &installation.version,
                if installation.is_valid { 1 } else { 0 },
                &installation.last_validated,
                &validation_errors_json
            ],
        )?;

        Ok(())
    }
}
