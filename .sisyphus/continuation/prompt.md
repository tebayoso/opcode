# Continuation Prompt - Unified Control Panel

**Use this prompt to continue the unified control panel work in a new session.**

---

## Context

This is a continuation of the unified tool management & control panel project for opcode. The planning phase is complete and implementation is ready to begin.

## Project Overview

**Goal**: Build a comprehensive unified control panel that manages the entire local development environment including:
- **8 Predefined CLI Tools**: Claude, Gemini, Codex, OpenCode, Cursor, Copilot, ESLint, Vite
- **Wild West Extensibility**: Users can define ANY tool via JSON/YAML specifications
- **Skills.sh Integration**: Full automation with search, install, uninstall, progress tracking
- **Universal MCP Registry**: Cross-tool MCP server sync with one-click enable/disable
- **Real-time Updates**: WebSocket-based status monitoring
- **Comprehensive Validation**: Binary existence, version, functional testing, config validation
- **New UI**: Built from scratch in Electron/Tauri, fully functional

## Architecture

**Backend (Rust)**:
- Axum web server with REST API and WebSocket
- SQLite database (rusqlite)
- Services: Tool Registry, MCP Registry, CLI Wrapper, Async Jobs, Validation Engine
- File system for configs and user-defined tool specs

**Frontend (React/TypeScript)**:
- Next.js App Router
- Zustand for state management
- React Query for server state
- shadcn/ui components
- WebSocket client for real-time updates

## Current Status

**Completed**:
- ✅ Comprehensive analysis of existing opcode codebase
- ✅ Architecture specification
- ✅ API specification
- ✅ Database schema design
- ✅ Phase 1 task breakdown

**Ready to Start**:
- Phase 1: Foundation (Database, Tool Registry, API, Frontend Shell)

## Key Documents

All planning documents are in `.sisyphus/`:

1. **`.sisyphus/plans/master-plan.md`** - Master work plan with full overview
2. **`.sisyphus/specs/architecture.md`** - Detailed architecture specification
3. **`.sisyphus/specs/api.md`** - Complete API specification
4. **`.sisyphus/tasks/phase-1.md`** - Phase 1 task breakdown

## User Requirements (Confirmed)

- **Full System**: All 8 tools from day 1 (not MVP approach)
- **Wild West**: ANY tool can be defined via JSON/YAML
- **New UI**: Built from scratch, replaces existing
- **Real-time**: WebSocket infrastructure
- **Robust CLI**: Async job system with progress, timeout, retry
- **Comprehensive Validation**: All checks (existence, version, functional, network)
- **Single Dashboard**: Overview + per-tool pages + skills panel + MCP panel + custom configs

## Immediate Next Steps

Start with **Phase 1: Foundation** (see `.sisyphus/tasks/phase-1.md`):

1. **Task 1.1**: Database schema migration (1 day)
   - Create migrations for 7 new tables
   - Test migrations

2. **Task 1.2**: Tool Specification schema (2 days)
   - Define ToolSpecification struct
   - Implement validation logic
   - Add JSON/YAML parsing

3. **Task 1.3**: Tool Registry service (3 days)
   - Implement ToolRegistry trait
   - Load predefined 8 tools
   - Load user-defined tools
   - Hot-reload with file watching

4. **Task 1.4**: REST API routes (2 days)
   - Implement all tool endpoints
   - Add error handling

5. **Task 1.5**: Frontend shell (3 days)
   - New layout with navigation
   - Route structure
   - State management setup

6. **Task 1.6**: Tool list page (2 days)
   - Display tools
   - Filter/search
   - Navigation to details

## Technical Stack

**Backend**:
- Rust 1.70+
- Axum (web framework)
- rusqlite (SQLite)
- tokio (async runtime)
- serde (serialization)
- jsonschema (validation)

**Frontend**:
- React 18
- TypeScript 5
- Next.js 14 (App Router)
- Zustand (state)
- TanStack Query (server state)
- shadcn/ui
- Tailwind CSS

## Important Notes

1. **Dual Mode Support**: Code must work in both Tauri (desktop) and web server mode
2. **Existing Codebase**: opcode already has CLI tools, MCP, and skills functionality - we're building ON TOP of it, not replacing entirely
3. **Test Strategy**: Tests after implementation (not TDD)
4. **File Locations**:
   - Backend: `src-tauri/src/`
   - Frontend: `src/`
   - Plans: `.sisyphus/`

## Commands to Know

```bash
# Development
bun run tauri dev          # Start Tauri dev mode
bun run dev                # Start frontend only
bun run check              # TypeScript + Cargo check
bun run lint               # ESLint check

# Backend tests
cd src-tauri && cargo test

# Database
sqlite3 ~/.opcode/data.db  # Access database directly
```

## Questions?

If unclear about requirements, refer to:
- `.sisyphus/plans/master-plan.md` for overview
- `.sisyphus/specs/architecture.md` for technical details
- `.sisyphus/specs/api.md` for API contracts

## Start Command

To begin implementation, run:
```bash
/start-work .sisyphus/plans/master-plan.md
```

Or start with Phase 1 specifically:
```bash
/start-work .sisyphus/tasks/phase-1.md
```

---

**Generated**: 2026-01-29  
**Status**: Ready for implementation  
**Next Milestone**: Complete Phase 1 (Foundation)
