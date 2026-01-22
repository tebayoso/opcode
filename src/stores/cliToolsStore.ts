import { create } from 'zustand';
import { subscribeWithSelector } from 'zustand/middleware';
import type { StateCreator } from 'zustand';
import { api } from '@/lib/api';
import type {
  CLIToolsStatus,
  CLIToolWithStatus,
  CLIToolType,
} from '@/types/cli-tools';

interface CLIToolsState {
  // Data
  status: CLIToolsStatus | null;

  // UI state
  isLoading: boolean;
  isRefreshing: boolean;
  error: string | null;
  lastFetchTime: number;

  // Cache duration in milliseconds (30 seconds)
  cacheDuration: number;

  // Actions
  fetchTools: (forceRefresh?: boolean) => Promise<void>;
  refreshTools: () => Promise<void>;
  setPreferredInstallation: (toolType: CLIToolType, path: string) => Promise<void>;
  getToolByType: (toolType: CLIToolType) => CLIToolWithStatus | undefined;
  clearError: () => void;
}

const cliToolsStore: StateCreator<
  CLIToolsState,
  [],
  [['zustand/subscribeWithSelector', never]],
  CLIToolsState
> = (set, get) => ({
  // Initial state
  status: null,
  isLoading: false,
  isRefreshing: false,
  error: null,
  lastFetchTime: 0,
  cacheDuration: 30000, // 30 seconds

  // Fetch CLI tools with caching
  fetchTools: async (forceRefresh = false) => {
    const now = Date.now();
    const { lastFetchTime, cacheDuration, status, isLoading } = get();

    // Prevent concurrent fetches
    if (isLoading) {
      return;
    }

    // Use cache unless forced or expired
    if (!forceRefresh && status && now - lastFetchTime < cacheDuration) {
      return;
    }

    set({ isLoading: true, error: null });

    try {
      const newStatus = await api.listCLITools();
      set({
        status: newStatus,
        isLoading: false,
        lastFetchTime: now,
      });
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to fetch CLI tools',
        isLoading: false,
      });
    }
  },

  // Force refresh CLI tools (bypasses cache and triggers full rescan)
  refreshTools: async () => {
    const { isRefreshing } = get();

    // Prevent concurrent refreshes
    if (isRefreshing) {
      return;
    }

    set({ isRefreshing: true, error: null });

    try {
      const newStatus = await api.refreshCLITools();
      set({
        status: newStatus,
        isRefreshing: false,
        lastFetchTime: Date.now(),
      });
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to refresh CLI tools',
        isRefreshing: false,
      });
    }
  },

  // Set preferred installation for a tool
  setPreferredInstallation: async (toolType: CLIToolType, path: string) => {
    const { status } = get();

    try {
      await api.setCLIToolPreferred(toolType, path);

      // Update local state optimistically
      if (status) {
        const updatedTools = status.tools.map((tool) => {
          if (tool.tool_type === toolType) {
            const preferredInstallation = tool.installations.find(
              (inst) => inst.path === path
            );
            return {
              ...tool,
              preferred_installation: preferredInstallation || null,
            };
          }
          return tool;
        });

        set({
          status: {
            ...status,
            tools: updatedTools,
          },
        });
      }
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to set preferred installation',
      });
      throw error;
    }
  },

  // Get a specific tool by type
  getToolByType: (toolType: CLIToolType) => {
    const { status } = get();
    return status?.tools.find((tool) => tool.tool_type === toolType);
  },

  // Clear error
  clearError: () => {
    set({ error: null });
  },
});

export const useCLIToolsStore = create<CLIToolsState>()(
  subscribeWithSelector(cliToolsStore)
);

// Selector hooks for common use cases
export const useCLIToolsStatus = () => useCLIToolsStore((state) => state.status);
export const useCLIToolsLoading = () => useCLIToolsStore((state) => state.isLoading || state.isRefreshing);
export const useCLIToolsError = () => useCLIToolsStore((state) => state.error);
export const useInstalledTools = () => useCLIToolsStore((state) =>
  state.status?.tools.filter((tool) => tool.is_installed) || []
);
