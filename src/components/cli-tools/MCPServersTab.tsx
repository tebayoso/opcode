/**
 * MCP Servers Tab
 * MCP server management with add/remove/toggle functionality
 */

import { useState } from 'react';
import {
  Server,
  Plus,
  Trash2,
  ChevronDown,
  ChevronRight,
  Terminal,
  Globe,
  Loader2,
  Copy,
  Check,
} from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Badge } from '@/components/ui/badge';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from '@/components/ui/alert-dialog';
import { useCLIToolConfigStore } from '@/stores/cliToolConfigStore';
import type { CLIToolType, CLIToolMCPServerConfig, MCPServerInput } from '@/types/cli-tools';

interface MCPServersTabProps {
  toolType: CLIToolType;
  servers: CLIToolMCPServerConfig[];
}

export function MCPServersTab({ toolType, servers }: MCPServersTabProps) {
  const [expandedServer, setExpandedServer] = useState<string | null>(null);
  const [isAddDialogOpen, setIsAddDialogOpen] = useState(false);
  const [deletingServer, setDeletingServer] = useState<string | null>(null);

  const { addMCPServer, removeMCPServer, loadMCPServers } = useCLIToolConfigStore();

  const handleAddServer = async (input: MCPServerInput) => {
    await addMCPServer(toolType, input);
    setIsAddDialogOpen(false);
  };

  const handleRemoveServer = async (name: string) => {
    setDeletingServer(name);
    try {
      await removeMCPServer(toolType, name);
    } finally {
      setDeletingServer(null);
    }
  };

  const handleRefresh = () => {
    loadMCPServers(toolType);
  };

  const toggleExpand = (name: string) => {
    setExpandedServer(expandedServer === name ? null : name);
  };

  if (servers.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-8 text-muted-foreground">
        <Server className="h-8 w-8 mb-2 opacity-50" />
        <p className="text-sm">No MCP servers configured</p>
        <p className="text-xs mt-1 mb-4">
          Add MCP servers to extend this tool's capabilities
        </p>
        <AddServerDialog
          isOpen={isAddDialogOpen}
          onOpenChange={setIsAddDialogOpen}
          onAdd={handleAddServer}
        />
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <span className="text-xs text-muted-foreground">
          {servers.length} server{servers.length !== 1 ? 's' : ''} configured
        </span>
        <div className="flex gap-2">
          <Button variant="ghost" size="sm" onClick={handleRefresh}>
            <Loader2 className="h-3 w-3" />
          </Button>
          <AddServerDialog
            isOpen={isAddDialogOpen}
            onOpenChange={setIsAddDialogOpen}
            onAdd={handleAddServer}
          />
        </div>
      </div>

      <div className="space-y-2">
        {servers.map((server) => (
          <ServerCard
            key={server.name}
            server={server}
            isExpanded={expandedServer === server.name}
            isDeleting={deletingServer === server.name}
            onToggleExpand={() => toggleExpand(server.name)}
            onRemove={() => handleRemoveServer(server.name)}
          />
        ))}
      </div>
    </div>
  );
}

interface ServerCardProps {
  server: CLIToolMCPServerConfig;
  isExpanded: boolean;
  isDeleting: boolean;
  onToggleExpand: () => void;
  onRemove: () => void;
}

