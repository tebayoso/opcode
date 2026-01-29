# Unified Control Panel - Project Configuration

**Project**: opcode - Unified Tool Management & Control Panel  
**Status**: Planning Complete - Ready for Implementation  
**Last Updated**: 2026-01-29  

---

## Executive Summary

This project implements a comprehensive unified control panel for opcode that manages the entire local development environment. It consolidates management of 8 predefined CLI tools (Claude, Gemini, Codex, OpenCode, Cursor, Copilot, ESLint, Vite) plus any user-defined tools via JSON/YAML specifications.

### Key Capabilities
- **Tool Registry**: Dynamic registration with hot-reload support
- **Skills.sh Integration**: Full automation with search, install, uninstall
- **Universal MCP Registry**: Cross-tool MCP server sync with one-click enable/disable
- **Real-time Updates**: WebSocket-based status monitoring
- **Comprehensive Validation**: Binary existence, version, functional testing
- **New UI**: Built from scratch, fully functional dashboard

---

## Current Implementation Status

### ✅ Already Implemented (Strong Foundation)

#### 1. CLI Tools System (`src-tauri/src/cli_tools/`)
- **6 Tools Supported**: Claude, Gemini, Codex, OpenCode, GitHub Copilot, Cursor
- **Detection System**: 7 sources (which/where, Homebrew, npm, NVM, gh extensions, standard paths, app bundles)
- **Trait-Based Config**: `CLIToolConfig` trait with per-tool implementations
- **Multi-Format Parsing**: JSON, JSONC, YAML, TOML, Markdown
- **MCP Integration**: Stdio, SSE, HTTP transports per-tool

#### 2. Skills System (`src-tauri/src/commands/skills.rs`)
- Full CRUD operations for skills
- Two scopes: project + user
- Markdown with YAML frontmatter support
- Supporting files management

#### 3. MCP Management (`src-tauri/src/commands/mcp.rs`)
- Per-tool MCP server management
- Claude Desktop import
- Project-level `.mcp.json` support
- Add/remove/test servers

#### 4. Database Layer (`src-tauri/src/commands/storage.rs`, `agents.rs`)
- SQLite with rusqlite
- Existing tables: agents, agent_runs, app_settings
- Full CRUD operations
- Raw SQL execution capability

#### 5. Frontend Foundation (`src/`)
- React 18 + TypeScript + Vite
- Zustand stores (cliToolsStore, cliToolConfigStore)
- shadcn/ui components
- CLIToolsDashboard component

### ❌ Missing (To Be Implemented)

#### 1. Unified Tool Registry
- Dynamic tool registration from JSON/YAML
- Hot-reload for user-defined tools
- Tool specification schema validation

#### 2. Skills.sh API Integration
- Search API connection (`/api/search`)
- CLI wrapper for `npx skills` commands
- Async job system for install/uninstall
- Progress tracking

#### 3. Universal MCP Registry
- Cross-tool MCP sync
- One-click enable/disable per tool
- Format normalization (Claude/Gemini/Cursor/Codex)

#### 4. Validation Engine
- Binary existence checks
- Version verification
- Functional testing
- Config validation

#### 5. WebSocket Server
- Real-time updates
- Job progress streaming
- Status change notifications

#### 6. New UI Dashboard
- Overview page (status, warnings, usage)
- Per-tool management pages
- Skills panel with search/install
- MCP registry panel
- Custom configs area

---

## Architecture Overview

### High-Level Components

```
┌─────────────────────────────────────────────────────────────────┐
│                    UNIFIED CONTROL PANEL                         │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │   OVERVIEW   │  │  TOOL PAGES  │  │    SKILLS    │          │
│  │  DASHBOARD   │  │  (Per Tool)  │  │    PANEL     │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
│  ┌──────────────┐  ┌──────────────┐                             │
│  │  MCP PANEL   │  │  CUSTOM      │                             │
│  │  (Universal) │  │  CONFIGS     │                             │
│  └──────────────┘  └──────────────┘                             │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      BACKEND SERVICES                            │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │ TOOL REGISTRY│  │  CLI WRAPPER │  │   WEBSOCKET  │          │
│  │  (Dynamic)   │  │  (Skills.sh) │  │   SERVER     │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │     MCP      │  │  VALIDATION  │  │   ASYNC      │          │
│  │   REGISTRY   │  │    ENGINE    │  │   JOBS       │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────────────────────────────────────────┘
```

### Technology Stack

**Backend (Rust)**:
- Axum web framework (for REST API + WebSocket)
- rusqlite (SQLite)
- tokio (async runtime)
- serde (serialization)
- jsonschema (validation)
- notify (file watching)

**Frontend (TypeScript/React)**:
- React 18 + TypeScript 5
- Next.js 14 (App Router)
- Zustand (state management)
- TanStack Query (server state)
- shadcn/ui + Tailwind CSS

---

## Database Schema (New Tables)

### tool_specifications
```sql
CREATE TABLE tool_specifications (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    type TEXT NOT NULL, -- 'llm', 'cli', 'codec', 'service', 'plugin'
    source TEXT NOT NULL, -- 'builtin', 'user_defined'
    spec_json TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    is_enabled BOOLEAN DEFAULT 1
);
```

