import React from 'react';
import { useUnifiedDashboard } from './UnifiedDashboardContext';
import { 
  LayoutDashboard, 
  Terminal, 
  Puzzle, 
  Server, 
  Settings,
  CheckCircle2,
  AlertCircle,
  Loader2
} from 'lucide-react';

const navItems = [
  { id: 'overview', label: 'Overview', icon: LayoutDashboard },
  { id: 'tools', label: 'Tools', icon: Terminal },
  { id: 'skills', label: 'Skills', icon: Puzzle },
  { id: 'mcp', label: 'MCP Servers', icon: Server },
  { id: 'configs', label: 'Configs', icon: Settings },
];

export function UnifiedDashboardSidebar() {
  const { activeSection, setActiveSection, stats } = useUnifiedDashboard();

  return (
    <aside className="w-64 bg-card border-r border-border flex flex-col h-full">
      <div className="p-6 border-b border-border">
        <h1 className="text-xl font-bold text-foreground">Opcode</h1>
        <p className="text-xs text-muted-foreground mt-1">Unified Control Panel</p>
      </div>

      <nav className="flex-1 p-4 space-y-1">
        {navItems.map((item) => {
          const Icon = item.icon;
          const isActive = activeSection === item.id;
          
          return (
            <button
              key={item.id}
              onClick={() => setActiveSection(item.id)}
              className={`w-full flex items-center gap-3 px-4 py-2.5 rounded-lg text-sm font-medium transition-colors ${
                isActive
                  ? 'bg-primary text-primary-foreground'
                  : 'text-muted-foreground hover:bg-muted hover:text-foreground'
              }`}
            >
              <Icon className="w-5 h-5" />
              {item.label}
              {item.id === 'tools' && stats.installed_tools > 0 && (
                <span className="ml-auto text-xs bg-background/20 px-2 py-0.5 rounded-full">
                  {stats.installed_tools}/{stats.total_tools}
                </span>
              )}
            </button>
          );
        })}
      </nav>

      <div className="p-4 border-t border-border space-y-2">
        <div className="flex items-center gap-2 text-xs text-muted-foreground">
          <CheckCircle2 className="w-4 h-4 text-green-500" />
          <span>{stats.installed_tools} tools ready</span>
        </div>
        {stats.active_warnings > 0 && (
          <div className="flex items-center gap-2 text-xs text-muted-foreground">
            <AlertCircle className="w-4 h-4 text-yellow-500" />
            <span>{stats.active_warnings} warnings</span>
          </div>
        )}
        {stats.running_jobs > 0 && (
          <div className="flex items-center gap-2 text-xs text-muted-foreground">
            <Loader2 className="w-4 h-4 text-blue-500 animate-spin" />
            <span>{stats.running_jobs} jobs running</span>
          </div>
        )}
      </div>
    </aside>
  );
}
