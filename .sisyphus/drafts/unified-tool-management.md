# Draft: Unified Tool Management & Control Panel

## User's Goal

> "finish all this integrations and to make sure we have a single control panel to manage my entire local setup of llms/clis/codecs/skills/etc. check every cli install on the list, and map all possible settings/commands/folders/configuration styles/etc to a dictionary or specification that all of them can use, make sure you can validate/verify/build the options panel for them all. Map all the mcp configs,schemas,etc so they can be shared across services with one click, checkbox, and then connect to this api: https://skills.sh/docs to be able to list/install/setup skills automatically on any client repo/ or uninstall them."

---

## Current Implementation Status

### ✅ What's Working Well

**CLI Tools System** (8 tools tracked):
1. Claude Code - Full support (files, settings, MCP, agents, usage)
2. Gemini CLI - Full support
3. Codex CLI - Full support
4. OpenCode - Full support
5. GitHub Copilot - Partial (MCP, agents, usage only)
6. Cursor CLI - Full support
7. ESLint - Basic (files, settings, usage only)
8. Vite - Basic (files, settings, usage only)

**Detection System** (7 sources):
- which/where commands
- Homebrew
- npm global
- NVM (Node version manager)
- GitHub extensions
- Standard paths
- App bundles (macOS)

**Configuration Management**:
- Trait-based architecture (`CLIToolConfig` trait)
- Per-tool config handlers (Claude, Gemini, Codex, Cursor, etc.)
- Multi-format parsing (JSON, JSONC, YAML, TOML, Markdown)
- Lazy loading with caching
- SQLite persistence for preferences

**MCP Integration**:
- 3 transport types (Stdio, SSE, HTTP)
- Per-tool MCP management
- Claude Desktop import
- Project-level `.mcp.json` support

**Skills System**:
- Full CRUD operations
- Two scopes (project + user)
- Markdown with YAML frontmatter support
- Supporting files management

**UI/State Management**:
- Zustand stores
- Tab-based navigation
- shadcn/ui components
- Lazy loading patterns

---

## Gaps & Missing Features

### 1. ❌ No Unified Tool Schema/Dictionary

**Current**: Each tool has hardcoded config implementations
**Need**: Generic tool specification that all tools can implement

```typescript
// Missing: Universal Tool Specification
interface ToolSpecification {
  name: string;
  type: 'llm' | 'cli' | 'codec' | 'plugin' | 'service';

  // Installation
  installation_sources: InstallationSource[];

  // Configuration
  config_files: ConfigFileSpec[];
  config_format: 'json' | 'yaml' | 'toml' | 'custom';
  config_dir_pattern: string; // "~/.{tool}/" or "./.{tool}/"

  // Capabilities
  capabilities: ToolCapabilities;

  // Settings schema (dynamic)
  settings_schema: SettingsSchema[];

  // Commands (if applicable)
  available_commands: CommandSpec[];
}
```

### 2. ❌ Skills.sh API Connection Incomplete

**Research Finding**:
- ✅ Search API available (`/api/search`)
- ✅ Skill matching API available (`/api/skills/search`)
- ✅ Update check API available
- ❌ **NO programmatic install/uninstall API**
- Must wrap CLI: `npx skills add <source>@<name> --yes --global`

**What's needed**:
- CLI wrapper integration in Rust
- Search UI that calls skills.sh search API
- "One-click install" that executes CLI command
- Installation progress tracking
- Error handling for CLI failures

### 3. ❌ MCP Sharing Across Services Not Implemented

**Current**: Each tool manages MCP servers independently
**Need**: Cross-tool MCP sync

```typescript
// Missing: Universal MCP Registry
interface UniversalMCPRegistry {
  servers: Record<string, UniversalMCPServer>;

  syncTo(tool: CLIToolType): Promise<void>;
  syncFrom(tool: CLIToolType): Promise<void>;
  syncAll(): Promise<void>;
}

interface UniversalMCPServer {
  id: string; // Unique across all tools
  name: string;
  transport: MCPServerConfig;
  enabled_tools: CLIToolType[]; // Which tools use this server
  environment: Record<string, string>;
  settings: Record<string, any>;
}
```

**Use Case**: "One click, checkbox" to enable an MCP server across Claude, Gemini, Cursor, etc.

### 4. ❌ No Dynamic Form Generation

**Current**: Hand-coded forms per tool
**Need**: Schema-driven form rendering

```typescript
// Current approach (hand-coded):
// SettingsTab.tsx - switch statement per setting type

// Needed approach (schema-driven):
interface FormSchema {
  sections: FormSection[];
  validation?: z.ZodSchema;
}

function DynamicForm({ schema }: { schema: FormSchema }) {
  // Automatically render controls based on schema
  return schema.sections.map(section => renderSection(section));
}
```

### 5. ❌ Validation & Verification Missing

**Current**: No runtime validation, no verification commands
**Need**:
```typescript
interface ToolVerification {
  verifyInstallation(tool: CLIToolType): Promise<VerificationResult>;
  validateConfig(tool: CLIToolType): Promise<ValidationResult>;
  testConnection(tool: CLIToolType): Promise<ConnectionTest>;
}

interface VerificationResult {
  installed: boolean;
  version: string;
  path: string;
  executable: boolean;
}

interface ValidationResult {
  valid: boolean;
  errors: ValidationError[];
  warnings: ValidationWarning[];
}
```

