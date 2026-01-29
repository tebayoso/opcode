# Unified Control Panel - Implementation Roadmap

**Project**: opcode - Unified Tool Management & Control Panel  
**Status**: Ready for Implementation  
**Generated**: 2026-01-29  

---

## Summary

This roadmap consolidates all planning documents and provides a clear path forward for implementing the unified control panel. The project builds on an already strong foundation in the opcode codebase.

### What Already Exists (Don't Rebuild)

1. **CLI Tools System** (`src-tauri/src/cli_tools/`)
   - 6 tools with detection (Claude, Gemini, Codex, OpenCode, Copilot, Cursor)
   - Trait-based config system (`CLIToolConfig`)
   - Multi-format parsers (JSON, JSONC, YAML, TOML, Markdown)
   - Detection from 7 sources

2. **Database Layer** (`src-tauri/src/commands/agents.rs`)
   - SQLite with rusqlite
   - Tables: agents, agent_runs, app_settings, cli_tool_preferences, cli_tool_usage
   - Initialization pattern in `init_database()`

3. **Skills System** (`src-tauri/src/commands/skills.rs`)
   - File-based CRUD operations
   - YAML frontmatter parsing
   - Project + user scopes

4. **MCP Management** (`src-tauri/src/commands/mcp.rs`)
   - Per-tool MCP server management
   - Stdio/SSE/HTTP transports
   - Claude Desktop import

5. **Frontend** (`src/`)
   - React + TypeScript + Vite
   - Zustand stores
   - shadcn/ui components
   - CLIToolsDashboard

### What Needs to Be Built

1. **Unified Tool Registry** - Dynamic tool registration from JSON/YAML
2. **Skills.sh API Integration** - Search, install, uninstall with progress
3. **Universal MCP Registry** - Cross-tool sync with one-click enable
4. **Validation Engine** - Binary checks, version verification, functional tests
5. **WebSocket Server** - Real-time updates
6. **Async Job System** - For long-running operations
7. **New UI Dashboard** - Overview + per-tool pages + skills + MCP panels

---

## Phase 1: Foundation (Week 1-2)

### Task 1.1: Database Schema Migration
**Priority**: Critical | **Estimated**: 1 day

Add 7 new tables to `init_database()` in `src-tauri/src/commands/agents.rs`:

```rust
// After existing table creation, add:

// 1. tool_specifications
conn.execute(
    "CREATE TABLE IF NOT EXISTS tool_specifications (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        type TEXT NOT NULL,
        source TEXT NOT NULL,
        spec_json TEXT NOT NULL,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        is_enabled BOOLEAN DEFAULT 1
    )",
    [],
)?;

// 2. tool_installations
conn.execute(
    "CREATE TABLE IF NOT EXISTS tool_installations (
        tool_id TEXT PRIMARY KEY,
        detected_paths TEXT,
        preferred_path TEXT,
        version TEXT,
        is_valid BOOLEAN,
        last_validated TIMESTAMP,
        validation_errors TEXT,
        FOREIGN KEY (tool_id) REFERENCES tool_specifications(id)
    )",
    [],
)?;

// 3. mcp_servers
conn.execute(
    "CREATE TABLE IF NOT EXISTS mcp_servers (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL UNIQUE,
        transport_type TEXT NOT NULL,
        config_json TEXT NOT NULL,
        is_enabled_globally BOOLEAN DEFAULT 1,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
    )",
    [],
)?;

// 4. mcp_tool_enablement
conn.execute(
    "CREATE TABLE IF NOT EXISTS mcp_tool_enablement (
        mcp_server_id TEXT,
        tool_id TEXT,
        is_enabled BOOLEAN DEFAULT 0,
        config_override TEXT,
        PRIMARY KEY (mcp_server_id, tool_id),
        FOREIGN KEY (mcp_server_id) REFERENCES mcp_servers(id),
        FOREIGN KEY (tool_id) REFERENCES tool_specifications(id)
    )",
    [],
)?;

// 5. async_jobs
conn.execute(
    "CREATE TABLE IF NOT EXISTS async_jobs (
        id TEXT PRIMARY KEY,
        job_type TEXT NOT NULL,
        status TEXT NOT NULL,
        params TEXT,
        progress INTEGER DEFAULT 0,
        result TEXT,
        error_message TEXT,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        started_at TIMESTAMP,
        completed_at TIMESTAMP,
        cancelled_at TIMESTAMP
    )",
    [],
)?;

// 6. validation_results
conn.execute(
    "CREATE TABLE IF NOT EXISTS validation_results (
        id TEXT PRIMARY KEY,
        tool_id TEXT NOT NULL,
        validation_type TEXT NOT NULL,
        status TEXT NOT NULL,
        details TEXT,
        checked_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY (tool_id) REFERENCES tool_specifications(id)
    )",
    [],
)?;

// 7. system_warnings
conn.execute(
    "CREATE TABLE IF NOT EXISTS system_warnings (
        id TEXT PRIMARY KEY,
        warning_type TEXT NOT NULL,
        severity TEXT NOT NULL,
        message TEXT NOT NULL,
        details TEXT,
        is_resolved BOOLEAN DEFAULT 0,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        resolved_at TIMESTAMP
    )",
    [],
)?;
```

