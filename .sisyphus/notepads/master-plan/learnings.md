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

## Next Steps
- Phase 2: Core Backend (Validation, WebSocket, Async Jobs, MCP Registry)
- Phase 3: Frontend Foundation
- Phase 4: Features integration
