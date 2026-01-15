import React, { useState, useEffect, useCallback } from "react";
import { api, type Plugin, type Marketplace, type PluginComponent } from "@/lib/api";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Badge } from "@/components/ui/badge";
import { Switch } from "@/components/ui/switch";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Package,
  Store,
  Download,
  Trash2,
  Plus,
  RefreshCw,
  Search,
  Folder,
  Code,
  Bot,
  Zap,
  FileCode,
  Server,
  Check,
  X,
  AlertCircle,
  Github,
  Globe,
  FolderOpen,
  Edit,
  Eye,
} from "lucide-react";
import { cn } from "@/lib/utils";
import MDEditor from "@uiw/react-md-editor";

interface PluginManagerProps {
  className?: string;
}

const COMPONENT_ICONS: Record<string, React.ReactNode> = {
  command: <Code className="h-4 w-4" />,
  agent: <Bot className="h-4 w-4" />,
  skill: <Zap className="h-4 w-4" />,
  hook: <FileCode className="h-4 w-4" />,
  mcp: <Server className="h-4 w-4" />,
};

const SOURCE_TYPE_ICONS: Record<string, React.ReactNode> = {
  github: <Github className="h-4 w-4" />,
  git: <Github className="h-4 w-4" />,
  url: <Globe className="h-4 w-4" />,
  local: <FolderOpen className="h-4 w-4" />,
};

