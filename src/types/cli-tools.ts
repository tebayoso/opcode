/**
 * Types for CLI Tools detection and management
 */

/**
 * Supported CLI tool types
 */
export type CLIToolType =
  | 'claude'
  | 'gemini'
  | 'codex'
  | 'opencode'
  | 'github_copilot'
  | 'cursor'
  | 'eslint'
  | 'vite';

/**
 * Source of installation discovery
 */
export type InstallationSource =
  | 'system'
  | 'homebrew'
  | 'npm_global'
  | 'nvm'
  | 'gh_extension'
  | 'script_install'
  | 'app_bundle'
  | 'custom';

/**
 * Represents a detected CLI tool installation
 */
export interface CLIToolInstallation {
  /** Type of CLI tool */
  tool_type: CLIToolType;
  /** Display name for this installation */
  name: string;
  /** Full path to the binary or command */
  path: string;
  /** Version string if detected */
  version: string | null;
  /** How this installation was discovered */
  source: InstallationSource;
  /** Whether the tool is currently available/working */
  is_available: boolean;
  /** The command to execute (may differ from path for extensions) */
  command: string;
  /** Additional source details (e.g., "nvm (v20.10.0)") */
  source_detail: string | null;
}

export interface CLIToolCapabilities {
  files: boolean;
  settings: boolean;
  mcp_servers: boolean;
  agents: boolean;
  usage: boolean;
}

/**
 * A CLI tool with its installation status
 */
export interface CLIToolWithStatus {
  /** Type of CLI tool */
  tool_type: CLIToolType;
  /** Display name */
  name: string;
  /** Description */
  description: string;
  /** Website URL */
  website: string;
  /** Install instructions */
  install_instructions: string;
  /** Whether any installation is available */
  is_installed: boolean;
  /** All found installations */
  installations: CLIToolInstallation[];
  /** The preferred installation (if set) */
  preferred_installation: CLIToolInstallation | null;
  /** Supported capabilities exposed by the tool */
  capabilities: CLIToolCapabilities;
}

/**
 * Status of all CLI tools
 */
export interface CLIToolsStatus {
  /** All detected tools with their installations */
  tools: CLIToolWithStatus[];
  /** When this status was last updated */
  last_updated: string;
}

/**
 * Display metadata for CLI tool types
 */
export const CLI_TOOL_DISPLAY: Record<
  CLIToolType,
  { name: string; icon: string; color: string }
> = {
  claude: {
    name: 'Claude Code',
    icon: '🤖',
    color: 'text-orange-500',
  },
  gemini: {
    name: 'Gemini CLI',
    icon: '✨',
    color: 'text-blue-500',
  },
  codex: {
    name: 'Codex CLI',
    icon: '🔮',
    color: 'text-green-500',
  },
  opencode: {
    name: 'OpenCode',
    icon: '📦',
    color: 'text-purple-500',
  },
  github_copilot: {
    name: 'GitHub Copilot CLI',
    icon: '🐙',
    color: 'text-gray-500',
  },
  cursor: {
    name: 'Cursor CLI',
    icon: '🖱️',
    color: 'text-indigo-500',
  },
  eslint: {
    name: 'ESLint',
    icon: '✅',
    color: 'text-amber-500',
  },
  vite: {
    name: 'Vite',
    icon: '⚡',
    color: 'text-yellow-500',
  },
};

const UNKNOWN_CLI_DISPLAY = {
  name: 'Unknown CLI',
  icon: '🧭',
  color: 'text-muted-foreground',
};

export function resolveCLIToolDisplay(toolType: string, fallbackName?: string) {
  const display = CLI_TOOL_DISPLAY[toolType as CLIToolType];
  if (display) {
    return display;
  }

  return {
    ...UNKNOWN_CLI_DISPLAY,
    name: fallbackName ?? UNKNOWN_CLI_DISPLAY.name,
  };
}

/**
 * Display metadata for installation sources
 */
export const INSTALLATION_SOURCE_DISPLAY: Record<
  InstallationSource,
  { name: string; description: string }
