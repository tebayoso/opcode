import React, { useState, useEffect, useCallback } from "react";
import { motion, AnimatePresence } from "framer-motion";
import {
  Plus,
  Trash2,
  Edit,
  Save,
  Sparkles,
  Globe,
  FolderOpen,
  FileCode,
  FileJson,
  File,
  AlertCircle,
  Loader2,
  Search,
  ChevronDown,
  ChevronRight,
  Code
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Card } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog";
import MDEditor from "@uiw/react-md-editor";
import { api, type Skill, type SupportingFile } from "@/lib/api";
import { cn } from "@/lib/utils";
import { COMMON_TOOL_MATCHERS } from "@/types/hooks";
import { useTrackEvent, useTheme } from "@/hooks";

interface SkillsManagerProps {
  projectPath?: string;
  className?: string;
  scopeFilter?: 'project' | 'user' | 'all';
}

interface SkillForm {
  name: string;
  content: string;
  fileType: 'markdown' | 'json';
  description: string;
  allowedTools: string[];
  scope: 'project' | 'user';
}

interface JsonError {
  message: string;
  line?: number;
}

// Get icon for skill based on its file type
const getSkillIcon = (skill: Skill): React.ElementType => {
  if (skill.file_type === "json") {return FileJson;}
  if (skill.file_type === "markdown") {return FileCode;}
  return File;
};

// Validate JSON and return any errors
const validateJson = (content: string): JsonError | null => {
  try {
    JSON.parse(content);
    return null;
  } catch (e) {
    if (e instanceof SyntaxError) {
      // Try to extract line number from error message
      const match = /position (\d+)/.exec(e.message);
      let line: number | undefined;
      if (match) {
        const pos = parseInt(match[1], 10);
        line = content.substring(0, pos).split('\n').length;
      }
      return { message: e.message, line };
    }
    return { message: "Invalid JSON" };
  }
};

// Format JSON content
const formatJson = (content: string): string => {
  try {
    return JSON.stringify(JSON.parse(content), null, 2);
  } catch {
    return content;
  }
};

/**
 * SkillsManager component for managing Claude skills
 * Provides a no-code interface for creating, editing, and deleting skills
 */
