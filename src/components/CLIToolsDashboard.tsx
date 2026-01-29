import React, { useEffect, useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import {
  RefreshCw,
  CheckCircle2,
  XCircle,
  ChevronDown,
  ChevronRight,
  ExternalLink,
  Star,
  Loader2,
  Terminal,
  AlertCircle,
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { cn } from '@/lib/utils';
import {
  useCLIToolsStore,
  useCLIToolsStatus,
  useCLIToolsLoading,
  useCLIToolsError,
} from '@/stores/cliToolsStore';
import type {
  CLIToolWithStatus,
  CLIToolInstallation,
  CLIToolType,
} from '@/types/cli-tools';
import { resolveCLIToolDisplay, INSTALLATION_SOURCE_DISPLAY } from '@/types/cli-tools';
import { CLIToolConfigPanel } from '@/components/cli-tools';

interface ToolCardProps {
  tool: CLIToolWithStatus;
  onSetPreferred: (toolType: CLIToolType, path: string) => Promise<void>;
}

const ToolCard: React.FC<ToolCardProps> = ({ tool, onSetPreferred }) => {
  const [isExpanded, setIsExpanded] = useState(false);
  const [isConfigExpanded, setIsConfigExpanded] = useState(false);
  const [isSettingPreferred, setIsSettingPreferred] = useState(false);
  const display = resolveCLIToolDisplay(tool.tool_type, tool.name);

  const handleSetPreferred = async (installation: CLIToolInstallation) => {
    if (isSettingPreferred) return;
    setIsSettingPreferred(true);
    try {
      await onSetPreferred(tool.tool_type, installation.path);
    } finally {
      setIsSettingPreferred(false);
    }
  };

  const preferredPath = tool.preferred_installation?.path;

  return (
    <Card className={cn(
      "transition-all duration-200",
      tool.is_installed ? "border-green-500/30 bg-green-500/5" : "border-muted"
    )}>
      <CardHeader className="pb-2">
        <div className="flex items-start justify-between">
          <div className="flex items-center gap-3">
            <span className="text-2xl">{display.icon}</span>
            <div>
              <CardTitle className="text-lg flex items-center gap-2">
                {display.name}
                {tool.is_installed ? (
                  <Badge variant="outline" className="text-green-600 border-green-600/50 bg-green-500/10">
                    <CheckCircle2 className="w-3 h-3 mr-1" />
                    Installed
                  </Badge>
                ) : (
                  <Badge variant="outline" className="text-muted-foreground">
                    <XCircle className="w-3 h-3 mr-1" />
                    Not Installed
                  </Badge>
                )}
              </CardTitle>
              <CardDescription className="mt-1">
                {tool.description}
              </CardDescription>
            </div>
          </div>
          <Button
            variant="ghost"
            size="sm"
            asChild
          >
            <a
              href={tool.website}
              target="_blank"
              rel="noopener noreferrer"
              className="flex items-center gap-1 text-xs"
            >
              <ExternalLink className="w-3 h-3" />
              Website
            </a>
          </Button>
        </div>
      </CardHeader>

      <CardContent>
        {tool.installations.length > 0 ? (
          <div className="space-y-2">
            <button
              onClick={() => setIsExpanded(!isExpanded)}
              className="flex items-center gap-2 text-sm text-muted-foreground hover:text-foreground transition-colors w-full"
            >
              {isExpanded ? (
                <ChevronDown className="w-4 h-4" />
              ) : (
                <ChevronRight className="w-4 h-4" />
              )}
              <span>
                {tool.installations.length} installation{tool.installations.length !== 1 ? 's' : ''} found
              </span>
            </button>

            <AnimatePresence>
              {isExpanded && (
                <motion.div
                  initial={{ opacity: 0, height: 0 }}
                  animate={{ opacity: 1, height: 'auto' }}
                  exit={{ opacity: 0, height: 0 }}
                  transition={{ duration: 0.2 }}
                  className="overflow-hidden"
                >
                  <div className="space-y-2 mt-3 pl-6">
                    {tool.installations.map((installation, index) => (
                      <InstallationItem
                        key={`${installation.path}-${index}`}
                        installation={installation}
                        isPreferred={installation.path === preferredPath}
                        onSetPreferred={() => handleSetPreferred(installation)}
                        isSettingPreferred={isSettingPreferred}
                        showPreferredButton={tool.installations.length > 1}
                      />
                    ))}
                  </div>
                </motion.div>
              )}
            </AnimatePresence>
          </div>
        ) : (
          <div className="text-sm text-muted-foreground">
            <p className="mb-2">To install {display.name}:</p>
            <code className="text-xs bg-muted px-2 py-1 rounded block overflow-x-auto">
              {tool.install_instructions}
            </code>
          </div>
        )}

        {/* Configuration Panel for installed tools */}
        {tool.is_installed && (
          <CLIToolConfigPanel
            tool={tool}
            isExpanded={isConfigExpanded}
            onToggle={() => setIsConfigExpanded(!isConfigExpanded)}
          />
        )}
      </CardContent>
    </Card>
  );
};

interface InstallationItemProps {
  installation: CLIToolInstallation;
  isPreferred: boolean;
  onSetPreferred: () => void;
  isSettingPreferred: boolean;
  showPreferredButton: boolean;
}

const InstallationItem: React.FC<InstallationItemProps> = ({
  installation,
  isPreferred,
  onSetPreferred,
  isSettingPreferred,
  showPreferredButton,
}) => {
  const sourceDisplay = INSTALLATION_SOURCE_DISPLAY[installation.source];

  return (
    <div className={cn(
      "p-3 rounded-lg border transition-colors",
      isPreferred ? "border-primary/50 bg-primary/5" : "border-border bg-muted/30",
      !installation.is_available && "opacity-60"
    )}>
      <div className="flex items-start justify-between gap-2">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1">
            <span className="text-sm font-medium truncate">{installation.name}</span>
            {isPreferred && (
              <Badge variant="default" className="text-xs">
                <Star className="w-3 h-3 mr-1" />
                Preferred
              </Badge>
            )}
            {!installation.is_available && (
              <Badge variant="destructive" className="text-xs">
                Unavailable
              </Badge>
            )}
          </div>
          <div className="space-y-1">
            <p className="text-xs text-muted-foreground font-mono truncate" title={installation.path}>
              {installation.path}
            </p>
            <div className="flex items-center gap-2 flex-wrap">
              <Badge variant="outline" className="text-xs">
                {sourceDisplay.name}
              </Badge>
              {installation.source_detail && (
                <span className="text-xs text-muted-foreground">
                  {installation.source_detail}
                </span>
              )}
              {installation.version && (
                <span className="text-xs text-muted-foreground">
                  v{installation.version}
                </span>
              )}
            </div>
          </div>
        </div>
        {showPreferredButton && !isPreferred && installation.is_available && (
          <Button
            variant="outline"
            size="sm"
            onClick={onSetPreferred}
            disabled={isSettingPreferred}
            className="text-xs shrink-0"
          >
            {isSettingPreferred ? (
              <Loader2 className="w-3 h-3 animate-spin" />
            ) : (
              'Set Preferred'
            )}
          </Button>
        )}
      </div>
    </div>
  );
};

export const CLIToolsDashboard: React.FC = () => {
  const status = useCLIToolsStatus();
  const isLoading = useCLIToolsLoading();
  const error = useCLIToolsError();
  const { fetchTools, refreshTools, setPreferredInstallation, clearError } = useCLIToolsStore();

  useEffect(() => {
    fetchTools();
  }, [fetchTools]);

  const handleRefresh = async () => {
    await refreshTools();
  };

  const installedTools = status?.tools.filter(t => t.is_installed) || [];
  const notInstalledTools = status?.tools.filter(t => !t.is_installed) || [];

  return (
    <div className="h-full overflow-y-auto">
      <div className="max-w-4xl mx-auto p-6 space-y-6">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-lg bg-primary/10">
              <Terminal className="w-6 h-6 text-primary" />
            </div>
            <div>
              <h1 className="text-2xl font-bold tracking-tight">CLI Tools</h1>
              <p className="text-sm text-muted-foreground">
                Manage your AI coding assistants
              </p>
            </div>
          </div>
          <Button
            variant="outline"
            onClick={handleRefresh}
            disabled={isLoading}
            className="gap-2"
          >
            <RefreshCw className={cn("w-4 h-4", isLoading && "animate-spin")} />
            {isLoading ? 'Scanning...' : 'Refresh'}
          </Button>
        </div>

        {/* Error Banner */}
        <AnimatePresence>
          {error && (
            <motion.div
              initial={{ opacity: 0, y: -10 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -10 }}
              className="flex items-center gap-3 p-4 rounded-lg border border-destructive/50 bg-destructive/10"
            >
              <AlertCircle className="w-5 h-5 text-destructive shrink-0" />
              <p className="text-sm text-destructive flex-1">{error}</p>
              <Button variant="ghost" size="sm" onClick={clearError}>
                Dismiss
              </Button>
            </motion.div>
          )}
        </AnimatePresence>

        {/* Loading State */}
        {isLoading && !status && (
          <div className="flex items-center justify-center py-12">
            <div className="flex flex-col items-center gap-3">
              <Loader2 className="w-8 h-8 animate-spin text-primary" />
              <p className="text-sm text-muted-foreground">Scanning for CLI tools...</p>
            </div>
          </div>
        )}

        {/* Content */}
        {status && (
          <div className="space-y-8">
            {/* Installed Tools */}
            {installedTools.length > 0 && (
              <section>
                <h2 className="text-lg font-semibold mb-4 flex items-center gap-2">
                  <CheckCircle2 className="w-5 h-5 text-green-500" />
                  Installed ({installedTools.length})
                </h2>
                <div className="grid gap-4">
                  {installedTools.map(tool => (
                    <ToolCard
                      key={tool.tool_type}
                      tool={tool}
                      onSetPreferred={setPreferredInstallation}
                    />
                  ))}
                </div>
              </section>
            )}

            {/* Not Installed Tools */}
            {notInstalledTools.length > 0 && (
              <section>
                <h2 className="text-lg font-semibold mb-4 flex items-center gap-2">
                  <XCircle className="w-5 h-5 text-muted-foreground" />
                  Available to Install ({notInstalledTools.length})
                </h2>
                <div className="grid gap-4">
                  {notInstalledTools.map(tool => (
                    <ToolCard
                      key={tool.tool_type}
                      tool={tool}
                      onSetPreferred={setPreferredInstallation}
                    />
                  ))}
                </div>
              </section>
            )}

            {/* Last Updated */}
            {status.last_updated && (
              <p className="text-xs text-muted-foreground text-center">
                Last scanned: {new Date(status.last_updated).toLocaleString()}
              </p>
            )}
          </div>
        )}
      </div>
    </div>
  );
};

export default CLIToolsDashboard;