> = {
  system: {
    name: 'System',
    description: 'Found in system PATH',
  },
  homebrew: {
    name: 'Homebrew',
    description: 'Installed via Homebrew',
  },
  npm_global: {
    name: 'npm (global)',
    description: 'Installed globally via npm',
  },
  nvm: {
    name: 'NVM',
    description: 'Installed via NVM-managed Node.js',
  },
  gh_extension: {
    name: 'GitHub CLI Extension',
    description: 'Installed as a gh CLI extension',
  },
  script_install: {
    name: 'Script Install',
    description: 'Installed via installation script',
  },
  app_bundle: {
    name: 'Application Bundle',
    description: 'Part of an application bundle',
  },
  custom: {
    name: 'Custom',
    description: 'User-specified path',
  },
};

// =============================================================================
// Configuration Management Types
// =============================================================================

/**
 * Type of configuration file
 */
export type ConfigFileType = 'json' | 'jsonc' | 'toml' | 'yaml' | 'markdown';

/**
 * Scope of a configuration file
 */
export type ConfigScope = 'user' | 'project' | 'system';

/**
 * Configuration file metadata (lazy loading - doesn't include content)
 */
export interface ConfigFileInfo {
  /** Full path to the config file */
  path: string;
  /** File name */
  name: string;
  /** Type of config file */
  file_type: ConfigFileType;
  /** File size in bytes */
  size_bytes: number;
  /** Last modified timestamp (ISO string) */
  modified: string;
  /** Scope of the configuration */
  scope: ConfigScope;
  /** Optional description of what this file configures */
  description: string | null;
}

/**
 * Configuration file content (loaded on demand)
 */
export interface ConfigFileContent {
  /** Full path to the config file */
  path: string;
  /** Raw content of the file */
  content: string;
  /** Parsed content as JSON (normalized from any format) */
  parsed: unknown | null;
  /** Type of config file */
  file_type: ConfigFileType;
}

/**
 * Type of setting value
 */
export type SettingType = 'string' | 'number' | 'boolean' | 'array' | 'object';

/**
 * Definition of a single setting
 */
export interface SettingDefinition {
  /** Setting key */
  key: string;
  /** Current value */
  value: unknown;
  /** Type of the value */
  value_type: SettingType;
  /** Description of the setting */
  description: string | null;
  /** Default value if any */
  default: unknown | null;
  /** Available options for enum-type settings */
  options: string[] | null;
  /** Whether the setting is read-only */
  readonly: boolean;
}

/**
 * Category of settings
 */
export interface SettingsCategory {
  /** Category name */
  name: string;
  /** Category description */
  description: string | null;
  /** Settings in this category */
  settings: SettingDefinition[];
}

/**
 * Tool settings organized by categories
 */
export interface ToolSettings {
  /** Tool type these settings belong to */
  tool_type: CLIToolType;
  /** Categories of settings */
  categories: SettingsCategory[];
}

/**
 * MCP transport configuration - Stdio variant
 */
export interface MCPTransportStdio {
  type: 'stdio';
  command: string;
  args: string[];
}

/**
 * MCP transport configuration - SSE variant
 */
export interface MCPTransportSse {
  type: 'sse';
  url: string;
  headers: Record<string, string>;
}

/**
 * MCP transport configuration - HTTP variant
 */
export interface MCPTransportHttp {
  type: 'http';
  url: string;
  headers: Record<string, string>;
}

/**
 * MCP transport configuration (union type)
 */
export type MCPTransport = MCPTransportStdio | MCPTransportSse | MCPTransportHttp;

/**
 * MCP server configuration (for CLI tool configs)
 */
export interface CLIToolMCPServerConfig {
  /** Server name/identifier */
  name: string;
  /** Transport configuration */
  transport: MCPTransport;
  /** Whether the server is enabled */
  enabled: boolean;
  /** Environment variables for the server */
  env: Record<string, string>;
  /** Optional description */
  description: string | null;
}

/**
 * Input type for adding an MCP server (simplified for API calls)
 */
export interface MCPServerInput {
  /** Server name */
  name: string;
  /** Transport type: "stdio", "sse", or "http" */
  transport_type: 'stdio' | 'sse' | 'http';
  /** Command for stdio transport */
  command?: string;
  /** Arguments for stdio transport */
  args?: string[];
  /** URL for sse/http transport */
  url?: string;
  /** Headers for sse/http transport */
  headers?: Record<string, string>;
  /** Whether the server is enabled */
  enabled?: boolean;
  /** Environment variables */
  env?: Record<string, string>;
  /** Optional description */
  description?: string;
}

