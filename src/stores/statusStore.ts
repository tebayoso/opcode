import { create } from 'zustand';
import { subscribeWithSelector } from 'zustand/middleware';
import { useShallow } from 'zustand/react/shallow';
import type { StateCreator } from 'zustand';
import { api } from '@/lib/api';
import type { CLIToolsStatus, CLIToolWithStatus, CLIToolType } from '@/types/cli-tools';

// Process info type for running processes
export interface ProcessInfo {
  id: string;
  name: string;
  type: 'session' | 'agent' | 'background';
  status: 'running' | 'stopped' | 'error';
  startedAt: Date;
  projectPath?: string;
}

// System status overview
export interface SystemStatus {
  cliTools: CLIToolsStatus | null;
  runningProcesses: ProcessInfo[];
  activeSessionCount: number;
  activeAgentCount: number;
}

interface StatusState {
  // Data
  systemStatus: SystemStatus;

  // UI State
  isLoading: boolean;
  isRefreshing: boolean;
  error: string | null;
  lastFetchTime: number;

  // Auto-refresh
  autoRefreshEnabled: boolean;
  autoRefreshInterval: number; // milliseconds
  refreshTimerId: NodeJS.Timeout | null;

  // Cache duration in milliseconds (5 seconds for status)
  cacheDuration: number;

  // CLI selection state
  selectedCLIToolType: CLIToolType | null;

  // Actions
  fetchStatus: (forceRefresh?: boolean) => Promise<void>;
  refreshStatus: () => Promise<void>;
  startAutoRefresh: () => void;
  stopAutoRefresh: () => void;
  setAutoRefreshInterval: (interval: number) => void;
  addProcess: (process: ProcessInfo) => void;
  removeProcess: (processId: string) => void;
  updateProcessStatus: (processId: string, status: ProcessInfo['status']) => void;
  getInstalledTools: () => CLIToolWithStatus[];
  getRunningProcesses: () => ProcessInfo[];
  clearError: () => void;
  selectCLITool: (toolType: CLIToolType | null) => void;
}

const statusStore: StateCreator<
  StatusState,
  [],
  [['zustand/subscribeWithSelector', never]],
  StatusState
