# Architecture Specification - Unified Control Panel

## 1. System Architecture

### 1.1 High-Level Design

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              FRONTEND (Electron/Tauri)                       │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                         React 18 + TypeScript                          │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │  │
│  │  │   Dashboard │  │ Tool Pages  │  │   Skills    │  │    MCP      │  │  │
│  │  │   (Overview)│  │  (Per Tool) │  │   Panel     │  │   Panel     │  │  │
│  │  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  │  │
│  │         │                │                │                │         │  │
│  │         └────────────────┴────────────────┴────────────────┘         │  │
│  │                              │                                       │  │
│  │                    ┌─────────┴─────────┐                             │  │
│  │                    │  Zustand Stores   │                             │  │
│  │                    │  (Global State)   │                             │  │
│  │                    └─────────┬─────────┘                             │  │
│  │                              │                                       │  │
│  │         ┌────────────────────┼────────────────────┐                  │  │
│  │         ▼                    ▼                    ▼                  │  │
│  │  ┌─────────────┐      ┌─────────────┐      ┌─────────────┐          │  │
│  │  │  REST API   │      │  WebSocket  │      │  React Query│          │  │
│  │  │   Client    │      │   Client    │      │   (Cache)   │          │  │
│  │  └──────┬──────┘      └──────┬──────┘      └─────────────┘          │  │
│  │         │                    │                                       │  │
│  │         └────────────────────┘                                       │  │
│  │                              │                                       │  │
│  └──────────────────────────────┼───────────────────────────────────────┘  │
│                                 │                                           │
│                                 ▼                                           │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                     Tauri Bridge / Web Server                          │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
                                              │
                                              ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                              BACKEND (Rust)                                  │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                         Axum Web Server                                │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │  │
│  │  │ REST Routes │  │ WebSocket   │  │  Middleware │  │   Static    │  │  │
│  │  │   (API)     │  │  Handler    │  │  (Auth/CORS)│  │    Files    │  │  │
│  │  └──────┬──────┘  └──────┬──────┘  └─────────────┘  └─────────────┘  │  │
│  │         │                │                                            │  │
│  │         └────────────────┘                                            │  │
│  │                    │                                                  │  │
│  │         ┌─────────┴─────────┐                                         │  │
│  │         │  Service Layer    │                                         │  │
│  │         │  (Business Logic) │                                         │  │
│  │         └─────────┬─────────┘                                         │  │
│  │                   │                                                   │  │
│  │  ┌────────────────┼────────────────┬────────────────┐                │  │
│  │  ▼                ▼                ▼                ▼                │  │
│  │ ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐ │  │
│  │ │  Tool    │  │   MCP    │  │   CLI    │  │  Async   │  │ Validation│ │  │
│  │ │ Registry │  │ Registry │  │ Wrapper  │  │  Jobs    │  │  Engine   │ │  │
│  │ └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘ │  │
│  │      │             │             │             │             │       │  │
│  │      └─────────────┴─────────────┴─────────────┴─────────────┘       │  │
│  │                              │                                        │  │
│  │                    ┌─────────┴─────────┐                              │  │
│  │                    │  Data Layer       │                              │  │
│  │                    └─────────┬─────────┘                              │  │
│  │                              │                                        │  │
│  │         ┌────────────────────┼────────────────────┐                   │  │
│  │         ▼                    ▼                    ▼                   │  │
│  │  ┌─────────────┐      ┌─────────────┐      ┌─────────────┐           │  │
│  │  │   SQLite    │      │  File System│      │ External    │           │  │
│  │  │  (rusqlite) │      │  (Configs)  │      │  Processes  │           │  │
│  │  └─────────────┘      └─────────────┘      └─────────────┘           │  │
│  │                                                                       │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 1.2 Component Interactions

#### Tool Registration Flow
```
1. User uploads JSON/YAML tool spec
2. Frontend validates schema (client-side)
3. POST /api/tools with spec
4. Backend validates (server-side)
5. Store in SQLite + file system
6. Broadcast via WebSocket: tool_registered
7. Frontend updates store
8. Tool appears in dashboard
```