/**
 * Agent/command definition for a CLI tool
 */
export interface CLIToolAgentDefinition {
  /** Agent name */
  name: string;
  /** Agent description */
  description: string | null;
  /** Model to use (if specified) */
  model: string | null;
  /** Available tools for the agent */
  tools: string[];
  /** System prompt */
  system_prompt: string | null;
  /** Source file where this agent is defined */
  source_file: string;
}

/**
 * Output from executing a CLI command
 */
export interface CommandOutput {
  /** Exit code from the command */
  exit_code: number;
  /** Standard output */
  stdout: string;
  /** Standard error */
  stderr: string;
  /** Whether the command succeeded */
  success: boolean;
}

/**
 * Display metadata for config file types
 */
export const CONFIG_FILE_TYPE_DISPLAY: Record<
  ConfigFileType,
  { name: string; icon: string; language: string }
> = {
  json: { name: 'JSON', icon: '📄', language: 'json' },
  jsonc: { name: 'JSONC', icon: '📄', language: 'jsonc' },
  toml: { name: 'TOML', icon: '⚙️', language: 'toml' },
  yaml: { name: 'YAML', icon: '📋', language: 'yaml' },
  markdown: { name: 'Markdown', icon: '📝', language: 'markdown' },
};

/**
 * Display metadata for config scopes
 */
export const CONFIG_SCOPE_DISPLAY: Record<
  ConfigScope,
  { name: string; description: string; icon: string }
> = {
  user: {
    name: 'User',
    description: 'User-level configuration (~/.config or ~/.<tool>)',
    icon: '👤',
  },
  project: {
    name: 'Project',
    description: 'Project-level configuration (./<tool>/ in project)',
    icon: '📁',
  },
  system: {
    name: 'System',
    description: 'System-level or managed configuration',
    icon: '💻',
  },
};

// =============================================================================
// Usage Tracking Types
// =============================================================================

/**
 * Types of usage actions that can be tracked
 */
export type UsageAction =
  | 'file_read'
  | 'file_write'
  | 'setting_change'
  | 'mcp_add'
  | 'mcp_remove'
  | 'command_execute'
  | 'agent_view'
  | 'config_refresh';

/**
 * A single usage entry
 */
export interface CLIToolUsageEntry {
  /** Entry ID */
  id: number;
  /** Tool type this action was for */
  tool_type: string;
  /** Type of action */
  action: string;
  /** Optional details about the action (JSON string) */
  details: string | null;
  /** When the action occurred (ISO timestamp) */
  timestamp: string;
}

/**
 * Usage statistics for a CLI tool
 */
export interface CLIToolUsageStats {
  /** Tool type */
  tool_type: string;
  /** Total number of actions */
  total_actions: number;
  /** Actions broken down by type */
  actions_by_type: Record<string, number>;
  /** Most recent actions */
  recent_actions: CLIToolUsageEntry[];
  /** Timestamp of first action (ISO string) */
  first_action: string | null;
  /** Timestamp of last action (ISO string) */
  last_action: string | null;
}

/**
 * Display metadata for usage actions
 */
export const USAGE_ACTION_DISPLAY: Record<
  UsageAction,
  { name: string; description: string; icon: string }
> = {
  file_read: {
    name: 'File Read',
    description: 'Configuration file was read',
    icon: '📖',
  },
  file_write: {
    name: 'File Write',
    description: 'Configuration file was modified',
    icon: '✏️',
  },
  setting_change: {
    name: 'Setting Change',
    description: 'A setting was modified',
    icon: '⚙️',
  },
  mcp_add: {
    name: 'MCP Server Added',
    description: 'An MCP server was added',
    icon: '➕',
  },
  mcp_remove: {
    name: 'MCP Server Removed',
    description: 'An MCP server was removed',
    icon: '➖',
  },
  command_execute: {
    name: 'Command Executed',
    description: 'A CLI command was executed',
    icon: '▶️',
  },
  agent_view: {
    name: 'Agent Viewed',
    description: 'An agent/rule was viewed',
    icon: '👁️',
  },
  config_refresh: {
    name: 'Config Refresh',
    description: 'Configuration was refreshed',
    icon: '🔄',
  },
};
