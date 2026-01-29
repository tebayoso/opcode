import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Plus, Loader2 } from 'lucide-react';

interface RegisterToolDialogProps {
  onToolRegistered: () => void;
}

export function RegisterToolDialog({ onToolRegistered }: RegisterToolDialogProps) {
  const [open, setOpen] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [formData, setFormData] = useState({
    id: '',
    name: '',
    description: '',
    tool_type: 'cli',
    binary_name: '',
    base_dir: '',
  });

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsSubmitting(true);

    try {
      const spec = {
        id: formData.id,
        name: formData.name,
        tool_type: formData.tool_type,
        source: 'user_defined',
        version: '1.0.0',
        description: formData.description || null,
        website: null,
        icon: null,
        installation: {
          binary_names: formData.binary_name ? [formData.binary_name] : null,
          homebrew_formulas: null,
          npm_packages: null,
          nvm_packages: null,
          gh_extensions: null,
          standard_paths: null,
          version_args: ['--version'],
          version_pattern: '(\\d+\\.\\d+\\.\\d+)',
        },
        config: {
          base_dir: formData.base_dir || '~/.config/' + formData.id,
          files: null,
          settings_format: null,
          settings_path: null,
        },
        capabilities: {
          files: true,
          settings: true,
          mcp_servers: false,
          agents: false,
          commands: true,
          usage: true,
        },
        settings_schema: null,
      };

      await invoke('tool_registry_register_tool', { spec });
      setOpen(false);
      onToolRegistered();
      setFormData({
        id: '',
        name: '',
        description: '',
        tool_type: 'cli',
        binary_name: '',
        base_dir: '',
      });
    } catch (error) {
      console.error('Failed to register tool:', error);
      alert('Failed to register tool: ' + error);
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button>
          <Plus className="w-4 h-4 mr-2" />
          Register Tool
        </Button>
      </DialogTrigger>
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Register Custom Tool</DialogTitle>
          <DialogDescription>
            Add a new tool to the unified control panel. Fill in the basic information below.
          </DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-6 mt-4">
          <div className="space-y-2">
            <Label htmlFor="id">Tool ID *</Label>
            <Input
              id="id"
              placeholder="my-custom-tool"
              value={formData.id}
              onChange={(e) => setFormData({ ...formData, id: e.target.value })}
              required
              pattern="[a-z0-9_-]+"
              title="Only lowercase letters, numbers, hyphens, and underscores allowed"
            />
            <p className="text-xs text-muted-foreground">
              Unique identifier (lowercase, no spaces)
            </p>
          </div>

          <div className="space-y-2">
            <Label htmlFor="name">Tool Name *</Label>
            <Input
              id="name"
              placeholder="My Custom Tool"
              value={formData.name}
              onChange={(e) => setFormData({ ...formData, name: e.target.value })}
              required
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="tool_type">Tool Type *</Label>
            <Select
              value={formData.tool_type}
              onValueChange={(value) => setFormData({ ...formData, tool_type: value })}
            >
              <SelectTrigger>
                <SelectValue placeholder="Select tool type" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="llm">LLM (AI Assistant)</SelectItem>
                <SelectItem value="cli">CLI Tool</SelectItem>
                <SelectItem value="codec">Codec (Formatter/Linter)</SelectItem>
                <SelectItem value="service">Service</SelectItem>
                <SelectItem value="plugin">Plugin</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div className="space-y-2">
            <Label htmlFor="description">Description</Label>
            <Textarea
              id="description"
              placeholder="Brief description of what this tool does..."
              value={formData.description}
              onChange={(e) => setFormData({ ...formData, description: e.target.value })}
              rows={3}
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="binary_name">Binary Name</Label>
            <Input
              id="binary_name"
              placeholder="my-tool"
              value={formData.binary_name}
              onChange={(e) => setFormData({ ...formData, binary_name: e.target.value })}
            />
            <p className="text-xs text-muted-foreground">
              Command name to execute the tool
            </p>
          </div>

          <div className="space-y-2">
            <Label htmlFor="base_dir">Config Directory</Label>
            <Input
              id="base_dir"
              placeholder="~/.config/my-tool"
              value={formData.base_dir}
              onChange={(e) => setFormData({ ...formData, base_dir: e.target.value })}
            />
            <p className="text-xs text-muted-foreground">
              Where the tool stores its configuration files
            </p>
          </div>

          <div className="flex justify-end gap-2">
            <Button type="button" variant="outline" onClick={() => setOpen(false)}>
              Cancel
            </Button>
            <Button type="submit" disabled={isSubmitting}>
              {isSubmitting ? (
                <><Loader2 className="w-4 h-4 mr-2 animate-spin" />Registering...</>
              ) : (
                'Register Tool'
              )}
            </Button>
          </div>
        </form>
      </DialogContent>
    </Dialog>
  );
}
