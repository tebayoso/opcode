use crate::secure_storage::{ApiKeyStore, EncryptedData, SecureStorage};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

pub struct SecureStorageState {
    pub storage: Arc<SecureStorage>,
}

#[derive(Serialize)]
pub struct ApiKeyResponse {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Deserialize)]
pub struct StoreApiKeyRequest {
    pub name: String,
    pub provider: String,
    pub api_key: String,
}

#[tauri::command]
pub async fn check_master_key_exists(
    state: State<'_, SecureStorageState>,
) -> Result<bool, String> {
    Ok(state.storage.is_initialized())
}

#[tauri::command]
pub async fn initialize_master_key(
    state: State<'_, SecureStorageState>,
    password: String,
) -> Result<(), String> {
    state
        .storage
        .initialize_with_password(&password)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn unlock_with_master_key(
    state: State<'_, SecureStorageState>,
    password: String,
) -> Result<(), String> {
    state
        .storage
        .initialize_with_password(&password)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn store_api_key(
    state: State<'_, SecureStorageState>,
    request: StoreApiKeyRequest,
) -> Result<String, String> {
    let store = ApiKeyStore::new(state.storage.clone());
    let api_key = store
        .store_api_key(
            &uuid::Uuid::new_v4().to_string(),
            &request.name,
            &request.provider,
            &request.api_key,
        )
        .map_err(|e| e.to_string())?;

    Ok(api_key.id)
}

#[tauri::command]
pub async fn list_api_keys(
    state: State<'_, SecureStorageState>,
) -> Result<Vec<ApiKeyResponse>, String> {
    let api_keys: Vec<ApiKeyResponse> = Vec::new();
    Ok(api_keys)
}

#[tauri::command]
pub async fn get_api_key_value(
    state: State<'_, SecureStorageState>,
    id: String,
) -> Result<String, String> {
    Err("Not implemented".to_string())
}

#[tauri::command]
pub async fn delete_api_key(
    state: State<'_, SecureStorageState>,
    id: String,
) -> Result<(), String> {
    Ok(())
}
