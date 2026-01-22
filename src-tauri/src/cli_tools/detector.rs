use super::registry::get_tool_definition;
use super::types::{CLIToolInstallation, CLIToolType, CLIToolWithStatus, InstallationSource};
use log::{debug, info, warn};
use regex::Regex;
use std::collections::HashSet;
use std::path::PathBuf;
use std::process::Command;

/// Detect all installations for a specific CLI tool
pub fn detect_tool_installations(tool_type: &CLIToolType) -> CLIToolWithStatus {
    info!("Detecting installations for {:?}", tool_type);

    let definition = get_tool_definition(tool_type);
    let mut status = CLIToolWithStatus::new(tool_type.clone());
    let mut seen_paths: HashSet<String> = HashSet::new();

    // 1. Try which/where command for each binary name
    for binary_name in &definition.detection.binary_names {
        if let Some(installation) = try_which_command(tool_type, binary_name) {
            if seen_paths.insert(installation.path.clone()) {
                status.installations.push(installation);
            }
        }
    }

    // 2. Check Homebrew installations
    for homebrew_name in &definition.detection.homebrew_names {
        if let Some(installation) = find_homebrew_installation(tool_type, homebrew_name) {
            if seen_paths.insert(installation.path.clone()) {
                status.installations.push(installation);
            }
        }
    }

    // 3. Check npm global installations
    for npm_package in &definition.detection.npm_packages {
        for installation in find_npm_global_installations(tool_type, npm_package) {
            if seen_paths.insert(installation.path.clone()) {
                status.installations.push(installation);
            }
        }
    }

    // 4. Check NVM installations (for npm-based tools)
    if !definition.detection.npm_packages.is_empty() {
        for binary_name in &definition.detection.binary_names {
            for installation in find_nvm_installations(tool_type, binary_name) {
                if seen_paths.insert(installation.path.clone()) {
                    status.installations.push(installation);
                }
            }
        }
    }

    // 5. Check standard paths
    for standard_path in &definition.detection.standard_paths {
        if let Some(installation) = check_standard_path(tool_type, standard_path) {
            if seen_paths.insert(installation.path.clone()) {
                status.installations.push(installation);
            }
        }
    }

    // 6. Check GitHub CLI extension (if applicable)
    if let Some(extension_name) = definition.detection.gh_extension {
        if let Some(installation) = check_gh_extension(tool_type, extension_name) {
            if seen_paths.insert(installation.path.clone()) {
                status.installations.push(installation);
            }
        }
    }

    // Get version for each installation
    for installation in &mut status.installations {
        if installation.version.is_none() {
            installation.version = get_tool_version(
                &installation.command,
                &definition.detection.version_args,
                definition.detection.version_pattern,
            );
        }
    }

    status.is_installed = !status.installations.is_empty();

    info!(
        "Found {} installations for {:?}",
        status.installations.len(),
        tool_type
    );

    status
}

/// Detect all CLI tools and their installations
pub fn detect_all_tools() -> Vec<CLIToolWithStatus> {
    info!("Detecting all CLI tools...");

    CLIToolType::all()
        .into_iter()
        .map(|tool_type| detect_tool_installations(&tool_type))
        .collect()
}

/// Try using the 'which' (Unix) or 'where' (Windows) command
#[cfg(unix)]
fn try_which_command(tool_type: &CLIToolType, binary_name: &str) -> Option<CLIToolInstallation> {
    debug!("Trying 'which {}' to find binary...", binary_name);

    match Command::new("which").arg(binary_name).output() {
        Ok(output) if output.status.success() => {
            let output_str = String::from_utf8_lossy(&output.stdout).trim().to_string();

            if output_str.is_empty() {
                return None;
            }

            // Parse aliased output: "cmd: aliased to /path/to/cmd"
            let path = if output_str.contains("aliased to") {
                output_str
                    .split("aliased to")
                    .nth(1)
                    .map(|s| s.trim().to_string())
            } else {
                Some(output_str)
            }?;

            debug!("'which' found {} at: {}", binary_name, path);

            // Verify the path exists
            if !PathBuf::from(&path).exists() {
                warn!("Path from 'which' does not exist: {}", path);
                return None;
            }

            Some(CLIToolInstallation {
                tool_type: tool_type.clone(),
                name: tool_type.display_name().to_string(),
                path: path.clone(),
                version: None,
                source: InstallationSource::System,
                is_available: true,
                command: path,
                source_detail: Some("which".to_string()),
            })
        }
        _ => None,
    }
}