> = (set, get) => ({
  // Initial state
  systemStatus: {
    cliTools: null,
    runningProcesses: [],
    activeSessionCount: 0,
    activeAgentCount: 0,
  },
  isLoading: false,
  isRefreshing: false,
  error: null,
  lastFetchTime: 0,
  autoRefreshEnabled: false,
  autoRefreshInterval: 5000, // 5 seconds default
  refreshTimerId: null,
  cacheDuration: 5000, // 5 seconds
  selectedCLIToolType: null,

  // Fetch system status with caching
  fetchStatus: async (forceRefresh = false) => {
    const now = Date.now();
    const { lastFetchTime, cacheDuration, systemStatus, isLoading } = get();

    // Prevent concurrent fetches
    if (isLoading) {
      return;
    }

    // Use cache unless forced or expired
    if (!forceRefresh && systemStatus.cliTools && now - lastFetchTime < cacheDuration) {
      return;
    }

    set({ isLoading: true, error: null });

    try {
      // Fetch CLI tools status
      const cliTools = await api.listCLITools();

      set((state) => ({
        systemStatus: {
          ...state.systemStatus,
          cliTools,
        },
        isLoading: false,
        lastFetchTime: now,
      }));
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to fetch system status',
        isLoading: false,
      });
    }
  },

  // Force refresh (bypasses cache)
  refreshStatus: async () => {
    const { isRefreshing } = get();

    if (isRefreshing) {
      return;
    }

    set({ isRefreshing: true, error: null });

    try {
      const cliTools = await api.refreshCLITools();

      set((state) => ({
        systemStatus: {
          ...state.systemStatus,
          cliTools,
        },
        isRefreshing: false,
        lastFetchTime: Date.now(),
      }));
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to refresh system status',
        isRefreshing: false,
      });
    }
  },

  // Start auto-refresh
  startAutoRefresh: () => {
    const { autoRefreshEnabled, refreshTimerId, autoRefreshInterval, fetchStatus } = get();

    if (autoRefreshEnabled && refreshTimerId) {
      return; // Already running
    }

    const timerId = setInterval(() => {
      fetchStatus(true);
    }, autoRefreshInterval);

    set({
      autoRefreshEnabled: true,
      refreshTimerId: timerId,
    });
  },

  // Stop auto-refresh
  stopAutoRefresh: () => {
    const { refreshTimerId } = get();

    if (refreshTimerId) {
      clearInterval(refreshTimerId);
    }

    set({
      autoRefreshEnabled: false,
      refreshTimerId: null,
    });
  },

  // Set auto-refresh interval
  setAutoRefreshInterval: (interval: number) => {
    const { autoRefreshEnabled, stopAutoRefresh, startAutoRefresh } = get();

    set({ autoRefreshInterval: interval });

    // Restart if already running
    if (autoRefreshEnabled) {
      stopAutoRefresh();
      startAutoRefresh();
    }
  },

  // Add a running process
  addProcess: (process: ProcessInfo) => {
    set((state) => {
      const existingIndex = state.systemStatus.runningProcesses.findIndex(
        (p) => p.id === process.id
      );

      let runningProcesses: ProcessInfo[];
      if (existingIndex >= 0) {
        // Update existing
        runningProcesses = [...state.systemStatus.runningProcesses];
        runningProcesses[existingIndex] = process;
      } else {
        // Add new
        runningProcesses = [...state.systemStatus.runningProcesses, process];
      }

      // Update counts
      const activeSessionCount = runningProcesses.filter(
        (p) => p.type === 'session' && p.status === 'running'
      ).length;
      const activeAgentCount = runningProcesses.filter(
        (p) => p.type === 'agent' && p.status === 'running'
      ).length;

      return {
        systemStatus: {
          ...state.systemStatus,
          runningProcesses,
          activeSessionCount,
          activeAgentCount,
        },
      };
    });
  },

  // Remove a process
  removeProcess: (processId: string) => {
    set((state) => {
      const runningProcesses = state.systemStatus.runningProcesses.filter(
        (p) => p.id !== processId
      );

      const activeSessionCount = runningProcesses.filter(
        (p) => p.type === 'session' && p.status === 'running'
      ).length;
      const activeAgentCount = runningProcesses.filter(
        (p) => p.type === 'agent' && p.status === 'running'
      ).length;

      return {
        systemStatus: {
          ...state.systemStatus,
          runningProcesses,
          activeSessionCount,
          activeAgentCount,
        },
      };
    });
  },

  // Update process status
  updateProcessStatus: (processId: string, status: ProcessInfo['status']) => {
    set((state) => {
      const runningProcesses = state.systemStatus.runningProcesses.map((p) =>
        p.id === processId ? { ...p, status } : p
      );

      const activeSessionCount = runningProcesses.filter(
        (p) => p.type === 'session' && p.status === 'running'
      ).length;
      const activeAgentCount = runningProcesses.filter(
        (p) => p.type === 'agent' && p.status === 'running'
      ).length;

      return {
        systemStatus: {
          ...state.systemStatus,
          runningProcesses,
          activeSessionCount,
          activeAgentCount,
        },
      };
    });
  },

  // Get installed tools
  getInstalledTools: () => {
    const { systemStatus } = get();
    return systemStatus.cliTools?.tools.filter((tool) => tool.is_installed) || [];
  },

  // Get running processes
  getRunningProcesses: () => {
    const { systemStatus } = get();
    return systemStatus.runningProcesses.filter((p) => p.status === 'running');
  },

  // Clear error
  clearError: () => {
    set({ error: null });
  },
  selectCLITool: (toolType: CLIToolType | null) => {
    set({ selectedCLIToolType: toolType });
  },
});

export const useStatusStore = create<StatusState>()(
  subscribeWithSelector(statusStore)
);

// Selector hooks for common use cases
// Using useShallow for selectors that return arrays/objects to prevent infinite re-renders
export const useSystemStatus = () => useStatusStore(
  useShallow((state) => state.systemStatus)
);
export const useStatusLoading = () => useStatusStore((state) => state.isLoading || state.isRefreshing);
export const useStatusError = () => useStatusStore((state) => state.error);
export const useInstalledTools = () => useStatusStore(
  useShallow((state) => state.systemStatus.cliTools?.tools.filter((tool) => tool.is_installed) || [])
);
export const useRunningProcesses = () => useStatusStore(
  useShallow((state) => state.systemStatus.runningProcesses.filter((p) => p.status === 'running'))
);
export const useActiveSessionCount = () => useStatusStore((state) => state.systemStatus.activeSessionCount);
export const useActiveAgentCount = () => useStatusStore((state) => state.systemStatus.activeAgentCount);
export const useSelectedCLIToolType = () => useStatusStore((state) => state.selectedCLIToolType);
export const useSelectedCLITool = () =>
  useStatusStore((state) => {
    const toolType = state.selectedCLIToolType;
    const tools = state.systemStatus.cliTools?.tools;
    if (!toolType || !tools) {
      return null;
    }
    return tools.find((tool) => tool.tool_type === toolType) || null;
  });
export const useSelectCLITool = () => useStatusStore((state) => state.selectCLITool);
