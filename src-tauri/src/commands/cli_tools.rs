//! Tauri commands for CLI tools detection and management

use crate::cli_tools::{
    detect_all_tools, detect_tool_installations, get_config_handler, AgentDefinition,
    CLIToolInstallation, CLIToolType, CLIToolWithStatus, CLIToolsStatus, ConfigFileContent,
    ConfigFileInfo, MCPServerConfig, MCPTransport, ToolSettings,
};
use crate::commands::agents::AgentDb;
use log::{debug, error, info};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

/// List all CLI tools with their installation status
#[tauri::command]
pub async fn cli_tools_list(db: State<'_, AgentDb>) -> Result<CLIToolsStatus, String> {
    info!("Listing all CLI tools...");

    let tools = detect_all_tools();

    // Load preferred installations from database
    let mut tools_with_prefs: Vec<CLIToolWithStatus> = Vec::new();

    for mut tool in tools {
        // Try to load preferred installation from database
        if let Ok(conn) = db.0.lock() {
            if let Ok(preferred_path) = conn.query_row(
                "SELECT preferred_path FROM cli_tool_preferences WHERE tool_type = ?1",
                params![tool.tool_type.to_db_string()],
                |row| row.get::<_, String>(0),
            ) {
                // Find the installation with this path
                tool.preferred_installation = tool
                    .installations
                    .iter()
                    .find(|i| i.path == preferred_path)
                    .cloned();

                debug!(
                    "Loaded preferred installation for {:?}: {}",
                    tool.tool_type, preferred_path
                );
            }
        }

        // If no preferred installation but there are installations, use the first one
        if tool.preferred_installation.is_none() && !tool.installations.is_empty() {
            tool.preferred_installation = Some(tool.installations[0].clone());
        }

        tools_with_prefs.push(tool);
    }

    Ok(CLIToolsStatus {
        tools: tools_with_prefs,
        last_updated: chrono::Utc::now(),
    })
}

/// Get installations for a specific CLI tool
#[tauri::command]
pub async fn cli_tool_get_installations(
    tool_type: String,
) -> Result<CLIToolWithStatus, String> {
    info!("Getting installations for tool type: {}", tool_type);

    let parsed_type = CLIToolType::from_db_string(&tool_type)
        .ok_or_else(|| format!("Invalid tool type: {}", tool_type))?;

    Ok(detect_tool_installations(&parsed_type))
}

/// Set the preferred installation for a CLI tool
#[tauri::command]
pub async fn cli_tool_set_preferred(
    db: State<'_, AgentDb>,
    tool_type: String,
    preferred_path: String,
) -> Result<(), String> {
    info!(
        "Setting preferred installation for {}: {}",
        tool_type, preferred_path
    );

    let conn = db.0.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT OR REPLACE INTO cli_tool_preferences (tool_type, preferred_path, updated_at)
         VALUES (?1, ?2, CURRENT_TIMESTAMP)",
        params![tool_type, preferred_path],
    )
    .map_err(|e| {
        error!("Failed to set preferred installation: {}", e);
        e.to_string()
    })?;

    info!("Preferred installation set successfully");
    Ok(())
}

/// Get the preferred installation for a CLI tool
#[tauri::command]
pub async fn cli_tool_get_preferred(
    db: State<'_, AgentDb>,
    tool_type: String,
) -> Result<Option<CLIToolInstallation>, String> {
    info!("Getting preferred installation for: {}", tool_type);

    let parsed_type = CLIToolType::from_db_string(&tool_type)
        .ok_or_else(|| format!("Invalid tool type: {}", tool_type))?;

    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let preferred_path: Option<String> = conn
        .query_row(
            "SELECT preferred_path FROM cli_tool_preferences WHERE tool_type = ?1",
            params![tool_type],
            |row| row.get(0),
        )
        .ok();

    if let Some(path) = preferred_path {
        // Detect current installations to find the preferred one
        let status = detect_tool_installations(&parsed_type);
        let installation = status.installations.into_iter().find(|i| i.path == path);
        Ok(installation)
    } else {
        Ok(None)
    }
}

/// Force refresh CLI tools detection (clear cache and rescan)
#[tauri::command]
pub async fn cli_tools_refresh(db: State<'_, AgentDb>) -> Result<CLIToolsStatus, String> {
    info!("Refreshing CLI tools detection...");
    // For now, just re-run the detection (we can add caching later)
    cli_tools_list(db).await
}

/// Check if a specific CLI tool is available
#[tauri::command]
pub async fn cli_tool_is_available(tool_type: String) -> Result<bool, String> {
    let parsed_type = CLIToolType::from_db_string(&tool_type)
        .ok_or_else(|| format!("Invalid tool type: {}", tool_type))?;

    let status = detect_tool_installations(&parsed_type);
    Ok(status.is_installed)
}

