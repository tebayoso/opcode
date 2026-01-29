import React from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Settings, FileText, Wrench } from 'lucide-react';

export function ConfigsPanel() {
  const configCategories = [
    {
      title: 'Tool Configurations',
      description: 'Manage individual tool settings',
      icon: Wrench,
      items: [
        { name: 'Claude Code', path: '~/.claude/settings.json' },
        { name: 'Cursor', path: '~/.cursor/mcp.json' },
        { name: 'ESLint', path: 'eslint.config.js' },
      ],
    },
    {
      title: 'Global Settings',
      description: 'Application-wide configuration',
      icon: Settings,
      items: [
        { name: 'Proxy Settings', status: 'Not configured' },
        { name: 'Theme', status: 'Dark mode' },
        { name: 'Auto-update', status: 'Enabled' },
      ],
    },
    {
      title: 'Custom Configs',
      description: 'User-defined configurations',
      icon: FileText,
      items: [
        { name: 'Custom Tools', count: 0 },
        { name: 'Environment Variables', count: 0 },
        { name: 'Custom Scripts', count: 0 },
      ],
    },
  ];

  return (
    <div className="p-6 space-y-6">
      <div>
        <h1 className="text-3xl font-bold text-foreground">Configurations</h1>
        <p className="text-muted-foreground mt-1">
          Manage tool settings and custom configurations
        </p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {configCategories.map((category) => {
          const Icon = category.icon;
          return (
            <Card key={category.title}>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Icon className="w-5 h-5" />
                  {category.title}
                </CardTitle>
                <CardDescription>{category.description}</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  {category.items.map((item: any) => (
                    <div
                      key={item.name}
                      className="flex items-center justify-between p-3 border rounded-lg hover:bg-muted/50 transition-colors cursor-pointer"
                    >
                      <span className="font-medium">{item.name}</span>
                      {item.path && (
                        <code className="text-xs bg-muted px-2 py-1 rounded">
                          {item.path}
                        </code>
                      )}
                      {item.status && (
                        <Badge variant="outline" className="text-xs">
                          {item.status}
                        </Badge>
                      )}
                      {item.count !== undefined && (
                        <Badge variant="secondary" className="text-xs">
                          {item.count} items
                        </Badge>
                      )}
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>
          );
        })}
      </div>
    </div>
  );
}
