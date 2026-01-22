/**
 * CLI Tool Configuration Panel
 * Main config panel with tabs for each configuration section
 */

import { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import {
  FileText,
  Settings,
  Server,
  Bot,
  FolderOpen,
  Loader2,
  ChevronRight,
  Activity,
} from 'lucide-react';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { useCLIToolConfigStore } from '@/stores/cliToolConfigStore';
import type { CLIToolType, CLIToolWithStatus } from '@/types/cli-tools';
import { ConfigFilesTab } from './ConfigFilesTab';
import { SettingsTab } from './SettingsTab';
import { MCPServersTab } from './MCPServersTab';
import { AgentsTab } from './AgentsTab';
import { UsageTab } from './UsageTab';

interface CLIToolConfigPanelProps {
  tool: CLIToolWithStatus;
  isExpanded: boolean;
  onToggle: () => void;
}

type TabId = 'overview' | 'files' | 'settings' | 'mcp' | 'agents' | 'usage';

interface TabConfig {
  id: TabId;
  label: string;
  icon: React.ReactNode;
  badge?: number | null;
}

export function CLIToolConfigPanel({
  tool,
  isExpanded,
  onToggle,
}: CLIToolConfigPanelProps) {
  const [activeTab, setActiveTab] = useState<TabId>('overview');
  const toolType = tool.tool_type as CLIToolType;

  const {
    configs,
    loading,
    loadConfigFiles,
    loadSettings,
    loadMCPServers,
    loadAgents,
    loadConfigDir,
  } = useCLIToolConfigStore();

  const toolConfig = configs[toolType];
  const isLoadingFiles = loading.files[toolType] || false;
  const isLoadingSettings = loading.settings[toolType] || false;
  const isLoadingMCP = loading.mcpServers[toolType] || false;
  const isLoadingAgents = loading.agents[toolType] || false;

  // Load config dir when panel expands
  useEffect(() => {
    if (isExpanded && tool.is_installed && !toolConfig?.configDir) {
      loadConfigDir(toolType);
    }
  }, [isExpanded, tool.is_installed, toolType, toolConfig?.configDir, loadConfigDir]);

  // Lazy load data based on active tab
  useEffect(() => {
    if (!isExpanded || !tool.is_installed) return;

    switch (activeTab) {
      case 'files':
        if (!toolConfig?.files) {
          loadConfigFiles(toolType);
        }
        break;
      case 'settings':
        if (!toolConfig?.settings) {
          loadSettings(toolType);
        }
        break;
      case 'mcp':
        if (!toolConfig?.mcpServers) {
          loadMCPServers(toolType);
        }
        break;
      case 'agents':
        if (!toolConfig?.agents) {
          loadAgents(toolType);
        }
        break;
    }
  }, [
    activeTab,
    isExpanded,
    tool.is_installed,
    toolType,
    toolConfig,
    loadConfigFiles,
    loadSettings,
    loadMCPServers,
    loadAgents,
  ]);

  const tabs: TabConfig[] = [
    { id: 'overview', label: 'Overview', icon: <FolderOpen className="h-4 w-4" /> },
    {
      id: 'files',
      label: 'Files',
      icon: <FileText className="h-4 w-4" />,
      badge: toolConfig?.files?.length,
    },
    { id: 'settings', label: 'Settings', icon: <Settings className="h-4 w-4" /> },
    {
      id: 'mcp',
      label: 'MCP Servers',
      icon: <Server className="h-4 w-4" />,
      badge: toolConfig?.mcpServers?.length,
    },
    {
      id: 'agents',
      label: 'Agents',
      icon: <Bot className="h-4 w-4" />,
      badge: toolConfig?.agents?.length,
    },
    {
      id: 'usage',
      label: 'Usage',
      icon: <Activity className="h-4 w-4" />,
      badge: toolConfig?.usageStats?.total_actions,
    },
  ];

  if (!tool.is_installed) {
    return null;
  }

  return (
    <div className="mt-3">
      <Button
        variant="ghost"
        size="sm"
        onClick={onToggle}
        className="w-full justify-between text-xs text-muted-foreground hover:text-foreground"
      >
        <span className="flex items-center gap-1">
          <Settings className="h-3 w-3" />
          Configuration
        </span>
        <motion.div
          animate={{ rotate: isExpanded ? 90 : 0 }}
          transition={{ duration: 0.2 }}
        >
          <ChevronRight className="h-3 w-3" />
        </motion.div>
      </Button>

      <AnimatePresence>
        {isExpanded && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: 'auto', opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            transition={{ duration: 0.2 }}
            className="overflow-hidden"
          >
            <div className="pt-3">
              <Tabs
                value={activeTab}
                onValueChange={(v) => setActiveTab(v as TabId)}
                className="w-full"
              >
                <TabsList className="w-full grid grid-cols-6 h-8">
                  {tabs.map((tab) => (
                    <TabsTrigger
                      key={tab.id}
                      value={tab.id}
                      className="text-xs gap-1 px-2"
                    >
                      {tab.icon}
                      <span className="hidden sm:inline">{tab.label}</span>
                      {tab.badge !== undefined && tab.badge !== null && tab.badge > 0 && (
                        <Badge variant="secondary" className="h-4 px-1 text-[10px]">
                          {tab.badge}
                        </Badge>
                      )}
                    </TabsTrigger>
                  ))}
                </TabsList>

                <div className="mt-3 min-h-[200px]">
                  <TabsContent value="overview" className="mt-0">
                    <OverviewTab tool={tool} configDir={toolConfig?.configDir} />
                  </TabsContent>

                  <TabsContent value="files" className="mt-0">
                    {isLoadingFiles ? (
                      <LoadingState message="Loading config files..." />
                    ) : (
                      <ConfigFilesTab
                        toolType={toolType}
                        files={toolConfig?.files || []}
                        loadedFiles={toolConfig?.loadedFiles || {}}
                      />
                    )}
                  </TabsContent>

                  <TabsContent value="settings" className="mt-0">
                    {isLoadingSettings ? (
                      <LoadingState message="Loading settings..." />
                    ) : (
                      <SettingsTab
                        toolType={toolType}
                        settings={toolConfig?.settings}
                      />
                    )}
                  </TabsContent>

                  <TabsContent value="mcp" className="mt-0">
                    {isLoadingMCP ? (
                      <LoadingState message="Loading MCP servers..." />
                    ) : (
                      <MCPServersTab
                        toolType={toolType}
                        servers={toolConfig?.mcpServers || []}
                      />
                    )}
                  </TabsContent>

                  <TabsContent value="agents" className="mt-0">
                    {isLoadingAgents ? (
                      <LoadingState message="Loading agents..." />
                    ) : (
                      <AgentsTab
                        toolType={toolType}
                        agents={toolConfig?.agents || []}
                      />
                    )}
                  </TabsContent>

                  <TabsContent value="usage" className="mt-0">
                    <UsageTab toolType={toolType} />
                  </TabsContent>
                </div>
              </Tabs>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}

function LoadingState({ message }: { message: string }) {
  return (
    <div className="flex flex-col items-center justify-center py-8 text-muted-foreground">
      <Loader2 className="h-6 w-6 animate-spin mb-2" />
      <span className="text-sm">{message}</span>
    </div>
  );
}

function OverviewTab({
  tool,
  configDir,
}: {
  tool: CLIToolWithStatus;
  configDir: string | null | undefined;
}) {
  const preferredInstall = tool.preferred_installation || tool.installations[0];

  return (
    <div className="space-y-4">
      <div className="grid gap-3 text-sm">
        <div className="flex justify-between">
          <span className="text-muted-foreground">Status</span>
          <Badge variant={tool.is_installed ? 'default' : 'secondary'}>
            {tool.is_installed ? 'Installed' : 'Not Installed'}
          </Badge>
        </div>

        {preferredInstall && (
          <>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Version</span>
              <span className="font-mono text-xs">
                {preferredInstall.version || 'Unknown'}
              </span>
            </div>

            <div className="flex justify-between">
              <span className="text-muted-foreground">Source</span>
              <span className="capitalize">{preferredInstall.source.replace('_', ' ')}</span>
            </div>

            <div className="flex flex-col gap-1">
              <span className="text-muted-foreground">Command</span>
              <code className="text-xs bg-muted px-2 py-1 rounded font-mono break-all">
                {preferredInstall.command}
              </code>
            </div>
          </>
        )}

        {configDir && (
          <div className="flex flex-col gap-1">
            <span className="text-muted-foreground">Config Directory</span>
            <code className="text-xs bg-muted px-2 py-1 rounded font-mono break-all">
              {configDir}
            </code>
          </div>
        )}

        <div className="flex justify-between">
          <span className="text-muted-foreground">Installations Found</span>
          <span>{tool.installations.length}</span>
        </div>
      </div>

      {tool.description && (
        <p className="text-xs text-muted-foreground border-t pt-3">
          {tool.description}
        </p>
      )}
    </div>
  );
}
