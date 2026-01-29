import React, { useEffect, useMemo } from 'react';
import {
  Activity,
  Bot,
  FileText,
  Loader2,
  RefreshCw,
  FolderOpen,
} from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { useCLIToolConfigStore } from '@/stores/cliToolConfigStore';
import {
  useSelectedCLITool,
  useSelectedCLIToolType,
} from '@/stores/statusStore';
import { resolveCLIToolDisplay, USAGE_ACTION_DISPLAY, CONFIG_SCOPE_DISPLAY } from '@/types/cli-tools';

export const CLIToolDetailsPanel: React.FC = () => {
  const selectedTool = useSelectedCLITool();
  const selectedToolType = useSelectedCLIToolType();
  const {
    configs,
    loading,
    loadConfigFiles,
    loadAgents,
    loadUsageStats,
  } = useCLIToolConfigStore();

  const toolConfig = selectedToolType ? configs[selectedToolType] : undefined;

  useEffect(() => {
    if (!selectedTool || !selectedToolType) {
      return;
    }

    if (selectedTool.capabilities.files) {
      loadConfigFiles(selectedToolType);
    }

    if (selectedTool.capabilities.agents) {
      loadAgents(selectedToolType);
    }

    if (selectedTool.capabilities.usage) {
      loadUsageStats(selectedToolType);
    }
  }, [
    selectedTool,
    selectedToolType,
    loadConfigFiles,
    loadAgents,
    loadUsageStats,
  ]);

  const usageStats = toolConfig?.usageStats;
  const usageLoading = selectedToolType ? loading.usage[selectedToolType] ?? false : false;
  const agents = toolConfig?.agents || [];
  const configFiles = toolConfig?.files || [];

  const usageEntries = useMemo(
    () =>
      usageStats
        ? Object.entries(usageStats.actions_by_type).map(([action, count]) => ({
            action,
            count,
            display: USAGE_ACTION_DISPLAY[action as keyof typeof USAGE_ACTION_DISPLAY],
          }))
        : [],
    [usageStats]
  );

  const display = selectedTool
    ? resolveCLIToolDisplay(selectedTool.tool_type, selectedTool.name)
    : null;

  return (
    <Card className="min-h-0">
      <CardHeader className="pb-2 pt-4 px-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <span className="text-2xl">{display?.icon ?? '🧭'}</span>
            <div>
              <CardTitle className="text-sm font-medium flex items-center gap-1">
                {display?.name ?? 'Select a CLI Tool'}
                {selectedTool && (
                <Badge variant="outline" className="text-[10px] px-1.5">
                    {selectedTool.is_installed ? 'Installed' : 'Missing'}
                  </Badge>
                )}
              </CardTitle>
              {selectedTool?.description && (
                <p className="text-xs text-muted-foreground">{selectedTool.description}</p>
              )}
            </div>
          </div>
        </div>
      </CardHeader>

      <CardContent className="px-4 pb-4">
        {!selectedTool ? (
          <div className="flex flex-col items-center justify-center gap-2 py-12 text-center text-sm text-muted-foreground">
            <Activity className="w-8 h-8 opacity-50" />
            <p>Click a CLI tool on the left to load its usage, agents, and configs.</p>
          </div>
        ) : (
          <div className="space-y-4">
            {selectedTool.capabilities.usage && (
              <section className="rounded-lg border border-border/50 bg-card/50 p-3">
                <div className="flex items-center justify-between mb-2">
                  <div className="flex items-center gap-2 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
                    <Activity className="w-4 h-4" />
                    Usage
                  </div>
                  <Button
                    variant="outline"
                    size="icon"
                    className="h-7 w-7"
                    onClick={() => selectedToolType && loadUsageStats(selectedToolType)}
                    disabled={usageLoading}
                    title="Refresh usage stats"
                  >
                    {usageLoading ? (
                      <Loader2 className="h-4 w-4 animate-spin" />
                    ) : (
                      <RefreshCw className="h-4 w-4" />
                    )}
                  </Button>
                </div>

                {usageLoading ? (
                  <div className="flex items-center justify-center py-8">
                    <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
                  </div>
                ) : usageStats ? (
                  <div className="space-y-2">
                    <p className="text-sm font-semibold">
                      {usageStats.total_actions} total action
                      {usageStats.total_actions !== 1 ? 's' : ''}
                    </p>
                    <div className="grid gap-2 sm:grid-cols-2">
                      {usageEntries.map((entry) => (
                        <div
                          key={entry.action}
                          className="flex items-center justify-between rounded-md border border-border/40 px-3 py-2 bg-background"
                        >
                          <span className="text-xs text-muted-foreground">
                            {entry.display?.name ?? entry.action}
                          </span>
                          <Badge variant="secondary" className="text-[10px]">
                            {entry.count}
                          </Badge>
                        </div>
                      ))}
                    </div>
                  </div>
                ) : (
                  <p className="text-xs text-muted-foreground">
                    No usage data recorded yet for this CLI tool.
                  </p>
                )}
              </section>
            )}

            {selectedTool.capabilities.agents && (
              <section className="rounded-lg border border-border/50 bg-card/50 p-3">
                <div className="flex items-center justify-between mb-2">
                  <div className="flex items-center gap-2 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
                    <Bot className="w-4 h-4" />
                    Linked Agents
                  </div>
                  <Badge variant="secondary" className="text-[10px]">
                    {agents.length}
                  </Badge>
                </div>

                {agents.length === 0 ? (
                  <p className="text-xs text-muted-foreground">
                    No agents are registered for this tool yet.
                  </p>
                ) : (
                  <div className="space-y-2">
                    {agents.slice(0, 3).map((agent) => (
                      <div
                        key={agent.name}
                        className="flex flex-col rounded-md border border-border/40 px-3 py-2 bg-background/60"
                      >
                        <span className="text-sm font-medium">{agent.name}</span>
                        {agent.description && (
                          <span className="text-xs text-muted-foreground">
                            {agent.description}
                          </span>
                        )}
                      </div>
                    ))}
                    {agents.length > 3 && (
                      <p className="text-xs text-muted-foreground">
                        Showing {Math.min(3, agents.length)} of {agents.length} agents.
                      </p>
                    )}
                  </div>
                )}
              </section>
            )}

            {selectedTool.capabilities.files && (
              <section className="rounded-lg border border-border/50 bg-card/50 p-3">
                <div className="flex items-center justify-between mb-2">
                  <div className="flex items-center gap-2 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
                    <FileText className="w-4 h-4" />
                    Config Files
                  </div>
                  {toolConfig?.configDir && (
                    <div className="flex items-center gap-1 text-xs text-muted-foreground">
                      <FolderOpen className="h-3 w-3" />
                      <span className="truncate">{toolConfig.configDir}</span>
                    </div>
                  )}
                </div>

                {configFiles.length === 0 ? (
                  <p className="text-xs text-muted-foreground">
                    Config files are still being discovered for this tool.
                  </p>
                ) : (
                  <div className="space-y-2">
                    {configFiles.slice(0, 4).map((file) => {
                      const scope = CONFIG_SCOPE_DISPLAY[file.scope];
                      return (
                        <div
                          key={file.path}
                          className="flex items-center justify-between rounded-md border border-border/40 px-3 py-2 bg-background/60"
                        >
                          <div className="min-w-0">
                            <p className="text-sm font-medium truncate">{file.name}</p>
                            <p className="text-[10px] text-muted-foreground truncate">
                              {file.path}
                            </p>
                          </div>
                          <Badge variant="outline" className="text-[10px] px-1.5">
                            {scope?.icon ?? '📁'} {scope?.name ?? file.scope}
                          </Badge>
                        </div>
                      );
                    })}
                    {configFiles.length > 4 && (
                      <p className="text-xs text-muted-foreground">
                        Showing {Math.min(4, configFiles.length)} of {configFiles.length} files.
                      </p>
                    )}
                  </div>
                )}
              </section>
            )}
          </div>
        )}
      </CardContent>
    </Card>
  );
};

export default CLIToolDetailsPanel;
