import React, { useState, useEffect, useCallback } from "react";
import { motion, AnimatePresence } from "framer-motion";
import MDEditor from "@uiw/react-md-editor";
import {
  Brain,
  X,
  FileText,
  Loader2,
  Save,
  RefreshCw,
  ChevronRight,
  Check,
  AlertCircle,
  Home,
  FolderOpen,
  Settings,
  Sparkles,
  Server,
  Bot,
  Terminal,
  BookOpen,
  Briefcase,
  Edit3,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { api, type ClaudeMdFile, type GlobalConfigFile } from "@/lib/api";
import { formatUnixTimestamp } from "@/lib/date-utils";
import { TooltipProvider, TooltipSimple } from "@/components/ui/tooltip-modern";
import { MarkdownRenderer } from "./MarkdownRenderer";

interface ClaudeMemoriesPanelProps {
  projectPath: string;
  isOpen: boolean;
  onClose: () => void;
  className?: string;
}

// Unified file interface that works for both project and global files
interface UnifiedFile {
  name: string;
  relative_path: string;
  absolute_path: string;
  category: string;
  size: number;
  modified: number;
  source: "project" | "global";
}

// Category configuration with icons and colors
const CATEGORY_CONFIG: Record<
  string,
  { icon: React.ReactNode; color: string; bgColor: string; order: number }
> = {
  Project: {
    icon: <FolderOpen className="h-4 w-4" />,
    color: "text-green-500",
    bgColor: "bg-green-500/10",
    order: 0,
  },
  "Core Instructions": {
    icon: <BookOpen className="h-4 w-4" />,
    color: "text-blue-500",
    bgColor: "bg-blue-500/10",
    order: 1,
  },
  Settings: {
    icon: <Settings className="h-4 w-4" />,
    color: "text-gray-500",
    bgColor: "bg-gray-500/10",
    order: 2,
  },
  Modes: {
    icon: <Sparkles className="h-4 w-4" />,
    color: "text-purple-500",
    bgColor: "bg-purple-500/10",
    order: 3,
  },
  "MCP Servers": {
    icon: <Server className="h-4 w-4" />,
    color: "text-cyan-500",
    bgColor: "bg-cyan-500/10",
    order: 4,
  },
  Agents: {
    icon: <Bot className="h-4 w-4" />,
    color: "text-orange-500",
    bgColor: "bg-orange-500/10",
    order: 5,
  },
  Commands: {
    icon: <Terminal className="h-4 w-4" />,
    color: "text-yellow-500",
    bgColor: "bg-yellow-500/10",
    order: 6,
  },
  Business: {
    icon: <Briefcase className="h-4 w-4" />,
    color: "text-pink-500",
    bgColor: "bg-pink-500/10",
    order: 7,
  },
  Research: {
    icon: <BookOpen className="h-4 w-4" />,
    color: "text-indigo-500",
    bgColor: "bg-indigo-500/10",
    order: 8,
  },
};

const getCategoryConfig = (category: string) => (
    CATEGORY_CONFIG[category] || {
      icon: <FileText className="h-4 w-4" />,
      color: "text-muted-foreground",
      bgColor: "bg-muted/50",
      order: 99,
    }
  );

export const ClaudeMemoriesPanel: React.FC<ClaudeMemoriesPanelProps> = ({
  projectPath,
  isOpen,
  onClose,
}) => {
  const [files, setFiles] = useState<UnifiedFile[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [collapsedCategories, setCollapsedCategories] = useState<Set<string>>(
    new Set()
  );
  const [selectedFile, setSelectedFile] = useState<UnifiedFile | null>(null);
  const [fileContent, setFileContent] = useState<string>("");
  const [editedContent, setEditedContent] = useState<string>("");
  const [isEditing, setIsEditing] = useState(false);
  const [isLoadingContent, setIsLoadingContent] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);
  const [saveSuccess, setSaveSuccess] = useState(false);
  const [sidebarWidth, setSidebarWidth] = useState(280);
  const [isResizing, setIsResizing] = useState(false);

  const loadFiles = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);

      // Fetch both project and global files in parallel
      const [projectFiles, globalFiles] = await Promise.all([
        projectPath
          ? api.findClaudeMdFiles(projectPath).catch(() => [])
          : Promise.resolve([]),
        api.findGlobalConfigFiles().catch(() => []),
      ]);

      // Convert project files to unified format
      const unifiedProjectFiles: UnifiedFile[] = projectFiles.map(
        (file: ClaudeMdFile) => ({
          name: file.relative_path.split("/").pop() || file.relative_path,
          relative_path: file.relative_path,
          absolute_path: file.absolute_path,
          category: "Project",
          size: file.size,
          modified: file.modified,
          source: "project" as const,
        })
      );

      // Convert global files to unified format
      const unifiedGlobalFiles: UnifiedFile[] = globalFiles.map(
        (file: GlobalConfigFile) => ({
          name: file.name,
          relative_path: file.relative_path,
          absolute_path: file.absolute_path,
          category: file.category,
          size: file.size,
          modified: file.modified,
          source: "global" as const,
        })
      );

      // Combine and sort by category order, then by name
      const allFiles = [...unifiedProjectFiles, ...unifiedGlobalFiles].sort(
        (a, b) => {
          const orderA = getCategoryConfig(a.category).order;
          const orderB = getCategoryConfig(b.category).order;
          if (orderA !== orderB) {return orderA - orderB;}
          return a.name.localeCompare(b.name);
        }
      );

      setFiles(allFiles);

      // Auto-select first project file if exists and nothing is selected
      if (allFiles.length > 0 && !selectedFile) {
        const firstProjectFile = allFiles.find((f) => f.category === "Project");
        if (firstProjectFile) {
          selectFile(firstProjectFile);
        } else {
          selectFile(allFiles[0]);
        }
      }
    } catch (err) {
      console.error("Failed to load config files:", err);
      setError("Failed to load configuration files");
    } finally {
      setLoading(false);
    }
  }, [projectPath, selectedFile]);

  useEffect(() => {
    if (isOpen) {
      loadFiles();
    }
  }, [isOpen, loadFiles]);

  const selectFile = async (file: UnifiedFile) => {
    setSelectedFile(file);
    setIsEditing(false);
    setSaveError(null);
    setSaveSuccess(false);
    setIsLoadingContent(true);

    try {
      const content = await api.readClaudeMdFile(file.absolute_path);
      setFileContent(content);
      setEditedContent(content);
    } catch (err) {
      console.error("Failed to load file content:", err);
      setFileContent("Error loading file content");
      setEditedContent("");
    } finally {
      setIsLoadingContent(false);
    }
  };

  const toggleCategory = (category: string) => {
    setCollapsedCategories((prev) => {
      const newSet = new Set(prev);
      if (newSet.has(category)) {
        newSet.delete(category);
      } else {
        newSet.add(category);
      }
      return newSet;
    });
  };

  const startEditing = () => {
    setIsEditing(true);
    setSaveError(null);
    setSaveSuccess(false);
  };

  const cancelEditing = () => {
    setIsEditing(false);
    setEditedContent(fileContent);
    setSaveError(null);
  };

  const saveFile = async () => {
    if (!selectedFile) {return;}

    setIsSaving(true);
    setSaveError(null);
    setSaveSuccess(false);

    try {
      await api.saveClaudeMdFile(selectedFile.absolute_path, editedContent);
      setFileContent(editedContent);
      setIsEditing(false);
      setSaveSuccess(true);

      setTimeout(() => {
        setSaveSuccess(false);
      }, 2000);
    } catch (err) {
      console.error("Failed to save file:", err);
      setSaveError("Failed to save file");
    } finally {
      setIsSaving(false);
    }
  };

  const formatFileSize = (bytes: number): string => {
    if (bytes < 1024) {return `${bytes} B`;}
    if (bytes < 1024 * 1024) {return `${(bytes / 1024).toFixed(1)} KB`;}
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  // Group files by category
  const filesByCategory = files.reduce<Record<string, UnifiedFile[]>>(
    (acc, file) => {
      if (!acc[file.category]) {
        acc[file.category] = [];
      }
      acc[file.category].push(file);
      return acc;
    },
    {}
  );

  // Sort categories by order
  const sortedCategories = Object.keys(filesByCategory).sort((a, b) => getCategoryConfig(a).order - getCategoryConfig(b).order);

  const hasChanges = editedContent !== fileContent;

  // Handle sidebar resize
  const handleMouseDown = (e: React.MouseEvent) => {
    e.preventDefault();
    setIsResizing(true);
  };

  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      if (!isResizing) {return;}
      const newWidth = Math.max(200, Math.min(500, e.clientX));
      setSidebarWidth(newWidth);
    };

    const handleMouseUp = () => {
      setIsResizing(false);
    };

    if (isResizing) {
      document.addEventListener("mousemove", handleMouseMove);
      document.addEventListener("mouseup", handleMouseUp);
    }

    return () => {
      document.removeEventListener("mousemove", handleMouseMove);
      document.removeEventListener("mouseup", handleMouseUp);
    };
  }, [isResizing]);

  // Don't render if not open
  if (!isOpen) {return null;}

  return (
    <TooltipProvider>
      <div className="h-full bg-background">
        <div className="h-full flex flex-col">
            {/* Header */}
            <div className="flex items-center justify-between px-4 py-3 border-b border-border bg-background/95 backdrop-blur-sm">
              <div className="flex items-center gap-3">
                <Brain className="h-5 w-5 text-primary" />
                <div>
                  <h2 className="text-lg font-semibold">Claude Memories</h2>
                  <p className="text-xs text-muted-foreground">
                    Configuration files that guide Claude's behavior
                  </p>
                </div>
              </div>
              <div className="flex items-center gap-2">
                <TooltipSimple content="Refresh files" side="bottom">
                  <Button
                    variant="ghost"
                    size="icon"
                    onClick={loadFiles}
                    disabled={loading}
                    className="h-8 w-8"
                  >
                    <RefreshCw
                      className={cn("h-4 w-4", loading && "animate-spin")}
                    />
                  </Button>
                </TooltipSimple>
                <Button
                  variant="ghost"
                  size="icon"
                  onClick={onClose}
                  className="h-8 w-8"
                >
                  <X className="h-4 w-4" />
                </Button>
              </div>
            </div>

            {/* Main Content */}
            <div className="flex-1 flex overflow-hidden">
              {/* Sidebar */}
              <div
                className="border-r border-border bg-muted/20 overflow-hidden flex flex-col"
                style={{ width: sidebarWidth }}
              >
                <div className="flex-1 overflow-y-auto">
                  {loading && files.length === 0 ? (
                    <div className="flex items-center justify-center h-32">
                      <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
                    </div>
                  ) : error ? (
                    <div className="flex items-center gap-2 p-4 m-4 rounded-lg bg-destructive/10 text-destructive">
                      <AlertCircle className="h-4 w-4" />
                      <span className="text-sm">{error}</span>
                    </div>
                  ) : files.length === 0 ? (
                    <div className="text-center py-8 px-4">
                      <FileText className="h-12 w-12 mx-auto text-muted-foreground/50 mb-3" />
                      <p className="text-sm text-muted-foreground">
                        No configuration files found
                      </p>
                    </div>
                  ) : (
                    <div className="py-2">
                      {sortedCategories.map((category) => {
                        const categoryFiles = filesByCategory[category];
                        const config = getCategoryConfig(category);
                        const isCollapsed = collapsedCategories.has(category);

                        return (
                          <div key={category} className="mb-1">
                            {/* Category Header */}
                            <button
                              onClick={() => { toggleCategory(category); }}
                              className="w-full flex items-center gap-2 px-3 py-2 hover:bg-muted/50 transition-colors"
                            >
                              <motion.div
                                animate={{ rotate: isCollapsed ? 0 : 90 }}
                                transition={{ duration: 0.15 }}
                              >
                                <ChevronRight className="h-3.5 w-3.5 text-muted-foreground" />
                              </motion.div>
                              <span
                                className={cn(
                                  "flex items-center gap-2",
                                  config.color
                                )}
                              >
                                {config.icon}
                              </span>
                              <span className="text-sm font-medium flex-1 text-left truncate">
                                {category}
                              </span>
                              <span className="text-[10px] text-muted-foreground px-1.5 py-0.5 rounded bg-muted">
                                {categoryFiles.length}
                              </span>
                            </button>

                            {/* Category Files */}
                            <AnimatePresence>
                              {!isCollapsed && (
                                <motion.div
                                  initial={{ height: 0, opacity: 0 }}
                                  animate={{ height: "auto", opacity: 1 }}
                                  exit={{ height: 0, opacity: 0 }}
                                  transition={{ duration: 0.15 }}
                                  className="overflow-hidden"
                                >
                                  {categoryFiles.map((file) => {
                                    const isSelected =
                                      selectedFile?.absolute_path ===
                                      file.absolute_path;

                                    return (
                                      <button
                                        key={file.absolute_path}
                                        onClick={() => selectFile(file)}
                                        className={cn(
                                          "w-full flex items-center gap-2 px-3 py-1.5 pl-9 text-left transition-colors",
                                          isSelected
                                            ? "bg-primary/10 text-primary border-l-2 border-primary"
                                            : "hover:bg-muted/50"
                                        )}
                                      >
                                        <FileText className="h-3.5 w-3.5 text-muted-foreground shrink-0" />
                                        <span className="text-sm truncate flex-1">
                                          {file.name}
                                        </span>
                                        {file.source === "global" && (
                                          <Home className="h-3 w-3 text-muted-foreground shrink-0" />
                                        )}
                                      </button>
                                    );
                                  })}
                                </motion.div>
                              )}
                            </AnimatePresence>
                          </div>
                        );
                      })}
                    </div>
                  )}
                </div>
              </div>

              {/* Resize Handle */}
              <div
                className={cn(
                  "w-1 cursor-col-resize hover:bg-primary/50 transition-colors",
                  isResizing && "bg-primary/50"
                )}
                onMouseDown={handleMouseDown}
              />

              {/* Main Content Area */}
              <div className="flex-1 flex flex-col overflow-hidden">
                {selectedFile ? (
                  <>
                    {/* File Header */}
                    <div className="flex items-center justify-between px-6 py-3 border-b border-border bg-background">
                      <div className="flex items-center gap-3 min-w-0">
                        <div
                          className={cn(
                            "p-2 rounded-md",
                            getCategoryConfig(selectedFile.category).bgColor
                          )}
                        >
                          <span
                            className={
                              getCategoryConfig(selectedFile.category).color
                            }
                          >
                            {getCategoryConfig(selectedFile.category).icon}
                          </span>
                        </div>
                        <div className="min-w-0">
                          <div className="flex items-center gap-2">
                            <h3 className="font-semibold truncate">
                              {selectedFile.name}
                            </h3>
                            {saveSuccess && (
                              <span className="inline-flex items-center gap-1 text-xs text-green-500 bg-green-500/10 px-2 py-0.5 rounded">
                                <Check className="h-3 w-3" />
                                Saved
                              </span>
                            )}
                          </div>
                          <div className="flex items-center gap-3 text-xs text-muted-foreground">
                            <span className="truncate max-w-[300px]">
                              {selectedFile.relative_path}
                            </span>
                            <span>•</span>
                            <span>{formatFileSize(selectedFile.size)}</span>
                            <span>•</span>
                            <span>
                              {formatUnixTimestamp(selectedFile.modified)}
                            </span>
                          </div>
                        </div>
                      </div>
                      <div className="flex items-center gap-2">
                        {isEditing ? (
                          <>
                            <Button
                              variant="outline"
                              size="sm"
                              onClick={cancelEditing}
                              disabled={isSaving}
                            >
                              Cancel
                            </Button>
                            <Button
                              size="sm"
                              onClick={saveFile}
                              disabled={isSaving || !hasChanges}
                              className="gap-1.5"
                            >
                              {isSaving ? (
                                <Loader2 className="h-3.5 w-3.5 animate-spin" />
                              ) : (
                                <Save className="h-3.5 w-3.5" />
                              )}
                              Save
                            </Button>
                          </>
                        ) : (
                          <Button
                            variant="outline"
                            size="sm"
                            onClick={startEditing}
                            className="gap-1.5"
                          >
                            <Edit3 className="h-3.5 w-3.5" />
                            Edit
                          </Button>
                        )}
                      </div>
                    </div>

                    {/* Error Display */}
                    {saveError && (
                      <div className="mx-6 mt-4 flex items-center gap-2 p-3 rounded-lg bg-destructive/10 text-destructive">
                        <AlertCircle className="h-4 w-4 shrink-0" />
                        <span className="text-sm">{saveError}</span>
                      </div>
                    )}

                    {/* Content Area */}
                    <div className="flex-1 overflow-hidden">
                      {isLoadingContent ? (
                        <div className="flex items-center justify-center h-full">
                          <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
                        </div>
                      ) : isEditing ? (
                        <div
                          className="h-full p-4"
                          data-color-mode="dark"
                        >
                          <MDEditor
                            value={editedContent}
                            onChange={(val) => { setEditedContent(val || ""); }}
                            preview="live"
                            height="100%"
                            visibleDragbar={false}
                            className="h-full"
                          />
                        </div>
                      ) : (
                        <div className="h-full overflow-y-auto p-6">
                          <div className="max-w-4xl mx-auto">
                            <MarkdownRenderer
                              content={fileContent}
                              size="base"
                              fullWidth
                            />
                          </div>
                        </div>
                      )}
                    </div>
                  </>
                ) : (
                  <div className="flex-1 flex items-center justify-center text-muted-foreground">
                    <div className="text-center">
                      <FileText className="h-16 w-16 mx-auto mb-4 opacity-50" />
                      <p className="text-lg font-medium">No file selected</p>
                      <p className="text-sm mt-1">
                        Select a file from the sidebar to view its contents
                      </p>
                    </div>
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>
    </TooltipProvider>
  );
};
