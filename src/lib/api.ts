import { apiCall } from './apiAdapter';
import type { HooksConfiguration } from '@/types/hooks';
import type {
  CLIToolsStatus,
  CLIToolInstallation,
  CLIToolType,
  ConfigFileInfo,
  ConfigFileContent,
  ToolSettings,
  CLIToolMCPServerConfig,
  MCPServerInput,
  CLIToolAgentDefinition,
  CommandOutput,
  CLIToolUsageEntry,
  CLIToolUsageStats,
  UsageAction,
} from '@/types/cli-tools';
import type {
  OperationResult as FileOperationResult,
  MergeOptions,
  MergePreview,
  FileOperation,
} from '@/types/file-ops';

/** Process type for tracking in ProcessRegistry */
export type ProcessType = 
  | { AgentRun: { agent_id: number; agent_name: string } }
  | { ClaudeSession: { session_id: string } };

/** Information about a running process */
export interface ProcessInfo {
  run_id: number;
  process_type: ProcessType;
  pid: number;
  started_at: string;
  project_path: string;
  task: string;
  model: string;
}

/**
 * Represents a project in the ~/.claude/projects directory
 */
export interface Project {
  /** The project ID (derived from the directory name) */
  id: string;
  /** The original project path (decoded from the directory name) */
  path: string;
  /** List of session IDs (JSONL file names without extension) */
  sessions: string[];
  /** Unix timestamp when the project directory was created */
  created_at: number;
  /** Unix timestamp of the most recent session (if any) */
  most_recent_session?: number;
}

/**
 * Represents a session with its metadata
 */
export interface Session {
  /** The session ID (UUID) */
  id: string;
  /** The project ID this session belongs to */
  project_id: string;
  /** The project path */
  project_path: string;
  /** Optional todo data associated with this session */
  todo_data?: any;
  /** Unix timestamp when the session file was created */
  created_at: number;
  /** First user message content (if available) */
  first_message?: string;
  /** Timestamp of the first user message (if available) */
  message_timestamp?: string;
}

/**
 * Represents the settings from ~/.claude/settings.json
 */
export type ClaudeSettings = Record<string, any>;

/**
 * Represents the Claude Code version status
 */
export interface ClaudeVersionStatus {
  /** Whether Claude Code is installed and working */
  is_installed: boolean;
  /** The version string if available */
  version?: string;
  /** The full output from the command */
  output: string;
}

/**
 * Represents a CLAUDE.md file found in the project
 */
export interface ClaudeMdFile {
  /** Relative path from the project root */
  relative_path: string;
  /** Absolute path to the file */
  absolute_path: string;
  /** File size in bytes */
  size: number;
  /** Last modified timestamp */
  modified: number;
}

/**
 * Represents a global config file from ~/.claude/
 */
export interface GlobalConfigFile {
  /** File name */
  name: string;
  /** Relative path from ~/.claude/ */
  relative_path: string;
  /** Absolute path to the file */
  absolute_path: string;
  /** Category of the config file */
  category: string;
  /** File size in bytes */
  size: number;
  /** Last modified timestamp */
  modified: number;
}

/**
 * Represents a file or directory entry
 */
export interface FileEntry {
  name: string;
  path: string;
  is_directory: boolean;
  size: number;
  extension?: string;
}

/**
 * Represents a Claude installation found on the system
 */
export interface ClaudeInstallation {
  /** Full path to the Claude binary */
  path: string;
  /** Version string if available */
  version?: string;
  /** Source of discovery (e.g., "nvm", "system", "homebrew", "which") */
  source: string;
  /** Type of installation */
  installation_type: "System" | "Custom";
}

// Agent API types
export interface Agent {
  id?: number;
  name: string;
  icon: string;
  system_prompt: string;
  default_task?: string;
  model: string;
  hooks?: string; // JSON string of HooksConfiguration
  created_at: string;
  updated_at: string;
}

export interface AgentExport {
  version: number;
  exported_at: string;
  agent: {
    name: string;
    icon: string;
    system_prompt: string;
    default_task?: string;
    model: string;
    hooks?: string;
  };
}

export interface GitHubAgentFile {
  name: string;
  path: string;
  download_url: string;
  size: number;
  sha: string;
}

export interface AgentRun {
  id?: number;
  agent_id: number;
  agent_name: string;
  agent_icon: string;
  task: string;
  model: string;
  project_path: string;
  session_id: string;
  status: string; // 'pending', 'running', 'completed', 'failed', 'cancelled'
  pid?: number;
  process_started_at?: string;
  created_at: string;
  completed_at?: string;
}

export interface AgentRunMetrics {
  duration_ms?: number;
  total_tokens?: number;
  cost_usd?: number;
  message_count?: number;
}

export interface AgentRunWithMetrics {
  id?: number;
  agent_id: number;
  agent_name: string;
  agent_icon: string;
  task: string;
  model: string;
  project_path: string;
  session_id: string;
  status: string; // 'pending', 'running', 'completed', 'failed', 'cancelled'
  pid?: number;
  duration_ms?: number;
  total_tokens?: number;
  process_started_at?: string;
  created_at: string;
  completed_at?: string;
  metrics?: AgentRunMetrics;
  output?: string; // Real-time JSONL content
}

// Usage Dashboard types
export interface UsageEntry {
  project: string;
  timestamp: string;
  model: string;
  input_tokens: number;
  output_tokens: number;
  cache_write_tokens: number;
  cache_read_tokens: number;
  cost: number;
}

export interface ModelUsage {
  model: string;
  total_cost: number;
  total_tokens: number;
  input_tokens: number;
  output_tokens: number;
  cache_creation_tokens: number;
  cache_read_tokens: number;
  session_count: number;
}

export interface DailyUsage {
  date: string;
  total_cost: number;
  total_tokens: number;
  models_used: string[];
}

export interface ProjectUsage {
  project_path: string;
  project_name: string;
  total_cost: number;
  total_tokens: number;
  session_count: number;
  last_used: string;
}

export interface UsageStats {
  total_cost: number;
  total_tokens: number;
  total_input_tokens: number;
  total_output_tokens: number;
  total_cache_creation_tokens: number;
  total_cache_read_tokens: number;
  total_sessions: number;
  by_model: ModelUsage[];
  by_date: DailyUsage[];
  by_project: ProjectUsage[];
}

/**
 * Represents a checkpoint in the session timeline
 */
export interface Checkpoint {
  id: string;
  sessionId: string;
  projectId: string;
  messageIndex: number;
  timestamp: string;
  description?: string;
  parentCheckpointId?: string;
  metadata: CheckpointMetadata;
}

/**
 * Metadata associated with a checkpoint
 */
export interface CheckpointMetadata {
  totalTokens: number;
  modelUsed: string;
  userPrompt: string;
  fileChanges: number;
  snapshotSize: number;
}

/**
 * Represents a file snapshot at a checkpoint
 */
export interface FileSnapshot {
  checkpointId: string;
  filePath: string;
  content: string;
  hash: string;
  isDeleted: boolean;
  permissions?: number;
  size: number;
}

/**
 * Represents a node in the timeline tree
 */
export interface TimelineNode {
  checkpoint: Checkpoint;
  children: TimelineNode[];
  fileSnapshotIds: string[];
}

/**
 * The complete timeline for a session
 */
export interface SessionTimeline {
  sessionId: string;
  rootNode?: TimelineNode;
  currentCheckpointId?: string;
  autoCheckpointEnabled: boolean;
  checkpointStrategy: CheckpointStrategy;
  totalCheckpoints: number;
}

/**
 * Strategy for automatic checkpoint creation
 */
export type CheckpointStrategy = 'manual' | 'per_prompt' | 'per_tool_use' | 'smart';

/**
 * Result of a checkpoint operation
 */
export interface CheckpointResult {
  checkpoint: Checkpoint;
  filesProcessed: number;
  warnings: string[];
}

/**
 * Diff between two checkpoints
 */
export interface CheckpointDiff {
  fromCheckpointId: string;
  toCheckpointId: string;
  modifiedFiles: FileDiff[];
  addedFiles: string[];
  deletedFiles: string[];
  tokenDelta: number;
}

/**
 * Diff for a single file
 */
export interface FileDiff {
  path: string;
  additions: number;
  deletions: number;
  diffContent?: string;
}

/**
 * Represents an MCP server configuration
 */
export interface MCPServer {
  /** Server name/identifier */
  name: string;
  /** Transport type: "stdio" or "sse" */
  transport: string;
  /** Command to execute (for stdio) */
  command?: string;
  /** Command arguments (for stdio) */
  args: string[];
  /** Environment variables */
  env: Record<string, string>;
  /** URL endpoint (for SSE) */
  url?: string;
  /** Configuration scope: "local", "project", or "user" */
  scope: string;
  /** Whether the server is currently active */
  is_active: boolean;
  /** Server status */
  status: ServerStatus;
}

/**
 * Server status information
 */
export interface ServerStatus {
  /** Whether the server is running */
  running: boolean;
  /** Last error message if any */
  error?: string;
  /** Last checked timestamp */
  last_checked?: number;
}

/**
 * MCP configuration for project scope (.mcp.json)
 */
export interface MCPProjectConfig {
  mcpServers: Record<string, MCPServerConfig>;
}

/**
 * Individual server configuration in .mcp.json
 */
export interface MCPServerConfig {
  command: string;
  args: string[];
  env: Record<string, string>;
}

/**
 * Represents a custom slash command
 */