#[cfg(windows)]
fn try_which_command(tool_type: &CLIToolType, binary_name: &str) -> Option<CLIToolInstallation> {
    debug!("Trying 'where {}' to find binary...", binary_name);

    // Try both with and without .exe extension
    let names_to_try = vec![binary_name.to_string(), format!("{}.exe", binary_name)];

    for name in names_to_try {
        match Command::new("where").arg(&name).output() {
            Ok(output) if output.status.success() => {
                let output_str = String::from_utf8_lossy(&output.stdout).trim().to_string();

                if output_str.is_empty() {
                    continue;
                }

                // 'where' can return multiple paths; take the first one
                let path = output_str.lines().next()?.trim().to_string();

                if path.is_empty() {
                    continue;
                }

                debug!("'where' found {} at: {}", name, path);

                if !PathBuf::from(&path).exists() {
                    warn!("Path from 'where' does not exist: {}", path);
                    continue;
                }

                return Some(CLIToolInstallation {
                    tool_type: tool_type.clone(),
                    name: tool_type.display_name().to_string(),
                    path: path.clone(),
                    version: None,
                    source: InstallationSource::System,
                    is_available: true,
                    command: path,
                    source_detail: Some("where".to_string()),
                });
            }
            _ => continue,
        }
    }

    None
}

/// Find Homebrew installation
#[cfg(unix)]
fn find_homebrew_installation(
    tool_type: &CLIToolType,
    formula_name: &str,
) -> Option<CLIToolInstallation> {
    debug!("Checking Homebrew for {}", formula_name);

    // Check common Homebrew paths
    let homebrew_paths = vec!["/opt/homebrew/bin", "/usr/local/bin", "/home/linuxbrew/.linuxbrew/bin"];

    for base_path in homebrew_paths {
        let path = PathBuf::from(base_path).join(formula_name);
        if path.exists() && path.is_file() {
            let path_str = path.to_string_lossy().to_string();
            debug!("Found Homebrew installation at: {}", path_str);

            return Some(CLIToolInstallation {
                tool_type: tool_type.clone(),
                name: tool_type.display_name().to_string(),
                path: path_str.clone(),
                version: None,
                source: InstallationSource::Homebrew,
                is_available: true,
                command: path_str,
                source_detail: Some(format!("Homebrew ({})", base_path)),
            });
        }
    }

    None
}

#[cfg(windows)]
fn find_homebrew_installation(
    _tool_type: &CLIToolType,
    _formula_name: &str,
) -> Option<CLIToolInstallation> {
    // Homebrew is not available on Windows
    None
}

/// Find npm global installations
fn find_npm_global_installations(
    tool_type: &CLIToolType,
    _package_name: &str,
) -> Vec<CLIToolInstallation> {
    let mut installations = Vec::new();

    // Get npm prefix
    if let Ok(output) = Command::new("npm").args(["config", "get", "prefix"]).output() {
        if output.status.success() {
            let prefix = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !prefix.is_empty() {
                #[cfg(unix)]
                let bin_path = PathBuf::from(&prefix).join("bin");
                #[cfg(windows)]
                let bin_path = PathBuf::from(&prefix);

                for binary_name in get_tool_definition(tool_type).detection.binary_names {
                    let tool_path = bin_path.join(binary_name);
                    if tool_path.exists() && tool_path.is_file() {
                        let path_str = tool_path.to_string_lossy().to_string();
                        debug!("Found npm global installation at: {}", path_str);

                        installations.push(CLIToolInstallation {
                            tool_type: tool_type.clone(),
                            name: tool_type.display_name().to_string(),
                            path: path_str.clone(),
                            version: None,
                            source: InstallationSource::NpmGlobal,
                            is_available: true,
                            command: path_str,
                            source_detail: Some(format!("npm prefix: {}", prefix)),
                        });
                    }
                }
            }
        }
    }

    installations
}