**Acceptance Criteria**:
- [ ] All 7 tables created successfully
- [ ] Foreign key constraints defined
- [ ] Tables appear in database after app restart
- [ ] No errors in database initialization

---

### Task 1.2: Tool Specification Schema
**Priority**: Critical | **Estimated**: 2 days

Create new module: `src-tauri/src/tool_registry/`

**Files to create**:
1. `src-tauri/src/tool_registry/mod.rs` - Module entry
2. `src-tauri/src/tool_registry/spec.rs` - Core types
3. `src-tauri/src/tool_registry/validation.rs` - Schema validation
4. `src-tauri/src/tool_registry/error.rs` - Error types

**spec.rs**:
```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tool specification - the core data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpecification {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub tool_type: ToolType,
    pub source: ToolSource,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    pub installation: InstallationConfig,
    pub config: ConfigSpec,
    pub capabilities: ToolCapabilities,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings_schema: Option<Vec<SettingSchema>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolType {
    Llm,
    Cli,
    Codec,
    Service,
    Plugin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolSource {
    Builtin,
    UserDefined,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binary_names: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homebrew_formulas: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub npm_packages: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nvm_packages: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gh_extensions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub standard_paths: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_args: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_pattern: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSpec {
    pub base_dir: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<ConfigFileSpec>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFileSpec {
    pub path: String,
    #[serde(rename = "type")]
    pub file_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCapabilities {
    pub files: bool,
    pub settings: bool,
    pub mcp_servers: bool,
    pub agents: bool,
    pub commands: bool,
    pub usage: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingSchema {
    pub key: String,
    #[serde(rename = "type")]
    pub value_type: SettingValueType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<String>>,
    #[serde(default)]
    pub readonly: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SettingValueType {
    String,
    Number,
    Boolean,
    Array,
    Object,
    Select,
    Textarea,
}
```

