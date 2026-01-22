/**
 * Agents Tab
 * Agent browser and viewer for CLI tool agents/commands/rules
 */

import { useState } from 'react';
import {
  Bot,
  ChevronDown,
  ChevronRight,
  FileText,
  Wrench,
  Copy,
  Check,
  RefreshCw,
} from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import { useCLIToolConfigStore } from '@/stores/cliToolConfigStore';
import type { CLIToolType, CLIToolAgentDefinition } from '@/types/cli-tools';

interface AgentsTabProps {
  toolType: CLIToolType;
  agents: CLIToolAgentDefinition[];
}

export function AgentsTab({ toolType, agents }: AgentsTabProps) {
  const [expandedAgent, setExpandedAgent] = useState<string | null>(null);

  const { loadAgents } = useCLIToolConfigStore();

  const handleRefresh = () => {
    loadAgents(toolType);
  };

  const toggleExpand = (name: string) => {
    setExpandedAgent(expandedAgent === name ? null : name);
  };

  // Get terminology based on tool type
  const getTerminology = () => {
    switch (toolType) {
      case 'cursor':
        return { singular: 'Rule', plural: 'Rules' };
      case 'claude':
        return { singular: 'Agent', plural: 'Agents' };
      default:
        return { singular: 'Agent', plural: 'Agents' };
    }
  };

  const { singular, plural } = getTerminology();

  if (agents.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-8 text-muted-foreground">
        <Bot className="h-8 w-8 mb-2 opacity-50" />
        <p className="text-sm">No {plural.toLowerCase()} found</p>
        <p className="text-xs mt-1">
          {singular}s are custom commands or prompts defined for this tool.
        </p>
        <Button variant="ghost" size="sm" onClick={handleRefresh} className="mt-3">
          <RefreshCw className="h-3 w-3 mr-1" />
          Refresh
        </Button>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <span className="text-xs text-muted-foreground">
          {agents.length} {agents.length === 1 ? singular.toLowerCase() : plural.toLowerCase()} found
        </span>
        <Button variant="ghost" size="sm" onClick={handleRefresh}>
          <RefreshCw className="h-3 w-3" />
        </Button>
      </div>

      <div className="space-y-2">
        {agents.map((agent) => (
          <AgentCard
            key={agent.name}
            agent={agent}
            isExpanded={expandedAgent === agent.name}
            onToggleExpand={() => toggleExpand(agent.name)}
            terminology={singular}
          />
        ))}
      </div>
    </div>
  );
}

interface AgentCardProps {
  agent: CLIToolAgentDefinition;
  isExpanded: boolean;
  onToggleExpand: () => void;
  terminology: string;
}

function AgentCard({
  agent,
  isExpanded,
  onToggleExpand,
  terminology: _terminology,
}: AgentCardProps) {
  const [copiedField, setCopiedField] = useState<string | null>(null);

  const handleCopy = async (value: string, field: string) => {
    await navigator.clipboard.writeText(value);
    setCopiedField(field);
    setTimeout(() => setCopiedField(null), 2000);
  };

  return (
    <div className="border rounded-md overflow-hidden">
      <button
        onClick={onToggleExpand}
        className="w-full flex items-center p-3 gap-3 hover:bg-muted/50 transition-colors text-left"
      >
        {isExpanded ? (
          <ChevronDown className="h-4 w-4 text-muted-foreground flex-shrink-0" />
        ) : (
          <ChevronRight className="h-4 w-4 text-muted-foreground flex-shrink-0" />
        )}
        <Bot className="h-4 w-4 flex-shrink-0" />
        <div className="flex-1 min-w-0">
          <span className="font-medium text-sm block truncate">{agent.name}</span>
          {agent.description && (
            <span className="text-xs text-muted-foreground block truncate">
              {agent.description}
            </span>
          )}
        </div>
        {agent.model && (
          <Badge variant="outline" className="text-[10px] flex-shrink-0">
            {agent.model}
          </Badge>
        )}
        {agent.tools.length > 0 && (
          <Badge variant="secondary" className="text-[10px] flex-shrink-0">
            {agent.tools.length} tools
          </Badge>
        )}
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
              {/* Source file */}
              <div className="flex items-center gap-2 text-xs">
                <FileText className="h-3 w-3 text-muted-foreground" />
                <TooltipProvider>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <button
                        onClick={() => handleCopy(agent.source_file, 'source')}
                        className="flex items-center gap-1 text-muted-foreground hover:text-foreground transition-colors"
                      >
                        {copiedField === 'source' ? (
                          <Check className="h-3 w-3 text-green-500" />
                        ) : (
                          <Copy className="h-3 w-3" />
                        )}
                        <span className="truncate max-w-[250px] font-mono">
                          {agent.source_file}
                        </span>
                      </button>
                    </TooltipTrigger>
                    <TooltipContent>
                      <p>Click to copy file path</p>
                    </TooltipContent>
                  </Tooltip>
                </TooltipProvider>
              </div>

              {/* Model */}
              {agent.model && (
                <div className="flex justify-between items-center text-sm">
                  <span className="text-muted-foreground">Model</span>
                  <code className="text-xs bg-muted px-2 py-0.5 rounded font-mono">
                    {agent.model}
                  </code>
                </div>
              )}

              {/* Tools */}
              {agent.tools.length > 0 && (
                <div className="space-y-2">
                  <div className="flex items-center gap-2 text-sm text-muted-foreground">
                    <Wrench className="h-3 w-3" />
                    <span>Available Tools</span>
                  </div>
                  <div className="flex flex-wrap gap-1">
                    {agent.tools.map((tool) => (
                      <Badge key={tool} variant="secondary" className="text-[10px]">
                        {tool}
                      </Badge>
                    ))}
                  </div>
                </div>
              )}

              {/* System Prompt */}
              {agent.system_prompt && (
                <div className="space-y-2">
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground">
                      System Prompt
                    </span>
                    <Button
                      variant="ghost"
                      size="sm"
                      className="h-6 px-2"
                      onClick={() =>
                        handleCopy(agent.system_prompt || '', 'prompt')
                      }
                    >
                      {copiedField === 'prompt' ? (
                        <Check className="h-3 w-3 text-green-500" />
                      ) : (
                        <Copy className="h-3 w-3" />
                      )}
                    </Button>
                  </div>
                  <pre className="bg-muted rounded p-2 text-xs font-mono overflow-auto max-h-[200px] whitespace-pre-wrap">
                    {agent.system_prompt}
                  </pre>
                </div>
              )}

              {/* No system prompt indicator */}
              {!agent.system_prompt && (
                <p className="text-xs text-muted-foreground italic">
                  No system prompt defined
                </p>
              )}
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
