import React, { useState, useEffect } from 'react';
import {
  FileText,
  Copy,
  Move,
  Merge,
  Loader2,
  FolderOpen,
  ExternalLink,
  Check,
} from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { cn } from '@/lib/utils';
import { api } from '@/lib/api';
import { useFileOpsStore, useSelectedFiles } from '@/stores/fileOpsStore';
import { useTabState } from '@/hooks/useTabState';

interface ConfigFile {
  name: string;
  path: string;
  exists: boolean;
  size?: number;
  modifiedAt?: string;
}

interface ConfigFileItemProps {
  file: ConfigFile;
  isSelected: boolean;
  onSelect: () => void;
  onOpen: () => void;
}

const ConfigFileItem: React.FC<ConfigFileItemProps> = ({
  file,
  isSelected,
  onSelect,
  onOpen,
}) => (
    <div
      className={cn(
        "flex items-center justify-between p-3 rounded-lg border transition-colors cursor-pointer",
        isSelected
          ? "border-primary/50 bg-primary/5"
          : "border-border/50 bg-card hover:bg-accent/5"
      )}
      onClick={onSelect}
    >
      <div className="flex items-center gap-3 min-w-0">
        <div
          className={cn(
            "p-2 rounded-md shrink-0",
            file.exists ? "bg-blue-500/10" : "bg-muted/50"
          )}
        >
          <FileText
            className={cn(
              "w-4 h-4",
              file.exists ? "text-blue-500" : "text-muted-foreground"
            )}
          />
        </div>
        <div className="min-w-0">
          <span className="text-sm font-medium truncate block">{file.name}</span>
          <span className="text-xs text-muted-foreground truncate block">
            {file.exists ? (
              file.modifiedAt ? `Modified ${file.modifiedAt}` : 'Available'
            ) : (
              'Not created yet'
            )}
          </span>
        </div>
      </div>

      <div className="flex items-center gap-1">
        {isSelected && (
          <Badge variant="default" className="mr-2">
            <Check className="w-3 h-3" />
          </Badge>
        )}
        {file.exists && (
          <Button
            variant="ghost"
            size="icon"
            className="h-7 w-7"
            onClick={(e) => {
              e.stopPropagation();
              onOpen();
            }}
            title="Open File"
          >
            <ExternalLink className="w-3.5 h-3.5" />
          </Button>
        )}
      </div>
    </div>
  );

interface ConfigFilesPanelProps {
  projectPath?: string;
}

// Define standard config files outside component to prevent recreation on each render
const STANDARD_CONFIG_FILES = [
  { name: 'CLAUDE.md', relativePath: 'CLAUDE.md' },
  { name: 'agents.md', relativePath: '.claude/agents.md' },
  { name: '.claudeignore', relativePath: '.claudeignore' },
  { name: 'mcp.json', relativePath: '.claude/mcp.json' },
] as const;

export const ConfigFilesPanel: React.FC<ConfigFilesPanelProps> = ({ projectPath }) => {
  const [configFiles, setConfigFiles] = useState<ConfigFile[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const selectedFiles = useSelectedFiles();
  const {
    selectFile,
    deselectFile,
    clearSelection,
    openMergeDialog,
    copyFile,
  } = useFileOpsStore();
  const { createClaudeMdTab } = useTabState();

  useEffect(() => {
    let isMounted = true;

    const checkConfigFiles = async () => {
      setIsLoading(true);

      const files: ConfigFile[] = [];

      for (const config of STANDARD_CONFIG_FILES) {
        const fullPath = projectPath
          ? `${projectPath}/${config.relativePath}`
          : config.relativePath;

        // Check if file exists (simplified - in real implementation would call API)
        try {
          const content = await api.readClaudeMdFile(projectPath || '');
          files.push({
            name: config.name,
            path: fullPath,
            exists: config.name === 'CLAUDE.md' && content.length > 0,
          });
        } catch {
          files.push({
            name: config.name,
            path: fullPath,
            exists: false,
          });
        }
      }

      // Only update state if component is still mounted
      if (isMounted) {
        setConfigFiles(files);
        setIsLoading(false);
      }
    };

    checkConfigFiles();

    return () => {
      isMounted = false;
    };
  }, [projectPath]);

  const handleSelectFile = (file: ConfigFile) => {
    if (selectedFiles.includes(file.path)) {
      deselectFile(file.path);
    } else {
      selectFile(file.path);
    }
  };

  const handleOpenFile = (file: ConfigFile) => {
    if (file.name === 'CLAUDE.md') {
      createClaudeMdTab();
    }
    // Add handlers for other file types
  };

  const handleCopy = () => {
    if (selectedFiles.length === 1) {
      const sourcePath = selectedFiles[0];
      const destPath = `${sourcePath}.backup`;
      copyFile(sourcePath, destPath);
    }
  };

  const handleMerge = () => {
    if (selectedFiles.length === 2) {
      openMergeDialog(selectedFiles[0], selectedFiles[1]);
    }
  };

  const canMerge = selectedFiles.length === 2;
  const canCopy = selectedFiles.length === 1;

  return (
    <Card className="flex-1 min-h-0 flex flex-col">
      <CardHeader className="pb-2 pt-4 px-4 shrink-0">
        <div className="flex items-center justify-between">
          <CardTitle className="text-sm font-medium flex items-center gap-2">
            <FileText className="w-4 h-4 text-blue-500" />
            Config Files
          </CardTitle>
          <Button
            variant="ghost"
            size="icon"
            className="h-6 w-6"
            title="Open Config Folder"
          >
            <FolderOpen className="w-4 h-4" />
          </Button>
        </div>
      </CardHeader>

      <CardContent className="px-4 pb-4 flex-1 overflow-y-auto">
        {isLoading ? (
          <div className="flex items-center justify-center py-8">
            <Loader2 className="w-6 h-6 animate-spin text-muted-foreground" />
          </div>
        ) : (
          <>
            <div className="space-y-2">
              {configFiles.map((file) => (
                <ConfigFileItem
                  key={file.path}
                  file={file}
                  isSelected={selectedFiles.includes(file.path)}
                  onSelect={() => handleSelectFile(file)}
                  onOpen={() => handleOpenFile(file)}
                />
              ))}
            </div>

            {/* File Operations Toolbar */}
            {selectedFiles.length > 0 && (
              <div className="mt-4 pt-3 border-t border-border/50">
                <div className="flex items-center justify-between mb-2">
                  <span className="text-xs text-muted-foreground">
                    {selectedFiles.length} selected
                  </span>
                  <Button
                    variant="ghost"
                    size="sm"
                    className="text-xs h-6"
                    onClick={clearSelection}
                  >
                    Clear
                  </Button>
                </div>
                <div className="flex items-center gap-2">
                  <Button
                    variant="outline"
                    size="sm"
                    className="flex-1 h-8 text-xs"
                    onClick={handleCopy}
                    disabled={!canCopy}
                  >
                    <Copy className="w-3 h-3 mr-1" />
                    Copy
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    className="flex-1 h-8 text-xs"
                    disabled={!canCopy}
                  >
                    <Move className="w-3 h-3 mr-1" />
                    Move
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    className="flex-1 h-8 text-xs"
                    onClick={handleMerge}
                    disabled={!canMerge}
                  >
                    <Merge className="w-3 h-3 mr-1" />
                    Merge
                  </Button>
                </div>
              </div>
            )}
          </>
        )}
      </CardContent>
    </Card>
  );
};

export default ConfigFilesPanel;
