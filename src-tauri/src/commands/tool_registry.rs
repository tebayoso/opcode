use crate::tool_registry::{
    ToolInstallation, ToolRegistry, ToolRegistryImpl, ToolSpecification, ValidationEngine, ValidationEngineImpl,
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