export interface SlashCommand {
  /** Unique identifier for the command */
  id: string;
  /** Command name (without prefix) */
  name: string;
  /** Full command with prefix (e.g., "/project:optimize") */
  full_command: string;
  /** Command scope: "project" or "user" */
  scope: string;
  /** Optional namespace (e.g., "frontend" in "/project:frontend:component") */
  namespace?: string;
  /** Path to the markdown file */
  file_path: string;
  /** Command content (markdown body) */
  content: string;
  /** Optional description from frontmatter */
  description?: string;
  /** Allowed tools from frontmatter */
  allowed_tools: string[];
  /** Whether the command has bash commands (!) */
  has_bash_commands: boolean;
  /** Whether the command has file references (@) */
  has_file_references: boolean;
  /** Whether the command uses $ARGUMENTS placeholder */
  accepts_arguments: boolean;
}

/**
 * Represents a supporting file in a skill directory
 */
export interface SupportingFile {
  name: string;
  path: string;
  file_type: string;
}

/**
 * Represents a Claude skill
 */
export interface Skill {
  /** Unique identifier for the skill */
  id: string;
  /** Skill name (derived from folder name) */
  name: string;
  /** Skill scope: "project" or "user" */
  scope: string;
  /** Path to the skill directory */
  dir_path: string;
  /** Path to the main SKILL.md file */
  file_path: string;
  /** File type: "markdown" or "json" */
  file_type: string;
  /** Skill content (markdown or JSON) */
  content: string;
  /** Optional description from frontmatter */
  description?: string;
  /** Allowed tools from frontmatter */
  allowed_tools: string[];
  /** Supporting files in the skill directory */
  supporting_files: SupportingFile[];
}

// =====================================
// Plugin Types
// =====================================

/**
 * Represents a plugin marketplace
 */
export interface Marketplace {
  /** Marketplace name */
  name: string;
  /** Source (GitHub repo, URL, or local path) */
  source: string;
  /** Source type: "github", "url", "local", "git" */
  source_type: string;
  /** Optional description */
  description?: string;
  /** Number of plugins in this marketplace */
  plugins_count: number;
}

/**
 * Represents a plugin author
 */
export interface PluginAuthor {
  name: string;
  email?: string;
}

/**
 * Represents a plugin component (command, agent, skill, hook, or MCP server)
 */
export interface PluginComponent {
  /** Component name */
  name: string;
  /** Type: "command", "agent", "skill", "hook", "mcp" */
  component_type: string;
  /** Optional description */
  description?: string;
}

/**
 * Represents a Claude plugin
 */
export interface Plugin {
  /** Plugin name */
  name: string;
  /** Version string */
  version?: string;
  /** Plugin description */
  description?: string;
  /** Author information */
  author?: PluginAuthor;
  /** Category (development, productivity, learning, security) */
  category?: string;
  /** Marketplace source (if from marketplace) */
  marketplace?: string;
  /** Whether the plugin is installed */
  installed: boolean;
  /** Whether the plugin is enabled */
  enabled: boolean;
  /** Installation scope: "local", "project", "managed" */
  scope?: string;
  /** Path to the plugin directory */
  path?: string;
  /** List of plugin components */
  components: PluginComponent[];
}

/**
 * Result of adding a server
 */
export interface AddServerResult {
  success: boolean;
  message: string;
  server_name?: string;
}

/**
 * Import result for multiple servers
 */
export interface ImportResult {
  imported_count: number;
  failed_count: number;
  servers: ImportServerResult[];
}

/**
 * Result for individual server import
 */
export interface ImportServerResult {
  name: string;
  success: boolean;
  error?: string;
}

/**
 * API client for interacting with the Rust backend
 */
