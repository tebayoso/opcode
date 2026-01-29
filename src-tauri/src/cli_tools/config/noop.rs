use super::traits::{
    CLIToolConfig, CommandOutput, ConfigError, ConfigFileContent, ConfigFileInfo, ToolSettings,
};
use crate::cli_tools::CLIToolType;
use std::path::PathBuf;

pub struct NoopConfig {
    tool_type: CLIToolType,
}

impl NoopConfig {
    pub fn new(tool_type: CLIToolType) -> Self {
        Self { tool_type }
    }
}

#[async_trait::async_trait]
impl CLIToolConfig for NoopConfig {
    fn tool_type(&self) -> CLIToolType {
        self.tool_type.clone()
    }

    fn config_dir(&self) -> PathBuf {
        dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
    }

    async fn list_config_files(&self) -> Result<Vec<ConfigFileInfo>, ConfigError> {
        Ok(Vec::new())
    }

    async fn read_config_file(&self, path: &str) -> Result<ConfigFileContent, ConfigError> {
        Err(ConfigError::NotFound(path.to_string()))
    }

    async fn write_config_file(&self, _path: &str, _content: &str) -> Result<(), ConfigError> {
        Err(ConfigError::NotSupported(
            "Config write not supported".to_string(),
        ))
    }

    async fn get_settings(&self) -> Result<ToolSettings, ConfigError> {
        Ok(ToolSettings {
            tool_type: self.tool_type(),
            categories: Vec::new(),
        })
    }

    async fn set_setting(&self, _key: &str, _value: serde_json::Value) -> Result<(), ConfigError> {
        Err(ConfigError::NotSupported(
            "Settings not supported".to_string(),
        ))
    }

    async fn execute_command(
        &self,
        _command: &str,
        _args: &[&str],
    ) -> Result<CommandOutput, ConfigError> {
        Err(ConfigError::NotSupported(
            "Command execution not supported".to_string(),
        ))
    }
}
