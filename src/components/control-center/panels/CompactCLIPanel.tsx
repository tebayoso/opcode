import React from 'react';
import { CheckCircle2, XCircle, Terminal, ChevronRight } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { cn } from '@/lib/utils';
import {
  useInstalledTools,
  useSelectedCLIToolType,
  useSelectCLITool,
} from '@/stores/statusStore';
import { useCLIToolsStore } from '@/stores/cliToolsStore';
import { resolveCLIToolDisplay } from '@/types/cli-tools';
import type { CLIToolWithStatus } from '@/types/cli-tools';

interface CompactToolItemProps {
  tool: CLIToolWithStatus;
  isSelected: boolean;
  onSelect: () => void;
}

const CompactToolItem: React.FC<CompactToolItemProps> = ({
  tool,
  isSelected,
  onSelect,
}) => {
  const display = resolveCLIToolDisplay(tool.tool_type, tool.name);
  const preferredInstall = tool.preferred_installation;

  return (
    <div
      className={cn(
        "flex items-center justify-between p-2 rounded-md transition-colors cursor-pointer",
        isSelected
          ? "border border-primary/60 bg-primary/10 shadow-inner"
          : tool.is_installed
            ? "bg-green-500/5 hover:bg-green-500/10"
            : "bg-muted/30 hover:bg-muted/50"
      )}
      onClick={onSelect}
    >
      <div className="flex items-center gap-2 min-w-0">
        <span className="text-lg shrink-0">{display.icon}</span>
        <div className="min-w-0">
          <span className="text-sm font-medium truncate block">{display.name}</span>
          {preferredInstall?.version && (
            <span className="text-xs text-muted-foreground">
              v{preferredInstall.version}
            </span>
          )}
        </div>
      </div>

      <div className="flex items-center gap-1">
        {tool.is_installed ? (
          <Badge
            variant="outline"
            className="text-xs px-1.5 py-0 text-green-600 border-green-600/50 bg-green-500/10"
          >
            <CheckCircle2 className="w-3 h-3" />
          </Badge>
        ) : (
          <Badge variant="outline" className="text-xs px-1.5 py-0 text-muted-foreground">
            <XCircle className="w-3 h-3" />
          </Badge>
        )}
      </div>
    </div>
  );
};

export const CompactCLIPanel: React.FC = () => {
  const installedTools = useInstalledTools();
  const selectedToolType = useSelectedCLIToolType();
  const selectCLITool = useSelectCLITool();
  const { status } = useCLIToolsStore();
  const allTools = status?.tools || [];

  const installedCount = installedTools.length;
  const totalCount = allTools.length;

  return (
    <Card className="shrink-0">
      <CardHeader className="pb-2 pt-4 px-4">
        <div className="flex items-center justify-between">
          <CardTitle className="text-sm font-medium flex items-center gap-2">
            <Terminal className="w-4 h-4 text-primary" />
            CLI Tools
          </CardTitle>
          <Badge variant="secondary" className="text-xs">
            {installedCount}/{totalCount}
          </Badge>
        </div>
      </CardHeader>

      <CardContent className="px-4 pb-4">
        <div className="space-y-1">
          {allTools.length > 0 ? (
            allTools.map((tool) => {
              const isSelected = selectedToolType === tool.tool_type;
              return (
                <CompactToolItem
                  key={tool.tool_type}
                  tool={tool}
                  isSelected={isSelected}
                  onSelect={() =>
                    selectCLITool(isSelected ? null : tool.tool_type)
                  }
                />
              );
            })
          ) : (
            <div className="text-sm text-muted-foreground text-center py-4">
              No CLI tools detected
            </div>
          )}
        </div>

        {/* View All Link */}
        <button
          className="w-full mt-3 flex items-center justify-center gap-1 text-xs text-muted-foreground hover:text-foreground transition-colors py-1"
          >
          View Details
          <ChevronRight className="w-3 h-3" />
        </button>
      </CardContent>
    </Card>
  );
};

export default CompactCLIPanel;
