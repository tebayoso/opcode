# Master Plan Learnings

## 2026-01-29 - Phase 1 Complete

### Database Schema Patterns
- SQLite with rusqlite in Tauri
- Use `CREATE TABLE IF NOT EXISTS` for idempotent migrations
- Use `TEXT` type for timestamps (no native TIMESTAMP in SQLite)
- Use `INTEGER` for booleans (0/1)
- Add triggers for automatic `updated_at` management
- Foreign key constraints work well for referential integrity

### Tool Registry Architecture
- Trait-based design (`ToolRegistry`) allows for easy testing and mocking
- `ToolRegistryImpl` wraps database connection and builtin tools cache
- Builtin tools stored in HashMap for fast lookup
- User-defined tools loaded from database on demand
- Async trait methods work well with Tauri's async runtime

### Predefined Tool Specs
All 8 tools defined with complete specs:
- Binary detection paths (which, npm, homebrew, gh extensions)
- Version extraction patterns (regex)
- Configuration file locations
- Capabilities flags (files, settings, mcp_servers, agents, commands, usage)

### API Design
- Tauri commands return `Result<ApiResponse<T>, String>` for consistency
- Separate request/response structs for type safety
- State management through Tauri's `State<AgentDb>`

### Git Workflow
- Atomic commits per logical unit
- Clear commit messages with scope and description
- Multiple commits for different concerns (schema, types, implementation, API)

## 2026-01-29 - Phase 2 Partial Complete

### Validation Engine
- Trait-based design (`ValidationEngine`) for testability
- 5 validation types: Installation, Configuration, Functional, Network, Permissions
- Binary detection via `which` command and standard paths
- Version extraction using configurable regex patterns
- Config file existence checks with base_dir expansion
- Permission checks using std::fs::metadata on Unix

### WebSocket Server
- Broadcast-based architecture using tokio::sync::broadcast
- ControlPanelState manages connected clients and event distribution
- Event-driven architecture with ControlPanelEvent enum
- Support for multiple client connections with proper cleanup
- Ping/pong for connection health checks
- Channel-based subscription model

### WebSocket Event Types
- ToolStatusChanged, ToolInstallationUpdated
- WarningAdded, WarningResolved
- ValidationCompleted
- JobProgress, JobCompleted
- MCPServerUpdated, MCPSyncCompleted
- UsageUpdated

## Current Progress
- Phase 1: 5/5 tasks (100%) ✅ COMPLETE
- Phase 2: 2/5 tasks (40%) - Validation ✅, WebSocket ✅, Async Jobs pending, CLI Wrapper pending, MCP Registry pending
- Total: 7/23 tasks (30%)

## Next Steps
- Phase 2.3: Async Job System
- Phase 2.4: CLI Wrapper for skills.sh
- Phase 2.5: MCP Registry service
- Phase 3: Frontend Foundation
- Phase 4: Features integration