/// Get the command to execute for a specific CLI tool
#[tauri::command]
pub async fn cli_tool_get_command(
    db: State<'_, AgentDb>,
    tool_type: String,
) -> Result<Option<String>, String> {
    info!("Getting command for tool: {}", tool_type);

    let parsed_type = CLIToolType::from_db_string(&tool_type)
        .ok_or_else(|| format!("Invalid tool type: {}", tool_type))?;

    // First try to get preferred installation
    if let Ok(Some(installation)) = cli_tool_get_preferred(db.clone(), tool_type.clone()).await {
        return Ok(Some(installation.command));
    }

    // Fall back to first available installation
    let status = detect_tool_installations(&parsed_type);
    Ok(status.installations.first().map(|i| i.command.clone()))
}

// =============================================================================
// Configuration Management Commands
// =============================================================================

/// List configuration files for a CLI tool (metadata only, lazy loading)
#[tauri::command]
pub async fn cli_tool_list_config_files(tool_type: String) -> Result<Vec<ConfigFileInfo>, String> {
    info!("Listing config files for tool: {}", tool_type);

    let parsed_type = CLIToolType::from_db_string(&tool_type)
        .ok_or_else(|| format!("Invalid tool type: {}", tool_type))?;

    let handler = get_config_handler(&parsed_type);
    handler
        .list_config_files()
        .await
        .map_err(|e| format!("Failed to list config files: {}", e))
}

/// Read a specific config file (on-demand loading)
#[tauri::command]
pub async fn cli_tool_read_config_file(
    tool_type: String,
    path: String,
) -> Result<ConfigFileContent, String> {
    info!("Reading config file for {}: {}", tool_type, path);

    let parsed_type = CLIToolType::from_db_string(&tool_type)
        .ok_or_else(|| format!("Invalid tool type: {}", tool_type))?;

    let handler = get_config_handler(&parsed_type);
    handler
        .read_config_file(&path)
        .await
        .map_err(|e| format!("Failed to read config file: {}", e))
}

/// Write a config file
#[tauri::command]
pub async fn cli_tool_write_config_file(
    tool_type: String,
    path: String,
    content: String,
) -> Result<(), String> {
    info!("Writing config file for {}: {}", tool_type, path);

    let parsed_type = CLIToolType::from_db_string(&tool_type)
        .ok_or_else(|| format!("Invalid tool type: {}", tool_type))?;

    let handler = get_config_handler(&parsed_type);
    handler
        .write_config_file(&path, &content)
        .await
        .map_err(|e| format!("Failed to write config file: {}", e))
}

/// Get structured settings for a CLI tool
#[tauri::command]
pub async fn cli_tool_get_settings(tool_type: String) -> Result<ToolSettings, String> {
    info!("Getting settings for tool: {}", tool_type);

    let parsed_type = CLIToolType::from_db_string(&tool_type)
        .ok_or_else(|| format!("Invalid tool type: {}", tool_type))?;

    let handler = get_config_handler(&parsed_type);
    handler
        .get_settings()
        .await
        .map_err(|e| format!("Failed to get settings: {}", e))
}

/// Set a specific setting for a CLI tool
#[tauri::command]
pub async fn cli_tool_set_setting(
    tool_type: String,
    key: String,
    value: serde_json::Value,
) -> Result<(), String> {
    info!("Setting {} for {}: {:?}", key, tool_type, value);

    let parsed_type = CLIToolType::from_db_string(&tool_type)
        .ok_or_else(|| format!("Invalid tool type: {}", tool_type))?;

    let handler = get_config_handler(&parsed_type);
    handler
        .set_setting(&key, value)
        .await
        .map_err(|e| format!("Failed to set setting: {}", e))
}

/// List MCP servers for a CLI tool
#[tauri::command]
pub async fn cli_tool_list_mcp_servers(tool_type: String) -> Result<Vec<MCPServerConfig>, String> {
    info!("Listing MCP servers for tool: {}", tool_type);

    let parsed_type = CLIToolType::from_db_string(&tool_type)
        .ok_or_else(|| format!("Invalid tool type: {}", tool_type))?;

    let handler = get_config_handler(&parsed_type);
    handler
        .list_mcp_servers()
        .await
        .map_err(|e| format!("Failed to list MCP servers: {}", e))
}

