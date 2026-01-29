import React, { useCallback } from 'react';
import { Activity, XCircle, MessageSquare, Bot, Cog } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';
import {
  useRunningProcesses,
  useActiveSessionCount,
  useActiveAgentCount,
  useStatusStore,
} from '@/stores/statusStore';
import type { ProcessInfo } from '@/stores/statusStore';

interface ProcessItemProps {
  process: ProcessInfo;
  onKill: (id: string) => void;
}

const ProcessItem: React.FC<ProcessItemProps> = ({ process, onKill }) => {
  const getIcon = () => {
    switch (process.type) {
      case 'session':
        return <MessageSquare className="w-4 h-4" />;
      case 'agent':
        return <Bot className="w-4 h-4" />;
      case 'background':
        return <Cog className="w-4 h-4" />;
      default:
        return <Activity className="w-4 h-4" />;
    }
  };

  const getStatusColor = () => {
    switch (process.status) {
      case 'running':
        return 'bg-green-500';
      case 'stopped':
        return 'bg-gray-400';
      case 'error':
        return 'bg-red-500';
      default:
        return 'bg-gray-400';
    }
  };

  const formatDuration = (startedAt: Date) => {
    const now = new Date();
    const diff = now.getTime() - startedAt.getTime();
    const minutes = Math.floor(diff / 60000);
    const seconds = Math.floor((diff % 60000) / 1000);

    if (minutes > 0) {
      return `${minutes}m ${seconds}s`;
    }
    return `${seconds}s`;
  };

  return (
    <div className="flex items-center justify-between p-2 rounded-md bg-muted/30 hover:bg-muted/50 transition-colors group">
      <div className="flex items-center gap-2 min-w-0">
        <div className="relative shrink-0">
          {getIcon()}
          <span
            className={cn(
              "absolute -bottom-0.5 -right-0.5 w-2 h-2 rounded-full border border-background",
              getStatusColor()
            )}
          />
        </div>
        <div className="min-w-0">
          <span className="text-sm font-medium truncate block">{process.name}</span>
          <span className="text-xs text-muted-foreground">
            {formatDuration(process.startedAt)}
          </span>
        </div>
      </div>

      <Button
        variant="ghost"
        size="icon"
        className="h-6 w-6 opacity-0 group-hover:opacity-100 transition-opacity text-destructive hover:text-destructive hover:bg-destructive/10"
        onClick={() => onKill(process.id)}
      >
        <XCircle className="w-4 h-4" />
      </Button>
    </div>
  );
};

export const RunningProcessesPanel: React.FC = () => {
  const runningProcesses = useRunningProcesses();
  const sessionCount = useActiveSessionCount();
  const agentCount = useActiveAgentCount();

  const handleKillProcess = useCallback((processId: string) => {
    // In a real implementation, this would also send a kill signal
    // to the actual process via Tauri command
    // Use getState() for stable reference to avoid re-renders
    useStatusStore.getState().removeProcess(processId);
  }, []);

  const totalRunning = sessionCount + agentCount;

  return (
    <Card className="flex-1 min-h-0">
      <CardHeader className="pb-2 pt-4 px-4">
        <div className="flex items-center justify-between">
          <CardTitle className="text-sm font-medium flex items-center gap-2">
            <Activity className="w-4 h-4 text-primary" />
            Running Processes
          </CardTitle>
          <div className="flex items-center gap-1">
            {sessionCount > 0 && (
              <Badge variant="outline" className="text-xs px-1.5 py-0">
                <MessageSquare className="w-3 h-3 mr-1" />
                {sessionCount}
              </Badge>
            )}
            {agentCount > 0 && (
              <Badge variant="outline" className="text-xs px-1.5 py-0">
                <Bot className="w-3 h-3 mr-1" />
                {agentCount}
              </Badge>
            )}
          </div>
        </div>
      </CardHeader>

      <CardContent className="px-4 pb-4 overflow-y-auto">
        {runningProcesses.length > 0 ? (
          <div className="space-y-1">
            {runningProcesses.map((process) => (
              <ProcessItem
                key={process.id}
                process={process}
                onKill={handleKillProcess}
              />
            ))}
          </div>
        ) : (
          <div className="flex flex-col items-center justify-center py-8 text-center">
            <Activity className="w-8 h-8 text-muted-foreground/50 mb-2" />
            <p className="text-sm text-muted-foreground">No running processes</p>
            <p className="text-xs text-muted-foreground/70 mt-1">
              Start a chat or agent to see them here
            </p>
          </div>
        )}

        {totalRunning > 0 && (
          <div className="mt-3 pt-3 border-t border-border/50">
            <div className="flex items-center justify-between text-xs text-muted-foreground">
              <span>Total active</span>
              <span className="font-medium">{totalRunning}</span>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
};

export default RunningProcessesPanel;
