use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Marketplace {
    pub name: String,
    pub source: String, // GitHub repo, URL, or local path
    pub source_type: String, // "github", "url", "local"
    pub description: Option<String>,
    pub plugins_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginAuthor {
    pub name: String,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginComponent {
    pub name: String,
    pub component_type: String, // "command", "agent", "skill", "hook", "mcp"
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub author: Option<PluginAuthor>,
    pub category: Option<String>,
    pub marketplace: Option<String>,
    pub installed: bool,
    pub enabled: bool,
    pub scope: Option<String>, // "local", "project", "managed"
    pub path: Option<String>,
    pub components: Vec<PluginComponent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub author: Option<PluginAuthor>,
    #[serde(rename = "mcpServers")]
    pub mcp_servers: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceManifest {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub owner: Option<PluginAuthor>,
    pub plugins: Vec<MarketplacePlugin>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplacePlugin {
    pub name: String,
    pub description: Option<String>,
    pub source: String,
    pub category: Option<String>,
    pub version: Option<String>,
    pub author: Option<PluginAuthor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeSettings {
    pub plugins: Option<Vec<PluginConfig>>,
    pub marketplaces: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub name: String,
    pub enabled: Option<bool>,
    pub scope: Option<String>,
}

fn get_claude_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
}

fn get_plugins_dir() -> PathBuf {
    get_claude_dir().join("plugins")
}

fn get_settings_path() -> PathBuf {
    get_claude_dir().join("settings.json")
}

fn read_settings() -> ClaudeSettings {
    let settings_path = get_settings_path();
    if settings_path.exists() {
        if let Ok(content) = fs::read_to_string(&settings_path) {
            if let Ok(settings) = serde_json::from_str(&content) {
                return settings;
            }
        }
    }
    ClaudeSettings {
        plugins: None,
        marketplaces: None,
    }
}

fn scan_plugin_components(plugin_path: &Path) -> Vec<PluginComponent> {
    let mut components = Vec::new();

    // Scan commands directory
    let commands_dir = plugin_path.join("commands");
    if commands_dir.exists() {
        if let Ok(entries) = fs::read_dir(&commands_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "md") {
                    let name = path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string();
                    components.push(PluginComponent {
                        name,
                        component_type: "command".to_string(),
                        description: None,
                    });
                }
            }
        }
    }

    // Scan agents directory
    let agents_dir = plugin_path.join("agents");
    if agents_dir.exists() {
        if let Ok(entries) = fs::read_dir(&agents_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "md") {
                    let name = path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string();
                    components.push(PluginComponent {
                        name,
                        component_type: "agent".to_string(),
                        description: None,
                    });
                }
            }
        }
    }

    // Scan skills directory
    let skills_dir = plugin_path.join("skills");
    if skills_dir.exists() {
        if let Ok(entries) = fs::read_dir(&skills_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let skill_file = path.join("SKILL.md");
                    if skill_file.exists() {
                        let name = path.file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or("unknown")
                            .to_string();
                        components.push(PluginComponent {
                            name,
                            component_type: "skill".to_string(),
                            description: None,
                        });
                    }
                }
            }
        }
    }

    // Check for hooks
    let hooks_file = plugin_path.join("hooks").join("hooks.json");
    if hooks_file.exists() {
        components.push(PluginComponent {
            name: "hooks".to_string(),
            component_type: "hook".to_string(),
            description: Some("Event handlers".to_string()),
        });
    }

    // Check for MCP servers
    let mcp_file = plugin_path.join(".mcp.json");
    if mcp_file.exists() {
        if let Ok(content) = fs::read_to_string(&mcp_file) {
            if let Ok(servers) = serde_json::from_str::<HashMap<String, serde_json::Value>>(&content) {
                for server_name in servers.keys() {
                    components.push(PluginComponent {
                        name: server_name.clone(),
                        component_type: "mcp".to_string(),
                        description: Some("MCP Server".to_string()),
                    });
                }
            }
        }
    }

    components
}