**validation.rs**:
```rust
use super::spec::ToolSpecification;
use super::error::RegistryError;
use jsonschema::{JSONSchema, CompilationError};
use serde_json::Value;

/// JSON Schema for tool specification validation
const TOOL_SPEC_SCHEMA: &str = r#"{
    "$schema": "http://json-schema.org/draft-07/schema#",
    "type": "object",
    "required": ["id", "name", "type", "installation", "config", "capabilities"],
    "properties": {
        "id": { "type": "string", "pattern": "^[a-z0-9_-]+$" },
        "name": { "type": "string", "minLength": 1 },
        "type": { "enum": ["llm", "cli", "codec", "service", "plugin"] },
        "installation": {
            "type": "object",
            "properties": {
                "binary_names": { "type": "array", "items": { "type": "string" } },
                "homebrew_formulas": { "type": "array", "items": { "type": "string" } },
                "npm_packages": { "type": "array", "items": { "type": "string" } },
                "version_args": { "type": "array", "items": { "type": "string" } }
            }
        },
        "config": {
            "type": "object",
            "required": ["base_dir"],
            "properties": {
                "base_dir": { "type": "string" }
            }
        },
        "capabilities": {
            "type": "object",
            "required": ["files", "settings", "mcp_servers", "agents", "commands", "usage"],
            "properties": {
                "files": { "type": "boolean" },
                "settings": { "type": "boolean" },
                "mcp_servers": { "type": "boolean" },
                "agents": { "type": "boolean" },
                "commands": { "type": "boolean" },
                "usage": { "type": "boolean" }
            }
        }
    }
}"#;

pub struct SpecValidator {
    schema: JSONSchema,
}

impl SpecValidator {
    pub fn new() -> Result<Self, CompilationError> {
        let schema: Value = serde_json::from_str(TOOL_SPEC_SCHEMA)?;
        let compiled = JSONSchema::compile(&schema)?;
        Ok(Self { schema: compiled })
    }

    pub fn validate(&self, spec: &ToolSpecification) -> Result<(), RegistryError> {
        let value = serde_json::to_value(spec)?;
        let result = self.schema.validate(&value);
        
        if let Err(errors) = result {
            let messages: Vec<String> = errors.map(|e| e.to_string()).collect();
            return Err(RegistryError::ValidationError(messages.join("; ")));
        }
        
        Ok(())
    }
}
```

**Acceptance Criteria**:
- [ ] All types defined with proper serialization
- [ ] JSON Schema validation compiles
- [ ] Can parse existing tool definitions
- [ ] Unit tests for validation (90%+ coverage)

---

### Task 1.3: Tool Registry Service
**Priority**: Critical | **Estimated**: 3 days

Create `src-tauri/src/tool_registry/registry.rs`:

