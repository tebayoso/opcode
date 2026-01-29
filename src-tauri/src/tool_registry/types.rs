use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ToolType {
    Llm,
    Cli,
    Codec,
    Service,
    Plugin,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ToolSource {
    Builtin,
    UserDefined,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpecification {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub tool_type: ToolType,
    pub source: ToolSource,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    pub installation: InstallationConfig,
    pub config: ConfigSpec,
    pub capabilities: ToolCapabilities,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings_schema: Option<Vec<SettingSchema>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binary_names: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homebrew_formulas: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub npm_packages: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nvm_packages: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gh_extensions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub standard_paths: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_args: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_pattern: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSpec {
    pub base_dir: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<ConfigFileSpec>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFileSpec {
    pub path: String,
    #[serde(rename = "type")]
    pub file_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCapabilities {
    pub files: bool,
    pub settings: bool,
    pub mcp_servers: bool,
    pub agents: bool,
    pub commands: bool,
    pub usage: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingSchema {
    pub key: String,
    #[serde(rename = "type")]
    pub value_type: SettingValueType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<String>>,
    #[serde(default)]
    pub readonly: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SettingValueType {
    String,
    Number,
    Boolean,
    Array,
    Object,
    Select,
    Textarea,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInstallation {
    pub tool_id: String,
    pub detected_paths: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub is_valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_validated: Option<String>,
    pub validation_errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPServer {
    pub id: String,
    pub name: String,
    pub transport_type: TransportType,
    pub config: MCPTransportConfig,
    pub is_enabled_globally: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransportType {
    Stdio,
    Sse,
    Http,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPTransportConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPToolEnablement {
    pub mcp_server_id: String,
    pub tool_id: String,
    pub is_enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_override: Option<MCPTransportConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncJob {
    pub id: String,
    pub job_type: JobType,
    pub status: JobStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
    pub progress: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancelled_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobType {
    SkillsInstall,
    SkillsUninstall,
    ToolValidation,
    MCPSync,
    ConfigSync,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub id: String,
    pub tool_id: String,
    pub validation_type: ValidationType,
    pub status: ValidationStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    pub checked_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationType {
    Installation,
    Configuration,
    Functional,
    Network,
    Permissions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationStatus {
    Valid,
    Invalid,
    Warning,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemWarning {
    pub id: String,
    pub warning_type: String,
    pub severity: Severity,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    pub is_resolved: bool,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}