#### Skills Install Flow
```
1. User clicks "Install" on skill
2. POST /api/skills/install
3. Backend creates async job
4. Returns jobId immediately
5. Frontend subscribes to job updates
6. Backend executes: npx skills add <source>@<name>
7. Progress streamed via WebSocket
8. Job completes/fails
9. Frontend shows result
```

#### MCP Sync Flow
```
1. User enables MCP server for tool
2. PUT /api/mcp/servers/:id/tools/:toolId
3. Backend updates enablement table
4. Triggers sync job
5. Backend reads server config
6. Converts to tool-specific format
7. Writes to tool's config file
8. Broadcasts: mcp_synced
9. Frontend updates UI
```

---

## 2. Service Specifications

### 2.1 Tool Registry Service

**Responsibilities**:
- Load and manage tool specifications
- Validate tool specs against schema
- Hot-reload user-defined tools
- Provide tool metadata to other services

**Interface**:
```rust
#[async_trait]
pub trait ToolRegistry: Send + Sync {
    /// Load all tools (builtin + user-defined)
    async fn load_tools(&self) -> Result<Vec<ToolSpec>>;
    
    /// Get specific tool by ID
    async fn get_tool(&self, tool_id: &str) -> Result<Option<ToolSpec>>;
    
    /// Register new user-defined tool
    async fn register_tool(&self, spec: ToolSpec) -> Result<String>;
    
    /// Update tool spec
    async fn update_tool(&self, tool_id: &str, spec: ToolSpec) -> Result<()>;
    
    /// Unregister user-defined tool
    async fn unregister_tool(&self, tool_id: &str) -> Result<()>;
    
    /// Validate tool specification
    async fn validate_spec(&self, spec: &ToolSpec) -> Result<ValidationResult>;
    
    /// Watch for file changes (hot-reload)
    async fn watch_user_tools(&self) -> Result<()>;
}
```

**Storage**:
- Builtin tools: Hardcoded in Rust
- User tools: `~/.opcode/tools/*.json` or `*.yaml`
- Metadata: SQLite `tool_specifications` table

### 2.2 MCP Registry Service

**Responsibilities**:
- Store universal MCP server definitions
- Manage cross-tool enablement
- Convert between tool-specific formats
- Test MCP connections

**Interface**:
```rust
#[async_trait]
pub trait MCPRegistry: Send + Sync {
    /// List all MCP servers
    async fn list_servers(&self) -> Result<Vec<MCPServer>>;
    
    /// Add new MCP server
    async fn add_server(&self, config: MCPServerConfig) -> Result<String>;
    
    /// Update MCP server
    async fn update_server(&self, id: &str, config: MCPServerConfig) -> Result<()>;
    
    /// Remove MCP server
    async fn remove_server(&self, id: &str) -> Result<()>;
    
    /// Enable/disable for specific tool
    async fn set_tool_enablement(
        &self, 
        server_id: &str, 
        tool_id: &str, 
        enabled: bool
    ) -> Result<()>;
    
    /// Get enablement status for all tools
    async fn get_tool_enablement(&self, server_id: &str) -> Result<Vec<ToolEnablement>>;
    
    /// Sync server config to specific tool
    async fn sync_to_tool(&self, server_id: &str, tool_id: &str) -> Result<()>;
    
    /// Sync all enabled servers to all tools
    async fn sync_all(&self) -> Result<SyncResult>;
    
    /// Test MCP server connection
    async fn test_connection(&self, server_id: &str) -> Result<ConnectionTest>;
}
```

**Format Conversion**:
```rust
// Universal format (internal)
struct MCPServerConfig {
    name: String,
    transport: MCPTransport, // Stdio | Sse | Http
    enabled: bool,
    env: HashMap<String, String>,
}

// Claude format (external)
// ~/.claude/mcp.json
{
  "mcpServers": {
    "server-name": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/path"],
      "env": { "KEY": "value" }
    }
  }
}

// Gemini format (external)
// ~/.gemini/settings.json
{
  "mcpServers": {
    "server-name": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/path"]
    }
  }
}

// Codex format (external)
// ~/.codex/config.toml
[mcp_servers.server-name]
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem", "/path"]
```

