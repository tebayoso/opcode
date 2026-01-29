# Phase 1: Foundation - Task Breakdown

**Phase Goal**: Establish the core infrastructure for the unified control panel  
**Duration**: 2 weeks  
**Deliverables**: Database schema, Tool Registry service, basic API endpoints  

---

## Week 1: Database & Core Infrastructure

### Task 1.1: Database Schema Migration
**Priority**: Critical  
**Estimated Time**: 1 day  

**Description**:  
Create database migrations for all new tables required by the unified control panel.

**Acceptance Criteria**:
- [ ] Migration files created in `src-tauri/migrations/`
- [ ] All 7 tables defined (tool_specifications, tool_installations, mcp_servers, mcp_tool_enablement, async_jobs, validation_results, system_warnings)
- [ ] Foreign key constraints properly defined
- [ ] Indexes created for frequently queried columns
- [ ] Migration runs successfully on fresh database
- [ ] Migration is idempotent (can run multiple times safely)

**Files to Create/Modify**:
- `src-tauri/migrations/001_create_tool_registry.sql`
- `src-tauri/migrations/002_create_mcp_registry.sql`
- `src-tauri/migrations/003_create_async_jobs.sql`
- `src-tauri/migrations/004_create_validation_system.sql`
- `src-tauri/src/storage/migrations.rs` - Migration runner

**Verification**:
```bash
# Run migrations
cd src-tauri && cargo test migrations::test_migrations

# Verify tables exist
sqlite3 ~/.opcode/data.db ".schema"
```

---

### Task 1.2: Tool Specification Schema Definition
**Priority**: Critical  
**Estimated Time**: 2 days  

**Description**:  
Define the complete ToolSpecification schema in Rust with validation logic.

**Acceptance Criteria**:
- [ ] `ToolSpecification` struct defined with all fields
- [ ] `InstallationConfig`, `ConfigSpec`, `ToolCapabilities` structs defined
- [ ] JSON Schema validation implemented
- [ ] YAML parsing support added
- [ ] Validation functions for each field
- [ ] Error types defined for validation failures
- [ ] Unit tests for validation logic (90%+ coverage)

**Files to Create/Modify**:
- `src-tauri/src/tool_registry/spec.rs` - Core spec types
- `src-tauri/src/tool_registry/validation.rs` - Validation logic
- `src-tauri/src/tool_registry/error.rs` - Error types

**Verification**:
```bash
cd src-tauri && cargo test tool_registry::spec
cd src-tauri && cargo test tool_registry::validation
```

---

### Task 1.3: Tool Registry Service Implementation
**Priority**: Critical  
**Estimated Time**: 3 days  

**Description**:  
Implement the ToolRegistry trait with all required methods.

**Acceptance Criteria**:
- [ ] `ToolRegistry` trait defined with all methods
- [ ] `ToolRegistryImpl` struct implementing the trait
- [ ] Load predefined 8 tools on startup
- [ ] Load user-defined tools from `~/.opcode/tools/`
- [ ] CRUD operations for user-defined tools
- [ ] Hot-reload with file watching
- [ ] SQLite persistence for tool metadata
- [ ] Unit tests for all methods

**Files to Create/Modify**:
- `src-tauri/src/tool_registry/mod.rs` - Module entry
- `src-tauri/src/tool_registry/registry.rs` - Implementation
- `src-tauri/src/tool_registry/watcher.rs` - File watcher
- `src-tauri/src/tool_registry/builtin.rs` - Predefined tools

**Verification**:
```bash
cd src-tauri && cargo test tool_registry::

# Test loading tools
curl http://localhost:8080/api/tools | jq '.data.tools | length'
# Expected: 8 (predefined tools)
```

---

## Week 2: API Foundation & Integration

### Task 1.4: REST API Routes - Tools
**Priority**: High  
**Estimated Time**: 2 days  

**Description**:  
Implement REST API endpoints for tool registry.

**Acceptance Criteria**:
- [ ] `GET /api/tools` - List all tools with filtering
- [ ] `GET /api/tools/:id` - Get tool details
- [ ] `POST /api/tools` - Register new tool
- [ ] `PUT /api/tools/:id` - Update tool
- [ ] `DELETE /api/tools/:id` - Unregister tool
- [ ] `POST /api/tools/:id/validate` - Validate tool
- [ ] `GET /api/tools/:id/validation` - Get validation history
- [ ] Proper error handling and status codes
- [ ] Request validation
- [ ] Integration tests

