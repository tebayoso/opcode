# API Specification - Unified Control Panel

## Base URL

- Development: `http://localhost:8080/api`
- Production: (TBD)

## Authentication

Currently no authentication required (local-only application).

## Response Format

All responses are JSON with the following structure:

```typescript
// Success
interface SuccessResponse<T> {
  data: T;
  meta?: {
    page?: number;
    per_page?: number;
    total?: number;
  };
}

// Error
interface ErrorResponse {
  error: {
    code: string;
    message: string;
    details?: unknown;
    request_id: string;
  };
}
```

## HTTP Status Codes

- `200 OK` - Success
- `201 Created` - Resource created
- `400 Bad Request` - Invalid request
- `404 Not Found` - Resource not found
- `409 Conflict` - Resource conflict
- `422 Unprocessable Entity` - Validation error
- `500 Internal Server Error` - Server error

---

## Tool Registry API

### List All Tools

```http
GET /tools
```

**Query Parameters:**
- `type` (optional): Filter by type (`llm`, `cli`, `codec`, `service`, `plugin`)
- `source` (optional): Filter by source (`builtin`, `user_defined`)
- `installed` (optional): Filter by installation status (`true`, `false`)

**Response:**
```typescript
interface ListToolsResponse {
  data: {
    tools: ToolSummary[];
  };
}

interface ToolSummary {
  id: string;
  name: string;
  type: string;
  source: 'builtin' | 'user_defined';
  is_installed: boolean;
  version?: string;
  capabilities: ToolCapabilities;
  status: 'valid' | 'invalid' | 'unknown';
}
```

**Example:**
```bash
curl http://localhost:8080/api/tools
curl http://localhost:8080/api/tools?type=llm&installed=true
```

---

### Get Tool Details

```http
GET /tools/:id
```

**Response:**
```typescript
interface GetToolResponse {
  data: {
    tool: ToolSpecification;
    installation?: ToolInstallation;
    validation?: ValidationReport;
  };
}
```

**Example:**
```bash
curl http://localhost:8080/api/tools/claude
```

---

### Register New Tool

```http
POST /tools
Content-Type: application/json
```

**Request Body:**
```typescript
interface RegisterToolRequest {
  spec: ToolSpecification;  // Full tool specification
}
```

**Response:**
```typescript
interface RegisterToolResponse {
  data: {
    tool_id: string;
    message: string;
  };
}
```

**Example:**
```bash
curl -X POST http://localhost:8080/api/tools \
  -H "Content-Type: application/json" \
  -d '{
    "spec": {
      "id": "my-custom-tool",
      "name": "My Custom Tool",
      "type": "cli",
      "installation": {
        "binary_names": ["my-tool"],
        "version_args": ["--version"]
      },
      "config": {
        "base_dir": "~/.my-tool/",
        "files": [{"path": "config.json", "type": "json"}]
      },
      "capabilities": {
        "files": true,
        "settings": true,
        "mcp_servers": false,
        "agents": false,
        "commands": true,
        "usage": false
      }
    }
  }'
```

---

### Update Tool

```http
PUT /tools/:id
Content-Type: application/json
```

**Request Body:**
```typescript
interface UpdateToolRequest {
  spec: Partial<ToolSpecification>;
}
```

**Response:** `200 OK`

**Example:**
```bash
curl -X PUT http://localhost:8080/api/tools/my-custom-tool \
  -H "Content-Type: application/json" \
  -d '{
    "spec": {
      "name": "My Custom Tool (Updated)"
    }
  }'
```

---

### Unregister Tool

```http
DELETE /tools/:id
```

**Note:** Only user-defined tools can be unregistered.

**Response:** `204 No Content`

**Example:**
```bash
curl -X DELETE http://localhost:8080/api/tools/my-custom-tool
```

---

### Validate Tool

```http
POST /tools/:id/validate
```

**Response:**
```typescript
interface ValidateToolResponse {
  data: ValidationReport;
}
```

**Example:**
```bash
curl -X POST http://localhost:8080/api/tools/claude/validate
```

---

### Get Tool Validation History

```http
GET /tools/:id/validation
```

**Query Parameters:**
- `limit` (optional): Number of results (default: 10, max: 100)
- `offset` (optional): Pagination offset

