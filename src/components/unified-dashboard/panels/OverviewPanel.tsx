import React from 'react';
import { useUnifiedDashboard } from '../UnifiedDashboardContext';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { 
  CheckCircle2, 
  AlertCircle, 
  Loader2, 
  Terminal,
  Puzzle,
  Server
} from 'lucide-react';

export function OverviewPanel() {
  const { tools, stats } = useUnifiedDashboard();

  const llmTools = tools.filter(t => t.tool_type === 'llm');
  const cliTools = tools.filter(t => t.tool_type === 'cli');
  const codecTools = tools.filter(t => t.tool_type === 'codec');

  return (
    <div className="p-6 space-y-6">
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-bold text-foreground">Dashboard</h1>
          <p className="text-muted-foreground mt-1">
            Manage your development environment
          </p>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Tools</CardTitle>
            <Terminal className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.total_tools}</div>
            <p className="text-xs text-muted-foreground">
              {stats.installed_tools} installed
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Ready</CardTitle>
            <CheckCircle2 className="h-4 w-4 text-green-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-green-600">
              {stats.installed_tools}
            </div>
            <p className="text-xs text-muted-foreground">
              Tools available
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Warnings</CardTitle>
            <AlertCircle className="h-4 w-4 text-yellow-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-yellow-600">
              {stats.active_warnings}
            </div>
            <p className="text-xs text-muted-foreground">
              Require attention
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Active Jobs</CardTitle>
            <Loader2 className="h-4 w-4 text-blue-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-blue-600">
              {stats.running_jobs}
            </div>
            <p className="text-xs text-muted-foreground">
              Currently running
            </p>
          </CardContent>
        </Card>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Terminal className="w-5 h-5" />
              LLM Tools
            </CardTitle>
            <CardDescription>
              AI-powered coding assistants
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              {llmTools.slice(0, 5).map((tool) => (
                <div key={tool.id} className="flex items-center justify-between">
                  <span className="text-sm">{tool.name}</span>
                  <Badge variant={tool.is_installed ? "default" : "secondary"}>
                    {tool.is_installed ? "Ready" : "Not Installed"}
                  </Badge>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Puzzle className="w-5 h-5" />
              CLI Tools
            </CardTitle>
            <CardDescription>
              Command-line utilities
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              {cliTools.slice(0, 5).map((tool) => (
                <div key={tool.id} className="flex items-center justify-between">
                  <span className="text-sm">{tool.name}</span>
                  <Badge variant={tool.is_installed ? "default" : "secondary"}>
                    {tool.is_installed ? "Ready" : "Not Installed"}
                  </Badge>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Server className="w-5 h-5" />
              Codecs
            </CardTitle>
            <CardDescription>
              Formatters and linters
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              {codecTools.slice(0, 5).map((tool) => (
                <div key={tool.id} className="flex items-center justify-between">
                  <span className="text-sm">{tool.name}</span>
                  <Badge variant={tool.is_installed ? "default" : "secondary"}>
                    {tool.is_installed ? "Ready" : "Not Installed"}
                  </Badge>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
