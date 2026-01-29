import React, { useEffect, useState } from 'react';
import { Sparkles, Edit, Copy, ChevronRight, Loader2, FolderOpen } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { api } from '@/lib/api';
import type { Skill } from '@/lib/api';
import { useFileOpsStore } from '@/stores/fileOpsStore';

interface SkillItemProps {
  skill: Skill;
  onEdit: (skill: Skill) => void;
  onClone: (skill: Skill) => void;
}

const SkillItem: React.FC<SkillItemProps> = ({ skill, onEdit, onClone }) => (
    <div className="flex items-center justify-between p-3 rounded-lg border border-border/50 bg-card hover:bg-accent/5 transition-colors group">
      <div className="flex items-center gap-3 min-w-0">
        <div className="p-2 rounded-md bg-amber-500/10 shrink-0">
          <Sparkles className="w-4 h-4 text-amber-500" />
        </div>
        <div className="min-w-0">
          <span className="text-sm font-medium truncate block">{skill.name}</span>
          {skill.description && (
            <span className="text-xs text-muted-foreground truncate block">
              {skill.description}
            </span>
          )}
        </div>
      </div>

      <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
        <Button
          variant="ghost"
          size="icon"
          className="h-7 w-7"
          onClick={() => onEdit(skill)}
          title="Edit Skill"
        >
          <Edit className="w-3.5 h-3.5" />
        </Button>
        <Button
          variant="ghost"
          size="icon"
          className="h-7 w-7"
          onClick={() => onClone(skill)}
          title="Clone Skill"
        >
          <Copy className="w-3.5 h-3.5" />
        </Button>
      </div>
    </div>
  );

interface SkillsPanelProps {
  projectPath?: string;
}

export const SkillsPanel: React.FC<SkillsPanelProps> = ({ projectPath }) => {
  const [skills, setSkills] = useState<Skill[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const { cloneSkill } = useFileOpsStore();

  useEffect(() => {
    let isMounted = true;

    const fetchSkills = async () => {
      setIsLoading(true);
      setError(null);

      try {
        const fetchedSkills = await api.skillsList(projectPath);
        if (isMounted) {
          setSkills(fetchedSkills);
        }
      } catch (err) {
        if (isMounted) {
          setError(err instanceof Error ? err.message : 'Failed to load skills');
        }
      } finally {
        if (isMounted) {
          setIsLoading(false);
        }
      }
    };

    fetchSkills();

    return () => {
      isMounted = false;
    };
  }, [projectPath]);

  const handleEditSkill = (skill: Skill) => {
    // Navigate to skill edit view or open editor
    console.log('Edit skill:', skill.name);
  };

  const handleCloneSkill = async (skill: Skill) => {
    const newName = `${skill.name}-copy`;
    try {
      await cloneSkill(skill.name, newName);
      // Refresh skills list
      const fetchedSkills = await api.skillsList(projectPath);
      setSkills(fetchedSkills);
    } catch (err) {
      console.error('Failed to clone skill:', err);
    }
  };

  const handleOpenSkillsFolder = async () => {
    // Open skills folder in file explorer
    // This would use Tauri's shell open command
    console.log('Open skills folder');
  };

  return (
    <Card className="flex-1 min-h-0 flex flex-col">
      <CardHeader className="pb-2 pt-4 px-4 shrink-0">
        <div className="flex items-center justify-between">
          <CardTitle className="text-sm font-medium flex items-center gap-2">
            <Sparkles className="w-4 h-4 text-amber-500" />
            Skills
          </CardTitle>
          <div className="flex items-center gap-2">
            <Badge variant="secondary" className="text-xs">
              {skills.length}
            </Badge>
            <Button
              variant="ghost"
              size="icon"
              className="h-6 w-6"
              onClick={handleOpenSkillsFolder}
              title="Open Skills Folder"
            >
              <FolderOpen className="w-4 h-4" />
            </Button>
          </div>
        </div>
      </CardHeader>

      <CardContent className="px-4 pb-4 flex-1 overflow-y-auto">
        {isLoading ? (
          <div className="flex items-center justify-center py-8">
            <Loader2 className="w-6 h-6 animate-spin text-muted-foreground" />
          </div>
        ) : error ? (
          <div className="flex flex-col items-center justify-center py-8 text-center">
            <p className="text-sm text-destructive">{error}</p>
          </div>
        ) : skills.length > 0 ? (
          <div className="space-y-2">
            {skills.map((skill) => (
              <SkillItem
                key={skill.name}
                skill={skill}
                onEdit={handleEditSkill}
                onClone={handleCloneSkill}
              />
            ))}
          </div>
        ) : (
          <div className="flex flex-col items-center justify-center py-8 text-center">
            <Sparkles className="w-8 h-8 text-muted-foreground/50 mb-2" />
            <p className="text-sm text-muted-foreground">No skills found</p>
            <p className="text-xs text-muted-foreground/70 mt-1">
              Skills are loaded from your project&apos;s .claude/skills folder
            </p>
          </div>
        )}

        {skills.length > 0 && (
          <button className="w-full mt-3 flex items-center justify-center gap-1 text-xs text-muted-foreground hover:text-foreground transition-colors py-1">
            View All Skills
            <ChevronRight className="w-3 h-3" />
          </button>
        )}
      </CardContent>
    </Card>
  );
};

export default SkillsPanel;
