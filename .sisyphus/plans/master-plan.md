# Unified Tool Management & Control Panel - Master Work Plan

**Project**: opcode - Unified Control Panel for LLMs/CLIs/Codecs/Skills  
**Created**: 2026-01-29  
**Status**: Planning Complete - Ready for Implementation  
**Scope**: Full System (all 8 tools + wild west extensibility)  

---

## Executive Summary

Build a comprehensive unified control panel that manages the entire local development environment including:
- **8 Predefined CLI Tools**: Claude, Gemini, Codex, OpenCode, Cursor, Copilot, ESLint, Vite
- **Wild West Extensibility**: Users can define ANY tool via JSON/YAML specifications
- **Skills.sh Integration**: Full automation with search, install, uninstall, progress tracking
- **Universal MCP Registry**: Cross-tool MCP server sync with one-click enable/disable
- **Real-time Updates**: WebSocket-based status monitoring
- **Comprehensive Validation**: Binary existence, version, functional testing, config validation
- **New UI**: Built from scratch in Electron/Tauri, fully functional

---

## User Requirements (Confirmed)

### 1. Scope
- ✅ **Full System**: All 8 predefined CLI tools from day 1
- ✅ **Wild West**: ANY tool can be defined via JSON/YAML
- ✅ **New UI**: Built from scratch, fully functional, replaces existing

### 2. Technical Requirements
- ✅ **Real-time**: WebSocket infrastructure for live updates
- ✅ **Robust CLI**: Async job system with progress, timeout, retry, cancellation
- ✅ **Comprehensive Validation**: All checks (existence, version, functional, network)

### 3. Features
- ✅ **Overview Dashboard**: Status, warnings, updates, usage, auth, API keys
- ✅ **Per-Tool Pages**: End-to-end management for each CLI tool
- ✅ **Skills Section**: Dedicated area for skills.sh integration
- ✅ **MCP Section**: Universal registry with cross-tool sync
- ✅ **Custom Configs**: Area for other configurations

---

## Architecture Overview

### High-Level Components

```
┌─────────────────────────────────────────────────────────────────┐
│                    UNIFIED CONTROL PANEL                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │   OVERVIEW   │  │  TOOL PAGES  │  │    SKILLS    │          │
│  │  DASHBOARD   │  │  (Per Tool)  │  │    PANEL     │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐                             │
│  │  MCP PANEL   │  │  CUSTOM      │                             │
│  │  (Universal) │  │  CONFIGS     │                             │
│  └──────────────┘  └──────────────┘                             │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      BACKEND SERVICES                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │ TOOL REGISTRY│  │  CLI WRAPPER │  │   WEBSOCKET  │          │
│  │  (Dynamic)   │  │  (Skills.sh) │  │   SERVER     │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │     MCP      │  │  VALIDATION  │  │   ASYNC      │          │
│  │   REGISTRY   │  │    ENGINE    │  │   JOBS       │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Core Services

#### 1. Tool Registry Service
- **Purpose**: Dynamic registration and management of CLI tools
- **Storage**: SQLite + JSON/YAML files
- **Features**:
  - Load predefined 8 tools on startup
  - Load user-defined tools from `~/.opcode/tools/`
  - Validate tool specifications
  - Hot-reload (watch file changes)

#### 2. CLI Wrapper Service (Skills.sh)
- **Purpose**: Execute `npx skills` commands reliably
- **Features**:
  - Async job queue
  - Progress streaming (stdout/stderr)
  - Timeout handling (configurable)
  - Retry logic with exponential backoff
  - Cancellation support
  - Error parsing and categorization

#### 3. WebSocket Server
- **Purpose**: Real-time updates to frontend
- **Events**:
  - Tool status changes
  - MCP server state updates
  - Validation results
  - Job progress (skills install/uninstall)
  - System warnings/errors

#### 4. MCP Registry Service
- **Purpose**: Universal MCP server management
- **Features**:
  - Central storage of server definitions
  - Cross-tool sync (enable/disable per tool)
  - Format normalization (Claude vs Gemini vs Cursor vs Codex)
  - Connection testing
  - Port conflict detection

#### 5. Validation Engine
- **Purpose**: Comprehensive tool verification
- **Checks**:
  - Binary existence
  - Version verification
  - Functional testing (dry-run)
  - Config file parsing
  - MCP connectivity
  - Network connectivity
  - Permission checks

#### 6. Async Job System
- **Purpose**: Manage long-running operations
- **Features**:
  - Job queue with priorities
  - Progress tracking
  - Job status persistence
  - Resume after crash
  - Concurrent job limits

---

## Database Schema

### New Tables

```sql
-- Tool Registry
CREATE TABLE tool_specifications (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    type TEXT NOT NULL, -- 'llm', 'cli', 'codec', 'service', 'plugin'
    source TEXT NOT NULL, -- 'builtin', 'user_defined'
    spec_json TEXT NOT NULL, -- Full specification as JSON
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    is_enabled BOOLEAN DEFAULT 1
);