**Response:**
```typescript
interface GetValidationHistoryResponse {
  data: {
    results: ValidationResult[];
    total: number;
  };
}
```

---

### Get Tool Config Files

```http
GET /tools/:id/config/files
```

**Response:**
```typescript
interface GetConfigFilesResponse {
  data: {
    files: ConfigFileInfo[];
  };
}

interface ConfigFileInfo {
  path: string;
  name: string;
  type: 'json' | 'yaml' | 'toml' | 'markdown';
  size_bytes: number;
  modified: string;  // ISO 8601
  scope: 'user' | 'project' | 'system';
}
```

---

### Read Config File

```http
GET /tools/:id/config/files/:path
```

**Note:** `:path` should be URL-encoded.

**Response:**
```typescript
interface ReadConfigFileResponse {
  data: {
    path: string;
    content: string;
    parsed?: unknown;  // Parsed content if valid
    type: string;
  };
}
```

---

### Write Config File

```http
PUT /tools/:id/config/files/:path
Content-Type: application/json
```

**Request Body:**
```typescript
interface WriteConfigFileRequest {
  content: string;
}
```

**Response:** `200 OK`

---

### Get Tool Settings

```http
GET /tools/:id/settings
```

**Response:**
```typescript
interface GetSettingsResponse {
  data: {
    settings: ToolSettings;
  };
}

interface ToolSettings {
  tool_id: string;
  categories: SettingsCategory[];
}

interface SettingsCategory {
  name: string;
  description?: string;
  settings: Setting[];
}

interface Setting {
  key: string;
  value: unknown;
  type: string;
  description?: string;
  default?: unknown;
  options?: string[];
  readonly: boolean;
}
```

---

### Update Setting

```http
PUT /tools/:id/settings/:key
Content-Type: application/json
```

**Request Body:**
```typescript
interface UpdateSettingRequest {
  value: unknown;
}
```

**Response:** `200 OK`

**Example:**
```bash
curl -X PUT http://localhost:8080/api/tools/claude/settings/theme \
  -H "Content-Type: application/json" \
  -d '{"value": "dark"}'
```

---

## Skills.sh API

### Search Skills

```http
GET /skills/search?q={query}
```

**Query Parameters:**
- `q` (required): Search query (min 2 characters)
- `limit` (optional): Max results (default: 10, max: 50)

**Response:**
```typescript
interface SearchSkillsResponse {
  data: {
    skills: SkillsShSkill[];
  };
}

interface SkillsShSkill {
  id: string;
  name: string;
  installs: number;
  top_source: string;  // owner/repo format
  description?: string;
}
```

**Example:**
```bash
curl "http://localhost:8080/api/skills/search?q=typescript&limit=10"
```

---

### Install Skill

```http
POST /skills/install
Content-Type: application/json
```

**Request Body:**
```typescript
interface InstallSkillRequest {
  source: string;      // e.g., "vercel-labs/agent-skills"
  name: string;        // e.g., "playwright"
  scope: 'global' | 'project';
  project_path?: string;  // Required if scope is 'project'
}
```

**Response:**
```typescript
interface InstallSkillResponse {
  data: {
    job_id: string;
    message: string;
  };
}
```

**Example:**
```bash
curl -X POST http://localhost:8080/api/skills/install \
  -H "Content-Type: application/json" \
  -d '{
    "source": "vercel-labs/agent-skills",
    "name": "playwright",
    "scope": "global"
  }'
```

**Note:** This returns immediately with a job ID. Subscribe to WebSocket `jobs` channel or poll `GET /jobs/:id` for progress.

---

### Uninstall Skill

```http
POST /skills/uninstall
Content-Type: application/json
```

**Request Body:**
```typescript
interface UninstallSkillRequest {
  name: string;
  scope: 'global' | 'project';
  project_path?: string;
}
```

**Response:**
```typescript
interface UninstallSkillResponse {
  data: {
    job_id: string;
    message: string;
  };
}
```

---

### List Installed Skills

```http
GET /skills/installed
```

**Query Parameters:**
- `scope` (optional): Filter by scope (`global`, `project`, `all`)
- `project_path` (optional): Required if scope is `project`