fn read_plugin_manifest(plugin_path: &Path) -> Option<PluginManifest> {
    let manifest_path = plugin_path.join(".claude-plugin").join("plugin.json");
    if manifest_path.exists() {
        if let Ok(content) = fs::read_to_string(&manifest_path) {
            if let Ok(manifest) = serde_json::from_str(&content) {
                return Some(manifest);
            }
        }
    }
    None
}

/// Known marketplace entry from known_marketplaces.json
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct KnownMarketplaceEntry {
    source: KnownMarketplaceSource,
    #[serde(rename = "installLocation")]
    install_location: Option<String>,
    #[serde(rename = "lastUpdated")]
    last_updated: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct KnownMarketplaceSource {
    source: String,
    repo: Option<String>,
    url: Option<String>,
}

/// List all registered marketplaces
#[tauri::command]
pub fn plugins_list_marketplaces() -> Result<Vec<Marketplace>, String> {
    let mut marketplaces = Vec::new();

    // Read from known_marketplaces.json
    let known_file = get_claude_dir().join("known_marketplaces.json");
    if known_file.exists() {
        if let Ok(content) = fs::read_to_string(&known_file) {
            if let Ok(known) = serde_json::from_str::<HashMap<String, KnownMarketplaceEntry>>(&content) {
                for (name, entry) in known {
                    let source = entry.source.repo.clone()
                        .or(entry.source.url.clone())
                        .unwrap_or_else(|| name.clone());

                    let source_type = entry.source.source.clone();

                    // Count plugins in marketplace
                    let mut plugins_count = 0;
                    if let Some(install_loc) = &entry.install_location {
                        let manifest_path = PathBuf::from(install_loc)
                            .join(".claude-plugin")
                            .join("marketplace.json");
                        if manifest_path.exists() {
                            if let Ok(manifest_content) = fs::read_to_string(&manifest_path) {
                                if let Ok(manifest) = serde_json::from_str::<MarketplaceManifest>(&manifest_content) {
                                    plugins_count = manifest.plugins.len();
                                }
                            }
                        }
                    }

                    marketplaces.push(Marketplace {
                        name,
                        source,
                        source_type,
                        description: None,
                        plugins_count,
                    });
                }
            }
        }
    }

    // If no known marketplaces, add default
    if marketplaces.is_empty() {
        marketplaces.push(Marketplace {
            name: "claude-plugins-official".to_string(),
            source: "anthropics/claude-plugins-official".to_string(),
            source_type: "github".to_string(),
            description: Some("Official Anthropic plugins".to_string()),
            plugins_count: 0,
        });
    }

    // Sort by name
    marketplaces.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(marketplaces)
}

/// Add a new marketplace
#[tauri::command]
pub fn plugins_add_marketplace(source: String) -> Result<Marketplace, String> {
    let mut settings = read_settings();
    let mut marketplaces = settings.marketplaces.unwrap_or_default();

    // Check if already exists
    if marketplaces.contains(&source) {
        return Err("Marketplace already registered".to_string());
    }

    // Determine source type and name
    let (name, source_type) = if source.starts_with("https://") || source.starts_with("http://") {
        let name = source.split('/').last().unwrap_or("unknown").replace(".json", "").replace(".git", "");
        let st = if source.ends_with(".json") { "url" } else { "git" };
        (name, st.to_string())
    } else if source.contains('/') && !source.starts_with('.') && !source.starts_with('/') {
        let name = source.split('/').last().unwrap_or("unknown").to_string();
        (name, "github".to_string())
    } else {
        let name = Path::new(&source)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("local")
            .to_string();
        (name, "local".to_string())
    };

    marketplaces.push(source.clone());
    settings.marketplaces = Some(marketplaces);

    // Save settings
    let settings_path = get_settings_path();
    if let Some(parent) = settings_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let content = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    fs::write(&settings_path, content)
        .map_err(|e| format!("Failed to write settings: {}", e))?;

    Ok(Marketplace {
        name,
        source,
        source_type,
        description: None,
        plugins_count: 0,
    })
}

/// Remove a marketplace
#[tauri::command]
pub fn plugins_remove_marketplace(source: String) -> Result<(), String> {
    let mut settings = read_settings();
    let mut marketplaces = settings.marketplaces.unwrap_or_default();

    marketplaces.retain(|m| m != &source);
    settings.marketplaces = Some(marketplaces);

    // Save settings
    let settings_path = get_settings_path();
    let content = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    fs::write(&settings_path, content)
        .map_err(|e| format!("Failed to write settings: {}", e))?;

    Ok(())
}

/// List all installed plugins
#[tauri::command]
pub fn plugins_list_installed() -> Result<Vec<Plugin>, String> {
    let mut plugins = Vec::new();
    let plugins_dir = get_plugins_dir();
    let settings = read_settings();
    let plugin_configs: HashMap<String, PluginConfig> = settings
        .plugins
        .unwrap_or_default()
        .into_iter()
        .map(|p| (p.name.clone(), p))
        .collect();

    // Scan user plugins directory
    if plugins_dir.exists() {
        if let Ok(entries) = fs::read_dir(&plugins_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let plugin_manifest_dir = path.join(".claude-plugin");
                    if plugin_manifest_dir.exists() {
                        let manifest = read_plugin_manifest(&path);
                        let name = manifest.as_ref()
                            .map(|m| m.name.clone())
                            .unwrap_or_else(|| {
                                path.file_name()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or("unknown")
                                    .to_string()
                            });

                        let config = plugin_configs.get(&name);
                        let components = scan_plugin_components(&path);

                        plugins.push(Plugin {
                            name: name.clone(),
                            version: manifest.as_ref().and_then(|m| m.version.clone()),
                            description: manifest.as_ref().and_then(|m| m.description.clone()),
                            author: manifest.as_ref().and_then(|m| m.author.clone()),
                            category: None,
                            marketplace: None,
                            installed: true,
                            enabled: config.and_then(|c| c.enabled).unwrap_or(true),
                            scope: config.and_then(|c| c.scope.clone()).or(Some("local".to_string())),
                            path: Some(path.to_string_lossy().to_string()),
                            components,
                        });
                    }
                }
            }
        }
    }

    Ok(plugins)
}

