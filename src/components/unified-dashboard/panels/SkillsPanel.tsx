import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { 
  Search, 
  Download, 
  Trash2, 
  RefreshCw,
  Puzzle,
  Loader2
} from 'lucide-react';

interface Skill {
  name: string;
  description: string;
  source: string;
  version: string;
  author: string;
}

export function SkillsPanel() {
  const [searchQuery, setSearchQuery] = useState('');
  const [searchResults, setSearchResults] = useState<Skill[]>([]);
  const [installedSkills, setInstalledSkills] = useState<Skill[]>([]);
  const [isSearching, setIsSearching] = useState(false);
  const [isLoadingInstalled, setIsLoadingInstalled] = useState(true);

  useEffect(() => {
    loadInstalledSkills();
  }, []);

  const loadInstalledSkills = async () => {
    try {
      setIsLoadingInstalled(true);
      const response = await invoke<{ data: Skill[] }>('skills_list_installed', { scope: null });
      setInstalledSkills(response.data);
    } catch (error) {
      console.error('Failed to load installed skills:', error);
    } finally {
      setIsLoadingInstalled(false);
    }
  };

  const handleSearch = async () => {
    if (!searchQuery.trim()) return;
    
    try {
      setIsSearching(true);
      const response = await invoke<{ data: Skill[] }>('skills_search', { query: searchQuery });
      setSearchResults(response.data);
    } catch (error) {
      console.error('Failed to search skills:', error);
    } finally {
      setIsSearching(false);
    }
  };

  const handleInstall = async (source: string, name: string) => {
    try {
      await invoke('skills_install', {
        request: { source, name, scope: 'global' }
      });
      loadInstalledSkills();
    } catch (error) {
      console.error('Failed to install skill:', error);
    }
  };

  const handleUninstall = async (name: string) => {
    try {
      await invoke('skills_uninstall', {
        request: { name, scope: 'global' }
      });
      loadInstalledSkills();
    } catch (error) {
      console.error('Failed to uninstall skill:', error);
    }
  };

  return (
    <div className="p-6 space-y-6">
      <div>
        <h1 className="text-3xl font-bold text-foreground">Skills</h1>
        <p className="text-muted-foreground mt-1">
          Manage Claude skills from skills.sh
        </p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Search className="w-5 h-5" />
            Search Skills
          </CardTitle>
          <CardDescription>
            Find and install skills from the registry
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex gap-2">
            <Input
              placeholder="Search for skills..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
              className="flex-1"
            />
            <Button onClick={handleSearch} disabled={isSearching}>
              {isSearching ? (
                <><Loader2 className="w-4 h-4 mr-2 animate-spin" />Searching...</>
              ) : (
                <><Search className="w-4 h-4 mr-2" />Search</>
              )}
            </Button>
          </div>

          {searchResults.length > 0 && (
            <div className="mt-4 space-y-2">
              {searchResults.map((skill) => (
                <div
                  key={`${skill.source}/${skill.name}`}
                  className="flex items-center justify-between p-3 border rounded-lg"
                >
                  <div>
                    <div className="font-medium">{skill.name}</div>
                    <div className="text-sm text-muted-foreground">
                      {skill.description}
                    </div>
                    <div className="text-xs text-muted-foreground mt-1">
                      {skill.source} • v{skill.version} • by {skill.author}
                    </div>
                  </div>
                  <Button
                    size="sm"
                    onClick={() => handleInstall(skill.source, skill.name)}
                  >
                    <Download className="w-4 h-4 mr-2" />
                    Install
                  </Button>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <div className="flex justify-between items-center">
            <div>
              <CardTitle className="flex items-center gap-2">
                <Puzzle className="w-5 h-5" />
                Installed Skills
              </CardTitle>
              <CardDescription>
                Skills currently available in your environment
              </CardDescription>
            </div>
            <Button variant="outline" size="sm" onClick={loadInstalledSkills}>
              <RefreshCw className="w-4 h-4 mr-2" />
              Refresh
            </Button>
          </div>
        </CardHeader>
        <CardContent>
          {isLoadingInstalled ? (
            <div className="flex items-center justify-center py-8">
              <Loader2 className="w-6 h-6 animate-spin text-primary" />
            </div>
          ) : installedSkills.length === 0 ? (
            <div className="text-center py-8 text-muted-foreground">
              No skills installed yet
            </div>
          ) : (
            <div className="space-y-2">
              {installedSkills.map((skill) => (
                <div
                  key={skill.name}
                  className="flex items-center justify-between p-3 border rounded-lg"
                >
                  <div>
                    <div className="font-medium">{skill.name}</div>
                    <div className="text-sm text-muted-foreground">
                      {skill.description}
                    </div>
                    <div className="flex gap-2 mt-1">
                      <Badge variant="outline" className="text-xs">
                        v{skill.version}
                      </Badge>
                      <Badge variant="outline" className="text-xs">
                        {skill.source}
                      </Badge>
                    </div>
                  </div>
                  <Button
                    size="sm"
                    variant="destructive"
                    onClick={() => handleUninstall(skill.name)}
                  >
                    <Trash2 className="w-4 h-4 mr-2" />
                    Uninstall
                  </Button>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
