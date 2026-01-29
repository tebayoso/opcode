use crate::tool_registry::{
    JobManager, JobManagerImpl, JobType, MCPRegistry, MCPRegistryImpl, MCPTransportConfig,
    SkillInfo, SkillsCLI, ToolInstallation, ToolRegistry, ToolRegistryImpl, ToolSpecification,
    TransportType, ValidationEngine, ValidationEngineImpl,
};
use crate::commands::agents::AgentDb;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub data: T,
}

#[derive(Serialize)]
pub struct ApiError {
    pub error: ErrorDetail,
}

#[derive(Serialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct ToolSummary {
    pub id: String,
    pub name: String,
    pub tool_type: String,
    pub source: String,
    pub is_installed: bool,
    pub capabilities: crate::tool_registry::ToolCapabilities,
}

#[derive(Deserialize)]
pub struct RegisterToolRequest {
    pub spec: ToolSpecification,
}

#[derive(Serialize)]
pub struct RegisterToolResponse {
    pub tool_id: String,
    pub message: String,
}

#[derive(Deserialize)]
pub struct UpdateToolRequest {
    pub spec: ToolSpecification,
}

#[derive(Serialize)]
pub struct ToolDetailResponse {
    pub tool: ToolSpecification,
    pub installation: Option<ToolInstallation>,
}

fn create_registry(db: State<'_, AgentDb>) -> Result<ToolRegistryImpl, String> {
    let db_arc = Arc::new(db.0.clone());
    ToolRegistryImpl::new(db_arc).map_err(|e| e.to_string())
}

fn create_job_manager(db: State<'_, AgentDb>) -> Result<JobManagerImpl, String> {
    let db_arc = Arc::new(db.0.clone());
    JobManagerImpl::new(db_arc).map_err(|e| e.to_string())
}