**Response:**
```typescript
interface ListInstalledSkillsResponse {
  data: {
    skills: InstalledSkill[];
  };
}

interface InstalledSkill {
  name: string;
  source: string;
  scope: 'global' | 'project';
  installed_at: string;
  updated_at: string;
  version?: string;
  has_update: boolean;
}
```

---

### Check for Updates

```http
POST /skills/check-updates
```

**Response:**
```typescript
interface CheckUpdatesResponse {
  data: {
    updates: SkillUpdate[];
  };
}

interface SkillUpdate {
  name: string;
  current_version?: string;
  latest_version?: string;
  source: string;
}
```

---

## MCP Registry API

### List MCP Servers

```http
GET /mcp/servers
```

**Response:**
```typescript
interface ListMCPServersResponse {
  data: {
    servers: MCPServer[];
  };
}

interface MCPServer {
  id: string;
  name: string;
  description?: string;
  transport: MCPTransport;
  is_enabled_globally: boolean;
  tool_enablement: ToolEnablement[];
  env: Record<string, string>;
  created_at: string;
  updated_at: string;
  last_tested?: string;
  test_status?: 'success' | 'failed';
}
```

---

### Add MCP Server

```http
POST /mcp/servers
Content-Type: application/json
```

**Request Body:**
```typescript
interface AddMCPServerRequest {
  name: string;
  description?: string;
  transport: MCPTransport;
  env?: Record<string, string>;
}

type MCPTransport =
  | { type: 'stdio'; command: string; args: string[] }
  | { type: 'sse'; url: string; headers?: Record<string, string> }
  | { type: 'http'; url: string; headers?: Record<string, string> };
```

**Response:**
```typescript
interface AddMCPServerResponse {
  data: {
    server_id: string;
    message: string;
  };
}
```

**Example:**
```bash
curl -X POST http://localhost:8080/api/mcp/servers \
  -H "Content-Type: application/json" \
  -d '{
    "name": "filesystem",
    "description": "Filesystem access",
    "transport": {
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/Users"]
    }
  }'
```

---

### Update MCP Server

```http
PUT /mcp/servers/:id
Content-Type: application/json
```

**Request Body:**
```typescript
interface UpdateMCPServerRequest {
  name?: string;
  description?: string;
  transport?: MCPTransport;
  env?: Record<string, string>;
}
```

**Response:** `200 OK`

---

### Remove MCP Server

```http
DELETE /mcp/servers/:id
```

**Response:** `204 No Content`

---

### Enable/Disable for Tool

```http
PUT /mcp/servers/:id/tools/:toolId
Content-Type: application/json
```

**Request Body:**
```typescript
interface UpdateToolEnablementRequest {
  enabled: boolean;
  config_override?: Partial<MCPTransport>;
}
```

**Response:** `200 OK`

**Example:**
```bash
# Enable for Claude
curl -X PUT http://localhost:8080/api/mcp/servers/filesystem/tools/claude \
  -H "Content-Type: application/json" \
  -d '{"enabled": true}'

# Disable for Gemini
curl -X PUT http://localhost:8080/api/mcp/servers/filesystem/tools/gemini \
  -H "Content-Type: application/json" \
  -d '{"enabled": false}'
```

---

### Test MCP Connection

```http
POST /mcp/servers/:id/test
```

**Response:**
```typescript
interface TestMCPConnectionResponse {
  data: {
    success: boolean;
    message: string;
    latency_ms?: number;
    error?: string;
  };
}
```

---

### Sync MCP to Tool

```http
POST /mcp/servers/:id/sync/:toolId
```

**Response:**
```typescript
interface SyncMCPResponse {
  data: {
    success: boolean;
    message: string;
    details?: string;
  };
}
```

---

### Sync All MCP Servers

```http
POST /mcp/sync
```

**Response:**
```typescript
interface SyncAllMCPResponse {
  data: {
    results: SyncResult[];
    summary: {
      total: number;
      success: number;
      failed: number;
    };
  };
}

interface SyncResult {
  server_id: string;
  tool_id: string;
  success: boolean;
  message: string;
}
```

---

## Async Jobs API

### Get Job Status

```http
GET /jobs/:id
```

