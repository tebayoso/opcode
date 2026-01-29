use crate::tool_registry::{
    MCPServer, MCPToolEnablement, MCPTransportConfig, TransportType,
};
use async_trait::async_trait;
use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

#[async_trait]
pub trait MCPRegistry: Send + Sync {
    async fn list_servers(&self) -> Result<Vec<MCPServer>, String>;

    async fn get_server(&self, server_id: &str) -> Result<Option<MCPServer>, String>;

    async fn add_server(
        &self,
        name: String,
        transport_type: TransportType,
        config: MCPTransportConfig,
    ) -> Result<String, String>;

    async fn update_server(
        &self,
        server_id: &str,
        name: Option<String>,
        transport_type: Option<TransportType>,
        config: Option<MCPTransportConfig>,
        is_enabled_globally: Option<bool>,
    ) -> Result<(), String>;

    async fn remove_server(&self, server_id: &str) -> Result<(), String>;

    async fn get_tool_enablement(
        &self,
        server_id: &str,
        tool_id: &str,
    ) -> Result<Option<MCPToolEnablement>, String>;

    async fn set_tool_enablement(
        &self,
        server_id: &str,
        tool_id: &str,
        is_enabled: bool,
        config_override: Option<MCPTransportConfig>,
    ) -> Result<(), String>;

    async fn list_enabled_tools(&self, server_id: &str) -> Result<Vec<MCPToolEnablement>, String>;

    async fn sync_server_to_tool(
        &self,
        server_id: &str,
        tool_id: &str,
    ) -> Result<(), String>;

    async fn test_connection(&self, server_id: &str) -> Result<bool, String>;
}

pub struct MCPRegistryImpl {
    db: Arc<Mutex<Connection>>,
}

impl MCPRegistryImpl {
    pub fn new(db: Arc<Mutex<Connection>>) -> Result<Self, String> {
        Ok(Self { db })
    }

