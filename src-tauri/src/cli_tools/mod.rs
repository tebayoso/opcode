//! CLI Tools detection and management module
//!
//! This module provides functionality to detect, manage, and interact with
//! various CLI code agents including Claude, Gemini, Codex, OpenCode,
//! GitHub Copilot, and Cursor.

pub mod config;
pub mod detector;
pub mod registry;
pub mod types;

pub use detector::{detect_all_tools, detect_tool_installations};
pub use registry::{get_all_tool_definitions, get_tool_definition};
pub use types::{
    CLIToolInstallation, CLIToolType, CLIToolWithStatus, CLIToolsStatus, DetectionConfig,
    InstallationSource,
};

// Re-export config types
pub use config::{
    get_config_handler, AgentDefinition, CLIToolConfig, CommandOutput, ConfigError,
    ConfigFileContent, ConfigFileInfo, ConfigFileType, ConfigScope, MCPServerConfig, MCPTransport,
    SettingDefinition, SettingType, SettingsCategory, ToolSettings,
};