**Response:**
```typescript
interface GetJobResponse {
  data: {
    job: AsyncJob;
  };
}

interface AsyncJob {
  id: string;
  job_type: string;
  status: 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';
  params: Record<string, unknown>;
  progress: number;
  current_stage?: string;
  stage_message?: string;
  result?: unknown;
  error_message?: string;
  created_at: string;
  started_at?: string;
  completed_at?: string;
  cancelled_at?: string;
}
```

---

### List Jobs

```http
GET /jobs
```

**Query Parameters:**
- `status` (optional): Filter by status
- `type` (optional): Filter by job type
- `limit` (optional): Default 20, max 100
- `offset` (optional): Pagination offset

**Response:**
```typescript
interface ListJobsResponse {
  data: {
    jobs: AsyncJob[];
    total: number;
  };
}
```

---

### Cancel Job

```http
POST /jobs/:id/cancel
```

**Response:**
```typescript
interface CancelJobResponse {
  data: {
    success: boolean;
    message: string;
  };
}
```

---

## Dashboard API

### Get Overview

```http
GET /dashboard/overview
```

**Response:**
```typescript
interface GetOverviewResponse {
  data: {
    status: SystemStatus;
    warnings: SystemWarning[];
    tools: ToolStatusSummary[];
    usage: UsageSummary;
    auth: AuthStatus;
    mcp: MCPStatusSummary;
    skills: SkillsStatusSummary;
  };
}

interface SystemStatus {
  overall: 'healthy' | 'degraded' | 'unhealthy';
  message?: string;
  last_updated: string;
}

interface SystemWarning {
  id: string;
  type: string;
  severity: 'info' | 'warning' | 'error' | 'critical';
  message: string;
  details?: unknown;
  created_at: string;
  is_resolved: boolean;
}

interface ToolStatusSummary {
  total: number;
  installed: number;
  valid: number;
  invalid: number;
  unknown: number;
}

interface UsageSummary {
  period: 'today' | 'week' | 'month';
  total_requests: number;
  total_tokens: number;
  estimated_cost: number;
  by_tool: Record<string, ToolUsage>;
}

interface AuthStatus {
  claude?: { authenticated: boolean; username?: string };
  gemini?: { authenticated: boolean; username?: string };
  // ... other tools
}

interface MCPStatusSummary {
  total_servers: number;
  enabled_globally: number;
  by_tool: Record<string, number>;  // Number of enabled servers per tool
}

interface SkillsStatusSummary {
  total_installed: number;
  global: number;
  by_project: Record<string, number>;
  updates_available: number;
}
```

---

## WebSocket API

### Connection

```
ws://localhost:8080/ws
```

### Subscribe to Channel

```json
{
  "type": "subscribe",
  "channel": "dashboard"
}
```

### Unsubscribe from Channel

```json
{
  "type": "unsubscribe",
  "channel": "dashboard"
}
```

### Ping/Pong

```json
// Client -> Server
{ "type": "ping" }

// Server -> Client
{ "type": "pong", "timestamp": "2026-01-29T10:00:00Z" }
```

### Events

See `.sisyphus/specs/architecture.md` for full WebSocket protocol specification.

---

## Error Codes

| Code | Description | HTTP Status |
|------|-------------|-------------|
| `TOOL_NOT_FOUND` | Tool ID not found | 404 |
| `TOOL_ALREADY_EXISTS` | Tool ID already registered | 409 |
| `INVALID_TOOL_SPEC` | Tool specification invalid | 422 |
| `TOOL_NOT_INSTALLED` | Tool binary not found | 400 |
| `MCP_SERVER_NOT_FOUND` | MCP server ID not found | 404 |
| `MCP_SERVER_EXISTS` | MCP server name already exists | 409 |
| `JOB_NOT_FOUND` | Job ID not found | 404 |
| `JOB_CANNOT_CANCEL` | Job cannot be cancelled (already completed/failed) | 400 |
| `SKILLS_SH_ERROR` | Error executing skills.sh CLI | 500 |
| `VALIDATION_ERROR` | Tool validation failed | 422 |
| `CONFIG_ERROR` | Configuration file error | 400 |
| `INTERNAL_ERROR` | Internal server error | 500 |

---

## Rate Limiting

- API: 100 requests per minute per IP
- WebSocket: 100 messages per second per connection
- Jobs: Max 10 concurrent jobs per type

---

**Next**: See `.sisyphus/tasks/phase-1.md` for implementation task breakdown.
