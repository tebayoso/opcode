# Unified Control Panel - Documentation

## Overview

The Unified Control Panel is a comprehensive management interface for opcode that consolidates control of CLI tools, MCP servers, and skills in a single dashboard.

## Features

### 1. Tool Management
- **8 Predefined Tools**: Claude, Gemini, Codex, OpenCode, Cursor, Copilot, ESLint, Vite
- **Custom Tool Registration**: Add your own tools via JSON/YAML specifications
- **Installation Detection**: Automatic detection via which, npm, homebrew, gh extensions
- **Version Tracking**: Extract and display version information
- **Capability Management**: Files, settings, MCP servers, agents, commands, usage

### 2. Skills.sh Integration
- **Search**: Find skills in the skills.sh registry
- **Install**: One-click installation with progress tracking
- **Uninstall**: Remove skills with cleanup
- **List**: View installed skills (global and project scope)
- **Updates**: Check for outdated skills

### 3. MCP Registry
- **Universal MCP Management**: Manage MCP servers across all tools
- **Transport Types**: Support for stdio, sse, http
- **Per-Tool Enablement**: Enable/disable servers per tool
- **Connection Testing**: Verify server connectivity
- **Cross-Tool Sync**: Sync MCP configuration across tools

### 4. Real-Time Updates
- **WebSocket Server**: Live updates for status changes
- **Progress Tracking**: Monitor async job progress
- **Event Types**:
  - ToolStatusChanged
  - ToolInstallationUpdated
  - WarningAdded/Resolved
  - ValidationCompleted
  - JobProgress/Completed
  - MCPServerUpdated
  - UsageUpdated

## Architecture

### Backend (Rust)

#### Database Schema
```sql
-- Core tables
tool_specifications      -- Tool metadata and specs
tool_installations       -- Installation status and paths
mcp_servers             -- Universal MCP server registry
mcp_tool_enablement     -- Cross-tool MCP enablement
async_jobs              -- Async job queue
validation_results      -- Tool validation results
system_warnings         -- System warnings and alerts
```

#### Key Modules

