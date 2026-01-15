use anyhow::{Context, Result};
use dirs;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Represents a Claude skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    /// Unique identifier for the skill
    pub id: String,
    /// Skill name (derived from folder name)
    pub name: String,
    /// Skill scope: "project" or "user"
    pub scope: String,
    /// Path to the skill directory
    pub dir_path: String,
    /// Path to the main SKILL.md file
    pub file_path: String,
    /// File type: "markdown" or "json"
    pub file_type: String,
    /// Skill content (markdown or JSON)
    pub content: String,
    /// Optional description from frontmatter
    pub description: Option<String>,
    /// Allowed tools from frontmatter
    pub allowed_tools: Vec<String>,
    /// Supporting files in the skill directory
    pub supporting_files: Vec<SupportingFile>,
}

/// Represents a supporting file in a skill directory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportingFile {
    pub name: String,
    pub path: String,
    pub file_type: String,
}

/// YAML frontmatter structure for skills
#[derive(Debug, Deserialize)]
struct SkillFrontmatter {
    #[serde(rename = "allowed-tools")]
    allowed_tools: Option<Vec<String>>,
    description: Option<String>,
}

/// Parse a markdown file with optional YAML frontmatter
fn parse_markdown_with_frontmatter(content: &str) -> Result<(Option<SkillFrontmatter>, String)> {
    let lines: Vec<&str> = content.lines().collect();

    // Check if the file starts with YAML frontmatter
    if lines.is_empty() || lines[0] != "---" {
        return Ok((None, content.to_string()));
    }

    // Find the end of frontmatter
    let mut frontmatter_end = None;
    for (i, line) in lines.iter().enumerate().skip(1) {
        if *line == "---" {
            frontmatter_end = Some(i);
            break;
        }
    }

    if let Some(end) = frontmatter_end {
        let frontmatter_content = lines[1..end].join("\n");
        let body_content = lines[(end + 1)..].join("\n");

        match serde_yaml::from_str::<SkillFrontmatter>(&frontmatter_content) {
            Ok(frontmatter) => Ok((Some(frontmatter), body_content)),
            Err(e) => {
                debug!("Failed to parse frontmatter: {}", e);
                Ok((None, content.to_string()))
            }
        }
    } else {
        Ok((None, content.to_string()))
    }
}

/// Find all skill directories in a base path
fn find_skill_directories(base_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut skill_dirs = Vec::new();

    if !base_dir.exists() {
        return Ok(skill_dirs);
    }

    for entry in fs::read_dir(base_dir)? {
        let entry = entry?;
        let path = entry.path();

        // Skip hidden directories
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with('.') {
                continue;
            }
        }

        if path.is_dir() {
            // Check if this directory contains a SKILL.md file
            let skill_file = path.join("SKILL.md");
            if skill_file.exists() {
                skill_dirs.push(path);
            }
        }
    }

    Ok(skill_dirs)
}

/// Load a skill from a directory
fn load_skill_from_directory(skill_dir: &Path, scope: &str) -> Result<Skill> {
    debug!("Loading skill from: {:?}", skill_dir);

    // Get skill name from directory name
    let name = skill_dir
        .file_name()
        .and_then(|n| n.to_str())
        .context("Failed to get skill directory name")?
        .to_string();

    // Look for SKILL.md or SKILL.json
    let skill_md_path = skill_dir.join("SKILL.md");
    let skill_json_path = skill_dir.join("SKILL.json");

    let (file_path, file_type, content, frontmatter) = if skill_md_path.exists() {
        let raw_content = fs::read_to_string(&skill_md_path)
            .context("Failed to read SKILL.md")?;
        let (fm, body) = parse_markdown_with_frontmatter(&raw_content)?;
        (skill_md_path, "markdown".to_string(), body, fm)
    } else if skill_json_path.exists() {
        let raw_content = fs::read_to_string(&skill_json_path)
            .context("Failed to read SKILL.json")?;
        (skill_json_path, "json".to_string(), raw_content, None)
    } else {
        return Err(anyhow::anyhow!("No SKILL.md or SKILL.json found in {:?}", skill_dir));
    };

    // Find supporting files
    let mut supporting_files = Vec::new();
    for entry in fs::read_dir(skill_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            // Skip the main skill file
            if file_name == "SKILL.md" || file_name == "SKILL.json" {
                continue;
            }

            // Skip hidden files
            if file_name.starts_with('.') {
                continue;
            }

            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_string();

            supporting_files.push(SupportingFile {
                name: file_name,
                path: path.to_string_lossy().to_string(),
                file_type: ext,
            });
        }
    }

    // Extract metadata from frontmatter
    let (description, allowed_tools) = if let Some(fm) = frontmatter {
        (fm.description, fm.allowed_tools.unwrap_or_default())
    } else {
        (None, Vec::new())
    };

    // Generate unique ID
    let id = format!(
        "{}-{}",
        scope,
        skill_dir.to_string_lossy().replace('/', "-")
    );

    Ok(Skill {
        id,
        name,
        scope: scope.to_string(),
        dir_path: skill_dir.to_string_lossy().to_string(),
        file_path: file_path.to_string_lossy().to_string(),
        file_type,
        content,
        description,
        allowed_tools,
        supporting_files,
    })
}