### 6. ❌ No Tool Registry/Plugin System

**Current**: Hardcoded tool list in `CLIToolType` enum
**Need**: Dynamic tool registration

```rust
// Missing: Tool Registry
trait ToolRegistry {
    fn register_tool(spec: ToolSpecification) -> Result<()>;
    fn unregister_tool(tool_type: String) -> Result<()>;
    fn list_tools() -> Vec<CLIToolType>;
    fn get_tool(tool_type: CLIToolType) -> Option<ToolSpecification>;
}
```

**Benefit**: Add new tools without code changes (plugins, community contributions).

### 7. ⚠️ Partial TODOs Found

1. **MCP Status Checking** (`mcp.rs:676`):
   - `mcp_get_server_status()` returns empty HashMap
   - Need actual health checks

2. **Environment Variable Display** (`mcp.rs:374`):
   - `mcp_get()` doesn't show env vars in UI

3. **Toast Notifications** (`useApiCall.ts:65,84`):
   - Marked as `// TODO: Implement toast notification`
   - Errors don't show user-friendly messages

4. **Attachment Support** (`ClaudeCodeSession.tsx:860`):
   - `has_attachments: false // TODO: Add attachment support`

5. **Webview Navigation** (`WebviewPreview.tsx:69,73,249,258`):
   - Navigation features disabled, awaiting implementation

6. **Missing Widgets** (`widgets/index.ts:6`):
   - Only LSWidget and TodoWidget exist
   - Other widgets listed but not implemented

---

## Architecture Patterns to Build On

### ✅ Solid Foundation:

1. **Trait-Based Config System**
   - `CLIToolConfig` trait defines standard interface
   - Each tool implements trait with tool-specific logic
   - Clean separation of concerns

2. **Lazy Loading & Caching**
   - Metadata first, content on-demand
   - 30-60 second cache durations
   - Prevents concurrent loads

3. **Multi-Format Parsing**
   - ConfigParser supports JSON, JSONC, YAML, TOML, Markdown
   - All normalized to JSON for UI

4. **Zustand State Management**
   - Per-tool config state
   - Selector-based reactivity
   - Centralized error handling

### 🔧 Improvements Needed:

1. **Schema-Driven UI** (Critical for unified panel)
2. **Tool Registration System** (Critical for extensibility)
3. **Universal MCP Sync** (Critical for cross-tool management)
4. **Validation Layer** (Important for reliability)
5. **CLI Wrapper** (Needed for skills.sh integration)

---

## Key Findings from Research

### Skills.sh API Summary:

**Available Endpoints:**
- `GET /api/search?q=<query>&limit=10` - Search skills
- `POST /api/skills/search` - Match installed skills
- `POST /check-updates` - Check for updates

**NOT Available:**
- No programmatic install endpoint
- No programmatic uninstall endpoint
- No skill content API

**Integration Strategy:**
```rust
// Must wrap CLI commands
fn install_skill(repo: &str, name: &str) -> Result<()> {
    let cmd = Command::new("npx")
        .args(["skills", "add", &format!("{}@{}", repo, name), "--yes", "--global"]);
    cmd.status()?;
}
```

---

## User's Clarified Vision

### User's Answers:

1. **CLI Scope**: User-customizable registry (let users define their own tool specifications with JSON/YAML)
2. **Skills Integration**: Full automation (CLI wrapper in Rust, search UI, install button with progress tracking, error handling)
3. **MCP Sync**: One-click enable per tool (checkbox UI for Claude/Gemini/Cursor/etc.)
4. **UI Layout**: **Single dashboard** with:
   - **Overview section**: Status/warnings/updates/usage/auth/API key management/etc.
   - **Per-tool pages**: For each CLI tool, a single page to manage end-to-end
   - **Skills section**: Dedicated area for skills management
   - **MCP section**: Dedicated area for MCP servers
   - **Custom configs section**: Area for other configurations

---

## Refined Requirements

### 1. User-Customizable Tool Registry System

**What's needed:**
- Allow users to define their own tools via JSON/YAML specifications
- Tool specification schema that includes:
  - Metadata (name, type, version, website)
  - Installation sources (binary names, npm packages, homebrew formulas, etc.)
  - Configuration directories and file patterns
  - Capabilities (files, settings, MCP, agents, usage, etc.)
  - Settings schema (dynamic definitions)
  - Commands (available CLI commands to expose)
- Tool registration system (load user definitions at runtime)
- Tool validation (verify spec is valid before loading)

### 2. Skills.sh Full Integration

**What's needed:**
- CLI wrapper module in Rust:
  - Execute `npx skills` commands
  - Capture stdout/stderr
  - Parse JSON output
  - Handle progress tracking
- Search UI component:
  - Connect to `/api/search` endpoint
  - Display results with skill metadata
  - Show install count
  - Filter/sort functionality
- Install workflow:
  - "One-click install" button
  - Progress indicator (CLI output)
  - Error handling and display
  - Success notification
