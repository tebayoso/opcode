use super::types::{CLIToolDefinition, CLIToolType, DetectionConfig};

/// Get the detection configuration for a specific CLI tool
pub fn get_tool_definition(tool_type: &CLIToolType) -> CLIToolDefinition {
    match tool_type {
        CLIToolType::Claude => CLIToolDefinition {
            tool_type: CLIToolType::Claude,
            detection: DetectionConfig {
                binary_names: vec!["claude"],
                homebrew_names: vec!["claude-code"],
                npm_packages: vec!["@anthropic-ai/claude-code"],
                standard_paths: vec![
                    "/usr/local/bin/claude",
                    "/opt/homebrew/bin/claude",
                    "/usr/bin/claude",
                    "~/.local/bin/claude",
                    "~/.npm-global/bin/claude",
                    "~/.yarn/bin/claude",
                    "~/.bun/bin/claude",
                    "~/.claude/local/claude",
                ],
                version_args: vec!["--version"],
                gh_extension: None,
                version_pattern: Some(r"(\d+\.\d+\.\d+)"),
            },
        },

        CLIToolType::Gemini => CLIToolDefinition {
            tool_type: CLIToolType::Gemini,
            detection: DetectionConfig {
                binary_names: vec!["gemini"],
                homebrew_names: vec![],
                npm_packages: vec!["@google/gemini-cli"],
                standard_paths: vec![
                    "/usr/local/bin/gemini",
                    "/opt/homebrew/bin/gemini",
                    "~/.local/bin/gemini",
                    "~/.npm-global/bin/gemini",
                    "~/.yarn/bin/gemini",
                    "~/.bun/bin/gemini",
                ],
                version_args: vec!["--version"],
                gh_extension: None,
                version_pattern: Some(r"(\d+\.\d+\.\d+)"),
            },
        },

        CLIToolType::Codex => CLIToolDefinition {
            tool_type: CLIToolType::Codex,
            detection: DetectionConfig {
                binary_names: vec!["codex"],
                homebrew_names: vec![],
                npm_packages: vec!["@openai/codex"],
                standard_paths: vec![
                    "/usr/local/bin/codex",
                    "/opt/homebrew/bin/codex",
                    "~/.local/bin/codex",
                    "~/.npm-global/bin/codex",
                    "~/.yarn/bin/codex",
                    "~/.bun/bin/codex",
                ],
                version_args: vec!["--version"],
                gh_extension: None,
                version_pattern: Some(r"(\d+\.\d+\.\d+)"),
            },
        },

        CLIToolType::OpenCode => CLIToolDefinition {
            tool_type: CLIToolType::OpenCode,
            detection: DetectionConfig {
                binary_names: vec!["opencode"],
                homebrew_names: vec![],
                npm_packages: vec![],
                standard_paths: vec![
                    "~/.local/bin/opencode",
                    "~/.opencode/bin/opencode",
                    "/usr/local/bin/opencode",
                ],
                version_args: vec!["--version"],
                gh_extension: None,
                version_pattern: Some(r"(\d+\.\d+\.\d+)"),
            },
        },

        CLIToolType::GitHubCopilot => CLIToolDefinition {
            tool_type: CLIToolType::GitHubCopilot,
            detection: DetectionConfig {
                // GitHub Copilot is a gh extension, not a standalone binary
                binary_names: vec![],
                homebrew_names: vec![],
                npm_packages: vec![],
                standard_paths: vec![],
                version_args: vec!["--version"],
                gh_extension: Some("github/gh-copilot"),
                version_pattern: Some(r"(\d+\.\d+\.\d+)"),
            },
        },

        CLIToolType::Cursor => CLIToolDefinition {
            tool_type: CLIToolType::Cursor,
            detection: DetectionConfig {
                binary_names: vec!["cursor-agent", "cursor"],
                homebrew_names: vec!["cursor"],
                npm_packages: vec![],
                standard_paths: vec![
                    "~/.local/bin/cursor-agent",
                    "~/.local/bin/cursor",
                    "/Applications/Cursor.app/Contents/MacOS/Cursor",
                    "/usr/local/bin/cursor",
                ],
                version_args: vec!["--version"],
                gh_extension: None,
                version_pattern: Some(r"(\d+\.\d+\.\d+)"),
            },
        },
    }
}

/// Get definitions for all supported CLI tools
pub fn get_all_tool_definitions() -> Vec<CLIToolDefinition> {
    CLIToolType::all()
        .into_iter()
        .map(|t| get_tool_definition(&t))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_tools_have_definitions() {
        for tool_type in CLIToolType::all() {
            let definition = get_tool_definition(&tool_type);
            assert_eq!(definition.tool_type, tool_type);
        }
    }

    #[test]
    fn test_claude_has_npm_package() {
        let def = get_tool_definition(&CLIToolType::Claude);
        assert!(!def.detection.npm_packages.is_empty());
    }

    #[test]
    fn test_github_copilot_has_gh_extension() {
        let def = get_tool_definition(&CLIToolType::GitHubCopilot);
        assert!(def.detection.gh_extension.is_some());
    }
}
