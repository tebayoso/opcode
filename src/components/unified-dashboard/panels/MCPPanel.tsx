import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { 
  Server, 
  Plus, 
  Trash2, 
  RefreshCw,
  CheckCircle2,
  XCircle,
  Loader2
} from 'lucide-react';

interface MCPServer {
  id: string;
  name: string;
  transport_type: string;
  config: {
    command?: string;
    args?: string[];
    env?: Record<string, string>;
    url?: string;
    headers?: Record<string, string>;
  };
  is_enabled_globally: boolean;
}

export function MCPPanel() {
  const [servers, setServers] = useState<MCPServer[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [testingServer, setTestingServer] = useState<string | null>(null);

  useEffect(() => {
    loadServers();
  }, []);

  const loadServers = async () => {
    try {
      setIsLoading(true);
      const response = await invoke<{ data: MCPServer[] }>('mcp_registry_list_servers');
      setServers(response.data);
    } catch (error) {
      console.error('Failed to load MCP servers:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleTestConnection = async (serverId: string) => {
    try {
      setTestingServer(serverId);
      const response = await invoke<{ data: boolean }>('mcp_registry_test_connection', { serverId });
      alert(response.data ? 'Connection successful!' : 'Connection failed');
    } catch (error) {
      console.error('Failed to test connection:', error);
      alert('Connection test failed');
    } finally {
      setTestingServer(null);
    }
  };

  const handleRemoveServer = async (serverId: string) => {
    try {
      await invoke('mcp_registry_remove_server', { serverId });
      loadServers();
    } catch (error) {
      console.error('Failed to remove server:', error);
    }
  };

  return (
    <div className="p-6 space-y-6">
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-bold text-foreground">MCP Servers</h1>
          <p className="text-muted-foreground mt-1">
            Manage Model Context Protocol servers
          </p>
        </div>
        <Button onClick={loadServers} variant="outline">
          <RefreshCw className="w-4 h-4 mr-2" />
          Refresh
        </Button>
      </div>

      {isLoading ? (
        <div className="flex items-center justify-center h-64">
          <Loader2 className="w-8 h-8 animate-spin text-primary" />
        </div>
      ) : servers.length === 0 ? (
        <Card>
          <CardContent className="flex flex-col items-center justify-center py-12">
            <Server className="w-12 h-12 text-muted-foreground mb-4" />
            <p className="text-muted-foreground text-center">
              No MCP servers configured yet
            </p>
            <Button className="mt-4">
              <Plus className="w-4 h-4 mr-2" />
              Add Server
            </Button>
          </CardContent>
        </Card>
      ) : (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
          {servers.map((server) => (
            <Card key={server.id}>
              <CardHeader>
                <div className="flex justify-between items-start">
                  <div>
                    <CardTitle className="flex items-center gap-2">
                      <Server className="w-5 h-5" />
                      {server.name}
                    </CardTitle>
                    <CardDescription className="capitalize">
                      {server.transport_type} transport
                    </CardDescription>
                  </div>
                  <Badge 
                    variant={server.is_enabled_globally ? "default" : "secondary"}
                  >
                    {server.is_enabled_globally ? (
                      <><CheckCircle2 className="w-3 h-3 mr-1" /> Enabled</>
                    ) : (
                      <><XCircle className="w-3 h-3 mr-1" /> Disabled</>
                    )}
                  </Badge>
                </div>
              </CardHeader>
              <CardContent>
                <div className="space-y-2 text-sm">
                  {server.config.command && (
                    <div>
                      <span className="text-muted-foreground">Command: </span>
                      <code className="bg-muted px-1 rounded">{server.config.command}</code>
                    </div>
                  )}
                  {server.config.url && (
                    <div>
                      <span className="text-muted-foreground">URL: </span>
                      <code className="bg-muted px-1 rounded">{server.config.url}</code>
                    </div>
                  )}
                  {server.config.args && server.config.args.length > 0 && (
                    <div>
                      <span className="text-muted-foreground">Args: </span>
                      <code className="bg-muted px-1 rounded">{server.config.args.join(' ')}</code>
                    </div>
                  )}
                </div>

                <div className="flex gap-2 mt-4">
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={() => handleTestConnection(server.id)}
                    disabled={testingServer === server.id}
                  >
                    {testingServer === server.id ? (
                      <><Loader2 className="w-4 h-4 mr-2 animate-spin" />Testing...</>
                    ) : (
                      'Test Connection'
                    )}
                  </Button>
                  <Button
                    size="sm"
                    variant="destructive"
                    onClick={() => handleRemoveServer(server.id)}
                  >
                    <Trash2 className="w-4 h-4 mr-2" />
                    Remove
                  </Button>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
}
