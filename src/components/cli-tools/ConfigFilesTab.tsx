/**
 * Config Files Tab
 * File browser with lazy loading and inline editing
 */

import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import {
  FileText,
  FileCode,
  FileJson,
  ChevronRight,
  Save,
  X,
  RefreshCw,
  Loader2,
  Copy,
  Check,
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Textarea } from '@/components/ui/textarea';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import { useCLIToolConfigStore } from '@/stores/cliToolConfigStore';
import type {
  CLIToolType,
  ConfigFileInfo,
  ConfigFileContent,
  ConfigFileType,
} from '@/types/cli-tools';

interface ConfigFilesTabProps {
  toolType: CLIToolType;
  files: ConfigFileInfo[];
  loadedFiles: Record<string, ConfigFileContent>;
}

export function ConfigFilesTab({
  toolType,
  files,
  loadedFiles,
}: ConfigFilesTabProps) {
  const [expandedFile, setExpandedFile] = useState<string | null>(null);
  const [editingFile, setEditingFile] = useState<string | null>(null);
  const [editContent, setEditContent] = useState<string>('');
  const [copiedPath, setCopiedPath] = useState<string | null>(null);

  const { loadConfigFile, saveConfigFile, loadConfigFiles, loading } =
    useCLIToolConfigStore();

  const isLoadingFile = (path: string) => loading.fileContent[`${toolType}:${path}`] || false;

  const handleExpand = async (file: ConfigFileInfo) => {
    if (expandedFile === file.path) {
      setExpandedFile(null);
      setEditingFile(null);
      return;
    }

    setExpandedFile(file.path);
    setEditingFile(null);

    // Load file content if not already loaded
    if (!loadedFiles[file.path]) {
      await loadConfigFile(toolType, file.path);
    }
  };

  const handleEdit = (file: ConfigFileInfo) => {
    const content = loadedFiles[file.path]?.content || '';
    setEditContent(content);
    setEditingFile(file.path);
  };

  const handleSave = async (path: string) => {
    await saveConfigFile(toolType, path, editContent);
    setEditingFile(null);
  };

  const handleCancelEdit = () => {
    setEditingFile(null);
    setEditContent('');
  };

  const handleCopyPath = async (path: string) => {
    await navigator.clipboard.writeText(path);
    setCopiedPath(path);
    setTimeout(() => setCopiedPath(null), 2000);
  };

  const handleRefresh = () => {
    loadConfigFiles(toolType);
  };

  const getFileIcon = (fileType: ConfigFileType) => {
    switch (fileType) {
      case 'json':
      case 'jsonc':
        return <FileJson className="h-4 w-4 text-yellow-500" />;
      case 'toml':
      case 'yaml':
        return <FileCode className="h-4 w-4 text-blue-500" />;
      case 'markdown':
        return <FileText className="h-4 w-4 text-purple-500" />;
      default:
        return <FileText className="h-4 w-4" />;
    }
  };

  const formatFileSize = (bytes: number): string => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  const formatDate = (dateStr: string): string => {
    try {
      return new Date(dateStr).toLocaleDateString(undefined, {
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
      });
    } catch {
      return dateStr;
    }
  };

  if (files.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-8 text-muted-foreground">
        <FileText className="h-8 w-8 mb-2 opacity-50" />
        <p className="text-sm">No configuration files found</p>
        <Button
          variant="ghost"
          size="sm"
          onClick={handleRefresh}
          className="mt-2"
        >
          <RefreshCw className="h-3 w-3 mr-1" />
          Refresh
        </Button>
      </div>
    );
  }

  // Group files by scope
  const filesByScope = files.reduce(
    (acc, file) => {
      if (!acc[file.scope]) acc[file.scope] = [];
      acc[file.scope].push(file);
      return acc;
    },
    {} as Record<string, ConfigFileInfo[]>
  );

  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <span className="text-xs text-muted-foreground">
          {files.length} file{files.length !== 1 ? 's' : ''} found
        </span>
        <Button variant="ghost" size="sm" onClick={handleRefresh}>
          <RefreshCw className="h-3 w-3" />
        </Button>
      </div>

      {Object.entries(filesByScope).map(([scope, scopeFiles]) => (
        <div key={scope} className="space-y-2">
          <div className="flex items-center gap-2">
            <span className="text-xs font-medium capitalize">{scope}</span>
            <Badge variant="outline" className="text-[10px]">
              {scopeFiles.length}
            </Badge>
          </div>

          <div className="space-y-1">
            {scopeFiles.map((file) => (
              <FileItem
                key={file.path}
                file={file}
                isExpanded={expandedFile === file.path}
                isEditing={editingFile === file.path}
                isLoading={isLoadingFile(file.path)}
                content={loadedFiles[file.path]}
                editContent={editContent}
                copiedPath={copiedPath}
                onExpand={() => handleExpand(file)}
                onEdit={() => handleEdit(file)}
                onSave={() => handleSave(file.path)}
                onCancelEdit={handleCancelEdit}
                onEditContentChange={setEditContent}
                onCopyPath={() => handleCopyPath(file.path)}
                getFileIcon={getFileIcon}
                formatFileSize={formatFileSize}
                formatDate={formatDate}
              />
            ))}
          </div>
        </div>
      ))}
    </div>
  );
}