/// Find installations in NVM directories
#[cfg(unix)]
fn find_nvm_installations(tool_type: &CLIToolType, binary_name: &str) -> Vec<CLIToolInstallation> {
    let mut installations = Vec::new();

    // First check NVM_BIN environment variable (current active NVM)
    if let Ok(nvm_bin) = std::env::var("NVM_BIN") {
        let tool_path = PathBuf::from(&nvm_bin).join(binary_name);
        if tool_path.exists() && tool_path.is_file() {
            let path_str = tool_path.to_string_lossy().to_string();
            debug!("Found {} via NVM_BIN: {}", binary_name, path_str);
            installations.push(CLIToolInstallation {
                tool_type: tool_type.clone(),
                name: tool_type.display_name().to_string(),
                path: path_str.clone(),
                version: None,
                source: InstallationSource::Nvm,
                is_available: true,
                command: path_str,
                source_detail: Some("NVM (active)".to_string()),
            });
        }
    }

    // Check all NVM directories
    if let Ok(home) = std::env::var("HOME") {
        let nvm_dir = PathBuf::from(&home).join(".nvm").join("versions").join("node");

        if let Ok(entries) = std::fs::read_dir(&nvm_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    let tool_path = entry.path().join("bin").join(binary_name);

                    if tool_path.exists() && tool_path.is_file() {
                        let path_str = tool_path.to_string_lossy().to_string();
                        let node_version = entry.file_name().to_string_lossy().to_string();

                        debug!(
                            "Found {} in NVM node {}: {}",
                            binary_name, node_version, path_str
                        );

                        installations.push(CLIToolInstallation {
                            tool_type: tool_type.clone(),
                            name: tool_type.display_name().to_string(),
                            path: path_str.clone(),
                            version: None,
                            source: InstallationSource::Nvm,
                            is_available: true,
                            command: path_str,
                            source_detail: Some(format!("NVM ({})", node_version)),
                        });
                    }
                }
            }
        }
    }

    installations
}

#[cfg(windows)]
fn find_nvm_installations(tool_type: &CLIToolType, binary_name: &str) -> Vec<CLIToolInstallation> {
    let mut installations = Vec::new();

    if let Ok(nvm_home) = std::env::var("NVM_HOME") {
        if let Ok(entries) = std::fs::read_dir(&nvm_home) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    let tool_path = entry.path().join(format!("{}.exe", binary_name));

                    if tool_path.exists() && tool_path.is_file() {
                        let path_str = tool_path.to_string_lossy().to_string();
                        let node_version = entry.file_name().to_string_lossy().to_string();

                        debug!(
                            "Found {} in NVM node {}: {}",
                            binary_name, node_version, path_str
                        );

                        installations.push(CLIToolInstallation {
                            tool_type: tool_type.clone(),
                            name: tool_type.display_name().to_string(),
                            path: path_str.clone(),
                            version: None,
                            source: InstallationSource::Nvm,
                            is_available: true,
                            command: path_str,
                            source_detail: Some(format!("NVM ({})", node_version)),
                        });
                    }
                }
            }
        }
    }

    installations
}

/// Check a standard path for installation
fn check_standard_path(tool_type: &CLIToolType, path_pattern: &str) -> Option<CLIToolInstallation> {
    let expanded_path = expand_path(path_pattern);
    let path_buf = PathBuf::from(&expanded_path);

    if path_buf.exists() {
        let is_file = path_buf.is_file();
        let is_app_bundle = expanded_path.ends_with(".app") || expanded_path.contains(".app/");

        if is_file || is_app_bundle {
            debug!("Found {} at standard path: {}", tool_type.display_name(), expanded_path);

            let source = if is_app_bundle {
                InstallationSource::AppBundle
            } else if expanded_path.contains("/.local/bin/") {
                InstallationSource::ScriptInstall
            } else {
                InstallationSource::System
            };

            return Some(CLIToolInstallation {
                tool_type: tool_type.clone(),
                name: tool_type.display_name().to_string(),
                path: expanded_path.clone(),
                version: None,
                source,
                is_available: true,
                command: expanded_path,
                source_detail: None,
            });
        }
    }

    None
}

