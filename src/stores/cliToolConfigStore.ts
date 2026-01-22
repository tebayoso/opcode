import { create } from 'zustand';
import { subscribeWithSelector } from 'zustand/middleware';
import type { StateCreator } from 'zustand';
import { api } from '@/lib/api';
import type {
  CLIToolType,
  ConfigFileInfo,
  ConfigFileContent,
  ToolSettings,
  CLIToolMCPServerConfig,
  CLIToolAgentDefinition,
  MCPServerInput,
  CLIToolUsageStats,
  UsageAction,
} from '@/types/cli-tools';

/**
 * Per-tool configuration state
 */
interface ToolConfigState {
  /** Configuration files metadata (lazy loaded) */
  files: ConfigFileInfo[] | null;
  /** Loaded file contents (keyed by path) */
  loadedFiles: Record<string, ConfigFileContent>;
  /** Tool settings (lazy loaded) */
  settings: ToolSettings | null;
  /** MCP servers (lazy loaded) */
  mcpServers: CLIToolMCPServerConfig[] | null;
  /** Agents/commands (lazy loaded) */
  agents: CLIToolAgentDefinition[] | null;
  /** Config directory path */
  configDir: string | null;
  /** Usage statistics (lazy loaded) */
  usageStats: CLIToolUsageStats | null;
}

/**
 * Loading states per tool and operation
 */
interface LoadingStates {
  files: Record<CLIToolType, boolean>;
  fileContent: Record<string, boolean>; // keyed by "toolType:path"
  settings: Record<CLIToolType, boolean>;
  mcpServers: Record<CLIToolType, boolean>;
  agents: Record<CLIToolType, boolean>;
  usage: Record<CLIToolType, boolean>;
}

/**
 * CLI Tool Configuration Store State
 */
interface CLIToolConfigState {
  // Per-tool config state
  configs: Partial<Record<CLIToolType, ToolConfigState>>;

  // Loading states
  loading: LoadingStates;

  // Error state
  error: string | null;

  // Cache management
  lastFetchTimes: Partial<Record<CLIToolType, Record<string, number>>>;
  cacheDuration: number; // 60 seconds for config data

  // Actions - Config files
  loadConfigFiles: (toolType: CLIToolType) => Promise<void>;
  loadConfigFile: (toolType: CLIToolType, path: string) => Promise<ConfigFileContent>;
  saveConfigFile: (toolType: CLIToolType, path: string, content: string) => Promise<void>;
  invalidateConfigFile: (toolType: CLIToolType, path: string) => void;

  // Actions - Settings
  loadSettings: (toolType: CLIToolType) => Promise<void>;
  updateSetting: (toolType: CLIToolType, key: string, value: unknown) => Promise<void>;

  // Actions - MCP Servers
  loadMCPServers: (toolType: CLIToolType) => Promise<void>;
  addMCPServer: (toolType: CLIToolType, config: MCPServerInput) => Promise<void>;
  removeMCPServer: (toolType: CLIToolType, name: string) => Promise<void>;

  // Actions - Agents
  loadAgents: (toolType: CLIToolType) => Promise<void>;
  getAgent: (toolType: CLIToolType, name: string) => Promise<CLIToolAgentDefinition>;

  // Actions - Config directory
  loadConfigDir: (toolType: CLIToolType) => Promise<void>;

  // Actions - Usage Tracking
  loadUsageStats: (toolType: CLIToolType) => Promise<void>;
  trackUsage: (toolType: CLIToolType, action: UsageAction, details?: string) => Promise<void>;
  clearUsage: (toolType: CLIToolType) => Promise<void>;

  // Utility actions
  clearError: () => void;
  clearToolCache: (toolType: CLIToolType) => void;
  clearAllCache: () => void;

  // Selectors
  getToolConfig: (toolType: CLIToolType) => ToolConfigState | undefined;
  isLoading: (toolType: CLIToolType, operation: keyof LoadingStates) => boolean;
}

/**
 * Create initial empty tool config state
 */
const createEmptyToolConfig = (): ToolConfigState => ({
  files: null,
  loadedFiles: {},
  settings: null,
  mcpServers: null,
  agents: null,
  configDir: null,
  usageStats: null,
});

/**
 * Create initial loading states
 */
