//! CLI Tools configuration management module
//!
//! This module provides configuration management capabilities for all supported
//! CLI tools, including reading/writing config files, managing MCP servers,
//! and handling tool-specific agents.

pub mod parsers;
pub mod traits;

// Tool-specific config implementations
pub mod claude;
pub mod codex;
pub mod copilot;
pub mod cursor;
pub mod gemini;
pub mod opencode;

pub use parsers::{ConfigParser, ParseError};
pub use traits::{
    AgentDefinition, CLIToolConfig, CommandOutput, ConfigError, ConfigFileContent, ConfigFileInfo,
    ConfigFileType, ConfigScope, MCPServerConfig, MCPTransport, SettingDefinition, SettingType,
    SettingsCategory, ToolSettings,
};

use crate::cli_tools::CLIToolType;
use std::sync::Arc;

/// Get the configuration handler for a specific CLI tool type
pub fn get_config_handler(tool_type: &CLIToolType) -> Arc<dyn CLIToolConfig> {
    match tool_type {
        CLIToolType::Claude => Arc::new(claude::ClaudeConfig::new()),
        CLIToolType::Gemini => Arc::new(gemini::GeminiConfig::new()),
        CLIToolType::Codex => Arc::new(codex::CodexConfig::new()),
        CLIToolType::OpenCode => Arc::new(opencode::OpenCodeConfig::new()),
        CLIToolType::GitHubCopilot => Arc::new(copilot::CopilotConfig::new()),
        CLIToolType::Cursor => Arc::new(cursor::CursorConfig::new()),
    }
}