export const api = {
  /**
   * Gets the user's home directory path
   * @returns Promise resolving to the home directory path
   */
  async getHomeDirectory(): Promise<string> {
    try {
      return await apiCall<string>("get_home_directory");
    } catch (error) {
      console.error("Failed to get home directory:", error);
      return "/";
    }
  },

  /**
   * Lists all projects in the ~/.claude/projects directory
   * @returns Promise resolving to an array of projects
   */
  async listProjects(): Promise<Project[]> {
    try {
      return await apiCall<Project[]>("list_projects");
    } catch (error) {
      console.error("Failed to list projects:", error);
      throw error;
    }
  },

  /**
   * Creates a new project for the given directory path
   * @param path - The directory path to create a project for
   * @returns Promise resolving to the created project
   */
  async createProject(path: string): Promise<Project> {
    try {
      return await apiCall<Project>('create_project', { path });
    } catch (error) {
      console.error("Failed to create project:", error);
      throw error;
    }
  },

  /**
   * Retrieves sessions for a specific project
   * @param projectId - The ID of the project to retrieve sessions for
   * @returns Promise resolving to an array of sessions
   */
  async getProjectSessions(projectId: string): Promise<Session[]> {
    try {
      return await apiCall<Session[]>('get_project_sessions', { projectId });
    } catch (error) {
      console.error("Failed to get project sessions:", error);
      throw error;
    }
  },

  /**
   * Fetch list of agents from GitHub repository
   * @returns Promise resolving to list of available agents on GitHub
   */
  async fetchGitHubAgents(): Promise<GitHubAgentFile[]> {
    try {
      return await apiCall<GitHubAgentFile[]>('fetch_github_agents');
    } catch (error) {
      console.error("Failed to fetch GitHub agents:", error);
      throw error;
    }
  },

  /**
   * Fetch and preview a specific agent from GitHub
   * @param downloadUrl - The download URL for the agent file
   * @returns Promise resolving to the agent export data
   */
  async fetchGitHubAgentContent(downloadUrl: string): Promise<AgentExport> {
    try {
      return await apiCall<AgentExport>('fetch_github_agent_content', { downloadUrl });
    } catch (error) {
      console.error("Failed to fetch GitHub agent content:", error);
      throw error;
    }
  },

  /**
   * Import an agent directly from GitHub
   * @param downloadUrl - The download URL for the agent file
   * @returns Promise resolving to the imported agent
   */
  async importAgentFromGitHub(downloadUrl: string): Promise<Agent> {
    try {
      return await apiCall<Agent>('import_agent_from_github', { downloadUrl });
    } catch (error) {
      console.error("Failed to import agent from GitHub:", error);
      throw error;
    }
  },

  /**
   * Reads the Claude settings file
   * @returns Promise resolving to the settings object
   */
  async getClaudeSettings(): Promise<ClaudeSettings> {
    try {
      const result = await apiCall<{ data: ClaudeSettings }>("get_claude_settings");
      console.log("Raw result from get_claude_settings:", result);
      
      // The Rust backend returns ClaudeSettings { data: ... }
      // We need to extract the data field
      if (result && typeof result === 'object' && 'data' in result) {
        return result.data;
      }
      
      // If the result is already the settings object, return it
      return result as ClaudeSettings;
    } catch (error) {
      console.error("Failed to get Claude settings:", error);
      throw error;
    }
  },

  /**
   * Opens a new Claude Code session
   * @param path - Optional path to open the session in
   * @returns Promise resolving when the session is opened
   */
  async openNewSession(path?: string): Promise<string> {
    try {
      return await apiCall<string>("open_new_session", { path });
    } catch (error) {
      console.error("Failed to open new session:", error);
      throw error;
    }
  },

  /**
   * Reads the CLAUDE.md system prompt file
   * @returns Promise resolving to the system prompt content
   */
  async getSystemPrompt(): Promise<string> {
    try {
      return await apiCall<string>("get_system_prompt");
    } catch (error) {
      console.error("Failed to get system prompt:", error);
      throw error;
    }
  },

  /**
   * Checks if Claude Code is installed and gets its version
   * @returns Promise resolving to the version status
   */
  async checkClaudeVersion(): Promise<ClaudeVersionStatus> {
    try {
      return await apiCall<ClaudeVersionStatus>("check_claude_version");
    } catch (error) {
      console.error("Failed to check Claude version:", error);
      throw error;
    }
  },

  /**
   * Saves the CLAUDE.md system prompt file
   * @param content - The new content for the system prompt
   * @returns Promise resolving when the file is saved
   */
  async saveSystemPrompt(content: string): Promise<string> {
    try {
      return await apiCall<string>("save_system_prompt", { content });
    } catch (error) {
      console.error("Failed to save system prompt:", error);
      throw error;
    }
  },

  /**
   * Saves the Claude settings file
   * @param settings - The settings object to save
   * @returns Promise resolving when the settings are saved
   */
  async saveClaudeSettings(settings: ClaudeSettings): Promise<string> {
    try {
      return await apiCall<string>("save_claude_settings", { settings });
    } catch (error) {
      console.error("Failed to save Claude settings:", error);
      throw error;
    }
  },

  /**
   * Finds all CLAUDE.md files in a project directory
   * @param projectPath - The absolute path to the project
   * @returns Promise resolving to an array of CLAUDE.md files
   */
  async findClaudeMdFiles(projectPath: string): Promise<ClaudeMdFile[]> {
    try {
      return await apiCall<ClaudeMdFile[]>("find_claude_md_files", { projectPath });
    } catch (error) {
      console.error("Failed to find CLAUDE.md files:", error);
      throw error;
    }
  },

  /**
   * Reads a specific CLAUDE.md file
   * @param filePath - The absolute path to the file
   * @returns Promise resolving to the file content
   */
  async readClaudeMdFile(filePath: string): Promise<string> {
    try {
      return await apiCall<string>("read_claude_md_file", { filePath });
    } catch (error) {
      console.error("Failed to read CLAUDE.md file:", error);
      throw error;
    }
  },

  /**
   * Saves a specific CLAUDE.md file
   * @param filePath - The absolute path to the file
   * @param content - The new content for the file
   * @returns Promise resolving when the file is saved
   */
  async saveClaudeMdFile(filePath: string, content: string): Promise<string> {
    try {
      return await apiCall<string>("save_claude_md_file", { filePath, content });
    } catch (error) {
      console.error("Failed to save CLAUDE.md file:", error);
      throw error;
    }
  },

  /**
   * Finds all global config files in ~/.claude/ directory
   * @returns Promise resolving to an array of global config files
   */
  async findGlobalConfigFiles(): Promise<GlobalConfigFile[]> {
    try {
      return await apiCall<GlobalConfigFile[]>("find_global_config_files", {});
    } catch (error) {
      console.error("Failed to find global config files:", error);
      throw error;
    }
  },

  // Agent API methods
  
  /**
   * Lists all CC agents
   * @returns Promise resolving to an array of agents
   */
  async listAgents(): Promise<Agent[]> {
    try {
      return await apiCall<Agent[]>('list_agents');
    } catch (error) {
      console.error("Failed to list agents:", error);
      throw error;
    }
  },

  /**
   * Creates a new agent
   * @param name - The agent name
   * @param icon - The icon identifier
   * @param system_prompt - The system prompt for the agent
   * @param default_task - Optional default task
   * @param model - Optional model (defaults to 'sonnet')
   * @param hooks - Optional hooks configuration as JSON string
   * @returns Promise resolving to the created agent
   */
  async createAgent(
    name: string, 
    icon: string, 
    system_prompt: string, 
    default_task?: string, 
    model?: string,
    hooks?: string
  ): Promise<Agent> {
    try {
      return await apiCall<Agent>('create_agent', { 
        name, 
        icon, 
        systemPrompt: system_prompt,
        defaultTask: default_task,
        model,
        hooks
      });
    } catch (error) {
      console.error("Failed to create agent:", error);
      throw error;
    }
  },

  /**
   * Updates an existing agent
   * @param id - The agent ID
   * @param name - The updated name
   * @param icon - The updated icon
   * @param system_prompt - The updated system prompt
   * @param default_task - Optional default task
   * @param model - Optional model
   * @param hooks - Optional hooks configuration as JSON string
   * @returns Promise resolving to the updated agent
   */
  async updateAgent(
    id: number, 
    name: string, 
    icon: string, 
    system_prompt: string, 
    default_task?: string, 
    model?: string,
    hooks?: string
  ): Promise<Agent> {
    try {
      return await apiCall<Agent>('update_agent', { 
        id, 
        name, 
        icon, 
        systemPrompt: system_prompt,
        defaultTask: default_task,
        model,
        hooks
      });
    } catch (error) {
      console.error("Failed to update agent:", error);
      throw error;
    }
  },

  /**
   * Deletes an agent
   * @param id - The agent ID to delete
   * @returns Promise resolving when the agent is deleted
   */
  async deleteAgent(id: number): Promise<void> {
    try {
      await apiCall('delete_agent', { id }); 
    } catch (error) {
      console.error("Failed to delete agent:", error);
      throw error;
    }
  },

  /**
   * Gets a single agent by ID
   * @param id - The agent ID
   * @returns Promise resolving to the agent
   */
  async getAgent(id: number): Promise<Agent> {
    try {
      return await apiCall<Agent>('get_agent', { id });
    } catch (error) {
      console.error("Failed to get agent:", error);
      throw error;
    }
  },

  /**
   * Exports a single agent to JSON format
   * @param id - The agent ID to export
   * @returns Promise resolving to the JSON string
   */
  async exportAgent(id: number): Promise<string> {
    try {
      return await apiCall<string>('export_agent', { id });
    } catch (error) {
      console.error("Failed to export agent:", error);
      throw error;
    }
  },

  /**
   * Imports an agent from JSON data
   * @param jsonData - The JSON string containing the agent export
   * @returns Promise resolving to the imported agent
   */
  async importAgent(jsonData: string): Promise<Agent> {
    try {
      return await apiCall<Agent>('import_agent', { jsonData });
    } catch (error) {
      console.error("Failed to import agent:", error);
      throw error;
    }
  },

  /**
   * Imports an agent from a file
   * @param filePath - The path to the JSON file
   * @returns Promise resolving to the imported agent
   */
  async importAgentFromFile(filePath: string): Promise<Agent> {
    try {
      return await apiCall<Agent>('import_agent_from_file', { filePath });
    } catch (error) {
      console.error("Failed to import agent from file:", error);
      throw error;
    }
  },

  /**
   * Executes an agent
   * @param agentId - The agent ID to execute
   * @param projectPath - The project path to run the agent in
   * @param task - The task description
   * @param model - Optional model override
   * @returns Promise resolving to the run ID when execution starts
   */
  async executeAgent(agentId: number, projectPath: string, task: string, model?: string): Promise<number> {
    try {
      return await apiCall<number>('execute_agent', { agentId, projectPath, task, model });
    } catch (error) {
      console.error("Failed to execute agent:", error);
      // Return a sentinel value to indicate error
      throw new Error(`Failed to execute agent: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  },

  /**
   * Lists agent runs without metrics (basic info only)
   * @param agentId - Optional agent ID to filter runs
   * @returns Promise resolving to an array of agent runs
   */
  async listAgentRuns(agentId?: number): Promise<AgentRunWithMetrics[]> {
    try {
      return await apiCall<AgentRunWithMetrics[]>('list_agent_runs', { agentId });
    } catch (error) {
      console.error("Failed to list agent runs:", error);
      // Return empty array instead of throwing to prevent UI crashes
      return [];
    }
  },

  /**
   * Lists agent runs with metrics (includes token counts and duration)
   * @param agentId - Optional agent ID to filter runs
   * @returns Promise resolving to an array of agent runs with metrics
   */
  async listAgentRunsWithMetrics(agentId?: number): Promise<AgentRunWithMetrics[]> {
    try {
      return await apiCall<AgentRunWithMetrics[]>('list_agent_runs_with_metrics', { agentId });
    } catch (error) {
      console.error("Failed to list agent runs with metrics:", error);
      // Return empty array instead of throwing to prevent UI crashes
      return [];
    }
  },

  /**
   * Gets a single agent run by ID with metrics
   * @param id - The run ID
   * @returns Promise resolving to the agent run with metrics
   */
  async getAgentRun(id: number): Promise<AgentRunWithMetrics> {
    try {
      return await apiCall<AgentRunWithMetrics>('get_agent_run', { id });
    } catch (error) {
      console.error("Failed to get agent run:", error);
      throw new Error(`Failed to get agent run: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  },

  /**
   * Gets a single agent run by ID with real-time metrics from JSONL
   * @param id - The run ID
   * @returns Promise resolving to the agent run with metrics
   */
  async getAgentRunWithRealTimeMetrics(id: number): Promise<AgentRunWithMetrics> {
    try {
      return await apiCall<AgentRunWithMetrics>('get_agent_run_with_real_time_metrics', { id });
    } catch (error) {
      console.error("Failed to get agent run with real-time metrics:", error);
      throw new Error(`Failed to get agent run with real-time metrics: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  },

  /**
   * Lists all currently running agent sessions
   * @returns Promise resolving to list of running agent sessions
   */
  async listRunningAgentSessions(): Promise<AgentRun[]> {
    try {
      return await apiCall<AgentRun[]>('list_running_sessions');
    } catch (error) {
      console.error("Failed to list running agent sessions:", error);
      throw new Error(`Failed to list running agent sessions: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  },

  /**
   * Kills a running agent session
   * @param runId - The run ID to kill
   * @returns Promise resolving to whether the session was successfully killed
   */
  async killAgentSession(runId: number): Promise<boolean> {
    try {
      return await apiCall<boolean>('kill_agent_session', { runId });
    } catch (error) {
      console.error("Failed to kill agent session:", error);
      throw new Error(`Failed to kill agent session: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  },

  /**
   * Gets the status of a specific agent session
   * @param runId - The run ID to check
   * @returns Promise resolving to the session status or null if not found
   */
  async getSessionStatus(runId: number): Promise<string | null> {
    try {
      return await apiCall<string | null>('get_session_status', { runId });
    } catch (error) {
      console.error("Failed to get session status:", error);
      throw new Error(`Failed to get session status: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  },

  /**
   * Cleanup finished processes and update their status
   * @returns Promise resolving to list of run IDs that were cleaned up
   */
  async cleanupFinishedProcesses(): Promise<number[]> {
    try {
      return await apiCall<number[]>('cleanup_finished_processes');
    } catch (error) {
      console.error("Failed to cleanup finished processes:", error);
      throw new Error(`Failed to cleanup finished processes: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  },

  /**
   * Get real-time output for a running session (with live output fallback)
   * @param runId - The run ID to get output for
   * @returns Promise resolving to the current session output (JSONL format)
   */
  async getSessionOutput(runId: number): Promise<string> {
    try {
      return await apiCall<string>('get_session_output', { runId });
    } catch (error) {
      console.error("Failed to get session output:", error);
      throw new Error(`Failed to get session output: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  },

  /**
   * Get live output directly from process stdout buffer
   * @param runId - The run ID to get live output for
   * @returns Promise resolving to the current live output
   */
  async getLiveSessionOutput(runId: number): Promise<string> {
    try {
      return await apiCall<string>('get_live_session_output', { runId });
    } catch (error) {
      console.error("Failed to get live session output:", error);
      throw new Error(`Failed to get live session output: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  },

  /**
   * Start streaming real-time output for a running session
   * @param runId - The run ID to stream output for
   * @returns Promise that resolves when streaming starts
   */
  async streamSessionOutput(runId: number): Promise<void> {
    try {
      await apiCall<void>('stream_session_output', { runId }); 
    } catch (error) {
      console.error("Failed to start streaming session output:", error);
      throw new Error(`Failed to start streaming session output: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  },

  /**
   * Loads the JSONL history for a specific session
   */
  async loadSessionHistory(sessionId: string, projectId: string): Promise<any[]> {
    return apiCall("load_session_history", { sessionId, projectId });
  },

  /**
   * Loads the JSONL history for a specific agent session
   * Similar to loadSessionHistory but searches across all project directories
   * @param sessionId - The session ID (UUID)
   * @returns Promise resolving to array of session messages
   */
  async loadAgentSessionHistory(sessionId: string): Promise<any[]> {
    try {
      return await apiCall<any[]>('load_agent_session_history', { sessionId });
    } catch (error) {
      console.error("Failed to load agent session history:", error);
      throw error;
    }
  },

  /**
   * Executes a new interactive Claude Code session with streaming output
   */
  async executeClaudeCode(projectPath: string, prompt: string, model: string): Promise<void> {
    return apiCall("execute_claude_code", { projectPath, prompt, model });
  },

  /**
   * Continues an existing Claude Code conversation with streaming output
   */
  async continueClaudeCode(projectPath: string, prompt: string, model: string): Promise<void> {
    return apiCall("continue_claude_code", { projectPath, prompt, model });
  },

  /**
   * Resumes an existing Claude Code session by ID with streaming output
   */
  async resumeClaudeCode(projectPath: string, sessionId: string, prompt: string, model: string): Promise<void> {
    return apiCall("resume_claude_code", { projectPath, sessionId, prompt, model });
  },

  /**
   * Cancels the currently running Claude Code execution
   * @param sessionId - Optional session ID to cancel a specific session
   */
  async cancelClaudeExecution(sessionId?: string): Promise<void> {
    return apiCall("cancel_claude_execution", { sessionId });
  },

  /**
   * Lists all currently running Claude sessions
   * @returns Promise resolving to list of running Claude sessions
   */
  async listRunningClaudeSessions(): Promise<any[]> {
    return apiCall("list_running_claude_sessions");
  },

  /**
   * Gets live output from a Claude session
   * @param sessionId - The session ID to get output for
   * @returns Promise resolving to the current live output
   */
  async getClaudeSessionOutput(sessionId: string): Promise<string> {
    return apiCall("get_claude_session_output", { sessionId });
  },

  /**
   * Lists files and directories in a given path
   */
  async listDirectoryContents(directoryPath: string): Promise<FileEntry[]> {
    return apiCall("list_directory_contents", { directoryPath });
  },

  /**
   * Searches for files and directories matching a pattern
   */
  async searchFiles(basePath: string, query: string): Promise<FileEntry[]> {
    return apiCall("search_files", { basePath, query });
  },

  /**
   * Gets overall usage statistics
   * @returns Promise resolving to usage statistics
   */
  async getUsageStats(): Promise<UsageStats> {
    try {
      return await apiCall<UsageStats>("get_usage_stats");
    } catch (error) {
      console.error("Failed to get usage stats:", error);
      throw error;
    }
  },

  /**
   * Gets usage statistics filtered by date range
   * @param startDate - Start date (ISO format)
   * @param endDate - End date (ISO format)
   * @returns Promise resolving to usage statistics
   */
  async getUsageByDateRange(startDate: string, endDate: string): Promise<UsageStats> {
    try {
      return await apiCall<UsageStats>("get_usage_by_date_range", { startDate, endDate });
    } catch (error) {
      console.error("Failed to get usage by date range:", error);
      throw error;
    }
  },

  /**
   * Gets usage statistics grouped by session
   * @param since - Optional start date (YYYYMMDD)
   * @param until - Optional end date (YYYYMMDD)
   * @param order - Optional sort order ('asc' or 'desc')
   * @returns Promise resolving to an array of session usage data
   */
  async getSessionStats(
    since?: string,
    until?: string,
    order?: "asc" | "desc"
  ): Promise<ProjectUsage[]> {
    try {
      return await apiCall<ProjectUsage[]>("get_session_stats", {
        since,
        until,
        order,
      });
    } catch (error) {
      console.error("Failed to get session stats:", error);
      throw error;
    }
  },

  /**
   * Gets detailed usage entries with optional filtering
   * @param limit - Optional limit for number of entries
   * @returns Promise resolving to array of usage entries
   */
  async getUsageDetails(limit?: number): Promise<UsageEntry[]> {
    try {
      return await apiCall<UsageEntry[]>("get_usage_details", { limit });
    } catch (error) {
      console.error("Failed to get usage details:", error);
      throw error;
    }
  },

  /**
   * Creates a checkpoint for the current session state
   */
  async createCheckpoint(
    sessionId: string,
    projectId: string,
    projectPath: string,
    messageIndex?: number,
    description?: string
  ): Promise<CheckpointResult> {
    return apiCall("create_checkpoint", {
      sessionId,
      projectId,
      projectPath,
      messageIndex,
      description
    });
  },

  /**
   * Restores a session to a specific checkpoint
   */
  async restoreCheckpoint(
    checkpointId: string,
    sessionId: string,
    projectId: string,
    projectPath: string
  ): Promise<CheckpointResult> {
    return apiCall("restore_checkpoint", {
      checkpointId,
      sessionId,
      projectId,
      projectPath
    });
  },

  /**
   * Lists all checkpoints for a session
   */
  async listCheckpoints(
    sessionId: string,
    projectId: string,
    projectPath: string
  ): Promise<Checkpoint[]> {
    return apiCall("list_checkpoints", {
      sessionId,
      projectId,
      projectPath
    });
  },

  /**
   * Forks a new timeline branch from a checkpoint
   */
  async forkFromCheckpoint(
    checkpointId: string,
    sessionId: string,
    projectId: string,
    projectPath: string,
    newSessionId: string,
    description?: string
  ): Promise<CheckpointResult> {
    return apiCall("fork_from_checkpoint", {
      checkpointId,
      sessionId,
      projectId,
      projectPath,
      newSessionId,
      description
    });
  },

  /**
   * Gets the timeline for a session
   */
  async getSessionTimeline(
    sessionId: string,
    projectId: string,
    projectPath: string
  ): Promise<SessionTimeline> {
    return apiCall("get_session_timeline", {
      sessionId,
      projectId,
      projectPath
    });
  },

  /**
   * Updates checkpoint settings for a session
   */
  async updateCheckpointSettings(
    sessionId: string,
    projectId: string,
    projectPath: string,
    autoCheckpointEnabled: boolean,
    checkpointStrategy: CheckpointStrategy
  ): Promise<void> {
    return apiCall("update_checkpoint_settings", {
      sessionId,
      projectId,
      projectPath,
      autoCheckpointEnabled,
      checkpointStrategy
    });
  },

  /**
   * Gets diff between two checkpoints
   */
  async getCheckpointDiff(
    fromCheckpointId: string,
    toCheckpointId: string,
    sessionId: string,
    projectId: string
  ): Promise<CheckpointDiff> {
    try {
      return await apiCall<CheckpointDiff>("get_checkpoint_diff", {
        fromCheckpointId,
        toCheckpointId,
        sessionId,
        projectId
      });
    } catch (error) {
      console.error("Failed to get checkpoint diff:", error);
      throw error;
    }
  },

  /**
   * Tracks a message for checkpointing
   */
  async trackCheckpointMessage(
    sessionId: string,
    projectId: string,
    projectPath: string,
    message: string
  ): Promise<void> {
    try {
      await apiCall("track_checkpoint_message", {
        sessionId,
        projectId,
        projectPath,
        message
      });
    } catch (error) {
      console.error("Failed to track checkpoint message:", error);
      throw error;
    }
  },

  /**
   * Checks if auto-checkpoint should be triggered
   */
  async checkAutoCheckpoint(
    sessionId: string,
    projectId: string,
    projectPath: string,
    message: string
  ): Promise<boolean> {
    try {
      return await apiCall<boolean>("check_auto_checkpoint", {
        sessionId,
        projectId,
        projectPath,
        message
      });
    } catch (error) {
      console.error("Failed to check auto checkpoint:", error);
      throw error;
    }
  },

  /**
   * Triggers cleanup of old checkpoints
   */
  async cleanupOldCheckpoints(
    sessionId: string,
    projectId: string,
    projectPath: string,
    keepCount: number
  ): Promise<number> {
    try {
      return await apiCall<number>("cleanup_old_checkpoints", {
        sessionId,
        projectId,
        projectPath,
        keepCount
      });
    } catch (error) {
      console.error("Failed to cleanup old checkpoints:", error);
      throw error;
    }
  },

  /**
   * Gets checkpoint settings for a session
   */
  async getCheckpointSettings(
    sessionId: string,
    projectId: string,
    projectPath: string
  ): Promise<{
    auto_checkpoint_enabled: boolean;
    checkpoint_strategy: CheckpointStrategy;
    total_checkpoints: number;
    current_checkpoint_id?: string;
  }> {
    try {
      return await apiCall("get_checkpoint_settings", {
        sessionId,
        projectId,
        projectPath
      });
    } catch (error) {
      console.error("Failed to get checkpoint settings:", error);
      throw error;
    }
  },

  /**
   * Clears checkpoint manager for a session (cleanup on session end)
   */
  async clearCheckpointManager(sessionId: string): Promise<void> {
    try {
      await apiCall("clear_checkpoint_manager", { sessionId });
    } catch (error) {
      console.error("Failed to clear checkpoint manager:", error);
      throw error;
    }
  },

  /**
   * Tracks a batch of messages for a session for checkpointing
   */
  trackSessionMessages: (
    sessionId: string, 
    projectId: string, 
    projectPath: string, 
    messages: string[]
  ): Promise<void> =>
    apiCall("track_session_messages", { sessionId, projectId, projectPath, messages }),

  /**
   * Adds a new MCP server
   */
  async mcpAdd(
    name: string,
    transport: string,
    command?: string,
    args: string[] = [],
    env: Record<string, string> = {},
    url?: string,
    scope = "local"
  ): Promise<AddServerResult> {
    try {
      return await apiCall<AddServerResult>("mcp_add", {
        name,
        transport,
        command,
        args,
        env,
        url,
        scope
      });
    } catch (error) {
      console.error("Failed to add MCP server:", error);
      throw error;
    }
  },

  /**
   * Lists all configured MCP servers
   */
  async mcpList(): Promise<MCPServer[]> {
    try {
      console.log("API: Calling mcp_list...");
      const result = await apiCall<MCPServer[]>("mcp_list");
      console.log("API: mcp_list returned:", result);
      return result;
    } catch (error) {
      console.error("API: Failed to list MCP servers:", error);
      throw error;
    }
  },

  /**
   * Gets details for a specific MCP server
   */
  async mcpGet(name: string): Promise<MCPServer> {
    try {
      return await apiCall<MCPServer>("mcp_get", { name });
    } catch (error) {
      console.error("Failed to get MCP server:", error);
      throw error;
    }
  },

  /**
   * Removes an MCP server
   */
  async mcpRemove(name: string): Promise<string> {
    try {
      return await apiCall<string>("mcp_remove", { name });
    } catch (error) {
      console.error("Failed to remove MCP server:", error);
      throw error;
    }
  },

  /**
   * Adds an MCP server from JSON configuration
   */
  async mcpAddJson(name: string, jsonConfig: string, scope = "local"): Promise<AddServerResult> {
    try {
      return await apiCall<AddServerResult>("mcp_add_json", { name, jsonConfig, scope });
    } catch (error) {
      console.error("Failed to add MCP server from JSON:", error);
      throw error;
    }
  },

  /**
   * Imports MCP servers from Claude Desktop
   */
  async mcpAddFromClaudeDesktop(scope = "local"): Promise<ImportResult> {
    try {
      return await apiCall<ImportResult>("mcp_add_from_claude_desktop", { scope });
    } catch (error) {
      console.error("Failed to import from Claude Desktop:", error);
      throw error;
    }
  },

  /**
   * Starts Claude Code as an MCP server
   */
  async mcpServe(): Promise<string> {
    try {
      return await apiCall<string>("mcp_serve");
    } catch (error) {
      console.error("Failed to start MCP server:", error);
      throw error;
    }
  },

  /**
   * Tests connection to an MCP server
   */
  async mcpTestConnection(name: string): Promise<string> {
    try {
      return await apiCall<string>("mcp_test_connection", { name });
    } catch (error) {
      console.error("Failed to test MCP connection:", error);
      throw error;
    }
  },

  /**
   * Resets project-scoped server approval choices
   */
  async mcpResetProjectChoices(): Promise<string> {
    try {
      return await apiCall<string>("mcp_reset_project_choices");
    } catch (error) {
      console.error("Failed to reset project choices:", error);
      throw error;
    }
  },

  /**
   * Gets the status of MCP servers
   */
  async mcpGetServerStatus(): Promise<Record<string, ServerStatus>> {
    try {
      return await apiCall<Record<string, ServerStatus>>("mcp_get_server_status");
    } catch (error) {
      console.error("Failed to get server status:", error);
      throw error;
    }
  },

  /**
   * Reads .mcp.json from the current project
   */
  async mcpReadProjectConfig(projectPath: string): Promise<MCPProjectConfig> {
    try {
      return await apiCall<MCPProjectConfig>("mcp_read_project_config", { projectPath });
    } catch (error) {
      console.error("Failed to read project MCP config:", error);
      throw error;
    }
  },

  /**
   * Saves .mcp.json to the current project
   */
  async mcpSaveProjectConfig(projectPath: string, config: MCPProjectConfig): Promise<string> {
    try {
      return await apiCall<string>("mcp_save_project_config", { projectPath, config });
    } catch (error) {
      console.error("Failed to save project MCP config:", error);
      throw error;
    }
  },

  /**
   * Get the stored Claude binary path from settings
   * @returns Promise resolving to the path if set, null otherwise
   */
  async getClaudeBinaryPath(): Promise<string | null> {
    try {
      return await apiCall<string | null>("get_claude_binary_path");
    } catch (error) {
      console.error("Failed to get Claude binary path:", error);
      throw error;
    }
  },

  /**
   * Set the Claude binary path in settings
   * @param path - The absolute path to the Claude binary
   * @returns Promise resolving when the path is saved
   */
  async setClaudeBinaryPath(path: string): Promise<void> {
    try {
      await apiCall<void>("set_claude_binary_path", { path }); 
    } catch (error) {
      console.error("Failed to set Claude binary path:", error);
      throw error;
    }
  },

  /**
   * List all available Claude installations on the system
   * @returns Promise resolving to an array of Claude installations
   */
  async listClaudeInstallations(): Promise<ClaudeInstallation[]> {
    try {
      return await apiCall<ClaudeInstallation[]>("list_claude_installations");
    } catch (error) {
      console.error("Failed to list Claude installations:", error);
      throw error;
    }
  },

  // Storage API methods

  /**
   * Lists all tables in the SQLite database
   * @returns Promise resolving to an array of table information
   */
  async storageListTables(): Promise<any[]> {
    try {
      return await apiCall<any[]>("storage_list_tables");
    } catch (error) {
      console.error("Failed to list tables:", error);
      throw error;
    }
  },

  /**
   * Reads table data with pagination
   * @param tableName - Name of the table to read
   * @param page - Page number (1-indexed)
   * @param pageSize - Number of rows per page
   * @param searchQuery - Optional search query
   * @returns Promise resolving to table data with pagination info
   */
  async storageReadTable(
    tableName: string,
    page: number,
    pageSize: number,
    searchQuery?: string
  ): Promise<any> {
    try {
      return await apiCall<any>("storage_read_table", {
        tableName,
        page,
        pageSize,
        searchQuery,
      });
    } catch (error) {
      console.error("Failed to read table:", error);
      throw error;
    }
  },

  /**
   * Updates a row in a table
   * @param tableName - Name of the table
   * @param primaryKeyValues - Map of primary key column names to values
   * @param updates - Map of column names to new values
   * @returns Promise resolving when the row is updated
   */
  async storageUpdateRow(
    tableName: string,
    primaryKeyValues: Record<string, any>,
    updates: Record<string, any>
  ): Promise<void> {
    try {
      await apiCall<void>("storage_update_row", {
        tableName,
        primaryKeyValues,
        updates,
      }); 
    } catch (error) {
      console.error("Failed to update row:", error);
      throw error;
    }
  },

  /**
   * Deletes a row from a table
   * @param tableName - Name of the table
   * @param primaryKeyValues - Map of primary key column names to values
   * @returns Promise resolving when the row is deleted
   */
  async storageDeleteRow(
    tableName: string,
    primaryKeyValues: Record<string, any>
  ): Promise<void> {
    try {
      await apiCall<void>("storage_delete_row", {
        tableName,
        primaryKeyValues,
      }); 
    } catch (error) {
      console.error("Failed to delete row:", error);
      throw error;
    }
  },

  /**
   * Inserts a new row into a table
   * @param tableName - Name of the table
   * @param values - Map of column names to values
   * @returns Promise resolving to the last insert row ID
   */
  async storageInsertRow(
    tableName: string,
    values: Record<string, any>
  ): Promise<number> {
    try {
      return await apiCall<number>("storage_insert_row", {
        tableName,
        values,
      });
    } catch (error) {
      console.error("Failed to insert row:", error);
      throw error;
    }
  },

  /**
   * Executes a raw SQL query
   * @param query - SQL query string
   * @returns Promise resolving to query result
   */
  async storageExecuteSql(query: string): Promise<any> {
    try {
      return await apiCall<any>("storage_execute_sql", { query });
    } catch (error) {
      console.error("Failed to execute SQL:", error);
      throw error;
    }
  },

  /**
   * Resets the entire database
   * @returns Promise resolving when the database is reset
   */
  async storageResetDatabase(): Promise<void> {
    try {
      await apiCall<void>("storage_reset_database"); 
    } catch (error) {
      console.error("Failed to reset database:", error);
      throw error;
    }
  },

  // Theme settings helpers

  /**
   * Gets a setting from the app_settings table
   * @param key - The setting key to retrieve
   * @returns Promise resolving to the setting value or null if not found
   */
  async getSetting(key: string): Promise<string | null> {
    try {
      // Fast path: check localStorage mirror to avoid startup flicker
      if (typeof window !== 'undefined' && 'localStorage' in window) {
        const cached = window.localStorage.getItem(`app_setting:${key}`);
        if (cached !== null) {
          return cached;
        }
      }
      // Use storageReadTable to safely query the app_settings table
      const result = await this.storageReadTable('app_settings', 1, 1000);
      const setting = result?.data?.find((row: any) => row.key === key);
      return setting?.value || null;
    } catch (error) {
      console.error(`Failed to get setting ${key}:`, error);
      return null;
    }
  },

  /**
   * Saves a setting to the app_settings table (insert or update)
   * @param key - The setting key
   * @param value - The setting value
   * @returns Promise resolving when the setting is saved
   */
  async saveSetting(key: string, value: string): Promise<void> {
    try {
      // Mirror to localStorage for instant availability on next startup
      if (typeof window !== 'undefined' && 'localStorage' in window) {
        try {
          window.localStorage.setItem(`app_setting:${key}`, value);
        } catch (_ignore) {
          // best-effort; continue to persist in DB
        }
      }
      // Try to update first
      try {
        await this.storageUpdateRow(
          'app_settings',
          { key },
          { value }
        );
      } catch (updateError) {
        // If update fails (row doesn't exist), insert new row
        await this.storageInsertRow('app_settings', { key, value });
      }
    } catch (error) {
      console.error(`Failed to save setting ${key}:`, error);
      throw error;
    }
  },

  /**
   * Get hooks configuration for a specific scope
   * @param scope - The configuration scope: 'user', 'project', or 'local'
   * @param projectPath - Project path (required for project and local scopes)
   * @returns Promise resolving to the hooks configuration
   */
  async getHooksConfig(scope: 'user' | 'project' | 'local', projectPath?: string): Promise<HooksConfiguration> {
    try {
      return await apiCall<HooksConfiguration>("get_hooks_config", { scope, projectPath });
    } catch (error) {
      console.error("Failed to get hooks config:", error);
      throw error;
    }
  },

  /**
   * Update hooks configuration for a specific scope
   * @param scope - The configuration scope: 'user', 'project', or 'local'
   * @param hooks - The hooks configuration to save
   * @param projectPath - Project path (required for project and local scopes)
   * @returns Promise resolving to success message
   */
  async updateHooksConfig(
    scope: 'user' | 'project' | 'local',
    hooks: HooksConfiguration,
    projectPath?: string
  ): Promise<string> {
    try {
      return await apiCall<string>("update_hooks_config", { scope, projectPath, hooks });
    } catch (error) {
      console.error("Failed to update hooks config:", error);
      throw error;
    }
  },

  /**
   * Validate a hook command syntax
   * @param command - The shell command to validate
   * @returns Promise resolving to validation result
   */
  async validateHookCommand(command: string): Promise<{ valid: boolean; message: string }> {
    try {
      return await apiCall<{ valid: boolean; message: string }>("validate_hook_command", { command });
    } catch (error) {
      console.error("Failed to validate hook command:", error);
      throw error;
    }
  },

  /**
   * Get merged hooks configuration (respecting priority)
   * @param projectPath - The project path
   * @returns Promise resolving to merged hooks configuration
   */
  async getMergedHooksConfig(projectPath: string): Promise<HooksConfiguration> {
    try {
      const [userHooks, projectHooks, localHooks] = await Promise.all([
        this.getHooksConfig('user'),
        this.getHooksConfig('project', projectPath),
        this.getHooksConfig('local', projectPath)
      ]);

      // Import HooksManager for merging
      const { HooksManager } = await import('@/lib/hooksManager');
      return HooksManager.mergeConfigs(userHooks, projectHooks, localHooks);
    } catch (error) {
      console.error("Failed to get merged hooks config:", error);
      throw error;
    }
  },

  // Slash Commands API methods

  /**
   * Lists all available slash commands
   * @param projectPath - Optional project path to include project-specific commands
   * @returns Promise resolving to array of slash commands
   */
  async slashCommandsList(projectPath?: string): Promise<SlashCommand[]> {
    try {
      return await apiCall<SlashCommand[]>("slash_commands_list", { projectPath });
    } catch (error) {
      console.error("Failed to list slash commands:", error);
      throw error;
    }
  },

  /**
   * Gets a single slash command by ID
   * @param commandId - Unique identifier of the command
   * @returns Promise resolving to the slash command
   */
  async slashCommandGet(commandId: string): Promise<SlashCommand> {
    try {
      return await apiCall<SlashCommand>("slash_command_get", { commandId });
    } catch (error) {
      console.error("Failed to get slash command:", error);
      throw error;
    }
  },

  /**
   * Creates or updates a slash command
   * @param scope - Command scope: "project" or "user"
   * @param name - Command name (without prefix)
   * @param namespace - Optional namespace for organization
   * @param content - Markdown content of the command
   * @param description - Optional description
   * @param allowedTools - List of allowed tools for this command
   * @param projectPath - Required for project scope commands
   * @returns Promise resolving to the saved command
   */
  async slashCommandSave(
    scope: string,
    name: string,
    namespace: string | undefined,
    content: string,
    description: string | undefined,
    allowedTools: string[],
    projectPath?: string
  ): Promise<SlashCommand> {
    try {
      return await apiCall<SlashCommand>("slash_command_save", {
        scope,
        name,
        namespace,
        content,
        description,
        allowedTools,
        projectPath
      });
    } catch (error) {
      console.error("Failed to save slash command:", error);
      throw error;
    }
  },

  /**
   * Deletes a slash command
   * @param commandId - Unique identifier of the command to delete
   * @param projectPath - Optional project path for deleting project commands
   * @returns Promise resolving to deletion message
   */
  async slashCommandDelete(commandId: string, projectPath?: string): Promise<string> {
    try {
      return await apiCall<string>("slash_command_delete", { commandId, projectPath });
    } catch (error) {
      console.error("Failed to delete slash command:", error);
      throw error;
    }
  },

  // =====================================
  // Skills API
  // =====================================

  /**
   * Lists all available skills
   * @param projectPath - Optional project path to include project-specific skills
   * @returns Promise resolving to array of skills
   */
  async skillsList(projectPath?: string): Promise<Skill[]> {
    try {
      return await apiCall<Skill[]>("skills_list", { projectPath });
    } catch (error) {
      console.error("Failed to list skills:", error);
      throw error;
    }
  },

  /**
   * Gets a single skill by ID
   * @param skillId - Unique identifier of the skill
   * @param projectPath - Optional project path
   * @returns Promise resolving to the skill
   */
  async skillGet(skillId: string, projectPath?: string): Promise<Skill> {
    try {
      return await apiCall<Skill>("skill_get", { skillId, projectPath });
    } catch (error) {
      console.error("Failed to get skill:", error);
      throw error;
    }
  },

  /**
   * Creates or updates a skill
   * @param scope - Skill scope: "project" or "user"
   * @param name - Skill name (folder name)
   * @param content - Content of the skill
   * @param fileType - File type: "markdown" or "json"
   * @param description - Optional description
   * @param allowedTools - List of allowed tools for this skill
   * @param projectPath - Required for project scope skills
   * @returns Promise resolving to the saved skill
   */
  async skillSave(
    scope: string,
    name: string,
    content: string,
    fileType: string,
    description: string | undefined,
    allowedTools: string[],
    projectPath?: string
  ): Promise<Skill> {
    try {
      return await apiCall<Skill>("skill_save", {
        scope,
        name,
        content,
        fileType,
        description,
        allowedTools,
        projectPath
      });
    } catch (error) {
      console.error("Failed to save skill:", error);
      throw error;
    }
  },

  /**
   * Deletes a skill
   * @param skillId - Unique identifier of the skill to delete
   * @param projectPath - Optional project path for deleting project skills
   * @returns Promise resolving to deletion message
   */
  async skillDelete(skillId: string, projectPath?: string): Promise<string> {
    try {
      return await apiCall<string>("skill_delete", { skillId, projectPath });
    } catch (error) {
      console.error("Failed to delete skill:", error);
      throw error;
    }
  },

  /**
   * Reads a supporting file from a skill
   * @param filePath - Path to the file
   * @returns Promise resolving to file content
   */
  async skillReadFile(filePath: string): Promise<string> {
    try {
      return await apiCall<string>("skill_read_file", { filePath });
    } catch (error) {
      console.error("Failed to read skill file:", error);
      throw error;
    }
  },

  /**
   * Saves a supporting file for a skill
   * @param skillDir - Path to the skill directory
   * @param fileName - Name of the file to save
   * @param content - Content to write
   * @returns Promise resolving to file path
   */
  async skillSaveFile(skillDir: string, fileName: string, content: string): Promise<string> {
    try {
      return await apiCall<string>("skill_save_file", { skillDir, fileName, content });
    } catch (error) {
      console.error("Failed to save skill file:", error);
      throw error;
    }
  },

  /**
   * Deletes a supporting file from a skill
   * @param filePath - Path to the file to delete
   * @returns Promise resolving to deletion message
   */
  async skillDeleteFile(filePath: string): Promise<string> {
    try {
      return await apiCall<string>("skill_delete_file", { filePath });
    } catch (error) {
      console.error("Failed to delete skill file:", error);
      throw error;
    }
  },

  // =====================================
  // Plugins API
  // =====================================

  /**
   * Lists all registered plugin marketplaces
   * @returns Promise resolving to array of marketplaces
   */
  async pluginsListMarketplaces(): Promise<Marketplace[]> {
    try {
      return await apiCall<Marketplace[]>("plugins_list_marketplaces");
    } catch (error) {
      console.error("Failed to list marketplaces:", error);
      throw error;
    }
  },

  /**
   * Adds a new marketplace
   * @param source - GitHub repo (owner/repo), URL, or local path
   * @returns Promise resolving to the added marketplace
   */
  async pluginsAddMarketplace(source: string): Promise<Marketplace> {
    try {
      return await apiCall<Marketplace>("plugins_add_marketplace", { source });
    } catch (error) {
      console.error("Failed to add marketplace:", error);
      throw error;
    }
  },

  /**
   * Removes a marketplace
   * @param source - The marketplace source to remove
   * @returns Promise resolving when removed
   */
  async pluginsRemoveMarketplace(source: string): Promise<void> {
    try {
      await apiCall("plugins_remove_marketplace", { source });
    } catch (error) {
      console.error("Failed to remove marketplace:", error);
      throw error;
    }
  },

  /**
   * Lists all installed plugins
   * @returns Promise resolving to array of installed plugins
   */
  async pluginsListInstalled(): Promise<Plugin[]> {
    try {
      return await apiCall<Plugin[]>("plugins_list_installed");
    } catch (error) {
      console.error("Failed to list installed plugins:", error);
      throw error;
    }
  },

  /**
   * Gets details for a specific plugin
   * @param pluginPath - Path to the plugin directory
   * @returns Promise resolving to the plugin details
   */
  async pluginsGetDetails(pluginPath: string): Promise<Plugin> {
    try {
      return await apiCall<Plugin>("plugins_get_details", { pluginPath });
    } catch (error) {
      console.error("Failed to get plugin details:", error);
      throw error;
    }
  },

  /**
   * Installs a plugin from a marketplace
   * @param pluginName - Name of the plugin to install
   * @param marketplace - Marketplace source
   * @param scope - Installation scope: "local" or "project"
   * @returns Promise resolving to the installed plugin
   */
  async pluginsInstall(pluginName: string, marketplace: string, scope: string): Promise<Plugin> {
    try {
      return await apiCall<Plugin>("plugins_install", { pluginName, marketplace, scope });
    } catch (error) {
      console.error("Failed to install plugin:", error);
      throw error;
    }
  },

  /**
   * Uninstalls a plugin
   * @param pluginName - Name of the plugin to uninstall
   * @returns Promise resolving when uninstalled
   */
  async pluginsUninstall(pluginName: string): Promise<void> {
    try {
      await apiCall("plugins_uninstall", { pluginName });
    } catch (error) {
      console.error("Failed to uninstall plugin:", error);
      throw error;
    }
  },

  /**
   * Enables a plugin
   * @param pluginName - Name of the plugin to enable
   * @returns Promise resolving when enabled
   */
  async pluginsEnable(pluginName: string): Promise<void> {
    try {
      await apiCall("plugins_enable", { pluginName });
    } catch (error) {
      console.error("Failed to enable plugin:", error);
      throw error;
    }
  },

  /**
   * Disables a plugin
   * @param pluginName - Name of the plugin to disable
   * @returns Promise resolving when disabled
   */
  async pluginsDisable(pluginName: string): Promise<void> {
    try {
      await apiCall("plugins_disable", { pluginName });
    } catch (error) {
      console.error("Failed to disable plugin:", error);
      throw error;
    }
  },

  /**
   * Fetches available plugins from a marketplace
   * @param source - Marketplace source
   * @returns Promise resolving to array of available plugins
   */
  async pluginsFetchMarketplace(source: string): Promise<Plugin[]> {
    try {
      return await apiCall<Plugin[]>("plugins_fetch_marketplace", { source });
    } catch (error) {
      console.error("Failed to fetch marketplace plugins:", error);
      throw error;
    }
  },

  /**
   * Reads a plugin's README
   * @param pluginPath - Path to the plugin directory
   * @returns Promise resolving to README content
   */
  async pluginsReadReadme(pluginPath: string): Promise<string> {
    try {
      return await apiCall<string>("plugins_read_readme", { pluginPath });
    } catch (error) {
      console.error("Failed to read plugin README:", error);
      throw error;
    }
  },

  /**
   * Reads a plugin component file
   * @param pluginPath - Path to the plugin directory
   * @param componentType - Type: "command", "agent", "skill", "hook", "mcp"
   * @param componentName - Name of the component
   * @returns Promise resolving to component content
   */
  async pluginsReadComponent(pluginPath: string, componentType: string, componentName: string): Promise<string> {
    try {
      return await apiCall<string>("plugins_read_component", { pluginPath, componentType, componentName });
    } catch (error) {
      console.error("Failed to read plugin component:", error);
      throw error;
    }
  },

  /**
   * Saves a plugin component file
   * @param pluginPath - Path to the plugin directory
   * @param componentType - Type: "command", "agent", "skill", "hook", "mcp"
   * @param componentName - Name of the component
   * @param content - Content to save
   * @returns Promise resolving when saved
   */
  async pluginsSaveComponent(pluginPath: string, componentType: string, componentName: string, content: string): Promise<void> {
    try {
      await apiCall("plugins_save_component", { pluginPath, componentType, componentName, content });
    } catch (error) {
      console.error("Failed to save plugin component:", error);
      throw error;
    }
  },

  /**
   * Creates a new plugin
   * @param name - Plugin name
   * @param description - Plugin description
   * @returns Promise resolving to the created plugin
   */
  async pluginsCreate(name: string, description: string): Promise<Plugin> {
    try {
      return await apiCall<Plugin>("plugins_create", { name, description });
    } catch (error) {
      console.error("Failed to create plugin:", error);
      throw error;
    }
  },

  /**
   * Deletes a plugin
   * @param pluginPath - Path to the plugin directory
   * @returns Promise resolving when deleted
   */
  async pluginsDelete(pluginPath: string): Promise<void> {
    try {
      await apiCall("plugins_delete", { pluginPath });
    } catch (error) {
      console.error("Failed to delete plugin:", error);
      throw error;
    }
  },

  // =====================================
  // CLI Tools API
  // =====================================

  /**
   * Lists all CLI tools with their installation status
   * @returns Promise resolving to CLI tools status
   */
  async listCLITools(): Promise<CLIToolsStatus> {
    try {
      return await apiCall<CLIToolsStatus>("cli_tools_list");
    } catch (error) {
      console.error("Failed to list CLI tools:", error);
      throw error;
    }
  },

  /**
   * Gets all installations for a specific CLI tool
   * @param toolType - The type of CLI tool
   * @returns Promise resolving to array of installations
   */
  async getCLIToolInstallations(toolType: CLIToolType): Promise<CLIToolInstallation[]> {
    try {
      return await apiCall<CLIToolInstallation[]>("cli_tool_get_installations", { toolType });
    } catch (error) {
      console.error("Failed to get CLI tool installations:", error);
      throw error;
    }
  },

  /**
   * Sets the preferred installation for a CLI tool
   * @param toolType - The type of CLI tool
   * @param path - The path to set as preferred
   * @returns Promise resolving when the preference is saved
   */
  async setCLIToolPreferred(toolType: CLIToolType, path: string): Promise<void> {
    try {
      await apiCall("cli_tool_set_preferred", { toolType, path });
    } catch (error) {
      console.error("Failed to set CLI tool preferred:", error);
      throw error;
    }
  },

  /**
   * Gets the preferred installation path for a CLI tool
   * @param toolType - The type of CLI tool
   * @returns Promise resolving to the preferred path or null
   */
  async getCLIToolPreferred(toolType: CLIToolType): Promise<string | null> {
    try {
      return await apiCall<string | null>("cli_tool_get_preferred", { toolType });
    } catch (error) {
      console.error("Failed to get CLI tool preferred:", error);
      throw error;
    }
  },

  /**
   * Forces a refresh of CLI tools detection
   * @returns Promise resolving to updated CLI tools status
   */
  async refreshCLITools(): Promise<CLIToolsStatus> {
    try {
      return await apiCall<CLIToolsStatus>("cli_tools_refresh");
    } catch (error) {
      console.error("Failed to refresh CLI tools:", error);
      throw error;
    }
  },

  /**
   * Checks if a CLI tool is available
   * @param toolType - The type of CLI tool
   * @returns Promise resolving to whether the tool is available
   */
  async isCLIToolAvailable(toolType: CLIToolType): Promise<boolean> {
    try {
      return await apiCall<boolean>("cli_tool_is_available", { toolType });
    } catch (error) {
      console.error("Failed to check CLI tool availability:", error);
      throw error;
    }
  },

  /**
   * Gets the command to execute for a CLI tool
   * @param toolType - The type of CLI tool
   * @returns Promise resolving to the command or null if not available
   */
  async getCLIToolCommand(toolType: CLIToolType): Promise<string | null> {
    try {
      return await apiCall<string | null>("cli_tool_get_command", { toolType });
    } catch (error) {
      console.error("Failed to get CLI tool command:", error);
      throw error;
    }
  },

  // =====================================
  // CLI Tools Configuration Management API
  // =====================================

  /**
   * Lists configuration files for a CLI tool (metadata only, lazy loading)
   * @param toolType - The type of CLI tool
   * @returns Promise resolving to array of config file info
   */
  async listCLIToolConfigFiles(toolType: CLIToolType): Promise<ConfigFileInfo[]> {
    try {
      return await apiCall<ConfigFileInfo[]>("cli_tool_list_config_files", { toolType });
    } catch (error) {
      console.error("Failed to list CLI tool config files:", error);
      throw error;
    }
  },

  /**
   * Reads a specific config file (on-demand loading)
   * @param toolType - The type of CLI tool
   * @param path - Path to the config file
   * @returns Promise resolving to the config file content
   */
  async readCLIToolConfigFile(toolType: CLIToolType, path: string): Promise<ConfigFileContent> {
    try {
      return await apiCall<ConfigFileContent>("cli_tool_read_config_file", { toolType, path });
    } catch (error) {
      console.error("Failed to read CLI tool config file:", error);
      throw error;
    }
  },

  /**
   * Writes a config file
   * @param toolType - The type of CLI tool
   * @param path - Path to the config file
   * @param content - Content to write
   * @returns Promise resolving when the file is written
   */
  async writeCLIToolConfigFile(toolType: CLIToolType, path: string, content: string): Promise<void> {
    try {
      await apiCall("cli_tool_write_config_file", { toolType, path, content });
    } catch (error) {
      console.error("Failed to write CLI tool config file:", error);
      throw error;
    }
  },

  /**
   * Gets structured settings for a CLI tool
   * @param toolType - The type of CLI tool
   * @returns Promise resolving to the tool settings
   */
  async getCLIToolSettings(toolType: CLIToolType): Promise<ToolSettings> {
    try {
      return await apiCall<ToolSettings>("cli_tool_get_settings", { toolType });
    } catch (error) {
      console.error("Failed to get CLI tool settings:", error);
      throw error;
    }
  },

  /**
   * Sets a specific setting for a CLI tool
   * @param toolType - The type of CLI tool
   * @param key - Setting key
   * @param value - Setting value
   * @returns Promise resolving when the setting is saved
   */
  async setCLIToolSetting(toolType: CLIToolType, key: string, value: unknown): Promise<void> {
    try {
      await apiCall("cli_tool_set_setting", { toolType, key, value });
    } catch (error) {
      console.error("Failed to set CLI tool setting:", error);
      throw error;
    }
  },

  /**
   * Lists MCP servers configured for a CLI tool
   * @param toolType - The type of CLI tool
   * @returns Promise resolving to array of MCP server configs
   */
  async listCLIToolMCPServers(toolType: CLIToolType): Promise<CLIToolMCPServerConfig[]> {
    try {
      return await apiCall<CLIToolMCPServerConfig[]>("cli_tool_list_mcp_servers", { toolType });
    } catch (error) {
      console.error("Failed to list CLI tool MCP servers:", error);
      throw error;
    }
  },

  /**
   * Adds an MCP server to a CLI tool
   * @param toolType - The type of CLI tool
   * @param config - MCP server configuration input
   * @returns Promise resolving when the server is added
   */
  async addCLIToolMCPServer(toolType: CLIToolType, config: MCPServerInput): Promise<void> {
    try {
      await apiCall("cli_tool_add_mcp_server", { toolType, ...config });
    } catch (error) {
      console.error("Failed to add CLI tool MCP server:", error);
      throw error;
    }
  },

  /**
   * Removes an MCP server from a CLI tool
   * @param toolType - The type of CLI tool
   * @param name - Name of the MCP server to remove
   * @returns Promise resolving when the server is removed
   */
  async removeCLIToolMCPServer(toolType: CLIToolType, name: string): Promise<void> {
    try {
      await apiCall("cli_tool_remove_mcp_server", { toolType, name });
    } catch (error) {
      console.error("Failed to remove CLI tool MCP server:", error);
      throw error;
    }
  },

  /**
   * Lists agents/commands for a CLI tool
   * @param toolType - The type of CLI tool
   * @returns Promise resolving to array of agent definitions
   */
  async listCLIToolAgents(toolType: CLIToolType): Promise<CLIToolAgentDefinition[]> {
    try {
      return await apiCall<CLIToolAgentDefinition[]>("cli_tool_list_agents", { toolType });
    } catch (error) {
      console.error("Failed to list CLI tool agents:", error);
      throw error;
    }
  },

  /**
   * Gets a specific agent/command for a CLI tool
   * @param toolType - The type of CLI tool
   * @param name - Name of the agent
   * @returns Promise resolving to the agent definition
   */
  async getCLIToolAgent(toolType: CLIToolType, name: string): Promise<CLIToolAgentDefinition> {
    try {
      return await apiCall<CLIToolAgentDefinition>("cli_tool_get_agent", { toolType, name });
    } catch (error) {
      console.error("Failed to get CLI tool agent:", error);
      throw error;
    }
  },

  /**
   * Executes a CLI tool command
   * @param toolType - The type of CLI tool
   * @param command - Command to execute
   * @param args - Command arguments
   * @returns Promise resolving to the command output
   */
  async executeCLIToolCommand(toolType: CLIToolType, command: string, args: string[]): Promise<CommandOutput> {
    try {
      return await apiCall<CommandOutput>("cli_tool_execute_cli_command", { toolType, command, args });
    } catch (error) {
      console.error("Failed to execute CLI tool command:", error);
      throw error;
    }
  },

  /**
   * Gets the config directory for a CLI tool
   * @param toolType - The type of CLI tool
   * @returns Promise resolving to the config directory path
   */
  async getCLIToolConfigDir(toolType: CLIToolType): Promise<string> {
    try {
      return await apiCall<string>("cli_tool_get_config_dir", { toolType });
    } catch (error) {
      console.error("Failed to get CLI tool config directory:", error);
      throw error;
    }
  },

  // =========================================
  // CLI Tools - Usage Tracking
  // =========================================

  /**
   * Tracks a usage action for a CLI tool
   * @param toolType - The type of CLI tool
   * @param action - The action being tracked
   * @param details - Optional details about the action
   */
  async trackCLIToolUsage(
    toolType: CLIToolType,
    action: UsageAction,
    details?: string
  ): Promise<void> {
    try {
      await apiCall("cli_tool_track_usage", { toolType, action, details });
    } catch (error) {
      console.error("Failed to track CLI tool usage:", error);
      throw error;
    }
  },

  /**
   * Gets usage history for a CLI tool
   * @param toolType - The type of CLI tool
   * @param limit - Maximum number of entries to return
   * @returns Promise resolving to an array of usage entries
   */
  async getCLIToolUsage(
    toolType: CLIToolType,
    limit: number = 100
  ): Promise<CLIToolUsageEntry[]> {
    try {
      return await apiCall<CLIToolUsageEntry[]>("cli_tool_get_usage", { toolType, limit });
    } catch (error) {
      console.error("Failed to get CLI tool usage:", error);
      throw error;
    }
  },

  /**
   * Gets usage statistics for a CLI tool
   * @param toolType - The type of CLI tool
   * @returns Promise resolving to usage statistics
   */
  async getCLIToolUsageStats(toolType: CLIToolType): Promise<CLIToolUsageStats> {
    try {
      return await apiCall<CLIToolUsageStats>("cli_tool_get_usage_stats", { toolType });
    } catch (error) {
      console.error("Failed to get CLI tool usage stats:", error);
      throw error;
    }
  },

  /**
   * Clears usage history for a CLI tool
   * @param toolType - The type of CLI tool
   */
  async clearCLIToolUsage(toolType: CLIToolType): Promise<void> {
    try {
      await apiCall("cli_tool_clear_usage", { toolType });
    } catch (error) {
      console.error("Failed to clear CLI tool usage:", error);
      throw error;
    }
  },

  // =========================================
  // File Operations API
  // =========================================

  /**
   * Copies a file or directory
   * @param source - Source path
   * @param destination - Destination path
   * @param overwrite - Whether to overwrite existing files
   * @returns Promise resolving to operation result
   */
  async fileOpsCopy(source: string, destination: string, overwrite: boolean = false): Promise<FileOperationResult> {
    try {
      return await apiCall<FileOperationResult>("file_ops_copy", { source, destination, overwrite });
    } catch (error) {
      console.error("Failed to copy file:", error);
      throw error;
    }
  },

  /**
   * Moves a file or directory
   * @param source - Source path
   * @param destination - Destination path
   * @param overwrite - Whether to overwrite existing files
   * @returns Promise resolving to operation result
   */
  async fileOpsMove(source: string, destination: string, overwrite: boolean = false): Promise<FileOperationResult> {
    try {
      return await apiCall<FileOperationResult>("file_ops_move", { source, destination, overwrite });
    } catch (error) {
      console.error("Failed to move file:", error);
      throw error;
    }
  },

  /**
   * Deletes a file or directory
   * @param path - Path to delete
   * @param createBackup - Whether to create a backup before deleting
   * @returns Promise resolving to operation result
   */
  async fileOpsDelete(path: string, createBackup: boolean = true): Promise<FileOperationResult> {
    try {
      return await apiCall<FileOperationResult>("file_ops_delete", { path, createBackup });
    } catch (error) {
      console.error("Failed to delete file:", error);
      throw error;
    }
  },

  /**
   * Previews a markdown merge operation
   * @param source - Source file path
   * @param target - Target file path
   * @param options - Merge options
   * @returns Promise resolving to merge preview
   */
  async fileOpsPreviewMerge(source: string, target: string, options?: MergeOptions): Promise<MergePreview> {
    try {
      return await apiCall<MergePreview>("file_ops_preview_merge", { source, target, options });
    } catch (error) {
      console.error("Failed to preview merge:", error);
      throw error;
    }
  },

  /**
   * Merges two markdown files
   * @param source - Source file path
   * @param target - Target file path
   * @param options - Merge options
   * @returns Promise resolving to operation result
   */
  async fileOpsMergeMarkdown(source: string, target: string, options?: MergeOptions): Promise<FileOperationResult> {
    try {
      return await apiCall<FileOperationResult>("file_ops_merge_markdown", { source, target, options });
    } catch (error) {
      console.error("Failed to merge markdown files:", error);
      throw error;
    }
  },

  /**
   * Executes bulk file operations
   * @param operations - Array of file operations
   * @param stopOnError - Whether to stop on first error
   * @returns Promise resolving to array of operation results
   */
  async fileOpsBulk(operations: FileOperation[], stopOnError: boolean = false): Promise<FileOperationResult[]> {
    try {
      return await apiCall<FileOperationResult[]>("file_ops_bulk", { operations, stopOnError });
    } catch (error) {
      console.error("Failed to execute bulk file operations:", error);
      throw error;
    }
  },

  /**
   * Clones a skill to a new name
   * @param skillName - Name of the skill to clone
   * @param newName - New name for the cloned skill
   * @returns Promise resolving to operation result
   */
  async fileOpsCloneSkill(skillName: string, newName: string): Promise<FileOperationResult> {
    try {
      return await apiCall<FileOperationResult>("file_ops_clone_skill", { skillName, newName });
    } catch (error) {
      console.error("Failed to clone skill:", error);
      throw error;
    }
  },

  /**
   * Clones an agent to a new name
   * @param agentId - ID of the agent to clone
   * @param newName - New name for the cloned agent
   * @returns Promise resolving to operation result
   */
  async fileOpsCloneAgent(agentId: number, newName: string): Promise<FileOperationResult> {
    try {
      return await apiCall<FileOperationResult>("file_ops_clone_agent", { agentId, newName });
    } catch (error) {
      console.error("Failed to clone agent:", error);
      throw error;
    }
  },

};