/// Get plugin details
#[tauri::command]
pub fn plugins_get_details(plugin_path: String) -> Result<Plugin, String> {
    let path = PathBuf::from(&plugin_path);
    if !path.exists() {
        return Err("Plugin path does not exist".to_string());
    }

    let manifest = read_plugin_manifest(&path);
    let components = scan_plugin_components(&path);

    let name = manifest.as_ref()
        .map(|m| m.name.clone())
        .unwrap_or_else(|| {
            path.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string()
        });

    // Try to read README
    let readme_path = path.join("README.md");
    let description = if readme_path.exists() {
        fs::read_to_string(&readme_path).ok()
    } else {
        manifest.as_ref().and_then(|m| m.description.clone())
    };

    Ok(Plugin {
        name,
        version: manifest.as_ref().and_then(|m| m.version.clone()),
        description,
        author: manifest.as_ref().and_then(|m| m.author.clone()),
        category: None,
        marketplace: None,
        installed: true,
        enabled: true,
        scope: Some("local".to_string()),
        path: Some(plugin_path),
        components,
    })
}

/// Install a plugin using Claude CLI
#[tauri::command]
pub async fn plugins_install(plugin_name: String, marketplace: String, scope: String) -> Result<Plugin, String> {
    // Use Claude CLI to install
    let install_arg = if marketplace.is_empty() {
        plugin_name.clone()
    } else {
        format!("{}@{}", plugin_name, marketplace)
    };

    let output = Command::new("claude")
        .args(["plugin", "install", &install_arg, "--scope", &scope])
        .output()
        .map_err(|e| format!("Failed to run claude command: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to install plugin: {}", stderr));
    }

    // Return the installed plugin info
    let plugins_dir = get_plugins_dir();
    let plugin_path = plugins_dir.join(&plugin_name);

    if plugin_path.exists() {
        plugins_get_details(plugin_path.to_string_lossy().to_string())
    } else {
        Ok(Plugin {
            name: plugin_name,
            version: None,
            description: None,
            author: None,
            category: None,
            marketplace: Some(marketplace),
            installed: true,
            enabled: true,
            scope: Some(scope),
            path: None,
            components: Vec::new(),
        })
    }
}

