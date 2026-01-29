//! Core traits and types for CLI tool configuration management

use crate::cli_tools::CLIToolType;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Error types for configuration operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigError {
    /// Configuration file not found
    NotFound(String),
    /// Permission denied
    PermissionDenied(String),
    /// Parse error
    ParseError(String),
    /// Write error
    WriteError(String),
    /// Tool not installed
    NotInstalled,
    /// Feature not supported by this tool
    NotSupported(String),
    /// Command execution error
    CommandError(String),
    /// I/O error
    IoError(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::NotFound(path) => write!(f, "Configuration not found: {}", path),
            ConfigError::PermissionDenied(path) => write!(f, "Permission denied: {}", path),
            ConfigError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ConfigError::WriteError(msg) => write!(f, "Write error: {}", msg),
            ConfigError::NotInstalled => write!(f, "Tool not installed"),
            ConfigError::NotSupported(feature) => write!(f, "Feature not supported: {}", feature),
            ConfigError::CommandError(msg) => write!(f, "Command error: {}", msg),
            ConfigError::IoError(msg) => write!(f, "I/O error: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        ConfigError::IoError(err.to_string())
    }
}

/// Type of configuration file
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConfigFileType {
    Json,
    Jsonc,
    Toml,
    Yaml,
    Markdown,
}

#[allow(dead_code)]
impl ConfigFileType {
    /// Get file type from extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "json" => Some(ConfigFileType::Json),
            "jsonc" => Some(ConfigFileType::Jsonc),
            "toml" => Some(ConfigFileType::Toml),
            "yaml" | "yml" => Some(ConfigFileType::Yaml),
            "md" | "markdown" => Some(ConfigFileType::Markdown),
            _ => None,
        }
    }

    /// Get the typical file extension for this type
    #[allow(dead_code)]
    pub fn extension(&self) -> &'static str {
        match self {
            ConfigFileType::Json => "json",
            ConfigFileType::Jsonc => "jsonc",
            ConfigFileType::Toml => "toml",
            ConfigFileType::Yaml => "yaml",
            ConfigFileType::Markdown => "md",
        }
    }
}

/// Scope of configuration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConfigScope {
    /// User-level configuration (~/.config/ or ~/.<tool>/)
    User,
    /// Project-level configuration (./<tool>/ in project)
    Project,
    /// System-level configuration (/etc/ or managed settings)
    System,
}

/// Metadata about a configuration file (for lazy loading)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFileInfo {
    /// Relative path from config directory
    pub path: String,
    /// Display name for the file
    pub name: String,
    /// Type of configuration file
    pub file_type: ConfigFileType,
    /// File size in bytes
    pub size_bytes: u64,
    /// Last modification time
    pub modified: DateTime<Utc>,
    /// Configuration scope
    pub scope: ConfigScope,
    /// Optional description of what this file configures
    pub description: Option<String>,
}

/// Content of a configuration file (loaded on demand)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFileContent {
    /// File path
    pub path: String,
    /// Raw content as string
    pub content: String,
    /// Parsed content as JSON (normalized from any format)
    pub parsed: Option<serde_json::Value>,
    /// Original file type
    pub file_type: ConfigFileType,
}

/// Type of a setting value
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SettingType {
    String,
    Number,
    Boolean,
    Array,
    Object,
    Enum,
}

/// Definition of a single setting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingDefinition {
    /// Setting key/path
    pub key: String,
    /// Current value
    pub value: serde_json::Value,
    /// Value type
    pub value_type: SettingType,
    /// Human-readable description
    pub description: Option<String>,
    /// Default value
    pub default: Option<serde_json::Value>,
    /// Available options for enum types
    pub options: Option<Vec<String>>,
    /// Whether this setting is read-only
    pub readonly: bool,
}

/// Category of settings for grouping in UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsCategory {
    /// Category name
    pub name: String,
    /// Category description
    pub description: Option<String>,
    /// Settings in this category
    pub settings: Vec<SettingDefinition>,
}

