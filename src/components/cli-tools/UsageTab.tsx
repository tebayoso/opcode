/**
 * Usage Tab
 * Usage statistics and history for CLI tool configuration operations
 */

import { useEffect, useState } from 'react';
import {
  Activity,
  Trash2,
  RefreshCw,
  FileText,
  Settings,
  Server,
  Terminal,
  Eye,
  Loader2,
  Clock,
  Hash,
} from 'lucide-react';
import { motion } from 'framer-motion';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from '@/components/ui/alert-dialog';
import { useCLIToolConfigStore, useToolUsageStats } from '@/stores/cliToolConfigStore';
import type { CLIToolType, UsageAction } from '@/types/cli-tools';
import { USAGE_ACTION_DISPLAY } from '@/types/cli-tools';

interface UsageTabProps {
  toolType: CLIToolType;
}

const getActionIcon = (action: string) => {
  switch (action) {
    case 'file_read':
      return <FileText className="h-3 w-3" />;
    case 'file_write':
      return <FileText className="h-3 w-3" />;
    case 'setting_change':
      return <Settings className="h-3 w-3" />;
    case 'mcp_add':
    case 'mcp_remove':
      return <Server className="h-3 w-3" />;
    case 'command_execute':
      return <Terminal className="h-3 w-3" />;
    case 'agent_view':
      return <Eye className="h-3 w-3" />;
    case 'config_refresh':
      return <RefreshCw className="h-3 w-3" />;
    default:
      return <Activity className="h-3 w-3" />;
  }
};

const formatTimestamp = (timestamp: string): string => {
  const date = new Date(timestamp);
  const now = new Date();
  const diff = now.getTime() - date.getTime();

  // Less than 1 minute
  if (diff < 60000) {
    return 'Just now';
  }
  // Less than 1 hour
  if (diff < 3600000) {
    const mins = Math.floor(diff / 60000);
    return `${mins}m ago`;
  }
  // Less than 24 hours
  if (diff < 86400000) {
    const hours = Math.floor(diff / 3600000);
    return `${hours}h ago`;
  }
  // Less than 7 days
  if (diff < 604800000) {
    const days = Math.floor(diff / 86400000);
    return `${days}d ago`;
  }
  // Otherwise show date
  return date.toLocaleDateString();
};

export function UsageTab({ toolType }: UsageTabProps) {
  const usageStats = useToolUsageStats(toolType);
  const { loadUsageStats, clearUsage } = useCLIToolConfigStore();
  const loading = useCLIToolConfigStore((state) => state.loading.usage[toolType] ?? false);
  const [isClearing, setIsClearing] = useState(false);

  useEffect(() => {
    loadUsageStats(toolType);
  }, [toolType, loadUsageStats]);

  const handleRefresh = () => {
    // Force refresh by clearing cache
    useCLIToolConfigStore.setState((state) => ({
      lastFetchTimes: {
        ...state.lastFetchTimes,
        [toolType]: {
          ...(state.lastFetchTimes[toolType] ?? {}),
          usage: 0,
        },
      },
    }));
    loadUsageStats(toolType);
  };

  const handleClear = async () => {
    setIsClearing(true);
    try {
      await clearUsage(toolType);
    } finally {
      setIsClearing(false);
    }
  };

  if (loading && !usageStats) {
    return (
      <div className="flex items-center justify-center py-8">
        <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (!usageStats || usageStats.total_actions === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-8 text-muted-foreground">
        <Activity className="h-8 w-8 mb-2 opacity-50" />
        <p className="text-sm">No usage history yet</p>
        <p className="text-xs mt-1">
          Usage will be tracked as you interact with this tool's configuration
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {/* Header with actions */}
      <div className="flex justify-between items-center">
        <div className="flex items-center gap-2">
          <Hash className="h-4 w-4 text-muted-foreground" />
          <span className="text-sm font-medium">
            {usageStats.total_actions} total action{usageStats.total_actions !== 1 ? 's' : ''}
          </span>
        </div>
        <div className="flex gap-2">
          <Button variant="ghost" size="sm" onClick={handleRefresh} disabled={loading}>
            <RefreshCw className={`h-3 w-3 ${loading ? 'animate-spin' : ''}`} />
          </Button>
          <AlertDialog>
            <AlertDialogTrigger asChild>
              <Button
                variant="ghost"
                size="sm"
                className="text-destructive hover:text-destructive"
                disabled={isClearing}
              >
                {isClearing ? (
                  <Loader2 className="h-3 w-3 animate-spin" />
                ) : (
                  <Trash2 className="h-3 w-3" />
                )}
              </Button>
            </AlertDialogTrigger>
            <AlertDialogContent>
              <AlertDialogHeader>
                <AlertDialogTitle>Clear Usage History</AlertDialogTitle>
                <AlertDialogDescription>
                  Are you sure you want to clear all usage history for this tool?
                  This action cannot be undone.
                </AlertDialogDescription>
              </AlertDialogHeader>
              <AlertDialogFooter>
                <AlertDialogCancel>Cancel</AlertDialogCancel>
                <AlertDialogAction onClick={handleClear}>Clear History</AlertDialogAction>
              </AlertDialogFooter>
            </AlertDialogContent>
          </AlertDialog>
        </div>
      </div>

      {/* Statistics summary */}
      <div className="grid grid-cols-2 gap-2">
        {Object.entries(usageStats.actions_by_type).map(([action, count]) => {
          const display = USAGE_ACTION_DISPLAY[action as UsageAction];
          return (
            <motion.div
              key={action}
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              className="flex items-center gap-2 p-2 rounded-md bg-muted/50"
            >
              <span className="text-lg">{display?.icon ?? '📊'}</span>
              <div className="flex-1 min-w-0">
                <p className="text-xs font-medium truncate">{display?.name ?? action}</p>
                <p className="text-xs text-muted-foreground">{count} time{count !== 1 ? 's' : ''}</p>
              </div>
            </motion.div>
          );
        })}
      </div>

      {/* Time range */}
      {usageStats.first_action && usageStats.last_action && (
        <div className="flex items-center gap-2 text-xs text-muted-foreground">
          <Clock className="h-3 w-3" />
          <span>
            {formatTimestamp(usageStats.first_action)} - {formatTimestamp(usageStats.last_action)}
          </span>
        </div>
      )}

      {/* Recent actions */}
      {usageStats.recent_actions.length > 0 && (
        <div className="space-y-2">
          <p className="text-xs font-medium text-muted-foreground">Recent Activity</p>
          <div className="space-y-1.5 max-h-[200px] overflow-y-auto">
            {usageStats.recent_actions.map((entry) => {
              const display = USAGE_ACTION_DISPLAY[entry.action as UsageAction];
              return (
                <motion.div
                  key={entry.id}
                  initial={{ opacity: 0, x: -10 }}
                  animate={{ opacity: 1, x: 0 }}
                  className="flex items-center gap-2 p-2 rounded-md border bg-background"
                >
                  <div className="flex items-center justify-center h-6 w-6 rounded bg-muted">
                    {getActionIcon(entry.action)}
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-1.5">
                      <p className="text-xs font-medium">{display?.name ?? entry.action}</p>
                      <Badge variant="outline" className="text-[9px] px-1 py-0">
                        {formatTimestamp(entry.timestamp)}
                      </Badge>
                    </div>
                    {entry.details && (
                      <p className="text-[10px] text-muted-foreground truncate">
                        {entry.details}
                      </p>
                    )}
                  </div>
                </motion.div>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
}
