import React, { createContext, useContext, useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface ToolSummary {
  id: string;
  name: string;
  tool_type: string;
  source: string;
  is_installed: boolean;
  capabilities: {
    files: boolean;
    settings: boolean;
    mcp_servers: boolean;
    agents: boolean;
    commands: boolean;
    usage: boolean;
  };
}

interface DashboardStats {
  total_tools: number;
  installed_tools: number;
  pending_validations: number;
  active_warnings: number;
  running_jobs: number;
}

interface UnifiedDashboardContextType {
  tools: ToolSummary[];
  stats: DashboardStats;
  isLoading: boolean;
  refreshTools: () => Promise<void>;
  selectedToolId: string | null;
  setSelectedToolId: (id: string | null) => void;
  activeSection: string;
  setActiveSection: (section: string) => void;
}

const UnifiedDashboardContext = createContext<UnifiedDashboardContextType | undefined>(undefined);

export function UnifiedDashboardProvider({ children }: { children: React.ReactNode }) {
  const [tools, setTools] = useState<ToolSummary[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [selectedToolId, setSelectedToolId] = useState<string | null>(null);
  const [activeSection, setActiveSection] = useState('overview');
  const [stats, setStats] = useState<DashboardStats>({
    total_tools: 0,
    installed_tools: 0,
    pending_validations: 0,
    active_warnings: 0,
    running_jobs: 0,
  });

  const refreshTools = async () => {
    try {
      setIsLoading(true);
      const response = await invoke<{ data: ToolSummary[] }>('tool_registry_list_tools');
      setTools(response.data);
      
      const installedCount = response.data.filter(t => t.is_installed).length;
      setStats(prev => ({
        ...prev,
        total_tools: response.data.length,
        installed_tools: installedCount,
      }));
    } catch (error) {
      console.error('Failed to load tools:', error);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    refreshTools();
  }, []);

  return (
    <UnifiedDashboardContext.Provider
      value={{
        tools,
        stats,
        isLoading,
        refreshTools,
        selectedToolId,
        setSelectedToolId,
        activeSection,
        setActiveSection,
      }}
    >
      {children}
    </UnifiedDashboardContext.Provider>
  );
}

export function useUnifiedDashboard() {
  const context = useContext(UnifiedDashboardContext);
  if (context === undefined) {
    throw new Error('useUnifiedDashboard must be used within a UnifiedDashboardProvider');
  }
  return context;
}