```rust
use super::spec::ToolSpecification;
use super::error::RegistryError;
use super::validation::SpecValidator;
use rusqlite::Connection;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use notify::{Watcher, RecursiveMode, DebouncedEvent};
use std::sync::mpsc::channel;
use std::time::Duration;

/// Trait for tool registry operations
#[async_trait::async_trait]
pub trait ToolRegistry: Send + Sync {
    /// Load all tools (builtin + user-defined)
    async fn load_tools(&self) -> Result<Vec<ToolSpecification>, RegistryError>;
    
    /// Get specific tool by ID
    async fn get_tool(&self, tool_id: &str
    ) -> Result<Option<ToolSpecification>, RegistryError>;
    
    /// Register new user-defined tool
    async fn register_tool(
        &self, 
        spec: ToolSpecification
    ) -> Result<String, RegistryError>;
    
    /// Update tool spec
    async fn update_tool(
        &self, 
        tool_id: &str, 
        spec: ToolSpecification
    ) -> Result<(), RegistryError>;
    
    /// Unregister user-defined tool
    async fn unregister_tool(&self, tool_id: &str) -> Result<(), RegistryError>;
    
    /// Validate tool specification
    async fn validate_spec(
        &self, 
        spec: &ToolSpecification
    ) -> Result<(), RegistryError>;
}

/// Implementation of tool registry
pub struct ToolRegistryImpl {
    db: Arc<Mutex<Connection>>,
    validator: SpecValidator,
    builtin_tools: HashMap<String, ToolSpecification>,
}

impl ToolRegistryImpl {
    pub fn new(db: Arc<Mutex<Connection>>) -> Result<Self, RegistryError> {
        let validator = SpecValidator::new()?;
        let mut registry = Self {
            db,
            validator,
            builtin_tools: HashMap::new(),
        };
        registry.load_builtin_tools();
        Ok(registry)
    }

    /// Load the 8 predefined tools
    fn load_builtin_tools(&mut self) {
        let tools = vec![
            self.create_claude_spec(),
            self.create_gemini_spec(),
            self.create_codex_spec(),
            self.create_opencode_spec(),
            self.create_cursor_spec(),
            self.create_copilot_spec(),
            self.create_eslint_spec(),
            self.create_vite_spec(),
        ];
        
        for tool in tools {
            self.builtin_tools.insert(tool.id.clone(), tool);
        }
    }

    fn create_claude_spec(&self) -> ToolSpecification {
        ToolSpecification {
            id: "claude".to_string(),
            name: "Claude Code".to_string(),
            tool_type: super::spec::ToolType::Llm,
            source: super::spec::ToolSource::Builtin,
            version: "1.0.0".to_string(),
            description: Some("Anthropic's AI coding assistant".to_string()),
            website: Some("https://claude.ai/code".to_string()),
            icon: None,
            installation: super::spec::InstallationConfig {
                binary_names: Some(vec!["claude".to_string()]),
                homebrew_formulas: Some(vec!["claude-code".to_string()]),
                npm_packages: Some(vec!["@anthropic-ai/claude-code".to_string()]),
                nvm_packages: None,
                gh_extensions: None,
                standard_paths: Some(vec![
                    "/usr/local/bin/claude".to_string(),
                    "/opt/homebrew/bin/claude".to_string(),
                    "~/.local/bin/claude".to_string(),
                ]),
                version_args: Some(vec!["--version".to_string()]),
                version_pattern: Some(r"(\d+\.\d+\.\d+)".to_string()),
            },
            config: super::spec::ConfigSpec {
                base_dir: "~/.claude".to_string(),
                files: Some(vec![
                    super::spec::ConfigFileSpec {
                        path: "settings.json".to_string(),
                        file_type: "json".to_string(),
                        description: Some("Claude settings".to_string()),
                        required: Some(false),
                    },
                ]),
                settings_format: Some("json".to_string()),
                settings_path: Some("settings.json".to_string()),
            },
            capabilities: super::spec::ToolCapabilities {
                files: true,
                settings: true,
                mcp_servers: true,
                agents: true,
                commands: true,
                usage: true,
            },
            settings_schema: None,
        }
    }

    // ... create_*_spec() for other tools
}

#[async_trait::async_trait]
impl ToolRegistry for ToolRegistryImpl {
    async fn load_tools(&self) -> Result<Vec<ToolSpecification>, RegistryError> {
        let mut tools: Vec<ToolSpecification> = self.builtin_tools.values().cloned().collect();
        
        // Load user-defined tools from database
        let conn = self.db.lock().map_err(|_| RegistryError::LockError)?;
        let mut stmt = conn.prepare(
            "SELECT spec_json FROM tool_specifications WHERE source = 'user_defined'"
        )?;
        
        let rows = stmt.query_map([], |row| {
            let json: String = row.get(0)?;
            Ok(json)
        })?;
        
        for row in rows {
            let json = row?;
            let spec: ToolSpecification = serde_json::from_str(&json)?;
            tools.push(spec);
        }
        
        Ok(tools)
    }

    async fn get_tool(
        &self, 
        tool_id: &str
    ) -> Result<Option<ToolSpecification>, RegistryError> {
        // Check builtin first
        if let Some(tool) = self.builtin_tools.get(tool_id) {
            return Ok(Some(tool.clone()));
        }
        
        // Check database
        let conn = self.db.lock().map_err(|_| RegistryError::LockError)?;
        let mut stmt = conn.prepare(
            "SELECT spec_json FROM tool_specifications WHERE id = ?1"
        )?;
        
        let result = stmt.query_row([tool_id], |row| {
            let json: String = row.get(0)?;
            Ok(json)
        });
        
        match result {
            Ok(json) => {
                let spec: ToolSpecification = serde_json::from_str(&json)?;
                Ok(Some(spec))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn register_tool(
        &self, 
        spec: ToolSpecification
    ) -> Result<String, RegistryError> {
        // Validate
        self.validator.validate(&spec)?;
        
        // Check if ID already exists
        if self.builtin_tools.contains_key(&spec.id) {
            return Err(RegistryError::ToolAlreadyExists(spec.id.clone()));
        }
        
        // Insert into database
        let conn = self.db.lock().map_err(|_| RegistryError::LockError)?;
        let json = serde_json::to_string(&spec)?;
        
        conn.execute(
            "INSERT INTO tool_specifications (id, name, type, source, spec_json, is_enabled)
             VALUES (?1, ?2, ?3, 'user_defined', ?4, 1)",
            [&spec.id, &spec.name, &format!("{:?}", spec.tool_type), &json],
        )?;
        
        Ok(spec.id)
    }

    // ... implement other methods
}
```