/// Input type for adding MCP server (to avoid complex nested deserialization)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPServerInput {
    pub name: String,
    #[serde(default)]
    pub transport_type: String, // "stdio", "sse", or "http"
    pub command: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    pub url: Option<String>,
    #[serde(default)]
    pub headers: std::collections::HashMap<String, String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
    pub description: Option<String>,
}

fn default_true() -> bool {
    true
}

impl From<MCPServerInput> for MCPServerConfig {
    fn from(input: MCPServerInput) -> Self {
        let transport = match input.transport_type.as_str() {
            "sse" => MCPTransport::Sse {
                url: input.url.unwrap_or_default(),
                headers: input.headers,
            },
            "http" => MCPTransport::Http {
                url: input.url.unwrap_or_default(),
                headers: input.headers,
            },
            _ => MCPTransport::Stdio {
                command: input.command.unwrap_or_default(),
                args: input.args,
            },
        };

        MCPServerConfig {
            name: input.name,
            transport,
            enabled: input.enabled,
            env: input.env,
            description: input.description,
        }
    }
}

/// Add an MCP server to a CLI tool
#[tauri::command]
pub async fn cli_tool_add_mcp_server(
    tool_type: String,
    config: MCPServerInput,
) -> Result<(), String> {
    info!("Adding MCP server {} to {}", config.name, tool_type);

    let parsed_type = CLIToolType::from_db_string(&tool_type)
        .ok_or_else(|| format!("Invalid tool type: {}", tool_type))?;

    let handler = get_config_handler(&parsed_type);
    let mcp_config: MCPServerConfig = config.into();

    handler
        .add_mcp_server(mcp_config)
        .await
        .map_err(|e| format!("Failed to add MCP server: {}", e))
}

/// Remove an MCP server from a CLI tool
#[tauri::command]
pub async fn cli_tool_remove_mcp_server(tool_type: String, name: String) -> Result<(), String> {
    info!("Removing MCP server {} from {}", name, tool_type);

    let parsed_type = CLIToolType::from_db_string(&tool_type)
        .ok_or_else(|| format!("Invalid tool type: {}", tool_type))?;

    let handler = get_config_handler(&parsed_type);
    handler
        .remove_mcp_server(&name)
        .await
        .map_err(|e| format!("Failed to remove MCP server: {}", e))
}

/// List agents/commands for a CLI tool
#[tauri::command]
pub async fn cli_tool_list_agents(tool_type: String) -> Result<Vec<AgentDefinition>, String> {
    info!("Listing agents for tool: {}", tool_type);

    let parsed_type = CLIToolType::from_db_string(&tool_type)
        .ok_or_else(|| format!("Invalid tool type: {}", tool_type))?;

    let handler = get_config_handler(&parsed_type);
    handler
        .list_agents()
        .await
        .map_err(|e| format!("Failed to list agents: {}", e))
}

/// Get a specific agent/command for a CLI tool
#[tauri::command]
pub async fn cli_tool_get_agent(
    tool_type: String,
    name: String,
) -> Result<AgentDefinition, String> {
    info!("Getting agent {} for tool: {}", name, tool_type);

    let parsed_type = CLIToolType::from_db_string(&tool_type)
        .ok_or_else(|| format!("Invalid tool type: {}", tool_type))?;

    let handler = get_config_handler(&parsed_type);
    handler
        .get_agent(&name)
        .await
        .map_err(|e| format!("Failed to get agent: {}", e))
}

/// Execute a CLI tool command
#[tauri::command]
pub async fn cli_tool_execute_cli_command(
    tool_type: String,
    command: String,
    args: Vec<String>,
) -> Result<crate::cli_tools::CommandOutput, String> {
    info!(
        "Executing command for {}: {} {:?}",
        tool_type, command, args
    );

    let parsed_type = CLIToolType::from_db_string(&tool_type)
        .ok_or_else(|| format!("Invalid tool type: {}", tool_type))?;

    let handler = get_config_handler(&parsed_type);
    let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    handler
        .execute_command(&command, &args_refs)
        .await
        .map_err(|e| format!("Failed to execute command: {}", e))
}

/// Get the config directory for a CLI tool
#[tauri::command]
pub async fn cli_tool_get_config_dir(tool_type: String) -> Result<String, String> {
    info!("Getting config directory for tool: {}", tool_type);

    let parsed_type = CLIToolType::from_db_string(&tool_type)
        .ok_or_else(|| format!("Invalid tool type: {}", tool_type))?;

    let handler = get_config_handler(&parsed_type);
    Ok(handler.config_dir().to_string_lossy().to_string())
}

// =============================================================================
// Usage Tracking Commands
// =============================================================================

/// Usage entry returned from the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CLIToolUsageEntry {
    pub id: i64,
    pub tool_type: String,
    pub action: String,
    pub details: Option<String>,
    pub timestamp: String,
}