/// Check for GitHub CLI extension
fn check_gh_extension(tool_type: &CLIToolType, extension_name: &str) -> Option<CLIToolInstallation> {
    debug!("Checking for gh extension: {}", extension_name);

    // First check if gh is available
    let gh_output = Command::new("gh").args(["extension", "list"]).output().ok()?;

    if !gh_output.status.success() {
        debug!("gh extension list failed");
        return None;
    }

    let output_str = String::from_utf8_lossy(&gh_output.stdout);

    // Parse extension list output (format: "gh copilot   github/gh-copilot   v1.0.0")
    for line in output_str.lines() {
        if line.contains(extension_name) || line.contains(&extension_name.replace("github/", "")) {
            debug!("Found gh extension: {}", extension_name);

            // Extract version if present
            let version = line.split_whitespace().last().map(|v| {
                v.trim_start_matches('v').to_string()
            });

            // The command for gh copilot is "gh copilot"
            let command = format!("gh {}", extension_name.split('/').last().unwrap_or(extension_name));

            return Some(CLIToolInstallation {
                tool_type: tool_type.clone(),
                name: tool_type.display_name().to_string(),
                path: format!("gh extension: {}", extension_name),
                version,
                source: InstallationSource::GhExtension,
                is_available: true,
                command,
                source_detail: Some(extension_name.to_string()),
            });
        }
    }

    None
}

/// Get version for a tool by running version command
fn get_tool_version(
    command: &str,
    version_args: &[&str],
    version_pattern: Option<&str>,
) -> Option<String> {
    // Handle gh extensions differently
    if command.starts_with("gh ") {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.len() >= 2 {
            let mut cmd = Command::new("gh");
            cmd.args(&parts[1..]);
            cmd.args(version_args);

            match cmd.output() {
                Ok(output) if output.status.success() => {
                    return extract_version(&output.stdout, version_pattern);
                }
                _ => return None,
            }
        }
    }

    // Standard binary version check
    match Command::new(command).args(version_args).output() {
        Ok(output) if output.status.success() => extract_version(&output.stdout, version_pattern),
        Ok(output) => {
            // Some tools output version to stderr
            extract_version(&output.stderr, version_pattern)
        }
        Err(e) => {
            debug!("Failed to get version for {}: {}", command, e);
            None
        }
    }
}

/// Extract version string from command output
fn extract_version(stdout: &[u8], pattern: Option<&str>) -> Option<String> {
    let output_str = String::from_utf8_lossy(stdout);
    debug!("Extracting version from: {:?}", output_str);

    let pattern_str = pattern.unwrap_or(r"(\d+\.\d+\.\d+(?:-[a-zA-Z0-9.-]+)?)");
    let regex = Regex::new(pattern_str).ok()?;

    if let Some(captures) = regex.captures(&output_str) {
        if let Some(version_match) = captures.get(1) {
            let version = version_match.as_str().to_string();
            debug!("Extracted version: {}", version);
            return Some(version);
        }
    }

    debug!("No version found in output");
    None
}

/// Expand ~ to home directory
fn expand_path(path: &str) -> String {
    if path.starts_with("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return path.replacen("~", &home, 1);
        }
        #[cfg(windows)]
        if let Ok(home) = std::env::var("USERPROFILE") {
            return path.replacen("~", &home, 1);
        }
    }
    path.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_path() {
        let expanded = expand_path("~/.local/bin/test");
        assert!(!expanded.starts_with("~/"));
    }

    #[test]
    fn test_extract_version() {
        let output = b"claude 1.0.41";
        let version = extract_version(output, Some(r"(\d+\.\d+\.\d+)"));
        assert_eq!(version, Some("1.0.41".to_string()));
    }

    #[test]
    fn test_extract_version_with_prefix() {
        let output = b"v2.3.4-beta.1";
        let version = extract_version(output, None);
        assert!(version.is_some());
    }
}