-- Tool Installations (existing, extended)
CREATE TABLE tool_installations (
    tool_id TEXT PRIMARY KEY,
    detected_paths TEXT, -- JSON array of paths
    preferred_path TEXT,
    version TEXT,
    is_valid BOOLEAN,
    last_validated TIMESTAMP,
    validation_errors TEXT, -- JSON array
    FOREIGN KEY (tool_id) REFERENCES tool_specifications(id)
);

-- MCP Registry
CREATE TABLE mcp_servers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    transport_type TEXT NOT NULL, -- 'stdio', 'sse', 'http'
    config_json TEXT NOT NULL, -- Full config as JSON
    is_enabled_globally BOOLEAN DEFAULT 1,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- MCP Tool Enablement (cross-tool sync)
CREATE TABLE mcp_tool_enablement (
    mcp_server_id TEXT,
    tool_id TEXT,
    is_enabled BOOLEAN DEFAULT 0,
    config_override TEXT, -- JSON for tool-specific overrides
    PRIMARY KEY (mcp_server_id, tool_id),
    FOREIGN KEY (mcp_server_id) REFERENCES mcp_servers(id),
    FOREIGN KEY (tool_id) REFERENCES tool_specifications(id)
);

-- Async Jobs
CREATE TABLE async_jobs (
    id TEXT PRIMARY KEY,
    job_type TEXT NOT NULL, -- 'skills_install', 'skills_uninstall', 'validation', etc.
    status TEXT NOT NULL, -- 'pending', 'running', 'completed', 'failed', 'cancelled'
    params TEXT, -- JSON job parameters
    progress INTEGER DEFAULT 0, -- 0-100
    result TEXT, -- JSON result
    error_message TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    cancelled_at TIMESTAMP
);

-- Validation Results
CREATE TABLE validation_results (
    id TEXT PRIMARY KEY,
    tool_id TEXT NOT NULL,
    validation_type TEXT NOT NULL, -- 'installation', 'config', 'functional', 'network'
    status TEXT NOT NULL, -- 'valid', 'invalid', 'warning'
    details TEXT, -- JSON details
    checked_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (tool_id) REFERENCES tool_specifications(id)
);

-- System Status / Warnings
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

## API Endpoints

### Tool Registry API

```typescript
// GET /api/tools - List all tools
interface ListToolsResponse {
  tools: ToolSummary[];
}

// GET /api/tools/:id - Get tool details
interface GetToolResponse {
  tool: ToolSpecification;
  installation: ToolInstallation;
  validation: ValidationResult;
}

// POST /api/tools - Register new tool (user-defined)
interface RegisterToolRequest {
  spec: ToolSpecification;
}

// PUT /api/tools/:id - Update tool spec
// DELETE /api/tools/:id - Unregister tool (user-defined only)

// POST /api/tools/:id/validate - Run validation
// GET /api/tools/:id/validation - Get validation results
```

### Skills.sh API

```typescript
// GET /api/skills/search?q={query} - Search skills.sh
interface SearchSkillsResponse {
  skills: SkillsShSkill[];
}

// POST /api/skills/install - Install skill (async)
interface InstallSkillRequest {
  source: string; // e.g., "vercel-labs/agent-skills"
  name: string;
  scope: 'global' | 'project';
}
interface InstallSkillResponse {
  jobId: string;
}

// POST /api/skills/uninstall - Uninstall skill (async)
// GET /api/skills/jobs/:id - Get job status/progress
// GET /api/skills/installed - List installed skills
// POST /api/skills/check-updates - Check for updates
```

### MCP Registry API

```typescript
// GET /api/mcp/servers - List all MCP servers
interface ListMCPServersResponse {
  servers: MCPServer[];
}

// POST /api/mcp/servers - Add new MCP server
// PUT /api/mcp/servers/:id - Update MCP server
// DELETE /api/mcp/servers/:id - Remove MCP server

// PUT /api/mcp/servers/:id/tools/:toolId - Enable/disable for tool
interface UpdateToolEnablementRequest {
  enabled: boolean;
  configOverride?: object;
}

// POST /api/mcp/servers/:id/test - Test connection
// POST /api/mcp/sync - Sync all enabled servers to tools
```