export const SkillsManager: React.FC<SkillsManagerProps> = ({
  projectPath,
  className,
  scopeFilter = 'all',
}) => {
  const [skills, setSkills] = useState<Skill[]>([]);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedScope, setSelectedScope] = useState<'all' | 'project' | 'user'>(
    scopeFilter === 'all' ? 'all' : scopeFilter
  );
  const [expandedSkills, setExpandedSkills] = useState<Set<string>>(new Set());

  // Edit dialog state
  const [editDialogOpen, setEditDialogOpen] = useState(false);
  const [editingSkill, setEditingSkill] = useState<Skill | null>(null);
  const [skillForm, setSkillForm] = useState<SkillForm>({
    name: "",
    content: "",
    fileType: 'markdown',
    description: "",
    allowedTools: [],
    scope: 'user'
  });
  const [jsonError, setJsonError] = useState<JsonError | null>(null);

  // Delete confirmation dialog state
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [skillToDelete, setSkillToDelete] = useState<Skill | null>(null);
  const [deleting, setDeleting] = useState(false);

  // Theme
  const { theme } = useTheme();

  // Analytics tracking
  const trackEvent = useTrackEvent();

  // Load skills on mount
  useEffect(() => {
    void loadSkills();
  }, [projectPath]);

  const loadSkills = async (): Promise<void> => {
    try {
      setLoading(true);
      setError(null);
      const loadedSkills = await api.skillsList(projectPath);
      setSkills(loadedSkills);
    } catch (err) {
      console.error("Failed to load skills:", err);
      setError("Failed to load skills");
    } finally {
      setLoading(false);
    }
  };

  const handleCreateNew = (): void => {
    setEditingSkill(null);
    setSkillForm({
      name: "",
      content: "",
      fileType: 'markdown',
      description: "",
      allowedTools: [],
      scope: scopeFilter !== 'all' ? scopeFilter : (projectPath ? 'project' : 'user')
    });
    setJsonError(null);
    setEditDialogOpen(true);
  };

  const handleEdit = (skill: Skill): void => {
    setEditingSkill(skill);
    setSkillForm({
      name: skill.name,
      content: skill.content,
      fileType: skill.file_type === 'json' ? 'json' : 'markdown',
      description: skill.description ?? "",
      allowedTools: skill.allowed_tools,
      scope: skill.scope as 'project' | 'user'
    });
    setJsonError(skill.file_type === 'json' ? validateJson(skill.content) : null);
    setEditDialogOpen(true);
  };

  const handleSave = async (): Promise<void> => {
    // Validate JSON if needed
    if (skillForm.fileType === 'json') {
      const error = validateJson(skillForm.content);
      if (error) {
        setJsonError(error);
        return;
      }
    }

    try {
      setSaving(true);
      setError(null);

      await api.skillSave(
        skillForm.scope,
        skillForm.name,
        skillForm.content,
        skillForm.fileType,
        skillForm.description || undefined,
        skillForm.allowedTools,
        skillForm.scope === 'project' ? projectPath : undefined
      );

      trackEvent.settingsChanged('skill_saved', skillForm.name);

      setEditDialogOpen(false);
      await loadSkills();
    } catch (err) {
      console.error("Failed to save skill:", err);
      setError(err instanceof Error ? err.message : "Failed to save skill");
    } finally {
      setSaving(false);
    }
  };

  const handleDeleteClick = (skill: Skill): void => {
    setSkillToDelete(skill);
    setDeleteDialogOpen(true);
  };

  const confirmDelete = async (): Promise<void> => {
    if (!skillToDelete) {return;}

    try {
      setDeleting(true);
      setError(null);
      await api.skillDelete(skillToDelete.id, projectPath);
      setDeleteDialogOpen(false);
      setSkillToDelete(null);
      await loadSkills();
    } catch (err) {
      console.error("Failed to delete skill:", err);
      const errorMessage = err instanceof Error ? err.message : "Failed to delete skill";
      setError(errorMessage);
    } finally {
      setDeleting(false);
    }
  };

  const cancelDelete = (): void => {
    setDeleteDialogOpen(false);
    setSkillToDelete(null);
  };

  const toggleExpanded = (skillId: string): void => {
    setExpandedSkills(prev => {
      const next = new Set(prev);
      if (next.has(skillId)) {
        next.delete(skillId);
      } else {
        next.add(skillId);
      }
      return next;
    });
  };

  const handleToolToggle = (tool: string): void => {
    setSkillForm(prev => ({
      ...prev,
      allowedTools: prev.allowedTools.includes(tool)
        ? prev.allowedTools.filter(t => t !== tool)
        : [...prev.allowedTools, tool]
    }));
  };

  const handleContentChange = useCallback((value: string | undefined): void => {
    const newContent = value ?? "";
    setSkillForm(prev => ({ ...prev, content: newContent }));

    // Validate JSON in real-time
    if (skillForm.fileType === 'json') {
      setJsonError(validateJson(newContent));
    }
  }, [skillForm.fileType]);

  const handleFormatJson = (): void => {
    if (skillForm.fileType === 'json') {
      const formatted = formatJson(skillForm.content);
      setSkillForm(prev => ({ ...prev, content: formatted }));
      setJsonError(validateJson(formatted));
    }
  };

  // Filter skills
  const filteredSkills = skills.filter(skill => {
    // Apply scopeFilter if set to specific scope
    if (scopeFilter !== 'all' && skill.scope !== scopeFilter) {
      return false;
    }

    // Scope filter
    if (selectedScope !== 'all' && skill.scope !== selectedScope) {
      return false;
    }

    // Search filter
    if (searchQuery) {
      const query = searchQuery.toLowerCase();
      return (
        skill.name.toLowerCase().includes(query) ||
        (skill.description?.toLowerCase().includes(query) ?? false)
      );
    }

    return true;
  });

  // Group skills by scope
  const groupedSkills = filteredSkills.reduce<Record<string, Skill[]>>((acc, skill) => {
    const key = skill.scope === 'project' ? 'Project Skills' : 'User Skills';
    if (!acc[key]) {
      acc[key] = [];
    }
    acc[key].push(skill);
    return acc;
  }, {});

  return (
    <div className={cn("space-y-4", className)}>
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-semibold">
            {scopeFilter === 'project' ? 'Project Skills' : 'Claude Skills'}
          </h3>
          <p className="text-sm text-muted-foreground mt-1">
            {scopeFilter === 'project'
              ? 'Create custom skills for this project'
              : 'Create custom skills to extend Claude\'s capabilities'}
          </p>
        </div>
        <Button onClick={handleCreateNew} size="sm" className="gap-2">
          <Plus className="h-4 w-4" />
          New Skill
        </Button>
      </div>

      {/* Filters */}
      <div className="flex items-center gap-4">
        <div className="flex-1">
          <div className="relative">
            <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
            <Input
              placeholder="Search skills..."
              value={searchQuery}
              onChange={(e) => { setSearchQuery(e.target.value); }}
              className="pl-9"
            />
          </div>
        </div>
        {scopeFilter === 'all' && (
          <Select
            value={selectedScope}
            onValueChange={(value: 'all' | 'project' | 'user') => { setSelectedScope(value); }}
          >
            <SelectTrigger className="w-[150px]">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">All Skills</SelectItem>
              <SelectItem value="project">Project</SelectItem>
              <SelectItem value="user">User</SelectItem>
            </SelectContent>
          </Select>
        )}
      </div>

      {/* Error Message */}
      {error && (
        <div className="flex items-center gap-2 p-3 rounded-lg bg-destructive/10 text-destructive">
          <AlertCircle className="h-4 w-4" />
          <span className="text-sm">{error}</span>
        </div>
      )}

      {/* Skills List */}
      {loading ? (
        <div className="flex items-center justify-center py-8">
          <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
        </div>
      ) : filteredSkills.length === 0 ? (
        <Card className="p-8">
          <div className="text-center">
            <Sparkles className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
            <p className="text-sm text-muted-foreground">
              {searchQuery
                ? "No skills found"
                : scopeFilter === 'project'
                  ? "No project skills created yet"
                  : "No skills created yet"}
            </p>
            {!searchQuery && (
              <Button onClick={handleCreateNew} variant="outline" size="sm" className="mt-4">
                {scopeFilter === 'project'
                  ? "Create your first project skill"
                  : "Create your first skill"}
              </Button>
            )}
          </div>
        </Card>
      ) : (
        <div className="space-y-4">
          {Object.entries(groupedSkills).map(([groupKey, groupSkills]) => (
            <Card key={groupKey} className="overflow-hidden">
              <div className="p-4 bg-muted/50 border-b">
                <h4 className="text-sm font-medium">
                  {groupKey}
                </h4>
              </div>

              <div className="divide-y">
                {groupSkills.map((skill) => {
                  const Icon = getSkillIcon(skill);
                  const isExpanded = expandedSkills.has(skill.id);

                  return (
                    <div key={skill.id}>
                      <div className="p-4">
                        <div className="flex items-start gap-4">
                          <Icon className="h-5 w-5 mt-0.5 text-muted-foreground flex-shrink-0" />

                          <div className="flex-1 min-w-0">
                            <div className="flex items-center gap-2 mb-1">
                              <span className="text-sm font-medium">
                                {skill.name}
                              </span>
                              <Badge variant="secondary" className="text-xs">
                                {skill.file_type}
                              </Badge>
                            </div>

                            {skill.description && (
                              <p className="text-sm text-muted-foreground mb-2">
                                {skill.description}
                              </p>
                            )}

                            <div className="flex items-center gap-4 text-xs">
                              {skill.allowed_tools.length > 0 && (
                                <span className="text-muted-foreground">
                                  {skill.allowed_tools.length} tool{skill.allowed_tools.length === 1 ? '' : 's'}
                                </span>
                              )}

                              {skill.supporting_files.length > 0 && (
                                <Badge variant="outline" className="text-xs">
                                  {skill.supporting_files.length} file{skill.supporting_files.length === 1 ? '' : 's'}
                                </Badge>
                              )}

                              <button
                                type="button"
                                onClick={() => { toggleExpanded(skill.id); }}
                                className="flex items-center gap-1 text-muted-foreground hover:text-foreground transition-colors"
                              >
                                {isExpanded ? (
                                  <>
                                    <ChevronDown className="h-3 w-3" />
                                    Hide content
                                  </>
                                ) : (
                                  <>
                                    <ChevronRight className="h-3 w-3" />
                                    Show content
                                  </>
                                )}
                              </button>
                            </div>
                          </div>

                          <div className="flex items-center gap-2">
                            <Button
                              variant="ghost"
                              size="icon"
                              onClick={() => { handleEdit(skill); }}
                              className="h-8 w-8"
                            >
                              <Edit className="h-4 w-4" />
                            </Button>
                            <Button
                              variant="ghost"
                              size="icon"
                              onClick={() => { handleDeleteClick(skill); }}
                              className="h-8 w-8 text-destructive hover:text-destructive"
                            >
                              <Trash2 className="h-4 w-4" />
                            </Button>
                          </div>
                        </div>

                        <AnimatePresence>
                          {isExpanded && (
                            <motion.div
                              initial={{ height: 0, opacity: 0 }}
                              animate={{ height: "auto", opacity: 1 }}
                              exit={{ height: 0, opacity: 0 }}
                              transition={{ duration: 0.2 }}
                              className="overflow-hidden"
                            >
                              <div className="mt-4 p-3 bg-muted/50 rounded-md">
                                <pre className="text-xs font-mono whitespace-pre-wrap max-h-64 overflow-y-auto">
                                  {skill.content}
                                </pre>
                              </div>

                              {skill.supporting_files.length > 0 && (
                                <div className="mt-3">
                                  <p className="text-xs font-medium mb-2">Supporting Files:</p>
                                  <div className="flex flex-wrap gap-2">
                                    {skill.supporting_files.map((file: SupportingFile) => (
                                      <Badge key={file.path} variant="outline" className="text-xs">
                                        {file.name}
                                      </Badge>
                                    ))}
                                  </div>
                                </div>
                              )}
                            </motion.div>
                          )}
                        </AnimatePresence>
                      </div>
                    </div>
                  );
                })}
              </div>
            </Card>
          ))}
        </div>
      )}

      {/* Edit Dialog */}
      <Dialog open={editDialogOpen} onOpenChange={setEditDialogOpen}>
        <DialogContent className="max-w-5xl max-h-[90vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>
              {editingSkill ? "Edit Skill" : "Create New Skill"}
            </DialogTitle>
          </DialogHeader>

          <div className="space-y-4 py-4">
            {/* Scope and File Type */}
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label>Scope</Label>
                <Select
                  value={skillForm.scope}
                  onValueChange={(value: 'project' | 'user') => { setSkillForm(prev => ({ ...prev, scope: value })); }}
                  disabled={scopeFilter !== 'all' || Boolean(editingSkill) || (!projectPath && skillForm.scope === 'project')}
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {(scopeFilter === 'all' || scopeFilter === 'user') && (
                      <SelectItem value="user">
                        <div className="flex items-center gap-2">
                          <Globe className="h-4 w-4" />
                          User (Global)
                        </div>
                      </SelectItem>
                    )}
                    {(scopeFilter === 'all' || scopeFilter === 'project') && (
                      <SelectItem value="project" disabled={!projectPath}>
                        <div className="flex items-center gap-2">
                          <FolderOpen className="h-4 w-4" />
                          Project
                        </div>
                      </SelectItem>
                    )}
                  </SelectContent>
                </Select>
              </div>

              <div className="space-y-2">
                <Label>File Type</Label>
                <Select
                  value={skillForm.fileType}
                  onValueChange={(value: 'markdown' | 'json') => {
                    setSkillForm(prev => ({ ...prev, fileType: value }));
                    if (value === 'json') {
                      setJsonError(validateJson(skillForm.content));
                    } else {
                      setJsonError(null);
                    }
                  }}
                  disabled={Boolean(editingSkill)}
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="markdown">
                      <div className="flex items-center gap-2">
                        <FileCode className="h-4 w-4" />
                        Markdown
                      </div>
                    </SelectItem>
                    <SelectItem value="json">
                      <div className="flex items-center gap-2">
                        <FileJson className="h-4 w-4" />
                        JSON
                      </div>
                    </SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>

            {/* Name */}
            <div className="space-y-2">
              <Label>Skill Name*</Label>
              <Input
                placeholder="e.g., code-review, api-generator"
                value={skillForm.name}
                onChange={(e) => { setSkillForm(prev => ({ ...prev, name: e.target.value })); }}
                disabled={Boolean(editingSkill)}
              />
              <p className="text-xs text-muted-foreground">
                This will be the folder name for the skill
              </p>
            </div>

            {/* Description */}
            <div className="space-y-2">
              <Label>Description (Optional)</Label>
              <Input
                placeholder="Brief description of what this skill does"
                value={skillForm.description}
                onChange={(e) => { setSkillForm(prev => ({ ...prev, description: e.target.value })); }}
              />
            </div>

            {/* Content */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <Label>Skill Content*</Label>
                {skillForm.fileType === 'json' && (
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    onClick={handleFormatJson}
                    className="gap-2"
                  >
                    <Code className="h-3 w-3" />
                    Format JSON
                  </Button>
                )}
              </div>

              {skillForm.fileType === 'markdown' ? (
                <div data-color-mode={theme === 'light' ? 'light' : 'dark'}>
                  <MDEditor
                    value={skillForm.content}
                    onChange={handleContentChange}
                    height={300}
                    preview="edit"
                  />
                </div>
              ) : (
                <div className="space-y-2">
                  <textarea
                    value={skillForm.content}
                    onChange={(e) => { handleContentChange(e.target.value); }}
                    className={cn(
                      "w-full h-64 p-3 rounded-md border font-mono text-sm resize-none",
                      "bg-background focus:outline-none focus:ring-2 focus:ring-ring",
                      jsonError ? "border-destructive focus:ring-destructive" : "border-input"
                    )}
                    placeholder='{\n  "key": "value"\n}'
                  />
                  {jsonError && (
                    <div className="flex items-center gap-2 text-destructive text-sm">
                      <AlertCircle className="h-4 w-4" />
                      <span>
                        {jsonError.line
                          ? `Line ${jsonError.line}: ${jsonError.message}`
                          : jsonError.message}
                      </span>
                    </div>
                  )}
                </div>
              )}
            </div>

            {/* Allowed Tools */}
            {skillForm.fileType === 'markdown' && (
              <div className="space-y-2">
                <Label>Allowed Tools</Label>
                <div className="flex flex-wrap gap-2">
                  {COMMON_TOOL_MATCHERS.map((tool) => (
                    <Button
                      key={tool}
                      variant={skillForm.allowedTools.includes(tool) ? "default" : "outline"}
                      size="sm"
                      onClick={() => { handleToolToggle(tool); }}
                      type="button"
                    >
                      {tool}
                    </Button>
                  ))}
                </div>
                <p className="text-xs text-muted-foreground">
                  Select which tools Claude can use with this skill
                </p>
              </div>
            )}

            {/* Location Preview */}
            {skillForm.name && (
              <div className="space-y-2">
                <Label>Location Preview</Label>
                <div className="p-3 bg-muted rounded-md">
                  <code className="text-sm">
                    {skillForm.scope === 'user'
                      ? `~/.claude/skills/${skillForm.name}/SKILL.${skillForm.fileType === 'json' ? 'json' : 'md'}`
                      : `.claude/skills/${skillForm.name}/SKILL.${skillForm.fileType === 'json' ? 'json' : 'md'}`
                    }
                  </code>
                </div>
              </div>
            )}
          </div>

          <DialogFooter>
            <Button variant="outline" onClick={() => { setEditDialogOpen(false); }}>
              Cancel
            </Button>
            <Button
              onClick={() => void handleSave()}
              disabled={
                !skillForm.name ||
                !skillForm.content ||
                saving ||
                (skillForm.fileType === 'json' && jsonError !== null)
              }
            >
              {saving ? (
                <>
                  <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                  Saving...
                </>
              ) : (
                <>
                  <Save className="h-4 w-4 mr-2" />
                  Save
                </>
              )}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Delete Confirmation Dialog */}
      <Dialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle>Delete Skill</DialogTitle>
          </DialogHeader>

          <div className="space-y-4 py-4">
            <p>Are you sure you want to delete this skill?</p>
            {skillToDelete && (
              <div className="p-3 bg-muted rounded-md">
                <div className="flex items-center gap-2">
                  <Sparkles className="h-4 w-4" />
                  <span className="text-sm font-medium">{skillToDelete.name}</span>
                </div>
                {skillToDelete.description && (
                  <p className="text-sm text-muted-foreground mt-1">{skillToDelete.description}</p>
                )}
              </div>
            )}
            <p className="text-sm text-muted-foreground">
              This action cannot be undone. The skill folder and all its files will be permanently deleted.
            </p>
          </div>

          <DialogFooter>
            <Button variant="outline" onClick={cancelDelete} disabled={deleting}>
              Cancel
            </Button>
            <Button
              variant="destructive"
              onClick={() => void confirmDelete()}
              disabled={deleting}
            >
              {deleting ? (
                <>
                  <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                  Deleting...
                </>
              ) : (
                <>
                  <Trash2 className="h-4 w-4 mr-2" />
                  Delete
                </>
              )}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
};