- Update checking:
  - Read `~/.agents/.skill-lock.json`
  - Call `/check-updates` API
  - Show "update available" badges
- Uninstall functionality:
  - Remove skill directory
  - Update skill-lock.json
  - Update UI state

### 3. MCP Cross-Tool Sync with One-Click Enable

**What's needed:**
- Universal MCP registry:
  - Central storage of MCP server definitions
  - Each server has list of tools that use it
  - Per-tool enable/disable flags
- Sync logic:
  - Read MCP servers from central registry
  - For each enabled tool: Apply to tool's config
  - Handle format differences (Claude vs Gemini vs Cursor vs Codex)
- UI for MCP management:
  - List all MCP servers
  - For each server: Show checkboxes for Claude, Gemini, Cursor, OpenCode, Codex
  - Enable/disable per-tool independently
  - Bulk actions: "Enable all", "Disable all"
- Validation:
  - Test MCP connection before enabling
  - Show server status (running/stopped/error)

### 4. Single Dashboard with Overview + Per-Tool Pages

**What's needed:**
- **Overview Dashboard** (main view):
  - System status (warnings, errors, updates)
  - Usage summary (token counts, costs by tool)
  - Authentication status (API keys, login status)
  - Quick actions (scan tools, check updates)
  - Navigation cards (click to open tool/skills/MCP pages)
- **Per-Tool Pages** (full management):
  - Tool metadata (name, version, install path, website)
  - Installation status (detected/not found)
  - Configuration files list with editor
  - Settings (dynamic form generation)
  - MCP servers (if supported, with sync from central registry)
  - Agents/commands (if supported, with execute button)
  - Usage analytics (charts, export data)
  - Actions (update tool, reinstall, open docs, run diagnostics)

### 5. Validation & Verification System

**What's needed:**
- Installation verification:
  - Check if tool binary exists and is executable
  - Verify version matches expected
  - Test basic functionality (--help or --version)
- Configuration validation:
  - Parse config files
  - Validate against schema
  - Check required fields
  - Test MCP server configs (can connect)
  - Validate skill formats (frontmatter, required fields)
- Health checking:
  - MCP server connectivity
  - Tool responsiveness
  - Config file permissions
- Diagnostics:
  - Comprehensive scan showing all issues
  - Fix suggestions (one-click fixes when possible)

### 6. Architecture Requirements

**Tool Specification Schema**:
```typescript
interface ToolSpecification {
  // Metadata
  id: string;
  name: string;
  type: 'llm' | 'cli' | 'codec' | 'service' | 'plugin';
  version: string;
  website: string;

  // Installation
  installation: {
    binary_names: string[];
    homebrew_formulas: string[];
    npm_packages: string[];
    nvm_packages: string[];
    gh_extensions: string[];
    custom_commands: CustomInstallCommand[];
  };

  // Configuration
  config: {
    base_dir: string;  // "~/.{id}/" or "./.{id}/"
    files: ConfigFileSpec[];
    settings_schema: SettingDefinition[];
  };

  // Capabilities
  capabilities: {
    files: boolean;
    settings: boolean;
    mcp_servers: boolean;
    agents: boolean;
    commands: boolean;
    usage: boolean;
  };
}
```

**Settings Schema** (from existing types):
```typescript
interface SettingDefinition {
  key: string;
  value_type: 'string' | 'number' | 'boolean' | 'array' | 'object' | 'select' | 'textarea';
  description: string;
  default: unknown;
  options: string[]; // For select
  readonly: boolean;
  validation?: ValidationRule;
}
```

---

## Implementation Options

### Option A: Incremental Evolution (Recommended)

1. Extract reusable components from current patterns
2. Add schema-driven form rendering
3. Create tool registration system
4. Build universal MCP sync
5. Integrate skills.sh CLI wrapper

**Pros:**
- Leverages existing solid architecture
- Backward compatible
- Lower risk
- Can deliver incrementally

**Cons:**
- Takes multiple iterations
- Some code duplication during transition

### Option B: Unified Rewrite

1. Design complete tool specification schema
2. Implement tool registry
3. Rewrite all config handlers to use schema
4. Build new control panel from scratch
5. Integrate all features

**Pros:**
- Clean architecture from start
- No legacy patterns
- Easier to maintain long-term

**Cons:**
- High risk
- Takes longer to deliver
- Breaks existing functionality

### Option C: Hybrid Approach

1. Build new unified system in parallel
2. Maintain legacy system during transition
3. Migrate tools incrementally
4. Deprecate old system when all tools migrated

**Pros:**
- Can test new architecture safely
- Gradual migration
- Can ship features as ready

**Cons:**
- More complex (two systems)
- Confusing for contributors
- Longer maintenance period

---

## Next Steps (Pending User Input)

1. Clarify scope of CLI tools to manage
2. Decide on skills.sh integration approach
3. Choose MCP sync strategy
4. Define control panel UX preference
5. Determine validation requirements
6. Select implementation approach (A/B/C)

After decisions, generate comprehensive work plan with:
- Detailed architecture
- Task breakdown
- Parallel execution strategy
- Verification approach