/// Uninstall a plugin
#[tauri::command]
pub async fn plugins_uninstall(plugin_name: String) -> Result<(), String> {
    let output = Command::new("claude")
        .args(["plugin", "uninstall", &plugin_name])
        .output()
        .map_err(|e| format!("Failed to run claude command: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to uninstall plugin: {}", stderr));
    }

    Ok(())
}

/// Enable a plugin
#[tauri::command]
pub fn plugins_enable(plugin_name: String) -> Result<(), String> {
    let mut settings = read_settings();
    let mut plugins = settings.plugins.unwrap_or_default();

    let mut found = false;
    for plugin in &mut plugins {
        if plugin.name == plugin_name {
            plugin.enabled = Some(true);
            found = true;
            break;
        }
    }

    if !found {
        plugins.push(PluginConfig {
            name: plugin_name,
            enabled: Some(true),
            scope: Some("local".to_string()),
        });
    }

    settings.plugins = Some(plugins);

    let settings_path = get_settings_path();
    let content = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    fs::write(&settings_path, content)
        .map_err(|e| format!("Failed to write settings: {}", e))?;

    Ok(())
}

/// Disable a plugin
#[tauri::command]
pub fn plugins_disable(plugin_name: String) -> Result<(), String> {
    let mut settings = read_settings();
    let mut plugins = settings.plugins.unwrap_or_default();

    let mut found = false;
    for plugin in &mut plugins {
        if plugin.name == plugin_name {
            plugin.enabled = Some(false);
            found = true;
            break;
        }
    }

    if !found {
        plugins.push(PluginConfig {
            name: plugin_name,
            enabled: Some(false),
            scope: Some("local".to_string()),
        });
    }

    settings.plugins = Some(plugins);

    let settings_path = get_settings_path();
    let content = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    fs::write(&settings_path, content)
        .map_err(|e| format!("Failed to write settings: {}", e))?;

    Ok(())
}

