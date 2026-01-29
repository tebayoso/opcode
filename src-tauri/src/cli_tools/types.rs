use serde::{Deserialize, Serialize};

/// Supported CLI tool types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum CLIToolType {
    Claude,
    Gemini,
    Codex,
    #[serde(rename = "opencode")]
    OpenCode,
    #[serde(rename = "github_copilot")]
    GitHubCopilot,
    Cursor,
    ESLint,
    Vite,
}

impl CLIToolType {
    pub fn all() -> Vec<CLIToolType> {
        vec![
            CLIToolType::Claude,
            CLIToolType::Gemini,
            CLIToolType::Codex,
            CLIToolType::OpenCode,
            CLIToolType::GitHubCopilot,
            CLIToolType::Cursor,
            CLIToolType::ESLint,
            CLIToolType::Vite,
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            CLIToolType::Claude => "Claude Code",
            CLIToolType::Gemini => "Gemini CLI",
            CLIToolType::Codex => "Codex CLI",
            CLIToolType::OpenCode => "OpenCode",
            CLIToolType::GitHubCopilot => "GitHub Copilot CLI",
            CLIToolType::Cursor => "Cursor CLI",
            CLIToolType::ESLint => "ESLint",
            CLIToolType::Vite => "Vite",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            CLIToolType::Claude => "Anthropic's AI coding assistant",
            CLIToolType::Gemini => "Google's AI-powered CLI for code assistance",
            CLIToolType::Codex => "OpenAI's command-line coding assistant",
            CLIToolType::OpenCode => "Open-source AI coding assistant",
            CLIToolType::GitHubCopilot => "GitHub's AI pair programmer in the terminal",
            CLIToolType::Cursor => "Cursor's AI coding agent",
            CLIToolType::ESLint => "Pluggable and configurable linter tool for identifying and reporting on patterns in JavaScript.",
            CLIToolType::Vite => "Next Generation Frontend Tooling.",
        }
    }

    pub fn website(&self) -> &'static str {
        match self {
            CLIToolType::Claude => "https://claude.ai/code",
            CLIToolType::Gemini => "https://github.com/google-gemini/gemini-cli",
            CLIToolType::Codex => "https://github.com/openai/codex",
            CLIToolType::OpenCode => "https://github.com/opencode-ai/opencode",
            CLIToolType::GitHubCopilot => "https://docs.github.com/en/copilot/using-github-copilot/using-github-copilot-in-the-command-line",
            CLIToolType::Cursor => "https://cursor.com",
            CLIToolType::ESLint => "https://eslint.org/",
            CLIToolType::Vite => "https://vitejs.dev/",
        }
    }

    pub fn install_instructions(&self) -> &'static str {
        match self {
            CLIToolType::Claude => "npm install -g @anthropic-ai/claude-code",
            CLIToolType::Gemini => "npm install -g @google/gemini-cli",
            CLIToolType::Codex => "npm install -g @openai/codex",
            CLIToolType::OpenCode => "curl -fsSL https://opencode.ai/install.sh | bash",
            CLIToolType::GitHubCopilot => "gh extension install github/gh-copilot",
            CLIToolType::Cursor => "Download from cursor.com and install Cursor app",
            CLIToolType::ESLint => "npm install -g eslint",
            CLIToolType::Vite => "npm install -g vite",
        }
    }

    /// Convert to string for database storage
    pub fn to_db_string(&self) -> String {
        serde_json::to_string(self)
            .unwrap_or_else(|_| format!("{:?}", self))
            .trim_matches('"')
            .to_string()
    }

    /// Parse from database string
    pub fn from_db_string(s: &str) -> Option<CLIToolType> {
        let json_str = format!("\"{}\"", s);
        serde_json::from_str(&json_str).ok()
    }
}