### 2.3 CLI Wrapper Service (Skills.sh)

**Responsibilities**:
- Execute skills.sh CLI commands
- Stream progress to frontend
- Handle timeouts and retries
- Parse output and errors

**Interface**:
```rust
#[async_trait]
pub trait CLIWrapper: Send + Sync {
    /// Execute command and stream output
    async fn execute(
        &self,
        command: CLICommand,
        job_id: &str,
    ) -> Result<CommandResult>;
    
    /// Search skills.sh
    async fn search_skills(&self, query: &str) -> Result<Vec<SkillInfo>>;
    
    /// Install skill
    async fn install_skill(
        &self,
        source: &str,
        name: &str,
        scope: InstallScope,
    ) -> Result<String>; // Returns job_id
    
    /// Uninstall skill
    async fn uninstall_skill(
        &self,
        name: &str,
        scope: InstallScope,
    ) -> Result<String>; // Returns job_id
    
    /// Check for updates
    async fn check_updates(&self) -> Result<Vec<SkillUpdate>>;
    
    /// Get installed skills
    async fn list_installed(&self) -> Result<Vec<InstalledSkill>>;
}

pub enum CLICommand {
    Search { query: String },
    Install { source: String, name: String, scope: InstallScope },
    Uninstall { name: String, scope: InstallScope },
    List,
    CheckUpdates,
}
```

**Progress Streaming**:
```rust
// Stream progress via WebSocket
pub struct CommandProgress {
    job_id: String,
    stage: String,      // "downloading", "installing", "verifying"
    progress: u8,       // 0-100
    message: String,    // Human-readable status
    timestamp: DateTime<Utc>,
}
```

### 2.4 Async Job Service

**Responsibilities**:
- Queue and execute long-running jobs
- Track progress and status
- Handle job lifecycle (create, start, complete, fail, cancel)
- Persist job state for crash recovery

**Interface**:
```rust
#[async_trait]
pub trait JobService: Send + Sync {
    /// Create new job
    async fn create_job(&self, job_type: JobType, params: Value) -> Result<String>;
    
    /// Get job status
    async fn get_job(&self, job_id: &str) -> Result<Option<Job>>;
    
    /// List jobs (with filters)
    async fn list_jobs(&self, filter: JobFilter) -> Result<Vec<Job>>;
    
    /// Update job progress
    async fn update_progress(&self, job_id: &str, progress: u8) -> Result<()>;
    
    /// Complete job
    async fn complete_job(&self, job_id: &str, result: Value) -> Result<()>;
    
    /// Fail job
    async fn fail_job(&self, job_id: &str, error: String) -> Result<()>;
    
    /// Cancel job
    async fn cancel_job(&self, job_id: &str) -> Result<()>;
    
    /// Subscribe to job updates
    async fn subscribe(&self, job_id: &str) -> Result<JobStream>;
}

pub struct Job {
    id: String,
    job_type: JobType,
    status: JobStatus,
    params: Value,
    progress: u8,
    result: Option<Value>,
    error: Option<String>,
    created_at: DateTime<Utc>,
    started_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
}

pub enum JobType {
    SkillsInstall,
    SkillsUninstall,
    ToolValidation,
    MCPSync,
    ConfigSync,
}

pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}
```

### 2.5 Validation Engine

**Responsibilities**:
- Validate tool installations
- Check configuration files
- Test functional behavior
- Verify network connectivity
- Report validation results

