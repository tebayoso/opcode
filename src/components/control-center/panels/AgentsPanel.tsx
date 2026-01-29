import React, { useEffect, useState } from 'react';
import { Bot, Play, Edit, Copy, Plus, ChevronRight, Loader2 } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { api } from '@/lib/api';
import type { Agent } from '@/lib/api';
import { useFileOpsStore } from '@/stores/fileOpsStore';
import { useTabState } from '@/hooks/useTabState';

interface AgentItemProps {
  agent: Agent;
  onRun: (agent: Agent) => void;
  onEdit: (agent: Agent) => void;
  onClone: (agent: Agent) => void;
}

const AgentItem: React.FC<AgentItemProps> = ({ agent, onRun, onEdit, onClone }) => (
    <div className="flex items-center justify-between p-3 rounded-lg border border-border/50 bg-card hover:bg-accent/5 transition-colors group">
      <div className="flex items-center gap-3 min-w-0">
        <div className="p-2 rounded-md bg-primary/10 shrink-0">
          <Bot className="w-4 h-4 text-primary" />
        </div>
        <div className="min-w-0">
          <span className="text-sm font-medium truncate block">{agent.name}</span>
          {agent.default_task && (
            <span className="text-xs text-muted-foreground truncate block">
              {agent.default_task}
            </span>
          )}
        </div>
      </div>

      <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
        <Button
          variant="ghost"
          size="icon"
          className="h-7 w-7"
          onClick={() => onRun(agent)}
          title="Run Agent"
        >
          <Play className="w-3.5 h-3.5" />
        </Button>
        <Button
          variant="ghost"
          size="icon"
          className="h-7 w-7"
          onClick={() => onEdit(agent)}
          title="Edit Agent"
        >
          <Edit className="w-3.5 h-3.5" />
        </Button>
        <Button
          variant="ghost"
          size="icon"
          className="h-7 w-7"
          onClick={() => onClone(agent)}
          title="Clone Agent"
        >
          <Copy className="w-3.5 h-3.5" />
        </Button>
      </div>
    </div>
  );

interface AgentsPanelProps {
  projectPath?: string;
}

export const AgentsPanel: React.FC<AgentsPanelProps> = ({ projectPath }) => {
  const [agents, setAgents] = useState<Agent[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const { cloneAgent } = useFileOpsStore();
  const { createAgentExecutionTab, createCreateAgentTab } = useTabState();

  useEffect(() => {
    let isMounted = true;

    const fetchAgents = async () => {
      setIsLoading(true);
      setError(null);

      try {
        const fetchedAgents = await api.listAgents();
        if (isMounted) {
          setAgents(fetchedAgents);
        }
      } catch (err) {
        if (isMounted) {
          setError(err instanceof Error ? err.message : 'Failed to load agents');
        }
      } finally {
        if (isMounted) {
          setIsLoading(false);
        }
      }
    };

    fetchAgents();

    return () => {
      isMounted = false;
    };
  }, []);

  const handleRunAgent = (agent: Agent) => {
    createAgentExecutionTab(agent, `agent-${agent.id}`, projectPath);
  };

  const handleEditAgent = (agent: Agent) => {
    // Navigate to agent edit view
    // This would typically open an edit tab or modal
    console.log('Edit agent:', agent.id);
  };

  const handleCloneAgent = async (agent: Agent) => {
    if (agent.id === undefined) {return;}
    const newName = `${agent.name} (Copy)`;
    try {
      await cloneAgent(agent.id, newName);
      // Refresh agents list
      const fetchedAgents = await api.listAgents();
      setAgents(fetchedAgents);
    } catch (err) {
      console.error('Failed to clone agent:', err);
    }
  };

  const handleCreateAgent = () => {
    createCreateAgentTab();
  };

  return (
    <Card className="flex-1 min-h-0 flex flex-col">
      <CardHeader className="pb-2 pt-4 px-4 shrink-0">
        <div className="flex items-center justify-between">
          <CardTitle className="text-sm font-medium flex items-center gap-2">
            <Bot className="w-4 h-4 text-primary" />
            Agents
          </CardTitle>
          <div className="flex items-center gap-2">
            <Badge variant="secondary" className="text-xs">
              {agents.length}
            </Badge>
            <Button
              variant="ghost"
              size="icon"
              className="h-6 w-6"
              onClick={handleCreateAgent}
              title="Create New Agent"
            >
              <Plus className="w-4 h-4" />
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
        ) : agents.length > 0 ? (
          <div className="space-y-2">
            {agents.map((agent) => (
              <AgentItem
                key={agent.id}
                agent={agent}
                onRun={handleRunAgent}
                onEdit={handleEditAgent}
                onClone={handleCloneAgent}
              />
            ))}
          </div>
        ) : (
          <div className="flex flex-col items-center justify-center py-8 text-center">
            <Bot className="w-8 h-8 text-muted-foreground/50 mb-2" />
            <p className="text-sm text-muted-foreground">No agents created yet</p>
            <Button
              variant="link"
              size="sm"
              className="mt-2"
              onClick={handleCreateAgent}
            >
              <Plus className="w-3 h-3 mr-1" />
              Create your first agent
            </Button>
          </div>
        )}

        {agents.length > 0 && (
          <button className="w-full mt-3 flex items-center justify-center gap-1 text-xs text-muted-foreground hover:text-foreground transition-colors py-1">
            View All Agents
            <ChevronRight className="w-3 h-3" />
          </button>
        )}
      </CardContent>
    </Card>
  );
};

export default AgentsPanel;
