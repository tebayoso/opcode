# Changelog

All notable changes to Opcode will be documented in this file.

## [0.3.0] - 2025-01-15

### Major Features

#### Plugin Management System
A complete plugin ecosystem has been integrated into Opcode, bringing the full power of Claude Code's plugin marketplace directly into the desktop application.

- **Plugin Discovery**: Browse all available plugins from the official Claude marketplace directly within the app
- **Local Cache Integration**: Seamlessly reads from Claude's local marketplace cache (`~/.claude/plugins/marketplaces/`) for instant access to plugin metadata
- **Plugin Installation**: Install, enable, and disable plugins with a single click
- **Marketplace Management**: View and manage registered plugin marketplaces
- **Three-Tab Interface**:
  - **Discover**: Browse all available plugins with descriptions, authors, and tags
  - **Installed**: Manage your currently installed plugins
  - **Marketplaces**: Configure plugin sources and view marketplace statistics

#### Skills Management System
Full CRUD operations for Claude Code skills, allowing users to create, edit, and manage custom skills.

- **Skills Editor**: Create and modify skills with syntax-highlighted YAML/Markdown editing
- **File Management**: Read, write, and delete skill files directly from the app
- **Skills Browser**: View all available skills with their configurations
- **Integration**: Skills sync with Claude Code's native skill system

#### Claude Memories Editor
A dedicated panel for viewing and editing Claude's memory files, providing transparency and control over persistent context.

- **Memory Browser**: View all stored memories across projects
- **Inline Editing**: Edit memory content directly in the app
- **Memory Management**: Create new memories and delete outdated ones

#### MCP (Model Context Protocol) Servers Tab
Enhanced MCP server management moved to a dedicated Settings tab for better organization.

- **Server List**: View all configured MCP servers with status indicators
- **Add Server**: Easily add new MCP servers with guided configuration
- **Server Status**: Real-time status checking for MCP server connectivity
- **Configuration Management**: Edit and remove server configurations

### Web Server Mode Enhancements

#### Mobile & Remote Access
The web server mode has been significantly improved for mobile browser access:

- **WebSocket Protocol Fix**: Automatic detection of HTTPS connections to use `wss://` protocol, fixing connectivity issues when accessing via ngrok or other HTTPS tunnels
- **Claude Installations Endpoint**: New `/api/settings/claude/installations` endpoint for web mode
- **Debug Command**: Added `just debug` command for troubleshooting Claude binary detection

### Cross-Platform Improvements

#### Windows Support
- Added Windows support for Claude binary detection
- Conditional compilation for Windows and Unix-like systems
- Platform-specific path handling

#### NVM Integration
- Improved Claude binary detection for active NVM environments
- Support for `NVM_BIN` environment variable detection
- Prioritizes currently active NVM environment over other installations
- Maintains backward compatibility with existing detection methods

### Bug Fixes

#### Project Path Display
Fixed a critical bug where project paths containing hyphens were incorrectly displayed:
- Projects like `~/projects/flipside/data-discovery` were showing as `~/projects/flipside/data/discovery`
- Root cause: Only the first line of JSONL session files was checked for `cwd` field
- Solution: Now checks up to 10 lines for valid, non-empty `cwd` values
- Added comprehensive unit tests covering the bug scenario and edge cases

#### IME Composition Handling
Improved input handling for users of CJK input methods and other IME systems:
- Added IME composition state tracking to prevent premature submission during input composition
- Handles `onCompositionStart`/`onCompositionEnd` events in text inputs and textareas
- Replaced `onKeyPress` with `onKeyDown` for better IME interaction handling
- Fixed in: AgentExecution, ClaudeCodeSession fork dialog, TimelineNavigator, WebviewPreview URL input, FloatingPromptInput

#### Message Scroll Behavior
- Implemented dual-phase scrolling: virtualizer positioning + native scroll to bottom
- Reduced excessive bottom padding from `pb-40` to `pb-20` for better viewport usage
- Unified scroll behavior across auto-scroll, history loading, and manual scroll
- Fixed issue where streamed content bottom couldn't reach viewport

#### Theme Improvements
- Fixed light theme code block backgrounds to use white
- Fixed light theme bash block backgrounds to use white

### Code Quality & Developer Experience

#### ESLint 9 Migration
- Migrated to ESLint 9 with flat config format
- TypeScript and React plugin integration
- Reduced strict rules for practical development workflow

#### Project Rename
- Completed rename from "Claudia" to "Opcode" throughout codebase
- Updated binary names, console messages, and documentation
- Updated web server binary from `claudia-web` to `opcode-web`

#### Dependency Fixes
- Pinned `image` crate to 0.25.1 to avoid edition2024 requirement
- Resolved TypeScript compilation errors for web/Tauri compatibility
- Fixed missing `installation_type` field in ClaudeInstallation struct

### Settings Page Redesign
The Settings page now features 12 organized tabs:
1. General
2. Models
3. Privacy
4. Claude Installation
5. Sessions
6. Slash Commands
7. Skills
8. Agents
9. Checkpoints
10. Memories
11. Plugins
12. MCP

### Technical Details

#### New Rust Modules
- `src-tauri/src/commands/plugins.rs` - Plugin management backend
- `src-tauri/src/commands/skills.rs` - Skills management backend

#### New React Components
- `src/components/PluginManager.tsx` - Plugin management UI
- `src/components/SkillsManager.tsx` - Skills management UI
- `src/components/ClaudeMemoriesPanel.tsx` - Memories editor

#### API Changes
- Added plugin management endpoints: `plugins_list_marketplaces`, `plugins_fetch_marketplace`, `plugins_install`, `plugins_uninstall`, etc.
- Added skills endpoints: `skills_list`, `skill_get`, `skill_save`, `skill_delete`, `skill_read_file`, `skill_save_file`, `skill_delete_file`
- Added web server endpoints for Claude installations

---

## [0.2.1] - Previous Release

- Version bump and configuration sync
- Initial agent editing capabilities
- README updates and documentation improvements

## [0.2.0] - Previous Release

- Core Opcode functionality
- Agent management system
- Session checkpoints and timeline
- MCP server integration
- Usage analytics

---

### Upgrade Notes

This release is fully backward compatible. No migration steps required.

### Known Issues

- Web server mode is limited to single concurrent session per connection
- Process cancellation in web mode requires additional implementation
- stderr handling not yet fully implemented in web mode

### Contributors

Built with Claude Code by the Opcode team.