fn create_mcp_registry(db: State<'_, AgentDb>) -> Result<MCPRegistryImpl, String> {
    let db_arc = Arc::new(db.0.clone());
    MCPRegistryImpl::new(db_arc).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn tool_registry_list_tools(
    db: State<'_, AgentDb>,
) -> Result<ApiResponse<Vec<ToolSummary>>, String> {
    let registry = create_registry(db)?;
    let tools = registry.load_tools().await.map_err(|e| e.to_string())?;

    let mut summaries = Vec::new();
    for tool in tools {
        let installation = registry
            .get_tool_installation(&tool.id)
            .await
            .map_err(|e| e.to_string())?;

        summaries.push(ToolSummary {
            id: tool.id.clone(),
            name: tool.name,
            tool_type: format!("{:?}", tool.tool_type).to_lowercase(),
            source: format!("{:?}", tool.source).to_lowercase(),
            is_installed: installation.as_ref().map(|i| i.is_valid).unwrap_or(false),
            capabilities: tool.capabilities,
        });
    }

    Ok(ApiResponse { data: summaries })
}

#[tauri::command]
pub async fn tool_registry_get_tool(
    db: State<'_, AgentDb>,
    tool_id: String,
) -> Result<ApiResponse<ToolDetailResponse>, String> {
    let registry = create_registry(db)?;

    let tool = registry
        .get_tool(&tool_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Tool not found: {}", tool_id))?;

    let installation = registry
        .get_tool_installation(&tool_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(ApiResponse {
        data: ToolDetailResponse { tool, installation },
    })
}

#[tauri::command]
pub async fn tool_registry_register_tool(
    db: State<'_, AgentDb>,
    request: RegisterToolRequest,
) -> Result<ApiResponse<RegisterToolResponse>, String> {
    let registry = create_registry(db)?;

    let tool_id = registry
        .register_tool(request.spec)
        .await
        .map_err(|e| e.to_string())?;

    Ok(ApiResponse {
        data: RegisterToolResponse {
            tool_id,
            message: "Tool registered successfully".to_string(),
        },
    })
}

#[tauri::command]
pub async fn tool_registry_update_tool(
    db: State<'_, AgentDb>,
    tool_id: String,
    request: UpdateToolRequest,
) -> Result<ApiResponse<String>, String> {
    let registry = create_registry(db)?;

    registry
        .update_tool(&tool_id, request.spec)
        .await
        .map_err(|e| e.to_string())?;

    Ok(ApiResponse {
        data: "Tool updated successfully".to_string(),
    })
}

#[tauri::command]
pub async fn tool_registry_unregister_tool(
    db: State<'_, AgentDb>,
    tool_id: String,
) -> Result<ApiResponse<String>, String> {
    let registry = create_registry(db)?;

    registry
        .unregister_tool(&tool_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(ApiResponse {
        data: "Tool unregistered successfully".to_string(),
    })
}

#[tauri::command]
pub async fn tool_registry_validate_tool(
    db: State<'_, AgentDb>,
    tool_id: String,
) -> Result<ApiResponse<String>, String> {
    let registry = create_registry(db)?;

    let tool = registry
        .get_tool(&tool_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Tool not found: {}", tool_id))?;

    registry
        .validate_spec(&tool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(ApiResponse {
        data: "Tool specification is valid".to_string(),
    })
}

#[tauri::command]
pub async fn tool_registry_get_installation(
    db: State<'_, AgentDb>,
    tool_id: String,
) -> Result<ApiResponse<Option<ToolInstallation>>, String> {
    let registry = create_registry(db)?;

    let installation = registry
        .get_tool_installation(&tool_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(ApiResponse { data: installation })
}

#[tauri::command]
pub async fn tool_registry_update_installation(
    db: State<'_, AgentDb>,
    installation: ToolInstallation,
) -> Result<ApiResponse<String>, String> {
    let registry = create_registry(db)?;

    registry
        .update_tool_installation(installation)
        .await
        .map_err(|e| e.to_string())?;

    Ok(ApiResponse {
        data: "Installation updated successfully".to_string(),
    })
}

#[tauri::command]
pub async fn tool_registry_run_validation(
    db: State<'_, AgentDb>,
    tool_id: String,
) -> Result<ApiResponse<Vec<crate::tool_registry::ValidationResult>>, String> {
    let registry = create_registry(db)?;

    let tool = registry
        .get_tool(&tool_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Tool not found: {}", tool_id))?;

    let validator = ValidationEngineImpl::new();
    let results = validator.validate_tool(&tool).await.map_err(|e| e.to_string())?;

    Ok(ApiResponse { data: results })
}

#[derive(Deserialize)]
pub struct CreateJobRequest {
    pub job_type: String,
    pub params: serde_json::Value,
}

#[derive(Serialize)]
pub struct CreateJobResponse {
    pub job_id: String,
}

#[tauri::command]
pub async fn tool_registry_create_job(
    db: State<'_, AgentDb>,
    request: CreateJobRequest,
) -> Result<ApiResponse<CreateJobResponse>, String> {
    let job_manager = create_job_manager(db)?;

    let job_type = match request.job_type.as_str() {
        "skills_install" => JobType::SkillsInstall,
        "skills_uninstall" => JobType::SkillsUninstall,
        "tool_validation" => JobType::ToolValidation,
        "mcp_sync" => JobType::MCPSync,
        "config_sync" => JobType::ConfigSync,
        _ => return Err(format!("Unknown job type: {}", request.job_type)),
    };

    let job_id = job_manager
        .create_job(job_type, request.params)
        .await
        .map_err(|e| e.to_string())?;

    Ok(ApiResponse {
        data: CreateJobResponse { job_id },
    })
}

#[tauri::command]
pub async fn tool_registry_get_job(
    db: State<'_, AgentDb>,
    job_id: String,
) -> Result<ApiResponse<Option<crate::tool_registry::AsyncJob>>, String> {
    let job_manager = create_job_manager(db)?;

    let job = job_manager
        .get_job(&job_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(ApiResponse { data: job })
}

#[tauri::command]
pub async fn tool_registry_list_jobs(
    db: State<'_, AgentDb>,
    status: Option<String>,
) -> Result<ApiResponse<Vec<crate::tool_registry::AsyncJob>>, String> {
    let job_manager = create_job_manager(db)?;

    let job_status = status.map(|s| match s.as_str() {
        "pending" => crate::tool_registry::JobStatus::Pending,
        "running" => crate::tool_registry::JobStatus::Running,
        "completed" => crate::tool_registry::JobStatus::Completed,
        "failed" => crate::tool_registry::JobStatus::Failed,
        "cancelled" => crate::tool_registry::JobStatus::Cancelled,
        _ => crate::tool_registry::JobStatus::Pending,
    });

    let jobs = job_manager
        .list_jobs(job_status)
        .await
        .map_err(|e| e.to_string())?;

    Ok(ApiResponse { data: jobs })
}

#[tauri::command]
pub async fn tool_registry_cancel_job(
    db: State<'_, AgentDb>,
    job_id: String,
) -> Result<ApiResponse<String>, String> {
    let job_manager = create_job_manager(db)?;

    job_manager
        .cancel_job(&job_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(ApiResponse {
        data: "Job cancelled successfully".to_string(),
    })
}

#[tauri::command]
pub async fn skills_search(query: String) -> Result<ApiResponse<Vec<SkillInfo>>, String> {
    let skills_cli = SkillsCLI::new();
    let result = skills_cli.search(&query).await.map_err(|e| e.to_string())?;
    Ok(ApiResponse { data: result.skills })
}

#[derive(Deserialize)]
pub struct SkillsInstallRequest {
    pub source: String,
    pub name: String,
    pub scope: String,
}

#[tauri::command]
pub async fn skills_install(
    db: State<'_, AgentDb>,
    request: SkillsInstallRequest,
) -> Result<ApiResponse<String>, String> {
    let job_manager = create_job_manager(db)?;

    let params = serde_json::json!({
        "source": request.source,
        "name": request.name,
        "scope": request.scope,
    });

    let job_id = job_manager
        .create_job(JobType::SkillsInstall, params)
        .await
        .map_err(|e| e.to_string())?;

    Ok(ApiResponse {
        data: job_id,
    })
}

#[derive(Deserialize)]
pub struct SkillsUninstallRequest {
    pub name: String,
    pub scope: String,
}

#[tauri::command]
pub async fn skills_uninstall(
    db: State<'_, AgentDb>,
    request: SkillsUninstallRequest,
) -> Result<ApiResponse<String>, String> {
    let job_manager = create_job_manager(db)?;

    let params = serde_json::json!({
        "name": request.name,
        "scope": request.scope,
    });

    let job_id = job_manager
        .create_job(JobType::SkillsUninstall, params)
        .await
        .map_err(|e| e.to_string())?;

    Ok(ApiResponse {
        data: job_id,
    })
}

#[tauri::command]
pub async fn skills_list_installed(
    scope: Option<String>,
) -> Result<ApiResponse<Vec<SkillInfo>>, String> {
    let skills_cli = SkillsCLI::new();
    let scope_ref = scope.as_deref();
    let skills = skills_cli.list_installed(scope_ref).await.map_err(|e| e.to_string())?;
    Ok(ApiResponse { data: skills })
}

#[tauri::command]
pub async fn skills_check_updates() -> Result<ApiResponse<Vec<SkillInfo>>, String> {
    let skills_cli = SkillsCLI::new();
    let skills = skills_cli.check_updates().await.map_err(|e| e.to_string())?;
    Ok(ApiResponse { data: skills })
}

#[tauri::command]
pub async fn mcp_registry_list_servers(
    db: State<'_, AgentDb>,
) -> Result<ApiResponse<Vec<crate::tool_registry::MCPServer>>, String> {
    let mcp_registry = create_mcp_registry(db)?;
    let servers = mcp_registry.list_servers().await.map_err(|e| e.to_string())?;
    Ok(ApiResponse { data: servers })
}

#[tauri::command]
pub async fn mcp_registry_get_server(
    db: State<'_, AgentDb>,
    server_id: String,
) -> Result<ApiResponse<Option<crate::tool_registry::MCPServer>>, String> {
    let mcp_registry = create_mcp_registry(db)?;
    let server = mcp_registry.get_server(&server_id).await.map_err(|e| e.to_string())?;
    Ok(ApiResponse { data: server })
}

#[derive(Deserialize)]
pub struct AddMCPServerRequest {
    pub name: String,
    pub transport_type: String,
    pub config: MCPTransportConfig,
}

#[tauri::command]
pub async fn mcp_registry_add_server(
    db: State<'_, AgentDb>,
    request: AddMCPServerRequest,
) -> Result<ApiResponse<String>, String> {
    let mcp_registry = create_mcp_registry(db)?;

    let transport_type = match request.transport_type.as_str() {
        "stdio" => TransportType::Stdio,
        "sse" => TransportType::Sse,
        "http" => TransportType::Http,
        _ => return Err(format!("Unknown transport type: {}", request.transport_type)),
    };

    let server_id = mcp_registry
        .add_server(request.name, transport_type, request.config)
        .await
        .map_err(|e| e.to_string())?;

    Ok(ApiResponse { data: server_id })
}

#[tauri::command]
pub async fn mcp_registry_remove_server(
    db: State<'_, AgentDb>,
    server_id: String,
) -> Result<ApiResponse<String>, String> {
    let mcp_registry = create_mcp_registry(db)?;
    mcp_registry.remove_server(&server_id).await.map_err(|e| e.to_string())?;
    Ok(ApiResponse {
        data: "Server removed successfully".to_string(),
    })
}

#[derive(Deserialize)]
pub struct SetToolEnablementRequest {
    pub server_id: String,
    pub tool_id: String,
    pub is_enabled: bool,
}

#[tauri::command]
pub async fn mcp_registry_set_tool_enablement(
    db: State<'_, AgentDb>,
    request: SetToolEnablementRequest,
) -> Result<ApiResponse<String>, String> {
    let mcp_registry = create_mcp_registry(db)?;
    mcp_registry
        .set_tool_enablement(&request.server_id, &request.tool_id, request.is_enabled, None)
        .await
        .map_err(|e| e.to_string())?;
    Ok(ApiResponse {
        data: "Tool enablement updated".to_string(),
    })
}

#[tauri::command]
pub async fn mcp_registry_test_connection(
    db: State<'_, AgentDb>,
    server_id: String,
) -> Result<ApiResponse<bool>, String> {
    let mcp_registry = create_mcp_registry(db)?;
    let result = mcp_registry.test_connection(&server_id).await.map_err(|e| e.to_string())?;
    Ok(ApiResponse { data: result })
}