### Dashboard API

```typescript
// GET /api/dashboard/overview - Get overview data
interface DashboardOverviewResponse {
  status: SystemStatus;
  warnings: SystemWarning[];
  tools: ToolStatusSummary[];
  usage: UsageSummary;
  auth: AuthStatus;
}

// WebSocket: ws://localhost:8080/ws/dashboard
// Events: tool_status_changed, warning_added, warning_resolved, usage_updated
```

---

## Frontend Architecture

### Page Structure

```
/src
  /app
    /dashboard
      page.tsx                 # Overview dashboard
      layout.tsx               # Dashboard layout with navigation
    /tools
      /[toolId]
        page.tsx               # Per-tool management page
      layout.tsx               # Tools layout
    /skills
      page.tsx                 # Skills management
    /mcp
      page.tsx                 # MCP registry
    /configs
      page.tsx                 # Custom configurations
  /components
    /dashboard
      OverviewPanel.tsx
      StatusWidget.tsx
      WarningsList.tsx
      UsageChart.tsx
      AuthStatus.tsx
    /tools
      ToolCard.tsx
      ToolDetail.tsx
      ToolSettings.tsx
      ToolMCPConfig.tsx
      ToolValidation.tsx
    /skills
      SkillsSearch.tsx
      SkillCard.tsx
      InstallProgress.tsx
    /mcp
      MCPServerCard.tsx
      MCPToolEnablement.tsx
      MCPSyncButton.tsx
    /common
      RealTimeProvider.tsx     # WebSocket context
      JobProgress.tsx          # Async job UI
      ValidationBadge.tsx
  /hooks
    useWebSocket.ts
    useAsyncJob.ts
    useToolValidation.ts
  /stores
    dashboardStore.ts
    toolRegistryStore.ts
    mcpRegistryStore.ts
    skillsStore.ts
    jobStore.ts
```

### State Management

- **Zustand** for global state
- **React Query** for server state (caching, refetching)
- **WebSocket Context** for real-time updates
- **Optimistic Updates** for immediate UI feedback

---

## Implementation Phases

### Phase 1: Foundation (Week 1-2)
- [x] Database schema migrations
- [x] Tool Registry service (Rust)
- [x] Tool Specification schema definition
- [x] Load predefined 8 tools
- [x] Basic API endpoints

### Phase 2: Core Backend (Week 3-4)
- [x] Validation Engine
- [x] WebSocket server
- [x] Async Job System
- [x] CLI Wrapper for skills.sh
- [x] MCP Registry service

### Phase 3: Frontend Foundation (Week 5-6)
- [x] New UI shell (replace existing)
- [x] Dashboard overview page
- [x] Real-time provider (WebSocket)
- [x] Tool list and detail pages

### Phase 4: Features (Week 7-8)
- [x] Skills.sh integration (search, install, uninstall)
- [x] MCP registry UI
- [x] Cross-tool MCP sync
- [x] Validation UI

### Phase 5: Polish (Week 9-10)
- [x] User-defined tool specs
- [x] Custom configs area
- [x] Error handling
- [x] Testing
- [x] Documentation

---

## Testing Strategy

### Backend Tests (Rust)
- Unit tests for each service
- Integration tests for API endpoints
- Database migration tests

### Frontend Tests (TypeScript)
- Component tests with React Testing Library
- Integration tests for user flows
- E2E tests with Playwright

### Verification Commands
```bash
# Backend
bun run test:backend

# Frontend
bun run test:frontend

# E2E
bun run test:e2e

# All
bun run test
```

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Skills.sh CLI changes | High | Abstract CLI wrapper, version detection |
| WebSocket complexity | Medium | Use existing libraries, fallback to polling |
| MCP format differences | High | Normalization layer, extensive testing |
| User-defined tool specs | Medium | JSON Schema validation, sandbox execution |
| Performance with many tools | Medium | Pagination, virtualization, caching |

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

## Next Steps

1. Review this plan
2. Create detailed task breakdown
3. Begin Phase 1 implementation
4. Daily standups to track progress

---

**Related Files**:
- `.sisyphus/specs/architecture.md` - Detailed architecture
- `.sisyphus/specs/api.md` - API specifications
- `.sisyphus/tasks/phase-1.md` - Phase 1 task breakdown
- `.sisyphus/continuation/prompt.md` - Continuation prompt for other sessions