const createInitialLoadingStates = (): LoadingStates => ({
  files: {} as Record<CLIToolType, boolean>,
  fileContent: {},
  settings: {} as Record<CLIToolType, boolean>,
  mcpServers: {} as Record<CLIToolType, boolean>,
  agents: {} as Record<CLIToolType, boolean>,
  usage: {} as Record<CLIToolType, boolean>,
});

const cliToolConfigStore: StateCreator<
  CLIToolConfigState,
  [],
  [['zustand/subscribeWithSelector', never]],
  CLIToolConfigState
> = (set, get) => ({
  // Initial state
  configs: {},
  loading: createInitialLoadingStates(),
  error: null,
  lastFetchTimes: {},
  cacheDuration: 60000, // 60 seconds

  // =========================================
  // Config Files Actions
  // =========================================

  loadConfigFiles: async (toolType: CLIToolType) => {
    const { loading, lastFetchTimes, cacheDuration, configs } = get();
    const now = Date.now();

    // Check cache
    const lastFetch = lastFetchTimes[toolType]?.files;
    if (lastFetch && now - lastFetch < cacheDuration && configs[toolType]?.files) {
      return;
    }

    // Prevent concurrent loads
    if (loading.files[toolType]) {
      return;
    }

    set((state) => ({
      loading: {
        ...state.loading,
        files: { ...state.loading.files, [toolType]: true },
      },
      error: null,
    }));

    try {
      const files = await api.listCLIToolConfigFiles(toolType);

      set((state) => ({
        configs: {
          ...state.configs,
          [toolType]: {
            ...(state.configs[toolType] ?? createEmptyToolConfig()),
            files,
          },
        },
        loading: {
          ...state.loading,
          files: { ...state.loading.files, [toolType]: false },
        },
        lastFetchTimes: {
          ...state.lastFetchTimes,
          [toolType]: {
            ...(state.lastFetchTimes[toolType] ?? {}),
            files: now,
          },
        },
      }));
    } catch (error) {
      set((state) => ({
        loading: {
          ...state.loading,
          files: { ...state.loading.files, [toolType]: false },
        },
        error: error instanceof Error ? error.message : 'Failed to load config files',
      }));
    }
  },

  loadConfigFile: async (toolType: CLIToolType, path: string) => {
    const { configs, loading } = get();
    const cacheKey = `${toolType}:${path}`;

    // Check if already loaded
    const existingContent = configs[toolType]?.loadedFiles[path];
    if (existingContent) {
      return existingContent;
    }

    // Prevent concurrent loads
    if (loading.fileContent[cacheKey]) {
      throw new Error('File is already being loaded');
    }

    set((state) => ({
      loading: {
        ...state.loading,
        fileContent: { ...state.loading.fileContent, [cacheKey]: true },
      },
      error: null,
    }));

    try {
      const content = await api.readCLIToolConfigFile(toolType, path);

      set((state) => ({
        configs: {
          ...state.configs,
          [toolType]: {
            ...(state.configs[toolType] ?? createEmptyToolConfig()),
            loadedFiles: {
              ...(state.configs[toolType]?.loadedFiles ?? {}),
              [path]: content,
            },
          },
        },
        loading: {
          ...state.loading,
          fileContent: { ...state.loading.fileContent, [cacheKey]: false },
        },
      }));

      return content;
    } catch (error) {
      set((state) => ({
        loading: {
          ...state.loading,
          fileContent: { ...state.loading.fileContent, [cacheKey]: false },
        },
        error: error instanceof Error ? error.message : 'Failed to load config file',
      }));
      throw error;
    }
  },

  saveConfigFile: async (toolType: CLIToolType, path: string, content: string) => {
    set({ error: null });

    try {
      await api.writeCLIToolConfigFile(toolType, path, content);

      // Invalidate the cached file content to force reload
      get().invalidateConfigFile(toolType, path);

      // Also refresh the file list to get updated metadata
      set((state) => ({
        lastFetchTimes: {
          ...state.lastFetchTimes,
          [toolType]: {
            ...(state.lastFetchTimes[toolType] ?? {}),
            files: 0, // Force refresh on next load
          },
        },
      }));
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to save config file',
      });
      throw error;
    }
  },

  invalidateConfigFile: (toolType: CLIToolType, path: string) => {
    set((state) => {
      const toolConfig = state.configs[toolType];
      if (!toolConfig) return state;

      const { [path]: _, ...remainingFiles } = toolConfig.loadedFiles;
      return {
        configs: {
          ...state.configs,
          [toolType]: {
            ...toolConfig,
            loadedFiles: remainingFiles,
          },
        },
      };
    });
  },

  // =========================================
  // Settings Actions
  // =========================================

  loadSettings: async (toolType: CLIToolType) => {
    const { loading, lastFetchTimes, cacheDuration, configs } = get();
    const now = Date.now();

    // Check cache
    const lastFetch = lastFetchTimes[toolType]?.settings;
    if (lastFetch && now - lastFetch < cacheDuration && configs[toolType]?.settings) {
      return;
    }

    // Prevent concurrent loads
    if (loading.settings[toolType]) {
      return;
    }

    set((state) => ({
      loading: {
        ...state.loading,
        settings: { ...state.loading.settings, [toolType]: true },
      },
      error: null,
    }));

    try {
      const settings = await api.getCLIToolSettings(toolType);

      set((state) => ({
        configs: {
          ...state.configs,
          [toolType]: {
            ...(state.configs[toolType] ?? createEmptyToolConfig()),
            settings,
          },
        },
        loading: {
          ...state.loading,
          settings: { ...state.loading.settings, [toolType]: false },
        },
        lastFetchTimes: {
          ...state.lastFetchTimes,
          [toolType]: {
            ...(state.lastFetchTimes[toolType] ?? {}),
            settings: now,
          },
        },
      }));
    } catch (error) {
      set((state) => ({
        loading: {
          ...state.loading,
          settings: { ...state.loading.settings, [toolType]: false },
        },
        error: error instanceof Error ? error.message : 'Failed to load settings',
      }));
    }
  },

  updateSetting: async (toolType: CLIToolType, key: string, value: unknown) => {
    set({ error: null });

    try {
      await api.setCLIToolSetting(toolType, key, value);

      // Invalidate settings cache to force reload
      set((state) => ({
        lastFetchTimes: {
          ...state.lastFetchTimes,
          [toolType]: {
            ...(state.lastFetchTimes[toolType] ?? {}),
            settings: 0,
          },
        },
      }));

      // Reload settings
      await get().loadSettings(toolType);
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to update setting',
      });
      throw error;
    }
  },

  // =========================================
  // MCP Servers Actions
  // =========================================

  loadMCPServers: async (toolType: CLIToolType) => {
    const { loading, lastFetchTimes, cacheDuration, configs } = get();
    const now = Date.now();

    // Check cache
    const lastFetch = lastFetchTimes[toolType]?.mcpServers;
    if (lastFetch && now - lastFetch < cacheDuration && configs[toolType]?.mcpServers) {
      return;
    }

    // Prevent concurrent loads
    if (loading.mcpServers[toolType]) {
      return;
    }

    set((state) => ({
      loading: {
        ...state.loading,
        mcpServers: { ...state.loading.mcpServers, [toolType]: true },
      },
      error: null,
    }));

    try {
      const mcpServers = await api.listCLIToolMCPServers(toolType);

      set((state) => ({
        configs: {
          ...state.configs,
          [toolType]: {
            ...(state.configs[toolType] ?? createEmptyToolConfig()),
            mcpServers,
          },
        },
        loading: {
          ...state.loading,
          mcpServers: { ...state.loading.mcpServers, [toolType]: false },
        },
        lastFetchTimes: {
          ...state.lastFetchTimes,
          [toolType]: {
            ...(state.lastFetchTimes[toolType] ?? {}),
            mcpServers: now,
          },
        },
      }));
    } catch (error) {
      set((state) => ({
        loading: {
          ...state.loading,
          mcpServers: { ...state.loading.mcpServers, [toolType]: false },
        },
        error: error instanceof Error ? error.message : 'Failed to load MCP servers',
      }));
    }
  },

  addMCPServer: async (toolType: CLIToolType, config: MCPServerInput) => {
    set({ error: null });

    try {
      await api.addCLIToolMCPServer(toolType, config);

      // Invalidate cache and reload
      set((state) => ({
        lastFetchTimes: {
          ...state.lastFetchTimes,
          [toolType]: {
            ...(state.lastFetchTimes[toolType] ?? {}),
            mcpServers: 0,
          },
        },
      }));

      await get().loadMCPServers(toolType);
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to add MCP server',
      });
      throw error;
    }
  },

  removeMCPServer: async (toolType: CLIToolType, name: string) => {
    set({ error: null });

    try {
      await api.removeCLIToolMCPServer(toolType, name);

      // Optimistic update
      set((state) => {
        const toolConfig = state.configs[toolType];
        if (!toolConfig?.mcpServers) return state;

        return {
          configs: {
            ...state.configs,
            [toolType]: {
              ...toolConfig,
              mcpServers: toolConfig.mcpServers.filter((s) => s.name !== name),
            },
          },
        };
      });
    } catch (error) {
      // Reload on error to restore state
      await get().loadMCPServers(toolType);
      set({
        error: error instanceof Error ? error.message : 'Failed to remove MCP server',
      });
      throw error;
    }
  },

  // =========================================
  // Agents Actions
  // =========================================

  loadAgents: async (toolType: CLIToolType) => {
    const { loading, lastFetchTimes, cacheDuration, configs } = get();
    const now = Date.now();

    // Check cache
    const lastFetch = lastFetchTimes[toolType]?.agents;
    if (lastFetch && now - lastFetch < cacheDuration && configs[toolType]?.agents) {
      return;
    }

    // Prevent concurrent loads
    if (loading.agents[toolType]) {
      return;
    }

    set((state) => ({
      loading: {
        ...state.loading,
        agents: { ...state.loading.agents, [toolType]: true },
      },
      error: null,
    }));

    try {
      const agents = await api.listCLIToolAgents(toolType);

      set((state) => ({
        configs: {
          ...state.configs,
          [toolType]: {
            ...(state.configs[toolType] ?? createEmptyToolConfig()),
            agents,
          },
        },
        loading: {
          ...state.loading,
          agents: { ...state.loading.agents, [toolType]: false },
        },
        lastFetchTimes: {
          ...state.lastFetchTimes,
          [toolType]: {
            ...(state.lastFetchTimes[toolType] ?? {}),
            agents: now,
          },
        },
      }));
    } catch (error) {
      set((state) => ({
        loading: {
          ...state.loading,
          agents: { ...state.loading.agents, [toolType]: false },
        },
        error: error instanceof Error ? error.message : 'Failed to load agents',
      }));
    }
  },

  getAgent: async (toolType: CLIToolType, name: string) => {
    set({ error: null });

    try {
      return await api.getCLIToolAgent(toolType, name);
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to get agent',
      });
      throw error;
    }
  },

  // =========================================
  // Config Directory Actions
  // =========================================

  loadConfigDir: async (toolType: CLIToolType) => {
    const { configs } = get();

    // Check if already loaded
    if (configs[toolType]?.configDir) {
      return;
    }

    set({ error: null });

    try {
      const configDir = await api.getCLIToolConfigDir(toolType);

      set((state) => ({
        configs: {
          ...state.configs,
          [toolType]: {
            ...(state.configs[toolType] ?? createEmptyToolConfig()),
            configDir,
          },
        },
      }));
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to get config directory',
      });
    }
  },

  // =========================================
  // Usage Tracking Actions
  // =========================================

  loadUsageStats: async (toolType: CLIToolType) => {
    const { loading, lastFetchTimes, cacheDuration, configs } = get();
    const now = Date.now();

    // Check cache
    const lastFetch = lastFetchTimes[toolType]?.usage;
    if (lastFetch && now - lastFetch < cacheDuration && configs[toolType]?.usageStats) {
      return;
    }

    // Prevent concurrent loads
    if (loading.usage[toolType]) {
      return;
    }

    set((state) => ({
      loading: {
        ...state.loading,
        usage: { ...state.loading.usage, [toolType]: true },
      },
      error: null,
    }));

    try {
      const usageStats = await api.getCLIToolUsageStats(toolType);

      set((state) => ({
        configs: {
          ...state.configs,
          [toolType]: {
            ...(state.configs[toolType] ?? createEmptyToolConfig()),
            usageStats,
          },
        },
        loading: {
          ...state.loading,
          usage: { ...state.loading.usage, [toolType]: false },
        },
        lastFetchTimes: {
          ...state.lastFetchTimes,
          [toolType]: {
            ...(state.lastFetchTimes[toolType] ?? {}),
            usage: now,
          },
        },
      }));
    } catch (error) {
      set((state) => ({
        loading: {
          ...state.loading,
          usage: { ...state.loading.usage, [toolType]: false },
        },
        error: error instanceof Error ? error.message : 'Failed to load usage stats',
      }));
    }
  },

  trackUsage: async (toolType: CLIToolType, action: UsageAction, details?: string) => {
    try {
      await api.trackCLIToolUsage(toolType, action, details);

      // Invalidate usage cache to refresh on next load
      set((state) => ({
        lastFetchTimes: {
          ...state.lastFetchTimes,
          [toolType]: {
            ...(state.lastFetchTimes[toolType] ?? {}),
            usage: 0,
          },
        },
      }));
    } catch (error) {
      // Silently ignore tracking errors to not disrupt user flow
      console.error('Failed to track usage:', error);
    }
  },

  clearUsage: async (toolType: CLIToolType) => {
    set({ error: null });

    try {
      await api.clearCLIToolUsage(toolType);

      // Clear local state
      set((state) => ({
        configs: {
          ...state.configs,
          [toolType]: {
            ...(state.configs[toolType] ?? createEmptyToolConfig()),
            usageStats: null,
          },
        },
        lastFetchTimes: {
          ...state.lastFetchTimes,
          [toolType]: {
            ...(state.lastFetchTimes[toolType] ?? {}),
            usage: 0,
          },
        },
      }));
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to clear usage history',
      });
      throw error;
    }
  },

  // =========================================
  // Utility Actions
  // =========================================

  clearError: () => {
    set({ error: null });
  },

  clearToolCache: (toolType: CLIToolType) => {
    set((state) => ({
      configs: {
        ...state.configs,
        [toolType]: createEmptyToolConfig(),
      },
      lastFetchTimes: {
        ...state.lastFetchTimes,
        [toolType]: {},
      },
    }));
  },

  clearAllCache: () => {
    set({
      configs: {},
      lastFetchTimes: {},
    });
  },

  // =========================================
  // Selectors
  // =========================================

  getToolConfig: (toolType: CLIToolType) => {
    return get().configs[toolType];
  },

  isLoading: (toolType: CLIToolType, operation: keyof LoadingStates) => {
    const loadingState = get().loading[operation];
    if (operation === 'fileContent') {
      // For fileContent, check if any file for this tool is loading
      const fileContentState = loadingState as Record<string, boolean>;
      return Object.keys(fileContentState).some(
        (key) => key.startsWith(`${toolType}:`) && fileContentState[key]
      );
    }
    return (loadingState as Record<CLIToolType, boolean>)[toolType] ?? false;
  },
});