/// Normalized settings structure for a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSettings {
    /// Tool type
    pub tool_type: CLIToolType,
    /// Settings grouped by category
    pub categories: Vec<SettingsCategory>,
}

/// MCP transport configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MCPTransport {
    /// Standard I/O transport
    Stdio {
        command: String,
        args: Vec<String>,
    },
    /// Server-sent events transport
    Sse {
        url: String,
        #[serde(default)]
        headers: HashMap<String, String>,
    },
    /// HTTP transport
    Http {
        url: String,
        #[serde(default)]
        headers: HashMap<String, String>,
    },
}

/// MCP server configuration (normalized across tools)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPServerConfig {
    /// Server name
    pub name: String,
    /// Transport configuration
    pub transport: MCPTransport,
    /// Whether the server is enabled
    pub enabled: bool,
    /// Environment variables
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// Optional description
    pub description: Option<String>,
}

/// Agent/command definition (normalized across tools)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDefinition {
    /// Agent name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Model to use (if configurable)
    pub model: Option<String>,
    /// Available tools/capabilities
    #[serde(default)]
    pub tools: Vec<String>,
    /// System prompt
    pub system_prompt: Option<String>,
    /// Source file path
    pub source_file: String,
}

/// Output from command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandOutput {
    /// Exit code
    pub exit_code: i32,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Whether the command succeeded
    pub success: bool,
}

/// Core trait for CLI tool configuration management
///
/// This trait defines the interface for reading and writing configuration
/// for each supported CLI tool. Implementations handle tool-specific
/// config file formats and locations.
#[async_trait::async_trait]
pub trait CLIToolConfig: Send + Sync {
    /// Get the tool type this config handler is for
    fn tool_type(&self) -> CLIToolType;

    /// Get the base configuration directory for this tool
    fn config_dir(&self) -> PathBuf;

    /// List all configuration files (lazy - metadata only)
    async fn list_config_files(&self) -> Result<Vec<ConfigFileInfo>, ConfigError>;

    /// Read a specific configuration file (on demand)
    async fn read_config_file(&self, path: &str) -> Result<ConfigFileContent, ConfigError>;

    /// Write content to a configuration file
    async fn write_config_file(&self, path: &str, content: &str) -> Result<(), ConfigError>;

    /// Get structured settings for this tool
    async fn get_settings(&self) -> Result<ToolSettings, ConfigError>;

    /// Update a specific setting
    async fn set_setting(&self, key: &str, value: serde_json::Value) -> Result<(), ConfigError>;

    /// List MCP servers (if supported)
    async fn list_mcp_servers(&self) -> Result<Vec<MCPServerConfig>, ConfigError> {
        Err(ConfigError::NotSupported(
            "MCP servers not supported".to_string(),
        ))
    }

    /// Add an MCP server (if supported)
    async fn add_mcp_server(&self, _config: MCPServerConfig) -> Result<(), ConfigError> {
        Err(ConfigError::NotSupported(
            "MCP servers not supported".to_string(),
        ))
    }

    /// Remove an MCP server (if supported)
    async fn remove_mcp_server(&self, _name: &str) -> Result<(), ConfigError> {
        Err(ConfigError::NotSupported(
            "MCP servers not supported".to_string(),
        ))
    }

    /// List agents/commands (if supported)
    async fn list_agents(&self) -> Result<Vec<AgentDefinition>, ConfigError> {
        Err(ConfigError::NotSupported("Agents not supported".to_string()))
    }

    /// Get a specific agent by name (if supported)
    async fn get_agent(&self, _name: &str) -> Result<AgentDefinition, ConfigError> {
        Err(ConfigError::NotSupported("Agents not supported".to_string()))
    }

    /// Execute a CLI command (on demand)
    async fn execute_command(
        &self,
        command: &str,
        args: &[&str],
    ) -> Result<CommandOutput, ConfigError>;
}