**tool_registry/**
- `types.rs` - Core type definitions
- `registry.rs` - ToolRegistry trait and implementation
- `validation.rs` - ValidationEngine for tool checks
- `jobs.rs` - JobManager for async operations
- `mcp_registry.rs` - MCPRegistry for MCP management
- `skills_cli.rs` - SkillsCLI wrapper for skills.sh
- `websocket.rs` - WebSocket server for real-time updates
- `error.rs` - Error types

**commands/**
- `tool_registry.rs` - Tauri commands for tool registry

#### API Commands

**Tool Registry**
- `tool_registry_list_tools` - List all tools
- `tool_registry_get_tool` - Get tool details
- `tool_registry_register_tool` - Register new tool
- `tool_registry_update_tool` - Update tool spec
- `tool_registry_unregister_tool` - Remove tool
- `tool_registry_validate_tool` - Validate tool spec
- `tool_registry_run_validation` - Run full validation
- `tool_registry_get_installation` - Get installation info
- `tool_registry_update_installation` - Update installation

**Async Jobs**
- `tool_registry_create_job` - Create async job
- `tool_registry_get_job` - Get job details
- `tool_registry_list_jobs` - List jobs
- `tool_registry_cancel_job` - Cancel job

**Skills.sh**
- `skills_search` - Search skills
- `skills_install` - Install skill (creates job)
- `skills_uninstall` - Uninstall skill (creates job)
- `skills_list_installed` - List installed skills
- `skills_check_updates` - Check for updates

**MCP Registry**
- `mcp_registry_list_servers` - List MCP servers
- `mcp_registry_get_server` - Get server details
- `mcp_registry_add_server` - Add MCP server
- `mcp_registry_remove_server` - Remove server
- `mcp_registry_set_tool_enablement` - Enable/disable for tool
- `mcp_registry_test_connection` - Test server connection

### Frontend (React/TypeScript)

#### Components

**unified-dashboard/**
- `UnifiedDashboard.tsx` - Main dashboard component
- `UnifiedDashboardContext.tsx` - React context for state
- `UnifiedDashboardSidebar.tsx` - Navigation sidebar
- `RegisterToolDialog.tsx` - Tool registration dialog
- `ToastProvider.tsx` - Toast notification system
- `useApiError.ts` - Error handling hook

**Panels**
- `OverviewPanel.tsx` - Dashboard overview with stats
- `ToolsPanel.tsx` - Tool management
- `SkillsPanel.tsx` - Skills.sh integration
- `MCPPanel.tsx` - MCP server management
- `ConfigsPanel.tsx` - Configuration management

## Usage

### Accessing the Dashboard

The unified dashboard can be accessed from the main application. It provides a sidebar navigation with sections:

1. **Overview** - Dashboard with statistics and summaries
2. **Tools** - Manage all CLI tools
3. **Skills** - Search and manage Claude skills
4. **MCP Servers** - Manage Model Context Protocol servers
5. **Configs** - Configuration management

### Registering a Custom Tool

1. Navigate to **Tools** section
2. Click **Register Tool** button
3. Fill in the form:
   - Tool ID (unique, lowercase, no spaces)
   - Tool Name (display name)
   - Tool Type (LLM, CLI, Codec, Service, Plugin)
   - Description (optional)
   - Binary Name (command to execute)
   - Config Directory (where settings are stored)
4. Click **Register Tool**

### Installing Skills

1. Navigate to **Skills** section
2. Enter search query in the search box
3. Click **Search**
4. Find the skill you want to install
5. Click **Install** button
6. Monitor progress in the Jobs section

### Managing MCP Servers

1. Navigate to **MCP Servers** section
2. View existing servers or add new ones
3. Click **Test Connection** to verify connectivity
4. Use toggle switches to enable/disable for specific tools

## Configuration

### Tool Specification Format

```json
{
  "id": "my-tool",
  "name": "My Tool",
  "type": "cli",
  "source": "user_defined",
  "version": "1.0.0",
  "description": "Description of my tool",
  "installation": {
    "binary_names": ["my-tool"],
    "version_args": ["--version"],
    "version_pattern": "(\\d+\\.\\d+\\.\\d+)"
  },
  "config": {
    "base_dir": "~/.config/my-tool"
  },
  "capabilities": {
    "files": true,
    "settings": true,
    "mcp_servers": false,
    "agents": false,
    "commands": true,
    "usage": true
  }
}
```

### MCP Server Configuration

**Stdio Transport**
```json
{
  "name": "My Server",
  "transport_type": "stdio",
  "config": {
    "command": "/path/to/server",
    "args": ["--stdio"],
    "env": {
      "API_KEY": "secret"
    }
  }
}
```

**HTTP/SSE Transport**
```json
{
  "name": "My Server",
  "transport_type": "http",
  "config": {
    "url": "http://localhost:3000/sse",
    "headers": {
      "Authorization": "Bearer token"
    }
  }
}
```

## Development

### Adding New Features

1. **Backend**: Add Tauri command in `src-tauri/src/commands/tool_registry.rs`
2. **Types**: Update types in `src-tauri/src/tool_registry/types.rs`
3. **Frontend**: Create/update components in `src/components/unified-dashboard/`
4. **Integration**: Wire up in `UnifiedDashboardContext.tsx`

### Testing

Run tests with:
```bash
# Backend tests
cd src-tauri && cargo test

# Frontend tests
bun test
```

### Building

```bash
# Development
bun run tauri dev

# Production build
bun run tauri build
```

## Troubleshooting

### Common Issues

**Tool not detected**
- Check binary is in PATH
- Verify binary name matches specification
- Run validation to check installation

**Skills install fails**
- Ensure Node.js and npm are installed
- Check network connectivity
- Verify skills.sh CLI is accessible

**MCP connection fails**
- Verify server is running
- Check URL/port configuration
- Review server logs for errors

### Debug Mode

Enable debug logging:
```bash
RUST_LOG=debug bun run tauri dev
```

## API Reference

See inline documentation in source files for detailed API reference.

## Contributing

1. Follow existing code patterns
2. Add tests for new features
3. Update documentation
4. Submit pull request

## License

MIT License - See LICENSE file for details