/// Fetch available plugins from a marketplace
#[tauri::command]
pub async fn plugins_fetch_marketplace(marketplace_name: String) -> Result<Vec<Plugin>, String> {
    let mut plugins = Vec::new();

    // First, try to read from local marketplace cache
    let marketplaces_dir = get_claude_dir().join("plugins").join("marketplaces");

    // Get installed plugins to mark them
    let installed_plugins: std::collections::HashSet<String> = plugins_list_installed()
        .unwrap_or_default()
        .into_iter()
        .map(|p| p.name)
        .collect();

    // Read known_marketplaces.json to find all marketplaces
    let known_file = get_claude_dir().join("known_marketplaces.json");
    let known_marketplaces: HashMap<String, serde_json::Value> = if known_file.exists() {
        fs::read_to_string(&known_file)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or_default()
    } else {
        HashMap::new()
    };

    // If marketplace_name is empty or "all", load from all marketplaces
    let marketplace_names: Vec<String> = if marketplace_name.is_empty() || marketplace_name == "all" {
        known_marketplaces.keys().cloned().collect()
    } else {
        // Try to find the marketplace by source or name
        let mut names = Vec::new();
        for (name, _value) in &known_marketplaces {
            if name == &marketplace_name || name.contains(&marketplace_name) {
                names.push(name.clone());
            }
        }
        if names.is_empty() {
            // Also try the source matching
            names.push(marketplace_name.clone());
        }
        names
    };

    for mp_name in marketplace_names {
        let mp_dir = marketplaces_dir.join(&mp_name);

        // Try to read marketplace.json from .claude-plugin directory
        let manifest_path = mp_dir.join(".claude-plugin").join("marketplace.json");
        if manifest_path.exists() {
            if let Ok(content) = fs::read_to_string(&manifest_path) {
                if let Ok(manifest) = serde_json::from_str::<MarketplaceManifest>(&content) {
                    for mp_plugin in manifest.plugins {
                        let is_installed = installed_plugins.contains(&mp_plugin.name);
                        plugins.push(Plugin {
                            name: mp_plugin.name,
                            version: mp_plugin.version,
                            description: mp_plugin.description,
                            author: mp_plugin.author,
                            category: mp_plugin.category,
                            marketplace: Some(mp_name.clone()),
                            installed: is_installed,
                            enabled: is_installed,
                            scope: None,
                            path: None,
                            components: Vec::new(),
                        });
                    }
                }
            }
        }

        // Also scan the plugins directory directly
        let plugins_dir = mp_dir.join("plugins");
        if plugins_dir.exists() {
            if let Ok(entries) = fs::read_dir(&plugins_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let name = path.file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or("unknown")
                            .to_string();

                        // Skip if already in list
                        if plugins.iter().any(|p| p.name == name) {
                            continue;
                        }

                        // Try to read plugin manifest
                        let plugin_manifest = read_plugin_manifest(&path);
                        let is_installed = installed_plugins.contains(&name);

                        plugins.push(Plugin {
                            name: name.clone(),
                            version: plugin_manifest.as_ref().and_then(|m| m.version.clone()),
                            description: plugin_manifest.as_ref().and_then(|m| m.description.clone()),
                            author: plugin_manifest.as_ref().and_then(|m| m.author.clone()),
                            category: None,
                            marketplace: Some(mp_name.clone()),
                            installed: is_installed,
                            enabled: is_installed,
                            scope: None,
                            path: Some(path.to_string_lossy().to_string()),
                            components: Vec::new(),
                        });
                    }
                }
            }
        }

        // Also scan external_plugins directory
        let external_dir = mp_dir.join("external_plugins");
        if external_dir.exists() {
            if let Ok(entries) = fs::read_dir(&external_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let name = path.file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or("unknown")
                            .to_string();

                        // Skip if already in list
                        if plugins.iter().any(|p| p.name == name) {
                            continue;
                        }

                        let plugin_manifest = read_plugin_manifest(&path);
                        let is_installed = installed_plugins.contains(&name);

                        plugins.push(Plugin {
                            name: name.clone(),
                            version: plugin_manifest.as_ref().and_then(|m| m.version.clone()),
                            description: plugin_manifest.as_ref().and_then(|m| m.description.clone()),
                            author: plugin_manifest.as_ref().and_then(|m| m.author.clone()),
                            category: Some("external".to_string()),
                            marketplace: Some(mp_name.clone()),
                            installed: is_installed,
                            enabled: is_installed,
                            scope: None,
                            path: Some(path.to_string_lossy().to_string()),
                            components: Vec::new(),
                        });
                    }
                }
            }
        }
    }

    // Sort by name
    plugins.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(plugins)
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GitHubContent {
    name: String,
    path: String,
    #[serde(rename = "type")]
    content_type: String,
}

/// Read a plugin's README
#[tauri::command]
pub fn plugins_read_readme(plugin_path: String) -> Result<String, String> {
    let path = PathBuf::from(&plugin_path);
    let readme_path = path.join("README.md");

    if readme_path.exists() {
        fs::read_to_string(&readme_path)
            .map_err(|e| format!("Failed to read README: {}", e))
    } else {
        Err("README.md not found".to_string())
    }
}