**Acceptance Criteria**:
- [ ] ToolRegistry trait implemented
- [ ] All 8 builtin tools load on startup
- [ ] User-defined tools persist to database
- [ ] CRUD operations work correctly

---

### Task 1.4: REST API Routes
**Priority**: High | **Estimated**: 2 days

Create `src-tauri/src/web_server/routes/tools.rs`:

```rust
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::tool_registry::{ToolRegistry, ToolSpecification};

/// API response wrapper
#[derive(Serialize)]
struct ApiResponse<T> {
    data: T,
}

#[derive(Serialize)]
struct ApiError {
    error: ErrorDetail,
}

#[derive(Serialize)]
struct ErrorDetail {
    code: String,
    message: String,
}

/// Query params for listing tools
#[derive(Deserialize)]
struct ListToolsQuery {
    #[serde(rename = "type")]
    tool_type: Option<String>,
    source: Option<String>,
    installed: Option<bool>,
}

/// List all tools
async fn list_tools(
    State(registry): State<Arc<dyn ToolRegistry>>,
    Query(query): Query<ListToolsQuery>,
) -> Result<Json<ApiResponse<Vec<ToolSummary>>>, StatusCode> {
    let tools = registry.load_tools().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let summaries: Vec<ToolSummary> = tools.into_iter()
        .map(|t| ToolSummary {
            id: t.id,
            name: t.name,
            tool_type: format!("{:?}", t.tool_type),
            source: format!("{:?}", t.source),
            is_installed: false, // TODO: check installation
            capabilities: t.capabilities,
        })
        .collect();
    
    Ok(Json(ApiResponse { data: summaries }))
}

/// Get tool details
async fn get_tool(
    State(registry): State<Arc<dyn ToolRegistry>>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<ToolSpecification>>, StatusCode> {
    let tool = registry.get_tool(&id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    Ok(Json(ApiResponse { data: tool }))
}

/// Register new tool
async fn register_tool(
    State(registry): State<Arc<dyn ToolRegistry>>,
    Json(req): Json<RegisterToolRequest>,
) -> Result<Json<ApiResponse<RegisterToolResponse>>, StatusCode> {
    let id = registry.register_tool(req.spec).await
        .map_err(|e| {
            match e {
                crate::tool_registry::RegistryError::ValidationError(_) => StatusCode::UNPROCESSABLE_ENTITY,
                crate::tool_registry::RegistryError::ToolAlreadyExists(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;
    
    Ok(Json(ApiResponse {
        data: RegisterToolResponse {
            tool_id: id,
            message: "Tool registered successfully".to_string(),
        }
    }))
}

// Request/response types
#[derive(Deserialize)]
struct RegisterToolRequest {
    spec: ToolSpecification,
}

#[derive(Serialize)]
struct RegisterToolResponse {
    tool_id: String,
    message: String,
}

#[derive(Serialize)]
struct ToolSummary {
    id: String,
    name: String,
    tool_type: String,
    source: String,
    is_installed: bool,
    capabilities: crate::tool_registry::ToolCapabilities,
}

/// Create router
pub fn tool_routes(registry: Arc<dyn ToolRegistry>) -> Router {
    Router::new()
        .route("/tools", get(list_tools).post(register_tool))
        .route("/tools/:id", get(get_tool).put(update_tool).delete(unregister_tool))
        .route("/tools/:id/validate", post(validate_tool))
        .with_state(registry)
}

// TODO: implement update_tool, unregister_tool, validate_tool
```

**Acceptance Criteria**:
- [ ] GET /api/tools - List all tools
- [ ] GET /api/tools/:id - Get tool details
- [ ] POST /api/tools - Register new tool
- [ ] PUT /api/tools/:id - Update tool
- [ ] DELETE /api/tools/:id - Unregister tool
- [ ] POST /api/tools/:id/validate - Validate tool