/// Source of installation discovery
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum InstallationSource {
    /// Found via which/where command
    System,
    /// Homebrew installation
    Homebrew,
    /// npm global installation
    NpmGlobal,
    /// NVM-managed Node.js installation
    Nvm,
    /// GitHub CLI extension
    GhExtension,
    /// Script-based installation (e.g., curl | bash)
    ScriptInstall,
    /// Application bundle (e.g., macOS .app)
    AppBundle,
    /// Custom user-specified path
    Custom,
}

impl InstallationSource {
    #[allow(dead_code)]
    pub fn display_name(&self) -> &'static str {
        match self {
            InstallationSource::System => "System",
            InstallationSource::Homebrew => "Homebrew",
            InstallationSource::NpmGlobal => "npm (global)",
            InstallationSource::Nvm => "NVM",
            InstallationSource::GhExtension => "GitHub CLI Extension",
            InstallationSource::ScriptInstall => "Script Install",
            InstallationSource::AppBundle => "Application Bundle",
            InstallationSource::Custom => "Custom",
        }
    }
}

/// Represents a detected CLI tool installation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CLIToolInstallation {
    /// Type of CLI tool
    pub tool_type: CLIToolType,
    /// Display name for this installation
    pub name: String,
    /// Full path to the binary or command
    pub path: String,
    /// Version string if detected
    pub version: Option<String>,
    /// How this installation was discovered
    pub source: InstallationSource,
    /// Whether the tool is currently available/working
    pub is_available: bool,
    /// The command to execute (may differ from path for extensions)
    pub command: String,
    /// Additional source details (e.g., "nvm (v20.10.0)")
    pub source_detail: Option<String>,
}

/// Configuration for detecting a specific CLI tool
#[derive(Debug, Clone)]
pub struct DetectionConfig {
    /// Binary names to search for (e.g., ["claude", "claude.exe"])
    pub binary_names: Vec<&'static str>,
    /// Homebrew formula/cask names
    pub homebrew_names: Vec<&'static str>,
    /// npm package names (for global installs)
    pub npm_packages: Vec<&'static str>,
    /// Standard paths to check
    pub standard_paths: Vec<&'static str>,
    /// Command to get version (e.g., "--version")
    pub version_args: Vec<&'static str>,
    /// GitHub CLI extension name (if applicable)
    pub gh_extension: Option<&'static str>,
    /// Pattern to extract version from output
    pub version_pattern: Option<&'static str>,
}

/// Capabilities of a CLI tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CLIToolCapabilities {
    pub files: bool,
    pub settings: bool,
    pub mcp_servers: bool,
    pub agents: bool,
    pub usage: bool,
}

/// Tool definition with display info and detection config
#[derive(Debug, Clone)]
pub struct CLIToolDefinition {
    #[allow(dead_code)]
    pub tool_type: CLIToolType,
    pub detection: DetectionConfig,
    pub capabilities: CLIToolCapabilities,
}

/// Status of all CLI tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CLIToolsStatus {
    /// All detected tools with their installations
    pub tools: Vec<CLIToolWithStatus>,
    /// When this status was last updated
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// A CLI tool with its installation status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CLIToolWithStatus {
    /// Type of CLI tool
    pub tool_type: CLIToolType,
    /// Display name
    pub name: String,
    /// Description
    pub description: String,
    /// Website URL
    pub website: String,
    /// Install instructions
    pub install_instructions: String,
    /// Whether any installation is available
    pub is_installed: bool,
    /// All found installations
    pub installations: Vec<CLIToolInstallation>,
    /// The preferred installation (if set)
    pub preferred_installation: Option<CLIToolInstallation>,
    /// Capabilities of the tool
    pub capabilities: CLIToolCapabilities,
}

impl CLIToolWithStatus {
    pub fn new(tool_type: CLIToolType, capabilities: CLIToolCapabilities) -> Self {
        Self {
            name: tool_type.display_name().to_string(),
            description: tool_type.description().to_string(),
            website: tool_type.website().to_string(),
            install_instructions: tool_type.install_instructions().to_string(),
            tool_type,
            is_installed: false,
            installations: Vec::new(),
            preferred_installation: None,
            capabilities,
        }
    }
}