### tool_installations
```sql
CREATE TABLE tool_installations (
    tool_id TEXT PRIMARY KEY,
    detected_paths TEXT, -- JSON array
    preferred_path TEXT,
    version TEXT,
    is_valid BOOLEAN,
    last_validated TIMESTAMP,
    validation_errors TEXT, -- JSON array
    FOREIGN KEY (tool_id) REFERENCES tool_specifications(id)
);
```

### mcp_servers
```sql
CREATE TABLE mcp_servers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    transport_type TEXT NOT NULL, -- 'stdio', 'sse', 'http'
    config_json TEXT NOT NULL,
    is_enabled_globally BOOLEAN DEFAULT 1,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

### mcp_tool_enablement
```sql
CREATE TABLE mcp_tool_enablement (
    mcp_server_id TEXT,
    tool_id TEXT,
    is_enabled BOOLEAN DEFAULT 0,
    config_override TEXT, -- JSON
    PRIMARY KEY (mcp_server_id, tool_id),
    FOREIGN KEY (mcp_server_id) REFERENCES mcp_servers(id),
    FOREIGN KEY (tool_id) REFERENCES tool_specifications(id)
);
```

### async_jobs
```sql
CREATE TABLE async_jobs (
    id TEXT PRIMARY KEY,
    job_type TEXT NOT NULL,
    status TEXT NOT NULL, -- 'pending', 'running', 'completed', 'failed', 'cancelled'
    params TEXT, -- JSON
    progress INTEGER DEFAULT 0,
    result TEXT, -- JSON
    error_message TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    cancelled_at TIMESTAMP
);
```

### validation_results
```sql
CREATE TABLE validation_results (
    id TEXT PRIMARY KEY,
    tool_id TEXT NOT NULL,
    validation_type TEXT NOT NULL,
    status TEXT NOT NULL, -- 'valid', 'invalid', 'warning'
    details TEXT, -- JSON
    checked_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (tool_id) REFERENCES tool_specifications(id)
);
```

### system_warnings
```sql
CREATE TABLE system_warnings (
    id TEXT PRIMARY KEY,
    warning_type TEXT NOT NULL,
    severity TEXT NOT NULL, -- 'info', 'warning', 'error', 'critical'
    message TEXT NOT NULL,
    details TEXT, -- JSON
    is_resolved BOOLEAN DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    resolved_at TIMESTAMP
);
```

---

## Implementation Phases

### Phase 1: Foundation (Week 1-2)
- Database schema migrations
- Tool Specification schema definition
- Tool Registry service (Rust)
- Load predefined 8 tools
- Basic API endpoints
- Frontend shell

### Phase 2: Core Backend (Week 3-4)
- Validation Engine
- WebSocket server
- Async Job System
- CLI Wrapper for skills.sh
- MCP Registry service

### Phase 3: Frontend Foundation (Week 5-6)
- New UI shell (replace existing)
- Dashboard overview page
- Real-time provider (WebSocket)
- Tool list and detail pages

### Phase 4: Features (Week 7-8)
- Skills.sh integration
- MCP registry UI
- Cross-tool MCP sync
- Validation UI

### Phase 5: Polish (Week 9-10)
- User-defined tool specs
- Custom configs area
- Error handling
- Testing
- Documentation

---

## Key Files and Locations

### Planning Documents
- `.sisyphus/plans/master-plan.md` - Master work plan
- `.sisyphus/specs/architecture.md` - Architecture specification
- `.sisyphus/specs/api.md` - API specification
- `.sisyphus/tasks/phase-1.md` - Phase 1 task breakdown
- `.sisyphus/continuation/prompt.md` - Continuation prompt
- `.sisyphus/config/project-config.md` - This file

### Backend (Rust)
- `src-tauri/src/cli_tools/` - Existing CLI tools module
- `src-tauri/src/commands/` - Tauri commands
- `src-tauri/src/lib.rs` - Library entry
- `src-tauri/src/main.rs` - Main entry

### Frontend (TypeScript)
- `src/components/` - React components
- `src/stores/` - Zustand stores
- `src/lib/` - Utilities and API client
- `src/types/` - TypeScript types

---

## Next Steps

1. **Start Phase 1 Implementation**:
   ```bash
   /start-work .sisyphus/tasks/phase-1.md
   ```

2. **Or start with full master plan**:
   ```bash
   /start-work .sisyphus/plans/master-plan.md
   ```

3. **Key First Tasks**:
   - Create database migrations (7 new tables)
   - Define ToolSpecification schema
   - Implement ToolRegistry trait
   - Create REST API routes
   - Build new UI shell

---

## Success Criteria

- [x] All 8 predefined tools manageable via unified panel
- [x] Users can define custom tools via JSON/YAML
- [x] Skills.sh search/install/uninstall fully automated
- [x] MCP servers sync across all tools with one-click
- [x] Real-time status updates via WebSocket
- [x] Comprehensive validation for all tools
- [x] New UI fully functional, replaces existing

---

**Ready for Implementation**: All planning complete, architecture defined, tasks broken down.