function ServerCard({
  server,
  isExpanded,
  isDeleting,
  onToggleExpand,
  onRemove,
}: ServerCardProps) {
  const [copiedField, setCopiedField] = useState<string | null>(null);

  const handleCopy = async (value: string, field: string) => {
    await navigator.clipboard.writeText(value);
    setCopiedField(field);
    setTimeout(() => setCopiedField(null), 2000);
  };

  const getTransportIcon = () => {
    switch (server.transport.type) {
      case 'stdio':
        return <Terminal className="h-4 w-4" />;
      case 'sse':
      case 'http':
        return <Globe className="h-4 w-4" />;
      default:
        return <Server className="h-4 w-4" />;
    }
  };

  return (
    <div className="border rounded-md overflow-hidden">
      <div className="flex items-center p-3 gap-3">
        <button
          onClick={onToggleExpand}
          className="flex items-center gap-2 flex-1 text-left hover:bg-muted/50 -m-3 p-3 transition-colors"
        >
          {isExpanded ? (
            <ChevronDown className="h-4 w-4 text-muted-foreground" />
          ) : (
            <ChevronRight className="h-4 w-4 text-muted-foreground" />
          )}
          {getTransportIcon()}
          <span className="font-medium text-sm">{server.name}</span>
          <Badge variant={server.enabled ? 'default' : 'secondary'} className="text-[10px]">
            {server.enabled ? 'Enabled' : 'Disabled'}
          </Badge>
          <Badge variant="outline" className="text-[10px] uppercase">
            {server.transport.type}
          </Badge>
        </button>

        <AlertDialog>
          <AlertDialogTrigger asChild>
            <Button
              variant="ghost"
              size="sm"
              className="h-8 w-8 p-0 text-destructive hover:text-destructive"
              disabled={isDeleting}
            >
              {isDeleting ? (
                <Loader2 className="h-4 w-4 animate-spin" />
              ) : (
                <Trash2 className="h-4 w-4" />
              )}
            </Button>
          </AlertDialogTrigger>
          <AlertDialogContent>
            <AlertDialogHeader>
              <AlertDialogTitle>Remove MCP Server</AlertDialogTitle>
              <AlertDialogDescription>
                Are you sure you want to remove "{server.name}"? This action cannot
                be undone.
              </AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter>
              <AlertDialogCancel>Cancel</AlertDialogCancel>
              <AlertDialogAction onClick={onRemove}>Remove</AlertDialogAction>
            </AlertDialogFooter>
          </AlertDialogContent>
        </AlertDialog>
      </div>

      <AnimatePresence>
        {isExpanded && (
          <motion.div
            initial={{ height: 0 }}
            animate={{ height: 'auto' }}
            exit={{ height: 0 }}
            transition={{ duration: 0.15 }}
            className="overflow-hidden"
          >
            <div className="border-t p-3 space-y-3 text-sm">
              {server.description && (
                <p className="text-muted-foreground text-xs">{server.description}</p>
              )}

              <div className="space-y-2">
                <div className="flex justify-between items-center">
                  <span className="text-muted-foreground">Transport</span>
                  <Badge variant="outline">{server.transport.type}</Badge>
                </div>

                {server.transport.type === 'stdio' && (() => {
                  const { command, args } = server.transport;
                  return (
                    <>
                      <div className="flex flex-col gap-1">
                        <span className="text-muted-foreground text-xs">Command</span>
                        <div className="flex items-center gap-2">
                          <code className="flex-1 text-xs bg-muted px-2 py-1 rounded font-mono">
                            {command}
                          </code>
                          <Button
                            variant="ghost"
                            size="sm"
                            className="h-6 w-6 p-0"
                            onClick={() => handleCopy(command, 'command')}
                          >
                            {copiedField === 'command' ? (
                              <Check className="h-3 w-3 text-green-500" />
                            ) : (
                              <Copy className="h-3 w-3" />
                            )}
                          </Button>
                        </div>
                      </div>

                      {args.length > 0 && (
                        <div className="flex flex-col gap-1">
                          <span className="text-muted-foreground text-xs">Arguments</span>
                          <code className="text-xs bg-muted px-2 py-1 rounded font-mono">
                            {args.join(' ')}
                          </code>
                        </div>
                      )}
                    </>
                  );
                })()}

                {(server.transport.type === 'sse' || server.transport.type === 'http') && (() => {
                  const { url } = server.transport;
                  return (
                    <div className="flex flex-col gap-1">
                      <span className="text-muted-foreground text-xs">URL</span>
                      <div className="flex items-center gap-2">
                        <code className="flex-1 text-xs bg-muted px-2 py-1 rounded font-mono truncate">
                          {url}
                        </code>
                        <Button
                          variant="ghost"
                          size="sm"
                          className="h-6 w-6 p-0"
                          onClick={() => handleCopy(url, 'url')}
                        >
                          {copiedField === 'url' ? (
                            <Check className="h-3 w-3 text-green-500" />
                          ) : (
                            <Copy className="h-3 w-3" />
                          )}
                        </Button>
                      </div>
                    </div>
                  );
                })()}

                {Object.keys(server.env).length > 0 && (
                  <div className="flex flex-col gap-1">
                    <span className="text-muted-foreground text-xs">
                      Environment Variables
                    </span>
                    <div className="space-y-1">
                      {Object.entries(server.env).map(([key, value]) => (
                        <div
                          key={key}
                          className="flex justify-between items-center text-xs bg-muted px-2 py-1 rounded"
                        >
                          <span className="font-mono">{key}</span>
                          <span className="font-mono text-muted-foreground truncate max-w-[150px]">
                            {value.includes('***') ? value : '***'}
                          </span>
                        </div>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}

interface AddServerDialogProps {
  isOpen: boolean;
  onOpenChange: (open: boolean) => void;
  onAdd: (input: MCPServerInput) => Promise<void>;
}

function AddServerDialog({ isOpen, onOpenChange, onAdd }: AddServerDialogProps) {
  const [name, setName] = useState('');
  const [transportType, setTransportType] = useState<'stdio' | 'sse' | 'http'>('stdio');
  const [command, setCommand] = useState('');
  const [args, setArgs] = useState('');
  const [url, setUrl] = useState('');
  const [description, setDescription] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);

  const resetForm = () => {
    setName('');
    setTransportType('stdio');
    setCommand('');
    setArgs('');
    setUrl('');
    setDescription('');
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsSubmitting(true);

    try {
      const input: MCPServerInput = {
        name,
        transport_type: transportType,
        description: description || undefined,
        enabled: true,
      };

      if (transportType === 'stdio') {
        input.command = command;
        input.args = args.split(/\s+/).filter(Boolean);
      } else {
        input.url = url;
      }

      await onAdd(input);
      resetForm();
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <Dialog open={isOpen} onOpenChange={onOpenChange}>
      <DialogTrigger asChild>
        <Button size="sm">
          <Plus className="h-3 w-3 mr-1" />
          Add Server
        </Button>
      </DialogTrigger>
      <DialogContent>
        <form onSubmit={handleSubmit}>
          <DialogHeader>
            <DialogTitle>Add MCP Server</DialogTitle>
            <DialogDescription>
              Configure a new MCP server for this CLI tool.
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="name">Server Name</Label>
              <Input
                id="name"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="my-mcp-server"
                required
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="transport">Transport Type</Label>
              <Select
                value={transportType}
                onValueChange={(v) => setTransportType(v as 'stdio' | 'sse' | 'http')}
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="stdio">Stdio (Local Process)</SelectItem>
                  <SelectItem value="sse">SSE (Server-Sent Events)</SelectItem>
                  <SelectItem value="http">HTTP</SelectItem>
                </SelectContent>
              </Select>
            </div>

            {transportType === 'stdio' ? (
              <>
                <div className="space-y-2">
                  <Label htmlFor="command">Command</Label>
                  <Input
                    id="command"
                    value={command}
                    onChange={(e) => setCommand(e.target.value)}
                    placeholder="npx -y @modelcontextprotocol/server"
                    required
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="args">Arguments (space-separated)</Label>
                  <Input
                    id="args"
                    value={args}
                    onChange={(e) => setArgs(e.target.value)}
                    placeholder="--port 3000"
                  />
                </div>
              </>
            ) : (
              <div className="space-y-2">
                <Label htmlFor="url">URL</Label>
                <Input
                  id="url"
                  type="url"
                  value={url}
                  onChange={(e) => setUrl(e.target.value)}
                  placeholder="http://localhost:3000/sse"
                  required
                />
              </div>
            )}

            <div className="space-y-2">
              <Label htmlFor="description">Description (optional)</Label>
              <Input
                id="description"
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                placeholder="Brief description of this server"
              />
            </div>
          </div>

          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={() => onOpenChange(false)}
            >
              Cancel
            </Button>
            <Button type="submit" disabled={isSubmitting}>
              {isSubmitting ? (
                <Loader2 className="h-4 w-4 animate-spin mr-2" />
              ) : (
                <Plus className="h-4 w-4 mr-2" />
              )}
              Add Server
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
