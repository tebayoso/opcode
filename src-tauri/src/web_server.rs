use axum::extract::ws::{Message, WebSocket};
use axum::http::Method;
use axum::{
    extract::{Path, State as AxumState, WebSocketUpgrade},
    response::{Html, Json, Response},
    routing::{get, post},
    Router,
};
use chrono;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use which;

use crate::commands;
use crate::cli_tools;

// Find Claude binary for web mode - use bundled binary first
fn find_claude_binary_web() -> Result<String, String> {
    // First try the bundled binary (same location as Tauri app uses)
    let bundled_binary = "src-tauri/binaries/claude-code-x86_64-unknown-linux-gnu";
    if std::path::Path::new(bundled_binary).exists() {
        println!(
            "[find_claude_binary_web] Using bundled binary: {}",
            bundled_binary
        );
        return Ok(bundled_binary.to_string());
    }

    // Fall back to system installation paths
    let home_path = format!(
        "{}/.local/bin/claude",
        std::env::var("HOME").unwrap_or_default()
    );
    let candidates = vec![
        "claude",
        "claude-code",
        "/usr/local/bin/claude",
        "/usr/bin/claude",
        "/opt/homebrew/bin/claude",
        &home_path,
    ];

    for candidate in candidates {
        if which::which(candidate).is_ok() {
            println!(
                "[find_claude_binary_web] Using system binary: {}",
                candidate
            );
            return Ok(candidate.to_string());
        }
    }

    Err("Claude binary not found in bundled location or system paths".to_string())
}

fn web_data_dir() -> Result<PathBuf, String> {
    if let Some(base_dir) = dirs::data_dir() {
        Ok(base_dir.join("opcode"))
    } else {
        Err("Failed to determine data directory".to_string())
    }
}

fn open_web_db() -> Result<rusqlite::Connection, String> {
    let db_path = web_data_dir()?.join("agents.db");
    crate::commands::agents::init_database_at_path(db_path).map_err(|e| e.to_string())
}

#[derive(Clone)]
pub struct AppState {
    // Track active WebSocket sessions for Claude execution
    pub active_sessions:
        Arc<Mutex<std::collections::HashMap<String, tokio::sync::mpsc::Sender<String>>>>,
    // Track active Claude processes by internal session key
    pub active_processes:
        Arc<Mutex<HashMap<String, Arc<tokio::sync::Mutex<tokio::process::Child>>>>>,
    // Map Claude session IDs to internal session keys
    pub session_map: Arc<Mutex<HashMap<String, String>>>,
    // Track cancelled sessions to avoid double-completion
    pub cancelled_sessions: Arc<Mutex<HashSet<String>>>,
    // Track process metadata for running sessions
    pub process_info: Arc<Mutex<HashMap<String, crate::process::ProcessInfo>>>,
    // Auto-incrementing run ID for process info
    pub next_run_id: Arc<Mutex<i64>>,
}

#[derive(Debug, Deserialize)]
pub struct ClaudeExecutionRequest {
    pub project_path: String,
    pub prompt: String,
    pub model: Option<String>,
    pub session_id: Option<String>,
    pub command_type: String, // "execute", "continue", or "resume"
}

#[derive(Deserialize)]
pub struct QueryParams {
    #[serde(default)]
    pub project_path: Option<String>,
}

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
        }
    }
}

/// Serve the React frontend
async fn serve_frontend() -> Html<&'static str> {
    Html(include_str!("../../dist/index.html"))
}