---

### Task 1.5: Frontend Shell
**Priority**: High | **Estimated**: 3 days

Create new app structure in `src/app/`:

```
src/app/
├── layout.tsx              # Root layout with providers
├── dashboard/
│   ├── layout.tsx          # Dashboard layout
│   └── page.tsx            # Overview page
├── tools/
│   ├── layout.tsx
│   ├── page.tsx            # Tools list
│   └── [toolId]/
│       └── page.tsx        # Tool detail page
├── skills/
│   └── page.tsx            # Skills management
├── mcp/
│   └── page.tsx            # MCP registry
└── configs/
    └── page.tsx            # Custom configs
```

**src/app/layout.tsx**:
```tsx
import { ThemeProvider } from "@/components/theme-provider";
import { RealTimeProvider } from "@/components/real-time-provider";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

const queryClient = new QueryClient();

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body>
        <QueryClientProvider client={queryClient}>
          <ThemeProvider defaultTheme="dark" storageKey="opcode-theme">
            <RealTimeProvider>
              {children}
            </RealTimeProvider>
          </ThemeProvider>
        </QueryClientProvider>
      </body>
    </html>
  );
}
```

**src/app/dashboard/layout.tsx**:
```tsx
import { Sidebar } from "@/components/layout/sidebar";
import { Header } from "@/components/layout/header";

export default function DashboardLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <div className="flex h-screen">
      <Sidebar />
      <div className="flex-1 flex flex-col">
        <Header />
        <main className="flex-1 overflow-auto p-6">
          {children}
        </main>
      </div>
    </div>
  );
}
```

**src/components/layout/sidebar.tsx**:
```tsx
"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { cn } from "@/lib/utils";
import {
  LayoutDashboard,
  Terminal,
  Puzzle,
  Server,
  Settings,
} from "lucide-react";

const navItems = [
  { href: "/dashboard", label: "Overview", icon: LayoutDashboard },
  { href: "/tools", label: "Tools", icon: Terminal },
  { href: "/skills", label: "Skills", icon: Puzzle },
  { href: "/mcp", label: "MCP Servers", icon: Server },
  { href: "/configs", label: "Configs", icon: Settings },
];

export function Sidebar() {
  const pathname = usePathname();

  return (
    <aside className="w-64 border-r bg-card">
      <div className="p-6">
        <h1 className="text-xl font-bold">opcode</h1>
      </div>
      <nav className="px-4 space-y-2">
        {navItems.map((item) => (
          <Link
            key={item.href}
            href={item.href}
            className={cn(
              "flex items-center gap-3 px-4 py-2 rounded-lg transition-colors",
              pathname === item.href
                ? "bg-primary text-primary-foreground"
                : "hover:bg-muted"
            )}
          >
            <item.icon className="w-5 h-5" />
            {item.label}
          </Link>
        ))}
      </nav>
    </aside>
  );
}
```

**Acceptance Criteria**:
- [ ] Navigation sidebar with all sections
- [ ] Route structure defined
- [ ] Theme provider setup
- [ ] API client configuration
- [ ] Error boundary component

---

### Task 1.6: Tool List Page
**Priority**: Medium | **Estimated**: 2 days

**src/app/tools/page.tsx**:
```tsx
"use client";

import { useQuery } from "@tanstack/react-query";
import { ToolCard } from "@/components/tools/tool-card";
import { ToolFilters } from "@/components/tools/tool-filters";
import { Button } from "@/components/ui/button";
import { Plus } from "lucide-react";
import Link from "next/link";

async function fetchTools() {
  const res = await fetch("http://localhost:8080/api/tools");
  if (!res.ok) throw new Error("Failed to fetch tools");
  return res.json();
}

export default function ToolsPage() {
  const { data, isLoading, error } = useQuery({
    queryKey: ["tools"],
    queryFn: fetchTools,
  });

  if (isLoading) return <div>Loading...</div>;
  if (error) return <div>Error: {error.message}</div>;

  const tools = data?.data || [];

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <h1 className="text-3xl font-bold">Tools</h1>
        <Button asChild>
          <Link href="/tools/register">
            <Plus className="w-4 h-4 mr-2" />
            Register Tool
          </Link>
        </Button>
      </div>

      <ToolFilters />

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {tools.map((tool: any) => (
          <ToolCard key={tool.id} tool={tool} />
        ))}
      </div>
    </div>
  );
}
```