interface FileItemProps {
  file: ConfigFileInfo;
  isExpanded: boolean;
  isEditing: boolean;
  isLoading: boolean;
  content: ConfigFileContent | undefined;
  editContent: string;
  copiedPath: string | null;
  onExpand: () => void;
  onEdit: () => void;
  onSave: () => void;
  onCancelEdit: () => void;
  onEditContentChange: (content: string) => void;
  onCopyPath: () => void;
  getFileIcon: (type: ConfigFileType) => React.ReactNode;
  formatFileSize: (bytes: number) => string;
  formatDate: (date: string) => string;
}

function FileItem({
  file,
  isExpanded,
  isEditing,
  isLoading,
  content,
  editContent,
  copiedPath,
  onExpand,
  onEdit,
  onSave,
  onCancelEdit,
  onEditContentChange,
  onCopyPath,
  getFileIcon,
  formatFileSize,
  formatDate,
}: FileItemProps) {
  return (
    <div className="border rounded-md overflow-hidden">
      <button
        onClick={onExpand}
        className="w-full flex items-center gap-2 p-2 hover:bg-muted/50 transition-colors text-left"
      >
        <motion.div
          animate={{ rotate: isExpanded ? 90 : 0 }}
          transition={{ duration: 0.15 }}
        >
          <ChevronRight className="h-3 w-3 text-muted-foreground" />
        </motion.div>

        {getFileIcon(file.file_type)}

        <span className="flex-1 text-sm truncate">{file.name}</span>

        <span className="text-xs text-muted-foreground">
          {formatFileSize(file.size_bytes)}
        </span>

        <Badge variant="outline" className="text-[10px]">
          {file.file_type}
        </Badge>
      </button>

      <AnimatePresence>
        {isExpanded && (
          <motion.div
            initial={{ height: 0 }}
            animate={{ height: 'auto' }}
            exit={{ height: 0 }}
            transition={{ duration: 0.15 }}
            className="overflow-hidden"
          >
            <div className="border-t p-3 space-y-3">
              {/* File metadata */}
              <div className="flex flex-wrap gap-x-4 gap-y-1 text-xs text-muted-foreground">
                <TooltipProvider>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          onCopyPath();
                        }}
                        className="flex items-center gap-1 hover:text-foreground transition-colors"
                      >
                        {copiedPath === file.path ? (
                          <Check className="h-3 w-3 text-green-500" />
                        ) : (
                          <Copy className="h-3 w-3" />
                        )}
                        <span className="truncate max-w-[200px]">{file.path}</span>
                      </button>
                    </TooltipTrigger>
                    <TooltipContent>
                      <p>Click to copy path</p>
                    </TooltipContent>
                  </Tooltip>
                </TooltipProvider>

                <span>Modified: {formatDate(file.modified)}</span>

                {file.description && (
                  <span className="italic">{file.description}</span>
                )}
              </div>

              {/* File content */}
              {isLoading ? (
                <div className="flex items-center justify-center py-4">
                  <Loader2 className="h-4 w-4 animate-spin mr-2" />
                  <span className="text-sm text-muted-foreground">
                    Loading content...
                  </span>
                </div>
              ) : content ? (
                <div className="space-y-2">
                  {isEditing ? (
                    <>
                      <Textarea
                        value={editContent}
                        onChange={(e) => onEditContentChange(e.target.value)}
                        className="font-mono text-xs min-h-[200px] resize-y"
                        placeholder="File content..."
                      />
                      <div className="flex gap-2 justify-end">
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={(e) => {
                            e.stopPropagation();
                            onCancelEdit();
                          }}
                        >
                          <X className="h-3 w-3 mr-1" />
                          Cancel
                        </Button>
                        <Button
                          size="sm"
                          onClick={(e) => {
                            e.stopPropagation();
                            onSave();
                          }}
                        >
                          <Save className="h-3 w-3 mr-1" />
                          Save
                        </Button>
                      </div>
                    </>
                  ) : (
                    <>
                      <pre className="bg-muted rounded p-3 text-xs font-mono overflow-auto max-h-[300px] whitespace-pre-wrap">
                        {content.content || '(empty file)'}
                      </pre>
                      <div className="flex justify-end">
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={(e) => {
                            e.stopPropagation();
                            onEdit();
                          }}
                        >
                          Edit
                        </Button>
                      </div>
                    </>
                  )}
                </div>
              ) : (
                <p className="text-sm text-muted-foreground italic">
                  Content not loaded
                </p>
              )}
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