export const useCLIToolConfigStore = create<CLIToolConfigState>()(
  subscribeWithSelector(cliToolConfigStore)
);

// Selector hooks for common use cases
export const useToolConfigFiles = (toolType: CLIToolType) =>
  useCLIToolConfigStore((state) => state.configs[toolType]?.files ?? null);

export const useToolSettings = (toolType: CLIToolType) =>
  useCLIToolConfigStore((state) => state.configs[toolType]?.settings ?? null);

export const useToolMCPServers = (toolType: CLIToolType) =>
  useCLIToolConfigStore((state) => state.configs[toolType]?.mcpServers ?? null);

export const useToolAgents = (toolType: CLIToolType) =>
  useCLIToolConfigStore((state) => state.configs[toolType]?.agents ?? null);

export const useToolConfigError = () =>
  useCLIToolConfigStore((state) => state.error);

export const useToolConfigLoading = (toolType: CLIToolType) =>
  useCLIToolConfigStore((state) => ({
    files: state.loading.files[toolType] ?? false,
    settings: state.loading.settings[toolType] ?? false,
    mcpServers: state.loading.mcpServers[toolType] ?? false,
    agents: state.loading.agents[toolType] ?? false,
    usage: state.loading.usage[toolType] ?? false,
  }));

export const useToolUsageStats = (toolType: CLIToolType) =>
  useCLIToolConfigStore((state) => state.configs[toolType]?.usageStats ?? null);