**src/components/tools/tool-card.tsx**:
```tsx
"use client";

import Link from "next/link";
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { ToolCapabilities } from "@/types/tool";

interface ToolCardProps {
  tool: {
    id: string;
    name: string;
    tool_type: string;
    source: string;
    is_installed: boolean;
    capabilities: ToolCapabilities;
  };
}

export function ToolCard({ tool }: ToolCardProps) {
  return (
    <Link href={`/tools/${tool.id}`}>
      <Card className="hover:border-primary transition-colors cursor-pointer">
        <CardHeader>
          <div className="flex justify-between items-start">
            <div>
              <CardTitle>{tool.name}</CardTitle>
              <CardDescription className="capitalize">
                {tool.tool_type.toLowerCase()}
              </CardDescription>
            </div>
            <Badge variant={tool.is_installed ? "default" : "secondary"}>
              {tool.is_installed ? "Installed" : "Not Installed"}
            </Badge>
          </div>
        </CardHeader>
        <CardContent>
          <div className="flex flex-wrap gap-2">
            {tool.capabilities.mcp_servers && (
              <Badge variant="outline">MCP</Badge>
            )}
            {tool.capabilities.agents && (
              <Badge variant="outline">Agents</Badge>
            )}
            {tool.capabilities.usage && (
              <Badge variant="outline">Usage</Badge>
            )}
          </div>
        </CardContent>
      </Card>
    </Link>
  );
}
```

**Acceptance Criteria**:
- [ ] Tool cards display correctly
- [ ] Installation status badges
- [ ] Filter by type/source
- [ ] Search functionality
- [ ] Click to navigate to detail

---

## Phase 2-5 Summary

### Phase 2: Core Backend (Week 3-4)
- Validation Engine - Binary/version/functional checks
- WebSocket Server - Real-time updates
- Async Job System - For skills install/uninstall
- CLI Wrapper for skills.sh
- MCP Registry service

### Phase 3: Frontend Foundation (Week 5-6)
- Dashboard overview page
- Real-time WebSocket provider
- Tool detail pages
- Skills panel UI
- MCP registry UI

### Phase 4: Features (Week 7-8)
- Skills.sh search integration
- One-click skill install/uninstall
- MCP cross-tool sync
- Validation UI
- Settings management

### Phase 5: Polish (Week 9-10)
- User-defined tool specs UI
- Custom configs area
- Error handling improvements
- Testing (unit + integration)
- Documentation

---

## How to Start

### Option 1: Start with Phase 1
```bash
/start-work .sisyphus/tasks/phase-1.md
```

### Option 2: Start with Full Master Plan
```bash
/start-work .sisyphus/plans/master-plan.md
```

### Option 3: Start with Specific Task
```bash
/start-work .sisyphus/roadmap/implementation-roadmap.md
```

---

## Verification Commands

```bash
# Run backend tests
cd src-tauri && cargo test

# Run frontend dev server
bun run dev

# Run full Tauri dev
bun run tauri dev

# Check database schema
sqlite3 ~/.local/share/opcode/agents.db ".schema"

# Test API endpoints
curl http://localhost:8080/api/tools | jq
```

---

## Success Criteria

- [ ] All 8 predefined tools manageable via unified panel
- [ ] Users can define custom tools via JSON/YAML
- [ ] Skills.sh search/install/uninstall fully automated
- [ ] MCP servers sync across all tools with one-click
- [ ] Real-time status updates via WebSocket
- [ ] Comprehensive validation for all tools
- [ ] New UI fully functional, replaces existing

---

**Ready for Implementation** ✅

All planning complete. Architecture defined. Tasks broken down. Existing codebase analyzed.