**Interface**:
```rust
#[async_trait]
pub trait ValidationEngine: Send + Sync {
    /// Run full validation on tool
    async fn validate_tool(&self, tool_id: &str) -> Result<ValidationReport>;
    
    /// Run specific validation type
    async fn validate_specific(
        &self,
        tool_id: &str,
        validation_type: ValidationType,
    ) -> Result<ValidationResult>;
    
    /// Schedule periodic validation
    async fn schedule_validation(&self, tool_id: &str, interval: Duration) -> Result<()>;
    
    /// Get validation history
    async fn get_validation_history(&self, tool_id: &str) -> Result<Vec<ValidationResult>>;
}

pub struct ValidationReport {
    tool_id: String,
    overall_status: ValidationStatus,
    results: Vec<ValidationResult>,
    checked_at: DateTime<Utc>,
}

pub struct ValidationResult {
    validation_type: ValidationType,
    status: ValidationStatus,
    details: String,
    metadata: Option<Value>,
}

pub enum ValidationType {
    Installation,  // Binary exists, version correct
    Configuration, // Config files valid, required fields present
    Functional,    // Tool executes, dry-run passes
    Network,       // Network connectivity (if required)
    Permissions,   // File permissions correct
}

pub enum ValidationStatus {
    Valid,
    Invalid,
    Warning,
    Skipped,
}
```

---

## 3. Data Models

### 3.1 Tool Specification

```typescript
interface ToolSpecification {
  // Metadata
  id: string;                    // Unique identifier
  name: string;                  // Display name
  type: 'llm' | 'cli' | 'codec' | 'service' | 'plugin';
  version: string;               // Spec version
  description?: string;
  website?: string;
  icon?: string;                 // Icon URL or emoji
  
  // Source
  source: 'builtin' | 'user_defined';
  created_at: string;
  updated_at: string;
  
  // Installation
  installation: InstallationConfig;
  
  // Configuration
  config: ConfigSpec;
  
  // Capabilities
  capabilities: ToolCapabilities;
  
  // Settings schema
  settings_schema?: SettingSchema[];
}

interface InstallationConfig {
  // Detection methods (OR logic - any match = installed)
  binary_names?: string[];           // e.g., ["claude", "claude.exe"]
  homebrew_formulas?: string[];      // e.g., ["claude-code"]
  npm_packages?: string[];           // e.g., ["@anthropic-ai/claude-code"]
  nvm_packages?: string[];
  gh_extensions?: string[];          // e.g., ["github/gh-copilot"]
  standard_paths?: string[];         // e.g., ["/usr/local/bin/claude"]
  
  // Version extraction
  version_args?: string[];           // e.g., ["--version"]
  version_pattern?: string;          // Regex to extract version
  
  // Custom commands
  custom_detection?: CustomDetectionCommand[];
}

interface ConfigSpec {
  // Base directory pattern
  base_dir: string;                  // e.g., "~/.{tool_id}/" or "./.{tool_id}/"
  
  // Config files
  files?: ConfigFile[];
  
  // Settings
  settings_format?: 'json' | 'yaml' | 'toml';
  settings_path?: string;            // Relative to base_dir
}

interface ConfigFile {
  path: string;                      // Relative to base_dir
  type: 'json' | 'yaml' | 'toml' | 'markdown';
  description?: string;
  required?: boolean;
  schema?: object;                   // JSON Schema for validation
}

interface ToolCapabilities {
  files: boolean;        // Can read/write config files
  settings: boolean;     // Has structured settings
  mcp_servers: boolean;  // Supports MCP servers
  agents: boolean;       // Has custom agents/commands
  commands: boolean;     // Can execute CLI commands
  usage: boolean;        // Tracks usage analytics
}

interface SettingSchema {
  key: string;
  type: 'string' | 'number' | 'boolean' | 'array' | 'object' | 'select' | 'textarea';
  label: string;
  description?: string;
  default?: unknown;
  options?: string[];    // For select type
  required?: boolean;
  validation?: ValidationRule;
}

interface ValidationRule {
  min?: number;
  max?: number;
  pattern?: string;      // Regex
  enum?: string[];
  custom?: string;       // Custom validation function (JavaScript)
}
```

### 3.2 MCP Server

