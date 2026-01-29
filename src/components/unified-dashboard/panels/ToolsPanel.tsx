import React from 'react';
import { useUnifiedDashboard } from '../UnifiedDashboardContext';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { 
  Terminal, 
  Settings, 
  FileText, 
  Play,
  CheckCircle2,
  AlertCircle,
  Loader2
} from 'lucide-react';

export function ToolsPanel() {
  const { tools, isLoading, refreshTools } = useUnifiedDashboard();

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <Loader2 className="w-8 h-8 animate-spin text-primary" />
      </div>
    );
  }

  const llmTools = tools.filter(t => t.tool_type === 'llm');
  const cliTools = tools.filter(t => t.tool_type === 'cli');
  const codecTools = tools.filter(t => t.tool_type === 'codec');

  const ToolCard = ({ tool }: { tool: typeof tools[0] }) => (
    <Card className="hover:border-primary/50 transition-colors">
      <CardHeader className="pb-3">
        <div className="flex justify-between items-start">
          <div>
            <CardTitle className="text-lg">{tool.name}</CardTitle>
            <CardDescription className="capitalize">
              {tool.tool_type}
            </CardDescription>
          </div>
          <Badge 
            variant={tool.is_installed ? "default" : "secondary"}
            className={tool.is_installed ? "bg-green-600" : ""}
          >
            {tool.is_installed ? (
              <><CheckCircle2 className="w-3 h-3 mr-1" /> Ready</>
            ) : (
              <><AlertCircle className="w-3 h-3 mr-1" /> Not Installed</>
            )}
          </Badge>
        </div>
      </CardHeader>
      <CardContent>
        <div className="flex flex-wrap gap-2 mb-4">
          {tool.capabilities.files && (
            <Badge variant="outline" className="text-xs"><FileText className="w-3 h-3 mr-1" />Files</Badge>
          )}
          {tool.capabilities.settings && (
            <Badge variant="outline" className="text-xs"><Settings className="w-3 h-3 mr-1" />Settings</Badge>
          )}
          {tool.capabilities.mcp_servers && (
            <Badge variant="outline" className="text-xs">MCP</Badge>
          )}
          {tool.capabilities.agents && (
            <Badge variant="outline" className="text-xs">Agents</Badge>
          )}
          {tool.capabilities.commands && (
            <Badge variant="outline" className="text-xs"><Terminal className="w-3 h-3 mr-1" />CLI</Badge>
          )}
          {tool.capabilities.usage && (
            <Badge variant="outline" className="text-xs">Usage</Badge>
          )}
        </div>
        <div className="flex gap-2">
          <Button size="sm" variant="outline" className="flex-1">
            <Settings className="w-4 h-4 mr-2" />
            Configure
          </Button>
          {tool.is_installed && (
            <Button size="sm" className="flex-1">
              <Play className="w-4 h-4 mr-2" />
              Launch
            </Button>
          )}
        </div>
      </CardContent>
    </Card>
  );

  return (
    <div className="p-6 space-y-6">
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-bold text-foreground">Tools</h1>
          <p className="text-muted-foreground mt-1">
            Manage your CLI tools and AI assistants
          </p>
        </div>
        <Button onClick={refreshTools} variant="outline">
          <Loader2 className="w-4 h-4 mr-2" />
          Refresh
        </Button>
      </div>

      {llmTools.length > 0 && (
        <section>
          <h2 className="text-xl font-semibold mb-4">LLM Tools</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {llmTools.map((tool) => (
              <ToolCard key={tool.id} tool={tool} />
            ))}
          </div>
        </section>
      )}

      {cliTools.length > 0 && (
        <section>
          <h2 className="text-xl font-semibold mb-4">CLI Tools</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {cliTools.map((tool) => (
              <ToolCard key={tool.id} tool={tool} />
            ))}
          </div>
        </section>
      )}

      {codecTools.length > 0 && (
        <section>
          <h2 className="text-xl font-semibold mb-4">Codecs</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {codecTools.map((tool) => (
              <ToolCard key={tool.id} tool={tool} />
            ))}
          </div>
        </section>
      )}
    </div>
  );
}
