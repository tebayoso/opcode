import React from 'react';
import { useUnifiedDashboard } from './UnifiedDashboardContext';
import { UnifiedDashboardSidebar } from './UnifiedDashboardSidebar';
import { OverviewPanel } from './panels/OverviewPanel';
import { ToolsPanel } from './panels/ToolsPanel';
import { SkillsPanel } from './panels/SkillsPanel';
import { MCPPanel } from './panels/MCPPanel';
import { ConfigsPanel } from './panels/ConfigsPanel';
import { Loader2 } from 'lucide-react';

export function UnifiedDashboard() {
  const { activeSection, isLoading } = useUnifiedDashboard();

  const renderPanel = () => {
    if (isLoading) {
      return (
        <div className="flex items-center justify-center h-full">
          <Loader2 className="w-8 h-8 animate-spin text-primary" />
        </div>
      );
    }

    switch (activeSection) {
      case 'overview':
        return <OverviewPanel />;
      case 'tools':
        return <ToolsPanel />;
      case 'skills':
        return <SkillsPanel />;
      case 'mcp':
        return <MCPPanel />;
      case 'configs':
        return <ConfigsPanel />;
      default:
        return <OverviewPanel />;
    }
  };

  return (
    <div className="flex h-screen bg-background">
      <UnifiedDashboardSidebar />
      <main className="flex-1 overflow-auto">
        {renderPanel()}
      </main>
    </div>
  );
}
