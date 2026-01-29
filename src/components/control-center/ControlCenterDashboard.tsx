import React, { useEffect, useCallback } from 'react';
import { motion } from 'framer-motion';
import {
  LayoutDashboard,
  RefreshCw,
  Loader2,
  AlertCircle,
  Maximize2,
  Minimize2,
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';
import { useStatusStore, useStatusLoading, useStatusError } from '@/stores/statusStore';
import { CompactCLIPanel } from './panels/CompactCLIPanel';
import { RunningProcessesPanel } from './panels/RunningProcessesPanel';
import { AgentsPanel } from './panels/AgentsPanel';
import { SkillsPanel } from './panels/SkillsPanel';
import { ConfigFilesPanel } from './panels/ConfigFilesPanel';
import { VersioningPanel } from './panels/VersioningPanel';
import { CLIToolDetailsPanel } from './panels/CLIToolDetailsPanel';
import { MergePreviewDialog } from './components/MergePreviewDialog';
import { useKeyboardShortcuts } from './hooks/useKeyboardShortcuts';

interface ControlCenterDashboardProps {
  projectPath?: string;
}

export const ControlCenterDashboard: React.FC<ControlCenterDashboardProps> = ({
  projectPath,
}) => {
  const isLoading = useStatusLoading();
  const error = useStatusError();

  // Get clearError directly from the store for event handlers
  const clearError = useStatusStore((state) => state.clearError);

  const [isFullscreen, setIsFullscreen] = React.useState(false);

  // Initialize keyboard shortcuts
  useKeyboardShortcuts({
    enabled: true,
  });

  // Initial fetch and auto-refresh setup
  // Use getState() to access stable store actions that don't trigger re-renders
  useEffect(() => {
    const { fetchStatus, startAutoRefresh, stopAutoRefresh } = useStatusStore.getState();

    fetchStatus();
    startAutoRefresh();

    return () => {
      stopAutoRefresh();
    };
  }, []); // Empty deps - store actions are stable and should only run on mount/unmount

  const handleRefresh = useCallback(async () => {
    await useStatusStore.getState().refreshStatus();
  }, []);

  const toggleFullscreen = useCallback(async () => {
    try {
      // Use Tauri window API if available
      if (window.__TAURI__) {
        const { getCurrentWindow } = await import('@tauri-apps/api/window');
        const window = getCurrentWindow();
        const isCurrentlyFullscreen = await window.isFullscreen();
        await window.setFullscreen(!isCurrentlyFullscreen);
        setIsFullscreen(!isCurrentlyFullscreen);
      } else {
        // Fallback for web mode
        if (!document.fullscreenElement) {
          await document.documentElement.requestFullscreen();
          setIsFullscreen(true);
        } else {
          await document.exitFullscreen();
          setIsFullscreen(false);
        }
      }
    } catch (err) {
      console.error('Failed to toggle fullscreen:', err);
    }
  }, []);

  return (
    <div className="h-full flex flex-col bg-background">
      {/* Header */}
      <div className="shrink-0 border-b border-border/50 bg-background/95 backdrop-blur-sm">
        <div className="flex items-center justify-between px-6 py-4">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-lg bg-primary/10">
              <LayoutDashboard className="w-6 h-6 text-primary" />
            </div>
            <div>
              <h1 className="text-xl font-bold tracking-tight">Control Center</h1>
              <p className="text-sm text-muted-foreground">
                Unified view of agents, tools, and configurations
              </p>
            </div>
          </div>

          <div className="flex items-center gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={handleRefresh}
              disabled={isLoading}
              className="gap-2"
            >
              <RefreshCw className={cn("w-4 h-4", isLoading && "animate-spin")} />
              {isLoading ? 'Refreshing...' : 'Refresh'}
            </Button>

            <Button
              variant="ghost"
              size="sm"
              onClick={toggleFullscreen}
              className="gap-2"
            >
              {isFullscreen ? (
                <Minimize2 className="w-4 h-4" />
              ) : (
                <Maximize2 className="w-4 h-4" />
              )}
            </Button>
          </div>
        </div>

        {/* Error Banner */}
        {error && (
          <motion.div
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: 'auto' }}
            exit={{ opacity: 0, height: 0 }}
            className="px-6 pb-4"
          >
            <div className="flex items-center gap-3 p-3 rounded-lg border border-destructive/50 bg-destructive/10">
              <AlertCircle className="w-5 h-5 text-destructive shrink-0" />
              <p className="text-sm text-destructive flex-1">{error}</p>
              <Button variant="ghost" size="sm" onClick={clearError}>
                Dismiss
              </Button>
            </div>
          </motion.div>
        )}
      </div>

      {/* Main Content - 3 Column Layout */}
      <div className="flex-1 overflow-hidden p-4">
        {isLoading && !useStatusStore.getState().systemStatus.cliTools ? (
          <div className="h-full flex items-center justify-center">
            <div className="flex flex-col items-center gap-3">
              <Loader2 className="w-8 h-8 animate-spin text-primary" />
              <p className="text-sm text-muted-foreground">Loading system status...</p>
            </div>
          </div>
        ) : (
          <div className="h-full grid grid-cols-[300px_1fr_350px] gap-4">
            {/* Left Column - CLI Tools & Running Processes */}
            <div className="flex flex-col gap-4 overflow-y-auto pr-1">
              <CompactCLIPanel />
              <RunningProcessesPanel />
            </div>

            {/* Center Column - Agents & Skills */}
            <div className="flex flex-col gap-4 overflow-y-auto px-1">
              <CLIToolDetailsPanel />
              <AgentsPanel projectPath={projectPath} />
              <SkillsPanel projectPath={projectPath} />
            </div>

            {/* Right Column - Config Files & Versioning */}
            <div className="flex flex-col gap-4 overflow-y-auto pl-1">
              <ConfigFilesPanel projectPath={projectPath} />
              <VersioningPanel projectPath={projectPath} />
            </div>
          </div>
        )}
      </div>

      {/* Merge Preview Dialog */}
      <MergePreviewDialog onMergeComplete={handleRefresh} />
    </div>
  );
};

export default ControlCenterDashboard;