/// List all skills
#[tauri::command]
pub async fn skills_list(project_path: Option<String>) -> Result<Vec<Skill>, String> {
    info!("Discovering skills");
    let mut skills = Vec::new();

    // Load project skills if project path is provided
    if let Some(proj_path) = project_path {
        let project_skills_dir = PathBuf::from(&proj_path).join(".claude").join("skills");
        if project_skills_dir.exists() {
            debug!("Scanning project skills at: {:?}", project_skills_dir);

            match find_skill_directories(&project_skills_dir) {
                Ok(skill_dirs) => {
                    for skill_dir in skill_dirs {
                        match load_skill_from_directory(&skill_dir, "project") {
                            Ok(skill) => {
                                debug!("Loaded project skill: {}", skill.name);
                                skills.push(skill);
                            }
                            Err(e) => {
                                error!("Failed to load skill from {:?}: {}", skill_dir, e);
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to find project skill directories: {}", e);
                }
            }
        }
    }

    // Load user skills
    if let Some(home_dir) = dirs::home_dir() {
        let user_skills_dir = home_dir.join(".claude").join("skills");
        if user_skills_dir.exists() {
            debug!("Scanning user skills at: {:?}", user_skills_dir);

            match find_skill_directories(&user_skills_dir) {
                Ok(skill_dirs) => {
                    for skill_dir in skill_dirs {
                        match load_skill_from_directory(&skill_dir, "user") {
                            Ok(skill) => {
                                debug!("Loaded user skill: {}", skill.name);
                                skills.push(skill);
                            }
                            Err(e) => {
                                error!("Failed to load skill from {:?}: {}", skill_dir, e);
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to find user skill directories: {}", e);
                }
            }
        }
    }

    info!("Found {} skills", skills.len());
    Ok(skills)
}

/// Get a single skill by ID
#[tauri::command]
pub async fn skill_get(skill_id: String, project_path: Option<String>) -> Result<Skill, String> {
    debug!("Getting skill: {}", skill_id);

    let skills = skills_list(project_path).await?;

    skills
        .into_iter()
        .find(|s| s.id == skill_id)
        .ok_or_else(|| format!("Skill not found: {}", skill_id))
}

/// Create or update a skill
#[tauri::command]
pub async fn skill_save(
    scope: String,
    name: String,
    content: String,
    file_type: String,
    description: Option<String>,
    allowed_tools: Vec<String>,
    project_path: Option<String>,
) -> Result<Skill, String> {
    info!("Saving skill: {} in scope: {}", name, scope);

    // Validate inputs
    if name.is_empty() {
        return Err("Skill name cannot be empty".to_string());
    }

    if !["project", "user"].contains(&scope.as_str()) {
        return Err("Invalid scope. Must be 'project' or 'user'".to_string());
    }

    if !["markdown", "json"].contains(&file_type.as_str()) {
        return Err("Invalid file type. Must be 'markdown' or 'json'".to_string());
    }

    // Determine base directory
    let base_dir = if scope == "project" {
        if let Some(proj_path) = project_path {
            PathBuf::from(proj_path).join(".claude").join("skills")
        } else {
            return Err("Project path required for project scope".to_string());
        }
    } else {
        dirs::home_dir()
            .ok_or_else(|| "Could not find home directory".to_string())?
            .join(".claude")
            .join("skills")
    };

    // Create skill directory
    let skill_dir = base_dir.join(&name);
    fs::create_dir_all(&skill_dir)
        .map_err(|e| format!("Failed to create skill directory: {}", e))?;

    // Determine file path
    let file_name = if file_type == "json" {
        "SKILL.json"
    } else {
        "SKILL.md"
    };
    let file_path = skill_dir.join(file_name);

    // Build content with frontmatter for markdown
    let full_content = if file_type == "markdown" && (description.is_some() || !allowed_tools.is_empty()) {
        let mut fm_content = String::from("---\n");

        if let Some(desc) = &description {
            fm_content.push_str(&format!("description: {}\n", desc));
        }

        if !allowed_tools.is_empty() {
            fm_content.push_str("allowed-tools:\n");
            for tool in &allowed_tools {
                fm_content.push_str(&format!("  - {}\n", tool));
            }
        }

        fm_content.push_str("---\n\n");
        fm_content.push_str(&content);
        fm_content
    } else {
        content.clone()
    };

    // Write file
    fs::write(&file_path, &full_content)
        .map_err(|e| format!("Failed to write skill file: {}", e))?;

    // Load and return the saved skill
    load_skill_from_directory(&skill_dir, &scope)
        .map_err(|e| format!("Failed to load saved skill: {}", e))
}

/// Delete a skill
#[tauri::command]
pub async fn skill_delete(
    skill_id: String,
    project_path: Option<String>,
) -> Result<String, String> {
    info!("Deleting skill: {}", skill_id);

    // Find the skill
    let skills = skills_list(project_path).await?;
    let skill = skills
        .into_iter()
        .find(|s| s.id == skill_id)
        .ok_or_else(|| format!("Skill not found: {}", skill_id))?;

    // Delete the entire skill directory
    fs::remove_dir_all(&skill.dir_path)
        .map_err(|e| format!("Failed to delete skill directory: {}", e))?;

    Ok(format!("Deleted skill: {}", skill.name))
}

/// Read a supporting file from a skill
#[tauri::command]
pub async fn skill_read_file(file_path: String) -> Result<String, String> {
    fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read file: {}", e))
}

/// Save a supporting file for a skill
#[tauri::command]
pub async fn skill_save_file(
    skill_dir: String,
    file_name: String,
    content: String,
) -> Result<String, String> {
    let file_path = PathBuf::from(&skill_dir).join(&file_name);

    fs::write(&file_path, &content)
        .map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(file_path.to_string_lossy().to_string())
}

/// Delete a supporting file from a skill
#[tauri::command]
pub async fn skill_delete_file(file_path: String) -> Result<String, String> {
    fs::remove_file(&file_path)
        .map_err(|e| format!("Failed to delete file: {}", e))?;

    Ok(format!("Deleted file: {}", file_path))
}