```typescript
interface MCPServer {
  id: string;
  name: string;
  description?: string;
  
  // Transport configuration
  transport: MCPTransport;
  
  // Global enablement
  is_enabled_globally: boolean;
  
  // Tool-specific enablement
  tool_enablement: ToolEnablement[];
  
  // Environment variables
  env: Record<string, string>;
  
  // Metadata
  created_at: string;
  updated_at: string;
}

type MCPTransport = 
  | { type: 'stdio'; command: string; args: string[] }
  | { type: 'sse'; url: string; headers: Record<string, string> }
  | { type: 'http'; url: string; headers: Record<string, string> };

interface ToolEnablement {
  tool_id: string;
  is_enabled: boolean;
  config_override?: Partial<MCPTransport>;
}
```

### 3.3 Async Job

```typescript
interface AsyncJob {
  id: string;
  job_type: 'skills_install' | 'skills_uninstall' | 'tool_validation' | 'mcp_sync' | 'config_sync';
  status: 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';
  
  // Parameters
  params: Record<string, unknown>;
  
  // Progress
  progress: number;  // 0-100
  current_stage?: string;
  stage_message?: string;
  
  // Result
  result?: unknown;
  error_message?: string;
  
  // Timestamps
  created_at: string;
  started_at?: string;
  completed_at?: string;
  cancelled_at?: string;
}
```

### 3.4 Validation Result

```typescript
interface ValidationResult {
  id: string;
  tool_id: string;
  validation_type: 'installation' | 'configuration' | 'functional' | 'network' | 'permissions';
  status: 'valid' | 'invalid' | 'warning' | 'skipped';
  
  // Details
  message: string;
  details?: Record<string, unknown>;
  
  // Metadata
  checked_at: string;
  duration_ms: number;
}

interface ValidationReport {
  tool_id: string;
  overall_status: 'valid' | 'invalid' | 'warning';
  results: ValidationResult[];
  summary: {
    total: number;
    valid: number;
    invalid: number;
    warning: number;
    skipped: number;
  };
  checked_at: string;
}
```

---

## 4. WebSocket Protocol

### 4.1 Connection

```
Client connects to: ws://localhost:8080/ws

Handshake:
1. Client sends authentication (if required)
2. Server accepts connection
3. Client subscribes to channels
```

### 4.2 Message Format

```typescript
// Client -> Server
interface ClientMessage {
  type: 'subscribe' | 'unsubscribe' | 'ping';
  channel?: string;     // For subscribe/unsubscribe
  payload?: unknown;    // Optional data
}

// Server -> Client
interface ServerMessage {
  type: 'event' | 'error' | 'pong';
  channel?: string;     // Event channel
  event?: string;       // Event name
  payload?: unknown;    // Event data
  timestamp: string;    // ISO 8601
}
```

### 4.3 Event Channels

#### Dashboard Channel
```typescript
// Subscribe: { type: 'subscribe', channel: 'dashboard' }

// Events:
interface ToolStatusChangedEvent {
  event: 'tool_status_changed';
  payload: {
    tool_id: string;
    status: 'installed' | 'not_installed' | 'invalid';
    version?: string;
  };
}

interface WarningAddedEvent {
  event: 'warning_added';
  payload: SystemWarning;
}

interface WarningResolvedEvent {
  event: 'warning_resolved';
  payload: { warning_id: string };
}

interface UsageUpdatedEvent {
  event: 'usage_updated';
  payload: UsageSummary;
}
```

#### Jobs Channel
```typescript
// Subscribe: { type: 'subscribe', channel: 'jobs' }
// Or: { type: 'subscribe', channel: 'jobs:job_id' } for specific job

// Events:
interface JobProgressEvent {
  event: 'job_progress';
  payload: {
    job_id: string;
    progress: number;
    stage: string;
    message: string;
  };
}

interface JobCompletedEvent {
  event: 'job_completed';
  payload: {
    job_id: string;
    result: unknown;
  };
}

interface JobFailedEvent {
  event: 'job_failed';
  payload: {
    job_id: string;
    error: string;
  };
}
```

#### MCP Channel
```typescript
// Subscribe: { type: 'subscribe', channel: 'mcp' }

// Events:
interface MCPServerUpdatedEvent {
  event: 'mcp_server_updated';
  payload: MCPServer;
}

interface MCPSyncCompletedEvent {
  event: 'mcp_sync_completed';
  payload: {
    server_id: string;
    tool_id: string;
    success: boolean;
  };
}
```