    fn generate_server_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    fn row_to_server(row: &rusqlite::Row) -> Result<MCPServer, rusqlite::Error> {
        let transport_type_str: String = row.get(2)?;
        let transport_type = match transport_type_str.as_str() {
            "stdio" => TransportType::Stdio,
            "sse" => TransportType::Sse,
            "http" => TransportType::Http,
            _ => TransportType::Stdio,
        };

        let config_json: String = row.get(3)?;
        let config: MCPTransportConfig = serde_json::from_str(&config_json)
            .unwrap_or_else(|_| MCPTransportConfig {
                command: None,
                args: vec![],
                env: HashMap::new(),
                url: None,
                headers: HashMap::new(),
            });

        Ok(MCPServer {
            id: row.get(0)?,
            name: row.get(1)?,
            transport_type,
            config,
            is_enabled_globally: row.get::<_, i32>(4).unwrap_or(1) != 0,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    }
}

#[async_trait]
impl MCPRegistry for MCPRegistryImpl {
    async fn list_servers(&self) -> Result<Vec<MCPServer>, String> {
        let conn = self.db.lock().map_err(|_| "Lock error")?;

        let mut stmt = conn.prepare(
            "SELECT id, name, transport_type, config_json, is_enabled_globally, created_at, updated_at
             FROM mcp_servers ORDER BY created_at DESC"
        ).map_err(|e| e.to_string())?;

        let servers = stmt
            .query_map([], Self::row_to_server)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(servers)
    }

    async fn get_server(&self, server_id: &str) -> Result<Option<MCPServer>, String> {
        let conn = self.db.lock().map_err(|_| "Lock error")?;

        let result = conn.query_row(
            "SELECT id, name, transport_type, config_json, is_enabled_globally, created_at, updated_at
             FROM mcp_servers WHERE id = ?1",
            [server_id],
            Self::row_to_server,
        );

        match result {
            Ok(server) => Ok(Some(server)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn add_server(
        &self,
        name: String,
        transport_type: TransportType,
        config: MCPTransportConfig,
    ) -> Result<String, String> {
        let server_id = Self::generate_server_id();
        let transport_type_str = format!("{:?}", transport_type).to_lowercase();
        let config_json = serde_json::to_string(&config).map_err(|e| e.to_string())?;

        let conn = self.db.lock().map_err(|_| "Lock error")?;

        conn.execute(
            "INSERT INTO mcp_servers (id, name, transport_type, config_json, is_enabled_globally, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
            params![&server_id, &name, &transport_type_str, &config_json
            ],
        ).map_err(|e| e.to_string())?;

        Ok(server_id)
    }

    async fn update_server(
        &self,
        server_id: &str,
        name: Option<String>,
        transport_type: Option<TransportType>,
        config: Option<MCPTransportConfig>,
        is_enabled_globally: Option<bool>,
    ) -> Result<(), String> {
        let conn = self.db.lock().map_err(|_| "Lock error")?;

        if let Some(n) = name {
            conn.execute(
                "UPDATE mcp_servers SET name = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
                params![&n, server_id],
            ).map_err(|e| e.to_string())?;
        }

        if let Some(tt) = transport_type {
            let tt_str = format!("{:?}", tt).to_lowercase();
            conn.execute(
                "UPDATE mcp_servers SET transport_type = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
                params![&tt_str, server_id],
            ).map_err(|e| e.to_string())?;
        }

        if let Some(c) = config {
            let config_json = serde_json::to_string(&c).map_err(|e| e.to_string())?;
            conn.execute(
                "UPDATE mcp_servers SET config_json = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
                params![&config_json, server_id],
            ).map_err(|e| e.to_string())?;
        }

        if let Some(enabled) = is_enabled_globally {
            let enabled_int = if enabled { 1 } else { 0 };
            conn.execute(
                "UPDATE mcp_servers SET is_enabled_globally = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
                params![enabled_int, server_id],
            ).map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    async fn remove_server(&self, server_id: &str) -> Result<(), String> {
        let conn = self.db.lock().map_err(|_| "Lock error")?;

        conn.execute(
            "DELETE FROM mcp_tool_enablement WHERE mcp_server_id = ?1",
            [server_id],
        ).map_err(|e| e.to_string())?;

        conn.execute(
            "DELETE FROM mcp_servers WHERE id = ?1",
            [server_id],
        ).map_err(|e| e.to_string())?;

        Ok(())
    }

    async fn get_tool_enablement(
        &self,
        server_id: &str,
        tool_id: &str,
    ) -> Result<Option<MCPToolEnablement>, String> {
        let conn = self.db.lock().map_err(|_| "Lock error")?;

        let result = conn.query_row(
            "SELECT mcp_server_id, tool_id, is_enabled, config_override
             FROM mcp_tool_enablement WHERE mcp_server_id = ?1 AND tool_id = ?2",
            params![server_id, tool_id],
            |row| {
                let config_override_str: Option<String> = row.get(3)?;
                let config_override = config_override_str
                    .and_then(|s| serde_json::from_str(&s).ok());

                Ok(MCPToolEnablement {
                    mcp_server_id: row.get(0)?,
                    tool_id: row.get(1)?,
                    is_enabled: row.get::<_, i32>(2).unwrap_or(0) != 0,
                    config_override,
                })
            },
        );

        match result {
            Ok(enablement) => Ok(Some(enablement)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn set_tool_enablement(
        &self,
        server_id: &str,
        tool_id: &str,
        is_enabled: bool,
        config_override: Option<MCPTransportConfig>,
    ) -> Result<(), String> {
        let conn = self.db.lock().map_err(|_| "Lock error")?;

        let config_override_json = config_override
            .map(|c| serde_json::to_string(&c).unwrap_or_default());

        conn.execute(
            "INSERT INTO mcp_tool_enablement (mcp_server_id, tool_id, is_enabled, config_override)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(mcp_server_id, tool_id) DO UPDATE SET
             is_enabled = excluded.is_enabled,
             config_override = excluded.config_override",
            params![
                server_id,
                tool_id,
                if is_enabled { 1 } else { 0 },
                config_override_json
            ],
        ).map_err(|e| e.to_string())?;

        Ok(())
    }

    async fn list_enabled_tools(&self, server_id: &str) -> Result<Vec<MCPToolEnablement>, String> {
        let conn = self.db.lock().map_err(|_| "Lock error")?;

        let mut stmt = conn.prepare(
            "SELECT mcp_server_id, tool_id, is_enabled, config_override
             FROM mcp_tool_enablement WHERE mcp_server_id = ?1 AND is_enabled = 1"
        ).map_err(|e| e.to_string())?;

        let enablements = stmt
            .query_map([server_id], |row| {
                let config_override_str: Option<String> = row.get(3)?;
                let config_override = config_override_str
                    .and_then(|s| serde_json::from_str(&s).ok());

                Ok(MCPToolEnablement {
                    mcp_server_id: row.get(0)?,
                    tool_id: row.get(1)?,
                    is_enabled: row.get::<_, i32>(2).unwrap_or(0) != 0,
                    config_override,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(enablements)
    }

    async fn sync_server_to_tool(
        &self,
        server_id: &str,
        tool_id: &str,
    ) -> Result<(), String> {
        let enablement = self.get_tool_enablement(server_id, tool_id).await?;

        if let Some(e) = enablement {
            if e.is_enabled {
                println!(
                    "Syncing MCP server {} to tool {}: ENABLED",
                    server_id, tool_id
                );
            } else {
                println!(
                    "Syncing MCP server {} to tool {}: DISABLED",
                    server_id, tool_id
                );
            }
        }

        Ok(())
    }

    async fn test_connection(&self, server_id: &str) -> Result<bool, String> {
        let server = self.get_server(server_id).await?;

        if let Some(s) = server {
            match s.transport_type {
                TransportType::Stdio => {
                    if let Some(cmd) = s.config.command {
                        match tokio::process::Command::new(&cmd)
                            .args(&s.config.args)
                            .envs(&s.config.env)
                            .output()
                            .await
                        {
                            Ok(_) => Ok(true),
                            Err(_) => Ok(false),
                        }
                    } else {
                        Ok(false)
                    }
                }
                TransportType::Sse | TransportType::Http => {
                    if let Some(url) = s.config.url {
                        match reqwest::get(&url).await {
                            Ok(response) => Ok(response.status().is_success()),
                            Err(_) => Ok(false),
                        }
                    } else {
                        Ok(false)
                    }
                }
            }
        } else {
            Err("Server not found".to_string())
        }
    }
}