**Files to Create/Modify**:
- `src-tauri/src/commands/tools.rs` - Tauri commands
- `src-tauri/src/web_server/routes/tools.rs` - Web server routes
- `src-tauri/src/api/tools.rs` - Shared API logic

**Verification**:
```bash
# Test list tools
curl http://localhost:8080/api/tools | jq

# Test get tool
curl http://localhost:8080/api/tools/claude | jq

# Test register tool
curl -X POST http://localhost:8080/api/tools \
  -H "Content-Type: application/json" \
  -d @test-tool.json | jq

# Test validation
curl -X POST http://localhost:8080/api/tools/claude/validate | jq
```

---

### Task 1.5: Frontend Foundation - New UI Shell
**Priority**: High  
**Estimated Time**: 3 days  

**Description**:  
Create the new UI shell that will replace the existing interface.

**Acceptance Criteria**:
- [ ] New layout component created
- [ ] Navigation sidebar with sections (Dashboard, Tools, Skills, MCP, Configs)
- [ ] Route structure defined (Next.js App Router)
- [ ] Theme provider (dark/light mode)
- [ ] Global state setup (Zustand stores)
- [ ] API client configuration
- [ ] Error boundary component
- [ ] Loading states

**Files to Create/Modify**:
- `src/app/layout.tsx` - Root layout
- `src/app/dashboard/layout.tsx` - Dashboard layout
- `src/app/dashboard/page.tsx` - Overview page (placeholder)
- `src/app/tools/layout.tsx` - Tools layout
- `src/app/tools/page.tsx` - Tools list page
- `src/components/layout/Sidebar.tsx` - Navigation
- `src/components/layout/Header.tsx` - Header
- `src/stores/index.ts` - Store exports
- `src/lib/api-client.ts` - API client

**Verification**:
```bash
# Start dev server
bun run dev

# Verify UI loads
open http://localhost:5173

# Check console for errors
```

---

### Task 1.6: Tool List Page (Frontend)
**Priority**: Medium  
**Estimated Time**: 2 days  

**Description**:  
Create the tools list page showing all registered tools.

**Acceptance Criteria**:
- [ ] Tool cards displaying name, type, status
- [ ] Installation status badges
- [ ] Filter by type/source/installed
- [ ] Search functionality
- [ ] Click to navigate to tool detail
- [ ] Register new tool button
- [ ] Loading states
- [ ] Error handling

**Files to Create/Modify**:
- `src/app/tools/page.tsx` - Tools list
- `src/components/tools/ToolCard.tsx` - Tool card component
- `src/components/tools/ToolFilters.tsx` - Filter component
- `src/components/tools/ToolSearch.tsx` - Search component
- `src/hooks/useTools.ts` - Data fetching hook

**Verification**:
```bash
# Start dev server
bun run dev

# Navigate to tools page
open http://localhost:5173/tools

# Verify all 8 tools displayed
# Test filtering
# Test search
```

---

## Task Dependencies

```
Task 1.1 (Database)
    │
    ▼
Task 1.2 (Spec Schema)
    │
    ▼
Task 1.3 (Registry Service)
    │
    ├──► Task 1.4 (API Routes)
    │
    └──► Task 1.5 (Frontend Shell)
             │
             ▼
        Task 1.6 (Tool List)
```

---

## Definition of Done for Phase 1

- [ ] All database migrations applied successfully
- [ ] Tool registry service fully functional
- [ ] All 8 predefined tools load on startup
- [ ] User can register custom tools via API
- [ ] Tool list page displays all tools
- [ ] API endpoints tested and working
- [ ] New UI shell renders without errors
- [ ] Code reviewed and approved
- [ ] Documentation updated

---

## Next Phase

**Phase 2**: Core Backend Services
- Validation Engine
- WebSocket Server
- Async Job System
- CLI Wrapper for skills.sh
- MCP Registry service

See `.sisyphus/tasks/phase-2.md` for details.