/// Read a plugin component file
#[tauri::command]
pub fn plugins_read_component(plugin_path: String, component_type: String, component_name: String) -> Result<String, String> {
    let path = PathBuf::from(&plugin_path);

    let file_path = match component_type.as_str() {
        "command" => path.join("commands").join(format!("{}.md", component_name)),
        "agent" => path.join("agents").join(format!("{}.md", component_name)),
        "skill" => path.join("skills").join(&component_name).join("SKILL.md"),
        "hook" => path.join("hooks").join("hooks.json"),
        "mcp" => path.join(".mcp.json"),
        _ => return Err("Unknown component type".to_string()),
    };

    if file_path.exists() {
        fs::read_to_string(&file_path)
            .map_err(|e| format!("Failed to read component: {}", e))
    } else {
        Err("Component file not found".to_string())
    }
}

/// Save a plugin component file
#[tauri::command]
pub fn plugins_save_component(plugin_path: String, component_type: String, component_name: String, content: String) -> Result<(), String> {
    let path = PathBuf::from(&plugin_path);

    let file_path = match component_type.as_str() {
        "command" => path.join("commands").join(format!("{}.md", component_name)),
        "agent" => path.join("agents").join(format!("{}.md", component_name)),
        "skill" => path.join("skills").join(&component_name).join("SKILL.md"),
        "hook" => path.join("hooks").join("hooks.json"),
        "mcp" => path.join(".mcp.json"),
        _ => return Err("Unknown component type".to_string()),
    };

    // Ensure parent directory exists
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    fs::write(&file_path, content)
        .map_err(|e| format!("Failed to write component: {}", e))?;

    Ok(())
}

/// Create a new plugin
#[tauri::command]
pub fn plugins_create(name: String, description: String) -> Result<Plugin, String> {
    let plugins_dir = get_plugins_dir();
    let plugin_path = plugins_dir.join(&name);

    if plugin_path.exists() {
        return Err("Plugin already exists".to_string());
    }

    // Create plugin structure
    fs::create_dir_all(&plugin_path)
        .map_err(|e| format!("Failed to create plugin directory: {}", e))?;

    let manifest_dir = plugin_path.join(".claude-plugin");
    fs::create_dir_all(&manifest_dir)
        .map_err(|e| format!("Failed to create manifest directory: {}", e))?;

    // Create plugin.json
    let manifest = PluginManifest {
        name: name.clone(),
        version: Some("1.0.0".to_string()),
        description: Some(description.clone()),
        author: None,
        mcp_servers: None,
    };

    let manifest_content = serde_json::to_string_pretty(&manifest)
        .map_err(|e| format!("Failed to serialize manifest: {}", e))?;
    fs::write(manifest_dir.join("plugin.json"), manifest_content)
        .map_err(|e| format!("Failed to write manifest: {}", e))?;

    // Create empty directories
    fs::create_dir_all(plugin_path.join("commands")).ok();
    fs::create_dir_all(plugin_path.join("agents")).ok();
    fs::create_dir_all(plugin_path.join("skills")).ok();

    // Create README
    let readme = format!("# {}\n\n{}\n", name, description);
    fs::write(plugin_path.join("README.md"), readme)
        .map_err(|e| format!("Failed to write README: {}", e))?;

    Ok(Plugin {
        name,
        version: Some("1.0.0".to_string()),
        description: Some(description),
        author: None,
        category: None,
        marketplace: None,
        installed: true,
        enabled: true,
        scope: Some("local".to_string()),
        path: Some(plugin_path.to_string_lossy().to_string()),
        components: Vec::new(),
    })
}

/// Delete a plugin
#[tauri::command]
pub fn plugins_delete(plugin_path: String) -> Result<(), String> {
    let path = PathBuf::from(&plugin_path);

    if !path.exists() {
        return Err("Plugin path does not exist".to_string());
    }

    // Safety check: ensure we're deleting from plugins directory
    let plugins_dir = get_plugins_dir();
    if !path.starts_with(&plugins_dir) {
        return Err("Can only delete plugins from the plugins directory".to_string());
    }

    fs::remove_dir_all(&path)
        .map_err(|e| format!("Failed to delete plugin: {}", e))?;

    Ok(())
}
