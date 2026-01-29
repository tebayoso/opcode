#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::Mutex;
    use rusqlite::Connection;

    fn setup_test_db() -> Arc<Mutex<Connection>> {
        let conn = Connection::open_in_memory().unwrap();
        
        conn.execute(
            "CREATE TABLE tool_specifications (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                type TEXT NOT NULL,
                source TEXT NOT NULL,
                spec_json TEXT NOT NULL,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
                is_enabled INTEGER DEFAULT 1
            )",
            [],
        ).unwrap();

        conn.execute(
            "CREATE TABLE tool_installations (
                tool_id TEXT PRIMARY KEY,
                detected_paths TEXT,
                preferred_path TEXT,
                version TEXT,
                is_valid INTEGER,
                last_validated TEXT,
                validation_errors TEXT,
                FOREIGN KEY (tool_id) REFERENCES tool_specifications(id)
            )",
            [],
        ).unwrap();

        conn.execute(
            "CREATE TABLE mcp_servers (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                transport_type TEXT NOT NULL,
                config_json TEXT NOT NULL,
                is_enabled_globally INTEGER DEFAULT 1,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        ).unwrap();

        conn.execute(
            "CREATE TABLE mcp_tool_enablement (
                mcp_server_id TEXT,
                tool_id TEXT,
                is_enabled INTEGER DEFAULT 0,
                config_override TEXT,
                PRIMARY KEY (mcp_server_id, tool_id)
            )",
            [],
        ).unwrap();

        conn.execute(
            "CREATE TABLE async_jobs (
                id TEXT PRIMARY KEY,
                job_type TEXT NOT NULL,
                status TEXT NOT NULL,
                params TEXT,
                progress INTEGER DEFAULT 0,
                result TEXT,
                error_message TEXT,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                started_at TEXT,
                completed_at TEXT,
                cancelled_at TEXT
            )",
            [],
        ).unwrap();

        Arc::new(Mutex::new(conn))
    }

    #[test]
    fn test_tool_registry_creation() {
        let db = setup_test_db();
        let registry = ToolRegistryImpl::new(db);
        assert!(registry.is_ok());
    }

    #[test]
    fn test_builtin_tools_loaded() {
        let db = setup_test_db();
        let registry = ToolRegistryImpl::new(db).unwrap();
        
        let tools = registry.builtin_tools;
        assert_eq!(tools.len(), 8);
        assert!(tools.contains_key("claude"));
        assert!(tools.contains_key("gemini"));
        assert!(tools.contains_key("codex"));
        assert!(tools.contains_key("opencode"));
        assert!(tools.contains_key("cursor"));
        assert!(tools.contains_key("copilot"));
        assert!(tools.contains_key("eslint"));
        assert!(tools.contains_key("vite"));
    }

    #[tokio::test]
    async fn test_register_user_defined_tool() {
        let db = setup_test_db();
        let registry = ToolRegistryImpl::new(db).unwrap();

        let spec = ToolSpecification {
            id: "test-tool".to_string(),
            name: "Test Tool".to_string(),
            tool_type: ToolType::Cli,
            source: ToolSource::UserDefined,
            version: "1.0.0".to_string(),
            description: Some("A test tool".to_string()),
            website: None,
            icon: None,
            installation: InstallationConfig {
                binary_names: Some(vec!["test-tool".to_string()]),
                homebrew_formulas: None,
                npm_packages: None,
                nvm_packages: None,
                gh_extensions: None,
                standard_paths: None,
                version_args: Some(vec!["--version".to_string()]),
                version_pattern: Some(r"(\d+\.\d+\.\d+)".to_string()),
            },
            config: ConfigSpec {
                base_dir: "~/.test-tool".to_string(),
                files: None,
                settings_format: None,
                settings_path: None,
            },
            capabilities: ToolCapabilities {
                files: true,
                settings: true,
                mcp_servers: false,
                agents: false,
                commands: true,
                usage: true,
            },
            settings_schema: None,
        };

        let result = registry.register_tool(spec).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test-tool");
    }

    #[tokio::test]
    async fn test_cannot_register_builtin_tool_id() {
        let db = setup_test_db();
        let registry = ToolRegistryImpl::new(db).unwrap();

        let spec = ToolSpecification {
            id: "claude".to_string(),
            name: "Fake Claude".to_string(),
            tool_type: ToolType::Llm,
            source: ToolSource::UserDefined,
            version: "1.0.0".to_string(),
            description: None,
            website: None,
            icon: None,
            installation: InstallationConfig {
                binary_names: None,
                homebrew_formulas: None,
                npm_packages: None,
                nvm_packages: None,
                gh_extensions: None,
                standard_paths: None,
                version_args: None,
                version_pattern: None,
            },
            config: ConfigSpec {
                base_dir: "~/.fake".to_string(),
                files: None,
                settings_format: None,
                settings_path: None,
            },
            capabilities: ToolCapabilities {
                files: false,
                settings: false,
                mcp_servers: false,
                agents: false,
                commands: false,
                usage: false,
            },
            settings_schema: None,
        };

        let result = registry.register_tool(spec).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            RegistryError::ToolAlreadyExists(id) => assert_eq!(id, "claude"),
            _ => panic!("Expected ToolAlreadyExists error"),
        }
    }

    #[tokio::test]
    async fn test_get_tool() {
        let db = setup_test_db();
        let registry = ToolRegistryImpl::new(db).unwrap();

        // Get builtin tool
        let tool = registry.get_tool("claude").await.unwrap();
        assert!(tool.is_some());
        assert_eq!(tool.unwrap().name, "Claude Code");
        let tool = registry.get_tool("non-existent").await.unwrap();
        assert!(tool.is_none());
    }

    #[tokio::test]
    async fn test_load_tools() {
        let db = setup_test_db();
        let registry = ToolRegistryImpl::new(db).unwrap();

        let tools = registry.load_tools().await.unwrap();
        assert_eq!(tools.len(), 8);
    }
}