/// Usage statistics for a CLI tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CLIToolUsageStats {
    pub tool_type: String,
    pub total_actions: i64,
    pub actions_by_type: std::collections::HashMap<String, i64>,
    pub recent_actions: Vec<CLIToolUsageEntry>,
    pub first_action: Option<String>,
    pub last_action: Option<String>,
}

/// Track a usage event for a CLI tool
#[tauri::command]
pub async fn cli_tool_track_usage(
    db: State<'_, AgentDb>,
    tool_type: String,
    action: String,
    details: Option<String>,
) -> Result<(), String> {
    debug!("Tracking usage for {}: {} - {:?}", tool_type, action, details);

    let conn = db.0.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO cli_tool_usage (tool_type, action, details) VALUES (?1, ?2, ?3)",
        params![tool_type, action, details],
    )
    .map_err(|e| {
        error!("Failed to track usage: {}", e);
        e.to_string()
    })?;

    Ok(())
}

/// Get usage history for a CLI tool
#[tauri::command]
pub async fn cli_tool_get_usage(
    db: State<'_, AgentDb>,
    tool_type: String,
    limit: Option<i64>,
) -> Result<Vec<CLIToolUsageEntry>, String> {
    debug!("Getting usage for {}", tool_type);

    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let limit = limit.unwrap_or(100);

    let mut stmt = conn
        .prepare(
            "SELECT id, tool_type, action, details, timestamp
             FROM cli_tool_usage
             WHERE tool_type = ?1
             ORDER BY timestamp DESC
             LIMIT ?2",
        )
        .map_err(|e| e.to_string())?;

    let entries = stmt
        .query_map(params![tool_type, limit], |row| {
            Ok(CLIToolUsageEntry {
                id: row.get(0)?,
                tool_type: row.get(1)?,
                action: row.get(2)?,
                details: row.get(3)?,
                timestamp: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(entries)
}

/// Get usage statistics for a CLI tool
#[tauri::command]
pub async fn cli_tool_get_usage_stats(
    db: State<'_, AgentDb>,
    tool_type: String,
) -> Result<CLIToolUsageStats, String> {
    debug!("Getting usage stats for {}", tool_type);

    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // Get total actions count
    let total_actions: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM cli_tool_usage WHERE tool_type = ?1",
            params![tool_type],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // Get actions by type
    let mut stmt = conn
        .prepare(
            "SELECT action, COUNT(*) as count
             FROM cli_tool_usage
             WHERE tool_type = ?1
             GROUP BY action",
        )
        .map_err(|e| e.to_string())?;

    let actions_by_type: std::collections::HashMap<String, i64> = stmt
        .query_map(params![tool_type], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    // Get recent actions (last 10)
    let mut stmt = conn
        .prepare(
            "SELECT id, tool_type, action, details, timestamp
             FROM cli_tool_usage
             WHERE tool_type = ?1
             ORDER BY timestamp DESC
             LIMIT 10",
        )
        .map_err(|e| e.to_string())?;

    let recent_actions: Vec<CLIToolUsageEntry> = stmt
        .query_map(params![tool_type], |row| {
            Ok(CLIToolUsageEntry {
                id: row.get(0)?,
                tool_type: row.get(1)?,
                details: row.get(3)?,
                action: row.get(2)?,
                timestamp: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    // Get first action timestamp
    let first_action: Option<String> = conn
        .query_row(
            "SELECT MIN(timestamp) FROM cli_tool_usage WHERE tool_type = ?1",
            params![tool_type],
            |row| row.get(0),
        )
        .ok();

    // Get last action timestamp
    let last_action: Option<String> = conn
        .query_row(
            "SELECT MAX(timestamp) FROM cli_tool_usage WHERE tool_type = ?1",
            params![tool_type],
            |row| row.get(0),
        )
        .ok();

    Ok(CLIToolUsageStats {
        tool_type,
        total_actions,
        actions_by_type,
        recent_actions,
        first_action,
        last_action,
    })
}

/// Clear usage history for a CLI tool
#[tauri::command]
pub async fn cli_tool_clear_usage(
    db: State<'_, AgentDb>,
    tool_type: String,
) -> Result<i64, String> {
    info!("Clearing usage history for {}", tool_type);

    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let deleted = conn
        .execute(
            "DELETE FROM cli_tool_usage WHERE tool_type = ?1",
            params![tool_type],
        )
        .map_err(|e| e.to_string())?;

    info!("Deleted {} usage entries for {}", deleted, tool_type);
    Ok(deleted as i64)
}