export const PluginManager: React.FC<PluginManagerProps> = ({ className }) => {
  const [activeTab, setActiveTab] = useState("installed");
  const [installedPlugins, setInstalledPlugins] = useState<Plugin[]>([]);
  const [marketplaces, setMarketplaces] = useState<Marketplace[]>([]);
  const [availablePlugins, setAvailablePlugins] = useState<Plugin[]>([]);
  const [selectedMarketplace, setSelectedMarketplace] = useState<string>("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");

  // Dialog states
  const [addMarketplaceOpen, setAddMarketplaceOpen] = useState(false);
  const [newMarketplaceSource, setNewMarketplaceSource] = useState("");
  const [createPluginOpen, setCreatePluginOpen] = useState(false);
  const [newPluginName, setNewPluginName] = useState("");
  const [newPluginDescription, setNewPluginDescription] = useState("");
  const [selectedPlugin, setSelectedPlugin] = useState<Plugin | null>(null);
  const [pluginDetailsOpen, setPluginDetailsOpen] = useState(false);
  const [componentEditorOpen, setComponentEditorOpen] = useState(false);
  const [editingComponent, setEditingComponent] = useState<{
    plugin: Plugin;
    component: PluginComponent;
    content: string;
  } | null>(null);
  const [installScope, setInstallScope] = useState<string>("local");
  const [installDialogOpen, setInstallDialogOpen] = useState(false);
  const [pluginToInstall, setPluginToInstall] = useState<Plugin | null>(null);

  // Load installed plugins
  const loadInstalledPlugins = useCallback(async () => {
    try {
      setLoading(true);
      const plugins = await api.pluginsListInstalled();
      setInstalledPlugins(plugins);
    } catch (err) {
      setError("Failed to load installed plugins");
      console.error(err);
    } finally {
      setLoading(false);
    }
  }, []);

  // Load marketplaces
  const loadMarketplaces = useCallback(async () => {
    try {
      const mps = await api.pluginsListMarketplaces();
      setMarketplaces(mps);
      if (mps.length > 0 && !selectedMarketplace) {
        setSelectedMarketplace(mps[0].source);
      }
    } catch (err) {
      console.error("Failed to load marketplaces:", err);
    }
  }, [selectedMarketplace]);

  // Load available plugins from selected marketplace
  const loadAvailablePlugins = useCallback(async () => {
    if (!selectedMarketplace) {
      return;
    }
    try {
      setLoading(true);
      const plugins = await api.pluginsFetchMarketplace(selectedMarketplace);
      setAvailablePlugins(plugins);
    } catch (err) {
      console.error("Failed to fetch marketplace plugins:", err);
      setAvailablePlugins([]);
    } finally {
      setLoading(false);
    }
  }, [selectedMarketplace]);

  useEffect(() => {
    loadInstalledPlugins();
    loadMarketplaces();
  }, [loadInstalledPlugins, loadMarketplaces]);

  useEffect(() => {
    if (activeTab === "discover" && selectedMarketplace) {
      loadAvailablePlugins();
    }
  }, [activeTab, selectedMarketplace, loadAvailablePlugins]);

  // Add marketplace
  const handleAddMarketplace = async () => {
    if (!newMarketplaceSource.trim()) {
      return;
    }
    try {
      setLoading(true);
      await api.pluginsAddMarketplace(newMarketplaceSource.trim());
      await loadMarketplaces();
      setNewMarketplaceSource("");
      setAddMarketplaceOpen(false);
    } catch (err) {
      setError("Failed to add marketplace");
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  // Remove marketplace
  const handleRemoveMarketplace = async (source: string) => {
    try {
      await api.pluginsRemoveMarketplace(source);
      await loadMarketplaces();
    } catch (err) {
      setError("Failed to remove marketplace");
      console.error(err);
    }
  };

  // Install plugin
  const handleInstallPlugin = async () => {
    if (!pluginToInstall) {
      return;
    }
    try {
      setLoading(true);
      await api.pluginsInstall(
        pluginToInstall.name,
        pluginToInstall.marketplace || selectedMarketplace,
        installScope
      );
      await loadInstalledPlugins();
      setInstallDialogOpen(false);
      setPluginToInstall(null);
    } catch (err) {
      setError("Failed to install plugin");
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  // Uninstall plugin
  const handleUninstallPlugin = async (plugin: Plugin) => {
    if (!plugin.path) {
      return;
    }
    try {
      await api.pluginsDelete(plugin.path);
      await loadInstalledPlugins();
    } catch (err) {
      setError("Failed to uninstall plugin");
      console.error(err);
    }
  };

  // Toggle plugin enabled state
  const handleTogglePlugin = async (plugin: Plugin) => {
    try {
      if (plugin.enabled) {
        await api.pluginsDisable(plugin.name);
      } else {
        await api.pluginsEnable(plugin.name);
      }
      await loadInstalledPlugins();
    } catch (err) {
      setError("Failed to toggle plugin");
      console.error(err);
    }
  };

  // Create plugin
  const handleCreatePlugin = async () => {
    if (!newPluginName.trim()) {
      return;
    }
    try {
      setLoading(true);
      await api.pluginsCreate(newPluginName.trim(), newPluginDescription);
      await loadInstalledPlugins();
      setNewPluginName("");
      setNewPluginDescription("");
      setCreatePluginOpen(false);
    } catch (err) {
      setError("Failed to create plugin");
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  // Open component editor
  const handleEditComponent = async (plugin: Plugin, component: PluginComponent) => {
    if (!plugin.path) {
      return;
    }
    try {
      const content = await api.pluginsReadComponent(
        plugin.path,
        component.component_type,
        component.name
      );
      setEditingComponent({ plugin, component, content });
      setComponentEditorOpen(true);
    } catch (err) {
      setError("Failed to load component");
      console.error(err);
    }
  };

  // Save component
  const handleSaveComponent = async () => {
    if (!editingComponent?.plugin.path) {
      return;
    }
    try {
      await api.pluginsSaveComponent(
        editingComponent.plugin.path,
        editingComponent.component.component_type,
        editingComponent.component.name,
        editingComponent.content
      );
      setComponentEditorOpen(false);
      setEditingComponent(null);
    } catch (err) {
      setError("Failed to save component");
      console.error(err);
    }
  };

  // Filter plugins by search
  const filteredInstalled = installedPlugins.filter(
    (p) =>
      p.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      p.description?.toLowerCase().includes(searchQuery.toLowerCase())
  );

  const filteredAvailable = availablePlugins.filter(
    (p) =>
      p.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      p.description?.toLowerCase().includes(searchQuery.toLowerCase())
  );

  // Render plugin card
  const renderPluginCard = (plugin: Plugin, showInstall = false) => (
    <Card key={plugin.name} className="hover:border-primary/50 transition-colors">
      <CardHeader className="pb-2">
        <div className="flex items-start justify-between">
          <div className="flex items-center gap-2">
            <Package className="h-5 w-5 text-primary" />
            <div>
              <CardTitle className="text-base">{plugin.name}</CardTitle>
              {plugin.version && (
                <span className="text-xs text-muted-foreground">v{plugin.version}</span>
              )}
            </div>
          </div>
          <div className="flex items-center gap-2">
            {plugin.category && (
              <Badge variant="outline" className="text-xs">
                {plugin.category}
              </Badge>
            )}
            {plugin.installed && (
              <Switch
                checked={plugin.enabled}
                onCheckedChange={() => handleTogglePlugin(plugin)}
                aria-label="Toggle plugin"
              />
            )}
          </div>
        </div>
      </CardHeader>
      <CardContent className="space-y-3">
        {plugin.description && (
          <p className="text-sm text-muted-foreground line-clamp-2">{plugin.description}</p>
        )}

        {plugin.author && (
          <p className="text-xs text-muted-foreground">by {plugin.author.name}</p>
        )}

        {plugin.components.length > 0 && (
          <div className="flex flex-wrap gap-1">
            {plugin.components.map((comp) => (
              <Badge
                key={`${comp.component_type}-${comp.name}`}
                variant="secondary"
                className="text-xs cursor-pointer hover:bg-secondary/80"
                onClick={() => plugin.installed && handleEditComponent(plugin, comp)}
              >
                {COMPONENT_ICONS[comp.component_type]}
                <span className="ml-1">{comp.name}</span>
              </Badge>
            ))}
          </div>
        )}

        <div className="flex items-center gap-2 pt-2">
          {showInstall ? (
            <Button
              size="sm"
              onClick={() => {
                setPluginToInstall(plugin);
                setInstallDialogOpen(true);
              }}
              disabled={loading}
            >
              <Download className="h-4 w-4 mr-1" />
              Install
            </Button>
          ) : (
            <>
              <Button
                size="sm"
                variant="outline"
                onClick={() => {
                  setSelectedPlugin(plugin);
                  setPluginDetailsOpen(true);
                }}
              >
                <Eye className="h-4 w-4 mr-1" />
                Details
              </Button>
              <Button
                size="sm"
                variant="destructive"
                onClick={() => handleUninstallPlugin(plugin)}
              >
                <Trash2 className="h-4 w-4 mr-1" />
                Remove
              </Button>
            </>
          )}
        </div>
      </CardContent>
    </Card>
  );

  return (
    <div className={cn("h-full flex flex-col", className)}>
      <div className="p-6 space-y-6 flex-1 overflow-y-auto">
        {error && (
          <div className="flex items-center gap-2 p-3 bg-destructive/10 text-destructive rounded-lg">
            <AlertCircle className="h-4 w-4" />
            <span className="text-sm">{error}</span>
            <Button size="sm" variant="ghost" onClick={() => setError(null)}>
              <X className="h-4 w-4" />
            </Button>
          </div>
        )}

        <Tabs value={activeTab} onValueChange={setActiveTab} className="space-y-6">
          <div className="flex items-center justify-between">
            <TabsList>
              <TabsTrigger value="installed">
                <Package className="h-4 w-4 mr-2" />
                Installed
              </TabsTrigger>
              <TabsTrigger value="discover">
                <Store className="h-4 w-4 mr-2" />
                Discover
              </TabsTrigger>
              <TabsTrigger value="marketplaces">
                <Globe className="h-4 w-4 mr-2" />
                Marketplaces
              </TabsTrigger>
            </TabsList>

            <div className="flex items-center gap-2">
              <div className="relative">
                <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                <Input
                  placeholder="Search plugins..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="pl-9 w-64"
                />
              </div>
              <Button
                variant="outline"
                size="icon"
                onClick={() => {
                  loadInstalledPlugins();
                  loadMarketplaces();
                }}
                disabled={loading}
              >
                <RefreshCw className={cn("h-4 w-4", loading && "animate-spin")} />
              </Button>
              <Button onClick={() => setCreatePluginOpen(true)}>
                <Plus className="h-4 w-4 mr-2" />
                Create Plugin
              </Button>
            </div>
          </div>

        {/* Installed Plugins */}
        <TabsContent value="installed" className="space-y-6 mt-0">
          {filteredInstalled.length === 0 ? (
            <Card className="py-16 px-8 text-center">
              <Package className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
              <h3 className="text-lg font-medium mb-2">No plugins installed</h3>
              <p className="text-muted-foreground mb-4 text-sm">
                Browse the Discover tab to find and install plugins
              </p>
              <Button onClick={() => setActiveTab("discover")}>
                <Store className="h-4 w-4 mr-2" />
                Discover Plugins
              </Button>
            </Card>
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              {filteredInstalled.map((plugin) => renderPluginCard(plugin))}
            </div>
          )}
        </TabsContent>

        {/* Discover Plugins */}
        <TabsContent value="discover" className="space-y-6 mt-0">
          <Card className="p-4">
            <div className="flex items-center gap-4">
              <Label className="text-sm font-medium whitespace-nowrap">Marketplace:</Label>
              <Select value={selectedMarketplace} onValueChange={setSelectedMarketplace}>
                <SelectTrigger className="w-72">
                  <SelectValue placeholder="Select a marketplace" />
                </SelectTrigger>
                <SelectContent>
                  {marketplaces.map((mp) => (
                    <SelectItem key={mp.source} value={mp.source}>
                      <div className="flex items-center gap-2">
                        {SOURCE_TYPE_ICONS[mp.source_type]}
                        <span>{mp.name}</span>
                      </div>
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <Button
                variant="outline"
                onClick={loadAvailablePlugins}
                disabled={loading || !selectedMarketplace}
              >
                <RefreshCw className={cn("h-4 w-4 mr-2", loading && "animate-spin")} />
                Refresh
              </Button>
            </div>
          </Card>

          {loading ? (
            <div className="flex items-center justify-center py-16">
              <RefreshCw className="h-8 w-8 animate-spin text-muted-foreground" />
            </div>
          ) : filteredAvailable.length === 0 ? (
            <Card className="py-16 px-8 text-center">
              <Store className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
              <h3 className="text-lg font-medium mb-2">No plugins found</h3>
              <p className="text-muted-foreground text-sm">
                {selectedMarketplace
                  ? "This marketplace doesn't have any plugins yet"
                  : "Select a marketplace to browse available plugins"}
              </p>
            </Card>
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              {filteredAvailable.map((plugin) => renderPluginCard(plugin, true))}
            </div>
          )}
        </TabsContent>

        {/* Marketplaces */}
        <TabsContent value="marketplaces" className="space-y-6 mt-0">
          <div className="flex justify-between items-center">
            <div>
              <h3 className="text-lg font-medium">Plugin Marketplaces</h3>
              <p className="text-sm text-muted-foreground">Manage plugin sources and repositories</p>
            </div>
            <Button onClick={() => setAddMarketplaceOpen(true)}>
              <Plus className="h-4 w-4 mr-2" />
              Add Marketplace
            </Button>
          </div>

          <div className="space-y-3">
            {marketplaces.map((mp) => (
              <Card key={mp.source} className="p-4">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <div className="h-10 w-10 rounded-lg bg-muted flex items-center justify-center">
                      {SOURCE_TYPE_ICONS[mp.source_type]}
                    </div>
                    <div>
                      <h4 className="font-medium">{mp.name}</h4>
                      <p className="text-sm text-muted-foreground font-mono">{mp.source}</p>
                      {mp.description && (
                        <p className="text-sm text-muted-foreground mt-1">{mp.description}</p>
                      )}
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    <Badge variant="outline">{mp.source_type}</Badge>
                    {mp.name !== "claude-plugins-official" && (
                      <Button
                        size="sm"
                        variant="destructive"
                        onClick={() => handleRemoveMarketplace(mp.source)}
                      >
                        <Trash2 className="h-4 w-4" />
                      </Button>
                    )}
                  </div>
                </div>
              </Card>
            ))}
          </div>

          <Card className="p-5 bg-muted/30 border-dashed">
            <h4 className="font-medium mb-3">Marketplace Sources</h4>
            <p className="text-sm text-muted-foreground mb-4">
              Add marketplaces using different formats:
            </p>
            <ul className="text-sm text-muted-foreground space-y-2">
              <li className="flex items-center gap-2">
                <code className="bg-background px-2 py-1 rounded text-xs font-mono">owner/repo</code>
                <span>- GitHub repository</span>
              </li>
              <li className="flex items-center gap-2">
                <code className="bg-background px-2 py-1 rounded text-xs font-mono">https://github.com/owner/repo</code>
                <span>- Git URL</span>
              </li>
              <li className="flex items-center gap-2">
                <code className="bg-background px-2 py-1 rounded text-xs font-mono">https://example.com/marketplace.json</code>
                <span>- Direct URL</span>
              </li>
              <li className="flex items-center gap-2">
                <code className="bg-background px-2 py-1 rounded text-xs font-mono">./local-path</code>
                <span>- Local directory</span>
              </li>
            </ul>
          </Card>
        </TabsContent>
        </Tabs>
      </div>

      {/* Add Marketplace Dialog */}
      <Dialog open={addMarketplaceOpen} onOpenChange={setAddMarketplaceOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Add Marketplace</DialogTitle>
            <DialogDescription>
              Add a new plugin marketplace source. You can use GitHub repos, Git URLs, or local
              paths.
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            <div>
              <Label>Source</Label>
              <Input
                placeholder="e.g., anthropics/claude-plugins-official"
                value={newMarketplaceSource}
                onChange={(e) => setNewMarketplaceSource(e.target.value)}
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setAddMarketplaceOpen(false)}>
              Cancel
            </Button>
            <Button onClick={handleAddMarketplace} disabled={loading || !newMarketplaceSource.trim()}>
              Add Marketplace
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Create Plugin Dialog */}
      <Dialog open={createPluginOpen} onOpenChange={setCreatePluginOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Create New Plugin</DialogTitle>
            <DialogDescription>Create a new local plugin with the basic structure.</DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            <div>
              <Label>Plugin Name</Label>
              <Input
                placeholder="my-plugin"
                value={newPluginName}
                onChange={(e) => setNewPluginName(e.target.value)}
              />
            </div>
            <div>
              <Label>Description</Label>
              <Textarea
                placeholder="A brief description of what this plugin does..."
                value={newPluginDescription}
                onChange={(e) => setNewPluginDescription(e.target.value)}
                rows={3}
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setCreatePluginOpen(false)}>
              Cancel
            </Button>
            <Button onClick={handleCreatePlugin} disabled={loading || !newPluginName.trim()}>
              Create Plugin
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Install Plugin Dialog */}
      <Dialog open={installDialogOpen} onOpenChange={setInstallDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Install Plugin</DialogTitle>
            <DialogDescription>
              Choose the installation scope for {pluginToInstall?.name}
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            <div>
              <Label>Installation Scope</Label>
              <Select value={installScope} onValueChange={setInstallScope}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="local">
                    <div className="flex items-center gap-2">
                      <Folder className="h-4 w-4" />
                      <span>Local (personal only)</span>
                    </div>
                  </SelectItem>
                  <SelectItem value="project">
                    <div className="flex items-center gap-2">
                      <FolderOpen className="h-4 w-4" />
                      <span>Project (shared with team)</span>
                    </div>
                  </SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setInstallDialogOpen(false)}>
              Cancel
            </Button>
            <Button onClick={handleInstallPlugin} disabled={loading}>
              <Download className="h-4 w-4 mr-2" />
              Install
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Plugin Details Dialog */}
      <Dialog open={pluginDetailsOpen} onOpenChange={setPluginDetailsOpen}>
        <DialogContent className="max-w-3xl max-h-[80vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <Package className="h-5 w-5" />
              {selectedPlugin?.name}
            </DialogTitle>
            {selectedPlugin?.version && (
              <Badge variant="outline">v{selectedPlugin.version}</Badge>
            )}
          </DialogHeader>

          {selectedPlugin && (
            <div className="space-y-4">
              {selectedPlugin.description && (
                <div>
                  <Label>Description</Label>
                  <p className="text-sm text-muted-foreground">{selectedPlugin.description}</p>
                </div>
              )}

              {selectedPlugin.author && (
                <div>
                  <Label>Author</Label>
                  <p className="text-sm">
                    {selectedPlugin.author.name}
                    {selectedPlugin.author.email && (
                      <span className="text-muted-foreground"> ({selectedPlugin.author.email})</span>
                    )}
                  </p>
                </div>
              )}

              {selectedPlugin.path && (
                <div>
                  <Label>Location</Label>
                  <p className="text-sm text-muted-foreground font-mono">{selectedPlugin.path}</p>
                </div>
              )}

              {selectedPlugin.components.length > 0 && (
                <div>
                  <Label>Components</Label>
                  <div className="grid grid-cols-1 gap-2 mt-2">
                    {selectedPlugin.components.map((comp) => (
                      <div
                        key={`${comp.component_type}-${comp.name}`}
                        className="flex items-center justify-between p-2 bg-muted rounded-lg"
                      >
                        <div className="flex items-center gap-2">
                          {COMPONENT_ICONS[comp.component_type]}
                          <span className="font-medium">{comp.name}</span>
                          <Badge variant="outline" className="text-xs">
                            {comp.component_type}
                          </Badge>
                        </div>
                        <Button
                          size="sm"
                          variant="ghost"
                          onClick={() => handleEditComponent(selectedPlugin, comp)}
                        >
                          <Edit className="h-4 w-4" />
                        </Button>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          )}
        </DialogContent>
      </Dialog>

      {/* Component Editor Dialog */}
      <Dialog open={componentEditorOpen} onOpenChange={setComponentEditorOpen}>
        <DialogContent className="max-w-4xl max-h-[90vh] overflow-hidden flex flex-col">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              {editingComponent && COMPONENT_ICONS[editingComponent.component.component_type]}
              {editingComponent?.component.name}
            </DialogTitle>
            <DialogDescription>
              Edit the {editingComponent?.component.component_type} component
            </DialogDescription>
          </DialogHeader>

          <div className="flex-1 min-h-0 overflow-hidden" data-color-mode="dark">
            <MDEditor
              value={editingComponent?.content || ""}
              onChange={(value) => {
                if (editingComponent) {
                  setEditingComponent({ ...editingComponent, content: value || "" });
                }
              }}
              height="100%"
              preview="edit"
            />
          </div>

          <DialogFooter>
            <Button variant="outline" onClick={() => setComponentEditorOpen(false)}>
              Cancel
            </Button>
            <Button onClick={handleSaveComponent}>
              <Check className="h-4 w-4 mr-2" />
              Save Changes
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
};