---

## 5. Error Handling

### 5.1 Error Categories

```rust
pub enum AppError {
    // Validation errors
    InvalidToolSpec { details: String },
    InvalidConfig { tool_id: String, path: String, reason: String },
    
    // Runtime errors
    ToolNotFound { tool_id: String },
    ToolNotInstalled { tool_id: String },
    MCPServerNotFound { server_id: String },
    JobNotFound { job_id: String },
    
    // Execution errors
    CommandFailed { command: String, exit_code: i32, stderr: String },
    Timeout { operation: String, duration: Duration },
    
    // External service errors
    SkillsShError { message: String },
    NetworkError { url: String, reason: String },
    
    // Internal errors
    DatabaseError { reason: String },
    SerializationError { reason: String },
    InternalError { reason: String },
}
```

### 5.2 Error Response Format

```typescript
interface ErrorResponse {
  error: {
    code: string;           // Machine-readable error code
    message: string;        // Human-readable message
    details?: unknown;      // Additional context
    request_id?: string;    // For tracking
  };
}

// Example: Tool not found
{
  "error": {
    "code": "TOOL_NOT_FOUND",
    "message": "Tool 'unknown-tool' not found in registry",
    "details": { "tool_id": "unknown-tool" },
    "request_id": "req_12345"
  }
}
```

---

## 6. Security Considerations

### 6.1 Tool Spec Validation

- JSON Schema validation before storing
- Sanitize all string inputs
- Prevent path traversal in config paths
- Validate file permissions before reading/writing

### 6.2 Command Execution

- Whitelist allowed commands (configurable)
- Timeout all external commands
- Run commands with restricted permissions where possible
- Log all command executions

### 6.3 Secrets Management

- Encrypt API keys at rest
- Mask secrets in UI (show ****)
- Use OS keychain where available
- Never log secrets

### 6.4 File System Access

- Validate all paths before access
- Prevent access outside allowed directories
- Use canonical paths to avoid traversal
- Audit file operations

---

## 7. Performance Considerations

### 7.1 Caching Strategy

```
Level 1: In-memory (Rust)
- Tool specifications
- MCP server configs
- Validation results (short TTL)

Level 2: SQLite
- Tool metadata
- Job history
- Validation history

Level 3: File System
- Config files
- User-defined tool specs
```

### 7.2 Pagination & Limits

- API list endpoints: Default 20, max 100 items
- WebSocket: Rate limit events (max 100/sec per client)
- Jobs: Max 10 concurrent jobs per type
- Validation: Stagger checks to avoid spikes

### 7.3 Lazy Loading

- Tool details: Load on demand
- Config files: List metadata first, content on click
- Validation history: Paginated
- MCP server status: Cached for 30 seconds

---

## 8. Deployment & Operations

### 8.1 Database Migrations

```rust
// Using refinery or similar
embed_migrations!("./migrations");

// On startup
pub async fn run_migrations(pool: &Pool) -> Result<()> {
    let mut conn = pool.get().await?;
    migrations::runner().run_async(&mut conn).await?;
    Ok(())
}
```

### 8.2 Configuration

```yaml
# ~/.opcode/config.yaml
server:
  port: 8080
  host: "127.0.0.1"
  
websocket:
  enabled: true
  max_connections: 100
  
jobs:
  max_concurrent: 10
  timeout_seconds: 300
  retry_attempts: 3
  
validation:
  enabled: true
  interval_minutes: 30
  
skills:
  cli_path: "npx"
  timeout_seconds: 600
  
logging:
  level: "info"
  file: "~/.opcode/logs/server.log"
```

### 8.3 Monitoring

- Health check endpoint: `GET /health`
- Metrics endpoint: `GET /metrics` (Prometheus format)
- Structured logging (JSON)
- Error tracking integration (optional)

---

**Next**: See `.sisyphus/specs/api.md` for detailed API specifications.