/// API endpoint to get projects (equivalent to Tauri command)
async fn get_projects() -> Json<ApiResponse<Vec<commands::claude::Project>>> {
    match commands::claude::list_projects().await {
        Ok(projects) => Json(ApiResponse::success(projects)),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

/// API endpoint to get sessions for a project
async fn get_sessions(
    Path(project_id): Path<String>,
) -> Json<ApiResponse<Vec<commands::claude::Session>>> {
    match commands::claude::get_project_sessions(project_id).await {
        Ok(sessions) => Json(ApiResponse::success(sessions)),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

/// Simple agents endpoint - return empty for now (needs DB state)
async fn get_agents() -> Json<ApiResponse<Vec<serde_json::Value>>> {
    Json(ApiResponse::success(vec![]))
}

/// Simple usage endpoint - return empty for now
async fn get_usage() -> Json<ApiResponse<Vec<serde_json::Value>>> {
    Json(ApiResponse::success(vec![]))
}

/// Get Claude settings - return basic defaults for web mode
async fn get_claude_settings() -> Json<ApiResponse<serde_json::Value>> {
    let default_settings = serde_json::json!({
        "data": {
            "model": "claude-3-5-sonnet-20241022",
            "max_tokens": 8192,
            "temperature": 0.0,
            "auto_save": true,
            "theme": "dark"
        }
    });
    Json(ApiResponse::success(default_settings))
}

/// Check Claude version - return mock status for web mode
async fn check_claude_version() -> Json<ApiResponse<serde_json::Value>> {
    let version_status = serde_json::json!({
        "status": "ok",
        "version": "web-mode",
        "message": "Running in web server mode"
    });
    Json(ApiResponse::success(version_status))
}

/// List all available Claude installations on the system
async fn list_claude_installations(
) -> Json<ApiResponse<Vec<crate::claude_binary::ClaudeInstallation>>> {
    let installations = crate::claude_binary::discover_claude_installations();

    if installations.is_empty() {
        Json(ApiResponse::error(
            "No Claude Code installations found on the system".to_string(),
        ))
    } else {
        Json(ApiResponse::success(installations))
    }
}

/// Get system prompt - return default for web mode
async fn get_system_prompt() -> Json<ApiResponse<String>> {
    let default_prompt =
        "You are Claude, an AI assistant created by Anthropic. You are running in web server mode."
            .to_string();
    Json(ApiResponse::success(default_prompt))
}

/// Open new session - mock for web mode
async fn open_new_session() -> Json<ApiResponse<String>> {
    let session_id = format!("web-session-{}", chrono::Utc::now().timestamp());
    Json(ApiResponse::success(session_id))
}

/// List slash commands - return empty for web mode
async fn list_slash_commands() -> Json<ApiResponse<Vec<serde_json::Value>>> {
    Json(ApiResponse::success(vec![]))
}

/// MCP list servers - return empty for web mode
async fn mcp_list() -> Json<ApiResponse<Vec<serde_json::Value>>> {
    Json(ApiResponse::success(vec![]))
}

// =============================================================================
// CLI Tools (Web)
// =============================================================================

#[derive(Deserialize)]
struct SetPreferredPayload {
    path: String,
}

#[derive(Deserialize)]
struct ConfigFilePayload {
    path: String,
    content: Option<String>,
}

#[derive(Deserialize)]
struct SetSettingPayload {
    key: String,
    value: serde_json::Value,
}

#[derive(Deserialize)]
struct ExecuteCommandPayload {
    command: String,
    #[serde(default)]
    args: Vec<String>,
}

#[derive(Deserialize)]
struct UsageTrackPayload {
    action: String,
    details: Option<String>,
}

#[derive(Deserialize)]
struct UsageQuery {
    limit: Option<i64>,
}

#[derive(Deserialize)]
struct RemoveMcpPayload {
    name: String,
}

fn parse_tool_type(tool_type: &str) -> Result<cli_tools::CLIToolType, String> {
    cli_tools::CLIToolType::from_db_string(tool_type)
        .ok_or_else(|| format!("Invalid tool type: {}", tool_type))
}

async fn cli_tools_list() -> Json<ApiResponse<cli_tools::CLIToolsStatus>> {
    let mut tools = cli_tools::detect_all_tools();
    let conn = match open_web_db() {
        Ok(conn) => conn,
        Err(e) => return Json(ApiResponse::error(e)),
    };

    let mut tools_with_prefs: Vec<cli_tools::CLIToolWithStatus> = Vec::new();
    for mut tool in tools.drain(..) {
        if let Ok(preferred_path) = conn.query_row(
            "SELECT preferred_path FROM cli_tool_preferences WHERE tool_type = ?1",
            rusqlite::params![tool.tool_type.to_db_string()],
            |row| row.get::<_, String>(0),
        ) {
            tool.preferred_installation = tool
                .installations
                .iter()
                .find(|i| i.path == preferred_path)
                .cloned();
        }

        if tool.preferred_installation.is_none() && !tool.installations.is_empty() {
            tool.preferred_installation = Some(tool.installations[0].clone());
        }

        tools_with_prefs.push(tool);
    }

    Json(ApiResponse::success(cli_tools::CLIToolsStatus {
        tools: tools_with_prefs,
        last_updated: chrono::Utc::now(),
    }))
}

async fn cli_tools_refresh() -> Json<ApiResponse<cli_tools::CLIToolsStatus>> {
    cli_tools_list().await
}

async fn cli_tool_get_installations(
    Path(tool_type): Path<String>,
) -> Json<ApiResponse<cli_tools::CLIToolWithStatus>> {
    match parse_tool_type(&tool_type) {
        Ok(parsed) => Json(ApiResponse::success(cli_tools::detect_tool_installations(&parsed))),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

async fn cli_tool_set_preferred(
    Path(tool_type): Path<String>,
    axum::Json(payload): axum::Json<SetPreferredPayload>,
) -> Json<ApiResponse<()>> {
    let conn = match open_web_db() {
        Ok(conn) => conn,
        Err(e) => return Json(ApiResponse::error(e)),
    };

    if let Err(e) = conn.execute(
        "INSERT OR REPLACE INTO cli_tool_preferences (tool_type, preferred_path, updated_at)
         VALUES (?1, ?2, CURRENT_TIMESTAMP)",
        rusqlite::params![tool_type, payload.path],
    ) {
        return Json(ApiResponse::error(e.to_string()));
    }

    Json(ApiResponse::success(()))
}

async fn cli_tool_get_preferred(
    Path(tool_type): Path<String>,
) -> Json<ApiResponse<Option<cli_tools::CLIToolInstallation>>> {
    let parsed_type = match parse_tool_type(&tool_type) {
        Ok(parsed) => parsed,
        Err(e) => return Json(ApiResponse::error(e)),
    };

    let conn = match open_web_db() {
        Ok(conn) => conn,
        Err(e) => return Json(ApiResponse::error(e)),
    };

    let preferred_path: Option<String> = conn
        .query_row(
            "SELECT preferred_path FROM cli_tool_preferences WHERE tool_type = ?1",
            rusqlite::params![tool_type],
            |row| row.get(0),
        )
        .ok();

    if let Some(path) = preferred_path {
        let status = cli_tools::detect_tool_installations(&parsed_type);
        let installation = status.installations.into_iter().find(|i| i.path == path);
        return Json(ApiResponse::success(installation));
    }

    Json(ApiResponse::success(None))
}

async fn cli_tool_is_available(Path(tool_type): Path<String>) -> Json<ApiResponse<bool>> {
    match parse_tool_type(&tool_type) {
        Ok(parsed) => {
            let status = cli_tools::detect_tool_installations(&parsed);
            Json(ApiResponse::success(status.is_installed))
        }
        Err(e) => Json(ApiResponse::error(e)),
    }
}

async fn cli_tool_get_command(
    Path(tool_type): Path<String>,
) -> Json<ApiResponse<Option<String>>> {
    let parsed_type = match parse_tool_type(&tool_type) {
        Ok(parsed) => parsed,
        Err(e) => return Json(ApiResponse::error(e)),
    };

    let conn = match open_web_db() {
        Ok(conn) => conn,
        Err(e) => return Json(ApiResponse::error(e)),
    };

    let preferred_path: Option<String> = conn
        .query_row(
            "SELECT preferred_path FROM cli_tool_preferences WHERE tool_type = ?1",
            rusqlite::params![tool_type],
            |row| row.get(0),
        )
        .ok();

    if let Some(path) = preferred_path {
        let status = cli_tools::detect_tool_installations(&parsed_type);
        let installation = status.installations.into_iter().find(|i| i.path == path);
        return Json(ApiResponse::success(installation.map(|i| i.command)));
    }

    let status = cli_tools::detect_tool_installations(&parsed_type);
    Json(ApiResponse::success(
        status.installations.first().map(|i| i.command.clone()),
    ))
}

async fn cli_tool_list_config_files(
    Path(tool_type): Path<String>,
) -> Json<ApiResponse<Vec<cli_tools::ConfigFileInfo>>> {
    let parsed_type = match parse_tool_type(&tool_type) {
        Ok(parsed) => parsed,
        Err(e) => return Json(ApiResponse::error(e)),
    };

    let handler = cli_tools::get_config_handler(&parsed_type);
    match handler.list_config_files().await {
        Ok(files) => Json(ApiResponse::success(files)),
        Err(e) => Json(ApiResponse::error(format!("Failed to list config files: {}", e))),
    }
}

async fn cli_tool_read_config_file(
    Path(tool_type): Path<String>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Json<ApiResponse<cli_tools::ConfigFileContent>> {
    let path = params.get("path").cloned().unwrap_or_default();
    if path.is_empty() {
        return Json(ApiResponse::error("path is required".to_string()));
    }

    let parsed_type = match parse_tool_type(&tool_type) {
        Ok(parsed) => parsed,
        Err(e) => return Json(ApiResponse::error(e)),
    };

    let handler = cli_tools::get_config_handler(&parsed_type);
    match handler.read_config_file(&path).await {
        Ok(content) => Json(ApiResponse::success(content)),
        Err(e) => Json(ApiResponse::error(format!("Failed to read config file: {}", e))),
    }
}

async fn cli_tool_write_config_file(
    Path(tool_type): Path<String>,
    axum::Json(payload): axum::Json<ConfigFilePayload>,
) -> Json<ApiResponse<()>> {
    if payload.path.is_empty() {
        return Json(ApiResponse::error("path is required".to_string()));
    }

    let parsed_type = match parse_tool_type(&tool_type) {
        Ok(parsed) => parsed,
        Err(e) => return Json(ApiResponse::error(e)),
    };

    let handler = cli_tools::get_config_handler(&parsed_type);
    match handler
        .write_config_file(&payload.path, payload.content.as_deref().unwrap_or_default())
        .await
    {
        Ok(_) => Json(ApiResponse::success(())),
        Err(e) => Json(ApiResponse::error(format!("Failed to write config file: {}", e))),
    }
}

async fn cli_tool_get_settings(
    Path(tool_type): Path<String>,
) -> Json<ApiResponse<cli_tools::ToolSettings>> {
    let parsed_type = match parse_tool_type(&tool_type) {
        Ok(parsed) => parsed,
        Err(e) => return Json(ApiResponse::error(e)),
    };

    let handler = cli_tools::get_config_handler(&parsed_type);
    match handler.get_settings().await {
        Ok(settings) => Json(ApiResponse::success(settings)),
        Err(e) => Json(ApiResponse::error(format!("Failed to get settings: {}", e))),
    }
}

async fn cli_tool_set_setting(
    Path(tool_type): Path<String>,
    axum::Json(payload): axum::Json<SetSettingPayload>,
) -> Json<ApiResponse<()>> {
    let parsed_type = match parse_tool_type(&tool_type) {
        Ok(parsed) => parsed,
        Err(e) => return Json(ApiResponse::error(e)),
    };

    let handler = cli_tools::get_config_handler(&parsed_type);
    match handler.set_setting(&payload.key, payload.value).await {
        Ok(_) => Json(ApiResponse::success(())),
        Err(e) => Json(ApiResponse::error(format!("Failed to set setting: {}", e))),
    }
}

async fn cli_tool_list_mcp_servers(
    Path(tool_type): Path<String>,
) -> Json<ApiResponse<Vec<cli_tools::MCPServerConfig>>> {
    let parsed_type = match parse_tool_type(&tool_type) {
        Ok(parsed) => parsed,
        Err(e) => return Json(ApiResponse::error(e)),
    };

    let handler = cli_tools::get_config_handler(&parsed_type);
    match handler.list_mcp_servers().await {
        Ok(servers) => Json(ApiResponse::success(servers)),
        Err(e) => Json(ApiResponse::error(format!("Failed to list MCP servers: {}", e))),
    }
}

async fn cli_tool_add_mcp_server(
    Path(tool_type): Path<String>,
    axum::Json(config): axum::Json<crate::commands::cli_tools::MCPServerInput>,
) -> Json<ApiResponse<()>> {
    let parsed_type = match parse_tool_type(&tool_type) {
        Ok(parsed) => parsed,
        Err(e) => return Json(ApiResponse::error(e)),
    };

    let handler = cli_tools::get_config_handler(&parsed_type);
    let mcp_config: cli_tools::MCPServerConfig = config.into();
    match handler.add_mcp_server(mcp_config).await {
        Ok(_) => Json(ApiResponse::success(())),
        Err(e) => Json(ApiResponse::error(format!("Failed to add MCP server: {}", e))),
    }
}

async fn cli_tool_remove_mcp_server(
    Path(tool_type): Path<String>,
    axum::Json(payload): axum::Json<RemoveMcpPayload>,
) -> Json<ApiResponse<()>> {
    let parsed_type = match parse_tool_type(&tool_type) {
        Ok(parsed) => parsed,
        Err(e) => return Json(ApiResponse::error(e)),
    };

    let handler = cli_tools::get_config_handler(&parsed_type);
    match handler.remove_mcp_server(&payload.name).await {
        Ok(_) => Json(ApiResponse::success(())),
        Err(e) => Json(ApiResponse::error(format!("Failed to remove MCP server: {}", e))),
    }
}

async fn cli_tool_list_agents(
    Path(tool_type): Path<String>,
) -> Json<ApiResponse<Vec<cli_tools::AgentDefinition>>> {
    let parsed_type = match parse_tool_type(&tool_type) {
        Ok(parsed) => parsed,
        Err(e) => return Json(ApiResponse::error(e)),
    };

    let handler = cli_tools::get_config_handler(&parsed_type);
    match handler.list_agents().await {
        Ok(agents) => Json(ApiResponse::success(agents)),
        Err(e) => Json(ApiResponse::error(format!("Failed to list agents: {}", e))),
    }
}

async fn cli_tool_get_agent(
    Path((tool_type, name)): Path<(String, String)>,
) -> Json<ApiResponse<cli_tools::AgentDefinition>> {
    let parsed_type = match parse_tool_type(&tool_type) {
        Ok(parsed) => parsed,
        Err(e) => return Json(ApiResponse::error(e)),
    };

    let handler = cli_tools::get_config_handler(&parsed_type);
    match handler.get_agent(&name).await {
        Ok(agent) => Json(ApiResponse::success(agent)),
        Err(e) => Json(ApiResponse::error(format!("Failed to get agent: {}", e))),
    }
}

async fn cli_tool_execute_cli_command(
    Path(tool_type): Path<String>,
    axum::Json(payload): axum::Json<ExecuteCommandPayload>,
) -> Json<ApiResponse<cli_tools::CommandOutput>> {
    let parsed_type = match parse_tool_type(&tool_type) {
        Ok(parsed) => parsed,
        Err(e) => return Json(ApiResponse::error(e)),
    };

    let handler = cli_tools::get_config_handler(&parsed_type);
    let args_refs: Vec<&str> = payload.args.iter().map(|s| s.as_str()).collect();
    match handler.execute_command(&payload.command, &args_refs).await {
        Ok(output) => Json(ApiResponse::success(output)),
        Err(e) => Json(ApiResponse::error(format!("Failed to execute command: {}", e))),
    }
}

async fn cli_tool_get_config_dir(
    Path(tool_type): Path<String>,
) -> Json<ApiResponse<String>> {
    let parsed_type = match parse_tool_type(&tool_type) {
        Ok(parsed) => parsed,
        Err(e) => return Json(ApiResponse::error(e)),
    };

    let handler = cli_tools::get_config_handler(&parsed_type);
    Json(ApiResponse::success(
        handler.config_dir().to_string_lossy().to_string(),
    ))
}

async fn cli_tool_track_usage(
    Path(tool_type): Path<String>,
    axum::Json(payload): axum::Json<UsageTrackPayload>,
) -> Json<ApiResponse<()>> {
    let conn = match open_web_db() {
        Ok(conn) => conn,
        Err(e) => return Json(ApiResponse::error(e)),
    };

    if let Err(e) = conn.execute(
        "INSERT INTO cli_tool_usage (tool_type, action, details) VALUES (?1, ?2, ?3)",
        rusqlite::params![tool_type, payload.action, payload.details],
    ) {
        return Json(ApiResponse::error(e.to_string()));
    }

    Json(ApiResponse::success(()))
}

async fn cli_tool_get_usage(
    Path(tool_type): Path<String>,
    axum::extract::Query(params): axum::extract::Query<UsageQuery>,
) -> Json<ApiResponse<Vec<crate::commands::cli_tools::CLIToolUsageEntry>>> {
    let conn = match open_web_db() {
        Ok(conn) => conn,
        Err(e) => return Json(ApiResponse::error(e)),
    };

    let limit = params.limit.unwrap_or(100);
    let mut stmt = match conn.prepare(
        "SELECT id, tool_type, action, details, timestamp
         FROM cli_tool_usage
         WHERE tool_type = ?1
         ORDER BY timestamp DESC
         LIMIT ?2",
    ) {
        Ok(stmt) => stmt,
        Err(e) => return Json(ApiResponse::error(e.to_string())),
    };

    let entries = stmt
        .query_map(rusqlite::params![tool_type, limit], |row| {
            Ok(crate::commands::cli_tools::CLIToolUsageEntry {
                id: row.get(0)?,
                tool_type: row.get(1)?,
                action: row.get(2)?,
                details: row.get(3)?,
                timestamp: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string());

    match entries {
        Ok(rows) => Json(ApiResponse::success(
            rows.filter_map(|r| r.ok()).collect(),
        )),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

async fn cli_tool_get_usage_stats(
    Path(tool_type): Path<String>,
) -> Json<ApiResponse<crate::commands::cli_tools::CLIToolUsageStats>> {
    let conn = match open_web_db() {
        Ok(conn) => conn,
        Err(e) => return Json(ApiResponse::error(e)),
    };

    let total_actions: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM cli_tool_usage WHERE tool_type = ?1",
            rusqlite::params![tool_type],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let mut stmt = match conn.prepare(
        "SELECT action, COUNT(*) as count
         FROM cli_tool_usage
         WHERE tool_type = ?1
         GROUP BY action",
    ) {
        Ok(stmt) => stmt,
        Err(e) => return Json(ApiResponse::error(e.to_string())),
    };

    let actions_by_type: std::collections::HashMap<String, i64> = match stmt
        .query_map(rusqlite::params![tool_type], |row| Ok((row.get(0)?, row.get(1)?)))
    {
        Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
        Err(e) => return Json(ApiResponse::error(e.to_string())),
    };

    let mut stmt = match conn.prepare(
        "SELECT id, tool_type, action, details, timestamp
         FROM cli_tool_usage
         WHERE tool_type = ?1
         ORDER BY timestamp DESC
         LIMIT 10",
    ) {
        Ok(stmt) => stmt,
        Err(e) => return Json(ApiResponse::error(e.to_string())),
    };

    let recent_actions: Vec<crate::commands::cli_tools::CLIToolUsageEntry> = match stmt
        .query_map(rusqlite::params![tool_type], |row| {
            Ok(crate::commands::cli_tools::CLIToolUsageEntry {
                id: row.get(0)?,
                tool_type: row.get(1)?,
                details: row.get(3)?,
                action: row.get(2)?,
                timestamp: row.get(4)?,
            })
        }) {
        Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
        Err(e) => return Json(ApiResponse::error(e.to_string())),
    };

    let first_action: Option<String> = conn
        .query_row(
            "SELECT MIN(timestamp) FROM cli_tool_usage WHERE tool_type = ?1",
            rusqlite::params![tool_type],
            |row| row.get(0),
        )
        .ok();

    let last_action: Option<String> = conn
        .query_row(
            "SELECT MAX(timestamp) FROM cli_tool_usage WHERE tool_type = ?1",
            rusqlite::params![tool_type],
            |row| row.get(0),
        )
        .ok();

    Json(ApiResponse::success(
        crate::commands::cli_tools::CLIToolUsageStats {
            tool_type,
            total_actions,
            actions_by_type,
            recent_actions,
            first_action,
            last_action,
        },
    ))
}

async fn cli_tool_clear_usage(
    Path(tool_type): Path<String>,
) -> Json<ApiResponse<i64>> {
    let conn = match open_web_db() {
        Ok(conn) => conn,
        Err(e) => return Json(ApiResponse::error(e)),
    };

    match conn.execute(
        "DELETE FROM cli_tool_usage WHERE tool_type = ?1",
        rusqlite::params![tool_type],
    ) {
        Ok(deleted) => Json(ApiResponse::success(deleted as i64)),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

/// Find global config files in ~/.claude/
async fn find_global_config_files() -> Json<ApiResponse<Vec<commands::claude::GlobalConfigFile>>> {
    match commands::claude::find_global_config_files().await {
        Ok(files) => Json(ApiResponse::success(files)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

/// Find CLAUDE.md files in a project
async fn find_claude_md_files(
    axum::extract::Query(params): axum::extract::Query<QueryParams>,
) -> Json<ApiResponse<Vec<commands::claude::ClaudeMdFile>>> {
    let project_path = params.project_path.unwrap_or_default();
    if project_path.is_empty() {
        return Json(ApiResponse::error("project_path query parameter is required".to_string()));
    }
    match commands::claude::find_claude_md_files(project_path).await {
        Ok(files) => Json(ApiResponse::success(files)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

/// Read a Claude.md or config file content
async fn read_claude_md_file(
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Json<ApiResponse<String>> {
    let file_path = params.get("path").cloned().unwrap_or_default();
    if file_path.is_empty() {
        return Json(ApiResponse::error("path query parameter is required".to_string()));
    }
    match commands::claude::read_claude_md_file(file_path).await {
        Ok(content) => Json(ApiResponse::success(content)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

/// Save a Claude.md or config file content
async fn save_claude_md_file(
    axum::Json(payload): axum::Json<std::collections::HashMap<String, String>>,
) -> Json<ApiResponse<String>> {
    let file_path = payload.get("path").cloned().unwrap_or_default();
    let content = payload.get("content").cloned().unwrap_or_default();
    if file_path.is_empty() {
        return Json(ApiResponse::error("path is required".to_string()));
    }
    match commands::claude::save_claude_md_file(file_path, content).await {
        Ok(result) => Json(ApiResponse::success(result)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

/// Load session history from JSONL file
async fn load_session_history(
    Path((session_id, project_id)): Path<(String, String)>,
) -> Json<ApiResponse<Vec<serde_json::Value>>> {
    match commands::claude::load_session_history(session_id, project_id).await {
        Ok(history) => Json(ApiResponse::success(history)),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

/// List running Claude sessions
async fn list_running_claude_sessions(
    AxumState(state): AxumState<AppState>,
) -> Json<ApiResponse<Vec<crate::process::ProcessInfo>>> {
    let sessions: Vec<crate::process::ProcessInfo> = state
        .process_info
        .lock()
        .await
        .values()
        .cloned()
        .collect();
    Json(ApiResponse::success(sessions))
}

/// Execute Claude code - mock for web mode
async fn execute_claude_code() -> Json<ApiResponse<serde_json::Value>> {
    Json(ApiResponse::error("Claude execution is not available in web mode. Please use the desktop app for running Claude commands.".to_string()))
}

/// Continue Claude code - mock for web mode
async fn continue_claude_code() -> Json<ApiResponse<serde_json::Value>> {
    Json(ApiResponse::error("Claude execution is not available in web mode. Please use the desktop app for running Claude commands.".to_string()))
}

/// Resume Claude code - mock for web mode  
async fn resume_claude_code() -> Json<ApiResponse<serde_json::Value>> {
    Json(ApiResponse::error("Claude execution is not available in web mode. Please use the desktop app for running Claude commands.".to_string()))
}

/// Cancel Claude execution
async fn cancel_claude_execution(
    Path(session_id): Path<String>,
    AxumState(state): AxumState<AppState>,
) -> Json<ApiResponse<()>> {
    println!("[TRACE] Cancel request for session: {}", session_id);

    let session_key = {
        let map = state.session_map.lock().await;
        map.get(&session_id).cloned().unwrap_or_else(|| session_id.clone())
    };

    state
        .cancelled_sessions
        .lock()
        .await
        .insert(session_key.clone());

    if let Some(child_handle) = state.active_processes.lock().await.remove(&session_key) {
        let mut child = child_handle.lock().await;
        let _ = child.kill().await;
    }
    state.process_info.lock().await.remove(&session_key);

    // Notify client of cancellation
    send_to_session(
        &state,
        &session_key,
        json!({
            "type": "cancelled",
            "message": "Execution cancelled"
        })
        .to_string(),
    )
    .await;

    // Clean up session map
    state.session_map.lock().await.remove(&session_id);

    Json(ApiResponse::success(()))
}

/// Get Claude session output
async fn get_claude_session_output(Path(session_id): Path<String>) -> Json<ApiResponse<String>> {
    // In web mode, output is streamed via WebSocket, not stored
    println!("[TRACE] Output request for session: {}", session_id);
    Json(ApiResponse::success(
        "Output available via WebSocket only".to_string(),
    ))
}

/// WebSocket handler for Claude execution with streaming output
async fn claude_websocket(ws: WebSocketUpgrade, AxumState(state): AxumState<AppState>) -> Response {
    ws.on_upgrade(move |socket| claude_websocket_handler(socket, state))
}

async fn claude_websocket_handler(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let session_key = uuid::Uuid::new_v4().to_string();

    println!(
        "[TRACE] WebSocket handler started - session_key: {}",
        session_key
    );

    // Channel for sending output to WebSocket
    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(100);

    // Store session in state
    {
        let mut sessions = state.active_sessions.lock().await;
        sessions.insert(session_key.clone(), tx);
        println!(
            "[TRACE] Session stored in state - active sessions count: {}",
            sessions.len()
        );
    }

    // Task to forward channel messages to WebSocket
    let session_id_for_forward = session_key.clone();
    let forward_task = tokio::spawn(async move {
        println!(
            "[TRACE] Forward task started for session {}",
            session_id_for_forward
        );
        while let Some(message) = rx.recv().await {
            println!("[TRACE] Forwarding message to WebSocket: {}", message);
            if sender.send(Message::Text(message.into())).await.is_err() {
                println!("[TRACE] Failed to send message to WebSocket - connection closed");
                break;
            }
        }
        println!(
            "[TRACE] Forward task ended for session {}",
            session_id_for_forward
        );
    });

    // Handle incoming messages from WebSocket
    println!("[TRACE] Starting to listen for WebSocket messages");
    while let Some(msg) = receiver.next().await {
        println!("[TRACE] Received WebSocket message: {:?}", msg);
        if let Ok(msg) = msg {
            if let Message::Text(text) = msg {
                println!(
                    "[TRACE] WebSocket text message received - length: {} chars",
                    text.len()
                );
                println!("[TRACE] WebSocket message content: {}", text);
                match serde_json::from_str::<ClaudeExecutionRequest>(&text) {
                    Ok(request) => {
                        println!("[TRACE] Successfully parsed request: {:?}", request);
                        println!("[TRACE] Command type: {}", request.command_type);
                        println!("[TRACE] Project path: {}", request.project_path);
                        println!("[TRACE] Prompt length: {} chars", request.prompt.len());

                        // Execute Claude command based on request type
                        let session_id_clone = session_key.clone();
                        let state_clone = state.clone();

                        println!(
                            "[TRACE] Spawning task to execute command: {}",
                            request.command_type
                        );

                        if let Some(ref claude_session_id) = request.session_id {
                            if !claude_session_id.is_empty() {
                                let mut map = state_clone.session_map.lock().await;
                                map.insert(claude_session_id.clone(), session_id_clone.clone());
                            }
                        }

                        tokio::spawn(async move {
                            println!("[TRACE] Task started for command execution");
                            let result = match request.command_type.as_str() {
                                "execute" => {
                                    println!("[TRACE] Calling execute_claude_command");
                                    execute_claude_command(
                                        request.project_path,
                                        request.prompt,
                                        request.model.unwrap_or_default(),
                                        session_id_clone.clone(),
                                        state_clone.clone(),
                                    )
                                    .await
                                }
                                "continue" => {
                                    println!("[TRACE] Calling continue_claude_command");
                                    continue_claude_command(
                                        request.project_path,
                                        request.prompt,
                                        request.model.unwrap_or_default(),
                                        session_id_clone.clone(),
                                        state_clone.clone(),
                                    )
                                    .await
                                }
                                "resume" => {
                                    println!("[TRACE] Calling resume_claude_command");
                                    resume_claude_command(
                                        request.project_path,
                                        request.session_id.unwrap_or_default(),
                                        request.prompt,
                                        request.model.unwrap_or_default(),
                                        session_id_clone.clone(),
                                        state_clone.clone(),
                                    )
                                    .await
                                }
                                _ => {
                                    println!(
                                        "[TRACE] Unknown command type: {}",
                                        request.command_type
                                    );
                                    Err("Unknown command type".to_string())
                                }
                            };

                            println!(
                                "[TRACE] Command execution finished with result: {:?}",
                                result
                            );

                            // Send completion message
                            let cancelled = {
                                let cancelled_sessions =
                                    state_clone.cancelled_sessions.lock().await;
                                cancelled_sessions.contains(&session_id_clone)
                            };

                            if let Some(sender) = state_clone
                                .active_sessions
                                .lock()
                                .await
                                .get(&session_id_clone)
                            {
                                let completion_msg = if cancelled {
                                    json!({
                                        "type": "completion",
                                        "status": "cancelled"
                                    })
                                } else {
                                    match result {
                                        Ok(_) => json!({
                                            "type": "completion",
                                            "status": "success"
                                        }),
                                        Err(e) => json!({
                                            "type": "completion",
                                            "status": "error",
                                            "error": e
                                        }),
                                    }
                                };
                                println!("[TRACE] Sending completion message: {}", completion_msg);
                                let _ = sender.send(completion_msg.to_string()).await;
                            } else {
                                println!(
                                    "[TRACE] Session not found in active sessions when sending completion"
                                );
                            }

                            state_clone
                                .cancelled_sessions
                                .lock()
                                .await
                                .remove(&session_id_clone);
                        });
                    }
                    Err(e) => {
                        println!("[TRACE] Failed to parse WebSocket request: {}", e);
                        println!("[TRACE] Raw message that failed to parse: {}", text);

                        // Send error back to client
                        let error_msg = json!({
                            "type": "error",
                            "message": format!("Failed to parse request: {}", e)
                        });
                        if let Some(sender_tx) =
                            state.active_sessions.lock().await.get(&session_key)
                        {
                            let _ = sender_tx.send(error_msg.to_string()).await;
                        }
                    }
                }
            } else if let Message::Close(_) = msg {
                println!("[TRACE] WebSocket close message received");
                break;
            } else {
                println!("[TRACE] Non-text WebSocket message received: {:?}", msg);
            }
        } else {
            println!("[TRACE] Error receiving WebSocket message");
        }
    }

    println!("[TRACE] WebSocket message loop ended");

    // Clean up session
    {
        let mut sessions = state.active_sessions.lock().await;
        sessions.remove(&session_key);
        println!(
            "[TRACE] Session {} removed from state - remaining sessions: {}",
            session_key,
            sessions.len()
        );
    }

    if let Some(child_handle) = state.active_processes.lock().await.remove(&session_key) {
        let mut child = child_handle.lock().await;
        let _ = child.kill().await;
    }

    let mut session_map = state.session_map.lock().await;
    session_map.retain(|_, value| value != &session_key);
    state.process_info.lock().await.remove(&session_key);
    state.cancelled_sessions.lock().await.remove(&session_key);

    forward_task.abort();
    println!("[TRACE] WebSocket handler ended for session {}", session_key);
}

// Claude command execution functions for WebSocket streaming
async fn execute_claude_command(
    project_path: String,
    prompt: String,
    model: String,
    session_id: String,
    state: AppState,
) -> Result<(), String> {
    use tokio::io::{AsyncBufReadExt, BufReader};
    use tokio::process::Command;

    println!("[TRACE] execute_claude_command called:");
    println!("[TRACE]   project_path: {}", project_path);
    println!("[TRACE]   prompt length: {} chars", prompt.len());
    println!("[TRACE]   model: {}", model);
    println!("[TRACE]   session_id: {}", session_id);

    // Send initial message
    println!("[TRACE] Sending initial start message");
    send_to_session(
        &state,
        &session_id,
        json!({
            "type": "start",
            "message": "Starting Claude execution..."
        })
        .to_string(),
    )
    .await;

    // Find Claude binary (simplified for web mode)
    println!("[TRACE] Finding Claude binary...");
    let claude_path = find_claude_binary_web().map_err(|e| {
        let error = format!("Claude binary not found: {}", e);
        println!("[TRACE] Error finding Claude binary: {}", error);
        error
    })?;
    println!("[TRACE] Found Claude binary: {}", claude_path);

    // Create Claude command
    println!("[TRACE] Creating Claude command...");
    let mut cmd = Command::new(&claude_path);
    let args = [
        "-p",
        &prompt,
        "--model",
        &model,
        "--output-format",
        "stream-json",
        "--verbose",
        "--dangerously-skip-permissions",
    ];
    cmd.args(args);
    cmd.current_dir(&project_path);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    println!(
        "[TRACE] Command: {} {:?} (in dir: {})",
        claude_path, args, project_path
    );

    // Spawn Claude process
    println!("[TRACE] Spawning Claude process...");
    let mut child = cmd.spawn().map_err(|e| {
        let error = format!("Failed to spawn Claude: {}", e);
        println!("[TRACE] Spawn error: {}", error);
        error
    })?;
    println!("[TRACE] Claude process spawned successfully");

    // Get stdout/stderr for streaming
    let stdout = child.stdout.take().ok_or_else(|| {
        println!("[TRACE] Failed to get stdout from child process");
        "Failed to get stdout".to_string()
    })?;
    let stderr = child.stderr.take().ok_or_else(|| {
        println!("[TRACE] Failed to get stderr from child process");
        "Failed to get stderr".to_string()
    })?;
    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    let child_handle = Arc::new(tokio::sync::Mutex::new(child));
    let run_id = next_run_id(&state).await;
    let pid = child_handle.lock().await.id().unwrap_or(0);
    let initial_session_id = session_id.clone();
    state.process_info.lock().await.insert(
        session_id.clone(),
        crate::process::ProcessInfo {
            run_id,
            process_type: crate::process::ProcessType::ClaudeSession {
                session_id: initial_session_id,
            },
            pid,
            started_at: chrono::Utc::now(),
            project_path: project_path.clone(),
            task: prompt.clone(),
            model: model.clone(),
        },
    );
    state
        .active_processes
        .lock()
        .await
        .insert(session_id.clone(), child_handle.clone());

    println!("[TRACE] Starting to read Claude output...");
    let session_id_for_stdout = session_id.clone();
    let state_for_stdout = state.clone();
    let stdout_task = tokio::spawn(async move {
        let mut lines = stdout_reader.lines();
        let mut line_count = 0;
        while let Ok(Some(line)) = lines.next_line().await {
            line_count += 1;
            println!("[TRACE] Claude output line {}: {}", line_count, line);

            if let Ok(msg) = serde_json::from_str::<serde_json::Value>(&line) {
                if msg["type"] == "system" && msg["subtype"] == "init" {
                    if let Some(claude_session_id) = msg["session_id"].as_str() {
                        let mut map = state_for_stdout.session_map.lock().await;
                        map.entry(claude_session_id.to_string())
                            .or_insert_with(|| session_id_for_stdout.clone());

                        let mut info_map = state_for_stdout.process_info.lock().await;
                        if let Some(info) = info_map.get_mut(&session_id_for_stdout) {
                            info.process_type = crate::process::ProcessType::ClaudeSession {
                                session_id: claude_session_id.to_string(),
                            };
                        }
                    }
                }
            }

            let message = json!({
                "type": "output",
                "content": line
            })
            .to_string();
            println!("[TRACE] Sending output message to session: {}", message);
            send_to_session(&state_for_stdout, &session_id_for_stdout, message).await;
        }

        println!(
            "[TRACE] Finished reading Claude output ({} lines total)",
            line_count
        );
    });

    let session_id_for_stderr = session_id.clone();
    let state_for_stderr = state.clone();
    let stderr_task = tokio::spawn(async move {
        let mut lines = stderr_reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            println!("[TRACE] Claude stderr: {}", line);
            let message = json!({
                "type": "stderr",
                "content": line
            })
            .to_string();
            send_to_session(&state_for_stderr, &session_id_for_stderr, message).await;
        }
    });

    // Wait for process to complete
    println!("[TRACE] Waiting for Claude process to complete...");
    let exit_status = {
        let mut child = child_handle.lock().await;
        child.wait().await
    }
    .map_err(|e| {
        let error = format!("Failed to wait for Claude: {}", e);
        println!("[TRACE] Wait error: {}", error);
        error
    })?;

    let _ = stdout_task.await;
    let _ = stderr_task.await;

    println!(
        "[TRACE] Claude process completed with status: {:?}",
        exit_status
    );

    state.active_processes.lock().await.remove(&session_id);
    state.process_info.lock().await.remove(&session_id);

    if !exit_status.success() {
        let error = format!(
            "Claude execution failed with exit code: {:?}",
            exit_status.code()
        );
        println!("[TRACE] Claude execution failed: {}", error);
        return Err(error);
    }

    println!("[TRACE] execute_claude_command completed successfully");
    Ok(())
}

async fn continue_claude_command(
    project_path: String,
    prompt: String,
    model: String,
    session_id: String,
    state: AppState,
) -> Result<(), String> {
    use tokio::io::{AsyncBufReadExt, BufReader};
    use tokio::process::Command;

    send_to_session(
        &state,
        &session_id,
        json!({
            "type": "start",
            "message": "Continuing Claude session..."
        })
        .to_string(),
    )
    .await;

    // Find Claude binary
    let claude_path =
        find_claude_binary_web().map_err(|e| format!("Claude binary not found: {}", e))?;

    // Create continue command
    let mut cmd = Command::new(&claude_path);
    cmd.args([
        "-c", // Continue flag
        "-p",
        &prompt,
        "--model",
        &model,
        "--output-format",
        "stream-json",
        "--verbose",
        "--dangerously-skip-permissions",
    ]);
    cmd.current_dir(&project_path);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    // Spawn and stream output
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn Claude: {}", e))?;
    let stdout = child.stdout.take().ok_or("Failed to get stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to get stderr")?;
    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    let child_handle = Arc::new(tokio::sync::Mutex::new(child));
    let run_id = next_run_id(&state).await;
    let pid = child_handle.lock().await.id().unwrap_or(0);
    state.process_info.lock().await.insert(
        session_id.clone(),
        crate::process::ProcessInfo {
            run_id,
            process_type: crate::process::ProcessType::ClaudeSession {
                session_id: session_id.clone(),
            },
            pid,
            started_at: chrono::Utc::now(),
            project_path: project_path.clone(),
            task: prompt.clone(),
            model: model.clone(),
        },
    );
    state
        .active_processes
        .lock()
        .await
        .insert(session_id.clone(), child_handle.clone());

    let session_id_for_stdout = session_id.clone();
    let state_for_stdout = state.clone();
    let stdout_task = tokio::spawn(async move {
        let mut lines = stdout_reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if let Ok(msg) = serde_json::from_str::<serde_json::Value>(&line) {
                if msg["type"] == "system" && msg["subtype"] == "init" {
                    if let Some(claude_session_id) = msg["session_id"].as_str() {
                        let mut map = state_for_stdout.session_map.lock().await;
                        map.entry(claude_session_id.to_string())
                            .or_insert_with(|| session_id_for_stdout.clone());

                        let mut info_map = state_for_stdout.process_info.lock().await;
                        if let Some(info) = info_map.get_mut(&session_id_for_stdout) {
                            info.process_type = crate::process::ProcessType::ClaudeSession {
                                session_id: claude_session_id.to_string(),
                            };
                        }
                    }
                }
            }

            send_to_session(
                &state_for_stdout,
                &session_id_for_stdout,
                json!({
                    "type": "output",
                    "content": line
                })
                .to_string(),
            )
            .await;
        }
    });

    let session_id_for_stderr = session_id.clone();
    let state_for_stderr = state.clone();
    let stderr_task = tokio::spawn(async move {
        let mut lines = stderr_reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            send_to_session(
                &state_for_stderr,
                &session_id_for_stderr,
                json!({
                    "type": "stderr",
                    "content": line
                })
                .to_string(),
            )
            .await;
        }
    });

    let exit_status = {
        let mut child = child_handle.lock().await;
        child.wait().await
    }
    .map_err(|e| format!("Failed to wait for Claude: {}", e))?;

    let _ = stdout_task.await;
    let _ = stderr_task.await;

    state.active_processes.lock().await.remove(&session_id);
    state.process_info.lock().await.remove(&session_id);

    if !exit_status.success() {
        return Err(format!(
            "Claude execution failed with exit code: {:?}",
            exit_status.code()
        ));
    }

    Ok(())
}

async fn resume_claude_command(
    project_path: String,
    claude_session_id: String,
    prompt: String,
    model: String,
    session_id: String,
    state: AppState,
) -> Result<(), String> {
    use tokio::io::{AsyncBufReadExt, BufReader};
    use tokio::process::Command;

    println!("[resume_claude_command] Starting with project_path: {}, claude_session_id: {}, prompt: {}, model: {}", 
             project_path, claude_session_id, prompt, model);

    send_to_session(
        &state,
        &session_id,
        json!({
            "type": "start",
            "message": "Resuming Claude session..."
        })
        .to_string(),
    )
    .await;

    // Find Claude binary
    println!("[resume_claude_command] Finding Claude binary...");
    let claude_path =
        find_claude_binary_web().map_err(|e| format!("Claude binary not found: {}", e))?;
    println!(
        "[resume_claude_command] Found Claude binary: {}",
        claude_path
    );

    // Create resume command
    println!("[resume_claude_command] Creating command...");
    let mut cmd = Command::new(&claude_path);
    let args = [
        "--resume",
        &claude_session_id,
        "-p",
        &prompt,
        "--model",
        &model,
        "--output-format",
        "stream-json",
        "--verbose",
        "--dangerously-skip-permissions",
    ];
    cmd.args(args);
    cmd.current_dir(&project_path);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    println!(
        "[resume_claude_command] Command: {} {:?} (in dir: {})",
        claude_path, args, project_path
    );

    // Spawn and stream output
    println!("[resume_claude_command] Spawning process...");
    let mut child = cmd.spawn().map_err(|e| {
        let error = format!("Failed to spawn Claude: {}", e);
        println!("[resume_claude_command] Spawn error: {}", error);
        error
    })?;
    println!("[resume_claude_command] Process spawned successfully");
    let stdout = child.stdout.take().ok_or("Failed to get stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to get stderr")?;
    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    let child_handle = Arc::new(tokio::sync::Mutex::new(child));
    let run_id = next_run_id(&state).await;
    let pid = child_handle.lock().await.id().unwrap_or(0);
    state.process_info.lock().await.insert(
        session_id.clone(),
        crate::process::ProcessInfo {
            run_id,
            process_type: crate::process::ProcessType::ClaudeSession {
                session_id: session_id.clone(),
            },
            pid,
            started_at: chrono::Utc::now(),
            project_path: project_path.clone(),
            task: prompt.clone(),
            model: model.clone(),
        },
    );
    state
        .active_processes
        .lock()
        .await
        .insert(session_id.clone(), child_handle.clone());

    let session_id_for_stdout = session_id.clone();
    let state_for_stdout = state.clone();
    let stdout_task = tokio::spawn(async move {
        let mut lines = stdout_reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if let Ok(msg) = serde_json::from_str::<serde_json::Value>(&line) {
                if msg["type"] == "system" && msg["subtype"] == "init" {
                    if let Some(claude_session_id) = msg["session_id"].as_str() {
                        let mut map = state_for_stdout.session_map.lock().await;
                        map.entry(claude_session_id.to_string())
                            .or_insert_with(|| session_id_for_stdout.clone());

                        let mut info_map = state_for_stdout.process_info.lock().await;
                        if let Some(info) = info_map.get_mut(&session_id_for_stdout) {
                            info.process_type = crate::process::ProcessType::ClaudeSession {
                                session_id: claude_session_id.to_string(),
                            };
                        }
                    }
                }
            }

            send_to_session(
                &state_for_stdout,
                &session_id_for_stdout,
                json!({
                    "type": "output",
                    "content": line
                })
                .to_string(),
            )
            .await;
        }
    });

    let session_id_for_stderr = session_id.clone();
    let state_for_stderr = state.clone();
    let stderr_task = tokio::spawn(async move {
        let mut lines = stderr_reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            send_to_session(
                &state_for_stderr,
                &session_id_for_stderr,
                json!({
                    "type": "stderr",
                    "content": line
                })
                .to_string(),
            )
            .await;
        }
    });

    let exit_status = {
        let mut child = child_handle.lock().await;
        child.wait().await
    }
    .map_err(|e| format!("Failed to wait for Claude: {}", e))?;

    let _ = stdout_task.await;
    let _ = stderr_task.await;

    state.active_processes.lock().await.remove(&session_id);
    state.process_info.lock().await.remove(&session_id);

    if !exit_status.success() {
        return Err(format!(
            "Claude execution failed with exit code: {:?}",
            exit_status.code()
        ));
    }

    Ok(())
}

async fn send_to_session(state: &AppState, session_id: &str, message: String) {
    println!("[TRACE] send_to_session called for session: {}", session_id);
    println!("[TRACE] Message: {}", message);

    let sessions = state.active_sessions.lock().await;
    if let Some(sender) = sessions.get(session_id) {
        println!("[TRACE] Found session in active sessions, sending message...");
        match sender.send(message).await {
            Ok(_) => println!("[TRACE] Message sent successfully"),
            Err(e) => println!("[TRACE] Failed to send message: {}", e),
        }
    } else {
        println!(
            "[TRACE] Session {} not found in active sessions",
            session_id
        );
        println!(
            "[TRACE] Active sessions: {:?}",
            sessions.keys().collect::<Vec<_>>()
        );
    }
}

async fn next_run_id(state: &AppState) -> i64 {
    let mut next_id = state.next_run_id.lock().await;
    let current = *next_id;
    *next_id += 1;
    current
}

/// Create the web server
pub async fn create_web_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let state = AppState {
        active_sessions: Arc::new(Mutex::new(std::collections::HashMap::new())),
        active_processes: Arc::new(Mutex::new(HashMap::new())),
        session_map: Arc::new(Mutex::new(HashMap::new())),
        cancelled_sessions: Arc::new(Mutex::new(HashSet::new())),
        process_info: Arc::new(Mutex::new(HashMap::new())),
        next_run_id: Arc::new(Mutex::new(1)),
    };

    // CORS layer to allow requests from phone browsers
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any);

    // Create router with API endpoints
    let app = Router::new()
        // Frontend routes
        .route("/", get(serve_frontend))
        .route("/index.html", get(serve_frontend))
        // API routes (REST API equivalent of Tauri commands)
        .route("/api/projects", get(get_projects))
        .route("/api/projects/{project_id}/sessions", get(get_sessions))
        .route("/api/agents", get(get_agents))
        .route("/api/usage", get(get_usage))
        // Settings and configuration
        .route("/api/settings/claude", get(get_claude_settings))
        .route("/api/settings/claude/version", get(check_claude_version))
        .route(
            "/api/settings/claude/installations",
            get(list_claude_installations),
        )
        .route("/api/settings/system-prompt", get(get_system_prompt))
        // Session management
        .route("/api/sessions/new", get(open_new_session))
        // Slash commands
        .route("/api/slash-commands", get(list_slash_commands))
        // MCP
        .route("/api/mcp/servers", get(mcp_list))
        // CLI Tools
        .route("/api/cli-tools", get(cli_tools_list))
        .route("/api/cli-tools/refresh", get(cli_tools_refresh))
        .route(
            "/api/cli-tools/{toolType}/installations",
            get(cli_tool_get_installations),
        )
        .route(
            "/api/cli-tools/{toolType}/preferred",
            get(cli_tool_get_preferred).post(cli_tool_set_preferred),
        )
        .route(
            "/api/cli-tools/{toolType}/available",
            get(cli_tool_is_available),
        )
        .route("/api/cli-tools/{toolType}/command", get(cli_tool_get_command))
        .route(
            "/api/cli-tools/{toolType}/config/files",
            get(cli_tool_list_config_files),
        )
        .route(
            "/api/cli-tools/{toolType}/config/file",
            get(cli_tool_read_config_file).post(cli_tool_write_config_file),
        )
        .route(
            "/api/cli-tools/{toolType}/settings",
            get(cli_tool_get_settings).post(cli_tool_set_setting),
        )
        .route(
            "/api/cli-tools/{toolType}/mcp",
            get(cli_tool_list_mcp_servers).post(cli_tool_add_mcp_server),
        )
        .route(
            "/api/cli-tools/{toolType}/mcp/remove",
            post(cli_tool_remove_mcp_server),
        )
        .route(
            "/api/cli-tools/{toolType}/agents",
            get(cli_tool_list_agents),
        )
        .route(
            "/api/cli-tools/{toolType}/agents/{name}",
            get(cli_tool_get_agent),
        )
        .route(
            "/api/cli-tools/{toolType}/execute",
            post(cli_tool_execute_cli_command),
        )
        .route(
            "/api/cli-tools/{toolType}/config-dir",
            get(cli_tool_get_config_dir),
        )
        .route(
            "/api/cli-tools/{toolType}/usage/track",
            post(cli_tool_track_usage),
        )
        .route(
            "/api/cli-tools/{toolType}/usage",
            get(cli_tool_get_usage),
        )
        .route(
            "/api/cli-tools/{toolType}/usage/stats",
            get(cli_tool_get_usage_stats),
        )
        .route(
            "/api/cli-tools/{toolType}/usage/clear",
            post(cli_tool_clear_usage),
        )
        // Config files / Memories
        .route("/api/config/global", get(find_global_config_files))
        .route("/api/config/project", get(find_claude_md_files))
        .route("/api/config/file", get(read_claude_md_file))
        .route("/api/config/file", axum::routing::post(save_claude_md_file))
        // Session history
        .route(
            "/api/sessions/{session_id}/history/{project_id}",
            get(load_session_history),
        )
        .route("/api/sessions/running", get(list_running_claude_sessions))
        // Claude execution endpoints (read-only in web mode)
        .route("/api/sessions/execute", get(execute_claude_code))
        .route("/api/sessions/continue", get(continue_claude_code))
        .route("/api/sessions/resume", get(resume_claude_code))
        .route(
            "/api/sessions/{sessionId}/cancel",
            get(cancel_claude_execution),
        )
        .route(
            "/api/sessions/{sessionId}/output",
            get(get_claude_session_output),
        )
        // WebSocket endpoint for real-time Claude execution
        .route("/ws/claude", get(claude_websocket))
        // Serve static assets
        .nest_service("/assets", ServeDir::new("../dist/assets"))
        .nest_service("/vite.svg", ServeDir::new("../dist/vite.svg"))
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("🌐 Web server running on http://0.0.0.0:{}", port);
    println!("📱 Access from phone: http://YOUR_PC_IP:{}", port);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Start web server mode (alternative to Tauri GUI)
pub async fn start_web_mode(port: Option<u16>) -> Result<(), Box<dyn std::error::Error>> {
    let port = port.unwrap_or(8080);

    println!("🚀 Starting Opcode in web server mode...");
    create_web_server(port).await
}
