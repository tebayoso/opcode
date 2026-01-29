use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Result of a file operation
#[derive(Debug, Serialize, Deserialize)]
pub struct OperationResult {
    pub success: bool,
    pub message: String,
    #[serde(rename = "affectedFiles")]
    pub affected_files: Vec<String>,
}

/// Merge strategy for markdown files
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MergeStrategy {
    Append,
    Prepend,
    SectionBased,
    Custom,
}

/// Options for merging markdown files
#[derive(Debug, Serialize, Deserialize)]
pub struct MergeOptions {
    pub strategy: MergeStrategy,
    #[serde(rename = "preserveHeaders")]
    pub preserve_headers: bool,
    #[serde(rename = "addSeparator")]
    pub add_separator: bool,
    pub separator: Option<String>,
}

impl Default for MergeOptions {
    fn default() -> Self {
        Self {
            strategy: MergeStrategy::Append,
            preserve_headers: true,
            add_separator: true,
            separator: Some("\n\n---\n\n".to_string()),
        }
    }
}

/// Preview of a merge operation
#[derive(Debug, Serialize, Deserialize)]
pub struct MergePreview {
    #[serde(rename = "sourceContent")]
    pub source_content: String,
    #[serde(rename = "targetContent")]
    pub target_content: String,
    #[serde(rename = "mergedContent")]
    pub merged_content: String,
    pub conflicts: Vec<MergeConflict>,
}

/// A conflict detected during merge
#[derive(Debug, Serialize, Deserialize)]
pub struct MergeConflict {
    #[serde(rename = "lineNumber")]
    pub line_number: usize,
    pub description: String,
    #[serde(rename = "sourceText")]
    pub source_text: String,
    #[serde(rename = "targetText")]
    pub target_text: String,
}

/// A single file operation for bulk operations
#[derive(Debug, Serialize, Deserialize)]
pub struct FileOperation {
    pub source: String,
    pub destination: String,
    #[serde(rename = "operationType")]
    pub operation_type: String, // "copy", "move", "merge"
}

/// Copy a file to a new location
#[tauri::command]
pub async fn file_ops_copy(
    source: String,
    destination: String,
    overwrite: bool,
) -> Result<OperationResult, String> {
    let source_path = Path::new(&source);
    let dest_path = Path::new(&destination);

    // Check if source exists
    if !source_path.exists() {
        return Ok(OperationResult {
            success: false,
            message: format!("Source file does not exist: {}", source),
            affected_files: vec![],
        });
    }

    // Check if destination exists and overwrite is not allowed
    if dest_path.exists() && !overwrite {
        return Ok(OperationResult {
            success: false,
            message: format!("Destination already exists: {}", destination),
            affected_files: vec![],
        });
    }

    // Create parent directories if needed
    if let Some(parent) = dest_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }

    // Create backup if overwriting
    let backup_path = if dest_path.exists() && overwrite {
        let backup = format!("{}.bak", destination);
        fs::copy(dest_path, &backup).map_err(|e| e.to_string())?;
        Some(backup)
    } else {
        None
    };

    // Perform copy
    match fs::copy(source_path, dest_path) {
        Ok(_) => {
            let mut affected = vec![destination.clone()];
            if let Some(backup) = backup_path {
                affected.push(backup);
            }
            Ok(OperationResult {
                success: true,
                message: format!("Successfully copied to {}", destination),
                affected_files: affected,
            })
        }
        Err(e) => Ok(OperationResult {
            success: false,
            message: format!("Failed to copy: {}", e),
            affected_files: vec![],
        }),
    }
}

/// Move a file to a new location
#[tauri::command]
pub async fn file_ops_move(
    source: String,
    destination: String,
    overwrite: bool,
) -> Result<OperationResult, String> {
    let source_path = Path::new(&source);
    let dest_path = Path::new(&destination);

    // Check if source exists
    if !source_path.exists() {
        return Ok(OperationResult {
            success: false,
            message: format!("Source file does not exist: {}", source),
            affected_files: vec![],
        });
    }

    // Check if destination exists and overwrite is not allowed
    if dest_path.exists() && !overwrite {
        return Ok(OperationResult {
            success: false,
            message: format!("Destination already exists: {}", destination),
            affected_files: vec![],
        });
    }

    // Create parent directories if needed
    if let Some(parent) = dest_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }

    // Create backup if overwriting
    let backup_path = if dest_path.exists() && overwrite {
        let backup = format!("{}.bak", destination);
        fs::copy(dest_path, &backup).map_err(|e| e.to_string())?;
        Some(backup)
    } else {
        None
    };

    // Perform move (rename)
    match fs::rename(source_path, dest_path) {
        Ok(_) => {
            let mut affected = vec![source.clone(), destination.clone()];
            if let Some(backup) = backup_path {
                affected.push(backup);
            }
            Ok(OperationResult {
                success: true,
                message: format!("Successfully moved to {}", destination),
                affected_files: affected,
            })
        }
        Err(e) => {
            // If rename fails (cross-device), try copy + delete
            match fs::copy(source_path, dest_path) {
                Ok(_) => {
                    fs::remove_file(source_path).map_err(|e| e.to_string())?;
                    let mut affected = vec![source.clone(), destination.clone()];
                    if let Some(backup) = backup_path {
                        affected.push(backup);
                    }
                    Ok(OperationResult {
                        success: true,
                        message: format!("Successfully moved to {}", destination),
                        affected_files: affected,
                    })
                }
                Err(copy_err) => Ok(OperationResult {
                    success: false,
                    message: format!("Failed to move: {} (copy fallback: {})", e, copy_err),
                    affected_files: vec![],
                }),
            }
        }
    }
}

/// Preview a markdown merge operation
#[tauri::command]
pub async fn file_ops_preview_merge(
    source: String,
    target: String,
    options: Option<MergeOptions>,
) -> Result<MergePreview, String> {
    let source_path = Path::new(&source);
    let target_path = Path::new(&target);

    let source_content = fs::read_to_string(source_path).map_err(|e| e.to_string())?;
    let target_content = if target_path.exists() {
        fs::read_to_string(target_path).map_err(|e| e.to_string())?
    } else {
        String::new()
    };

    let opts = options.unwrap_or_default();
    let merged_content = merge_markdown_content(&source_content, &target_content, &opts);
    let conflicts = detect_conflicts(&source_content, &target_content);

    Ok(MergePreview {
        source_content,
        target_content,
        merged_content,
        conflicts,
    })
}

/// Merge two markdown files
#[tauri::command]
pub async fn file_ops_merge_markdown(
    source: String,
    target: String,
    options: Option<MergeOptions>,
) -> Result<OperationResult, String> {
    let source_path = Path::new(&source);
    let target_path = Path::new(&target);

    // Read source content
    let source_content = fs::read_to_string(source_path).map_err(|e| e.to_string())?;

    // Read or create target content
    let target_content = if target_path.exists() {
        fs::read_to_string(target_path).map_err(|e| e.to_string())?
    } else {
        String::new()
    };

    // Create backup of target if it exists
    let backup_path = if target_path.exists() {
        let backup = format!("{}.bak", target);
        fs::copy(target_path, &backup).map_err(|e| e.to_string())?;
        Some(backup)
    } else {
        None
    };

    // Create parent directories if needed
    if let Some(parent) = target_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }

    // Merge content
    let opts = options.unwrap_or_default();
    let merged_content = merge_markdown_content(&source_content, &target_content, &opts);

    // Write merged content
    match fs::write(target_path, &merged_content) {
        Ok(_) => {
            let mut affected = vec![target.clone()];
            if let Some(backup) = backup_path {
                affected.push(backup);
            }
            Ok(OperationResult {
                success: true,
                message: format!("Successfully merged into {}", target),
                affected_files: affected,
            })
        }
        Err(e) => Ok(OperationResult {
            success: false,
            message: format!("Failed to write merged content: {}", e),
            affected_files: vec![],
        }),
    }
}

/// Execute multiple file operations in sequence
#[tauri::command]
pub async fn file_ops_bulk(
    operations: Vec<FileOperation>,
    stop_on_error: bool,
) -> Result<Vec<OperationResult>, String> {
    let mut results = Vec::new();

    for op in operations {
        let result = match op.operation_type.as_str() {
            "copy" => file_ops_copy(op.source, op.destination, false).await?,
            "move" => file_ops_move(op.source, op.destination, false).await?,
            _ => OperationResult {
                success: false,
                message: format!("Unknown operation type: {}", op.operation_type),
                affected_files: vec![],
            },
        };

        let failed = !result.success;
        results.push(result);

        if failed && stop_on_error {
            break;
        }
    }

    Ok(results)
}

/// Clone a skill by copying its files
#[tauri::command]
pub async fn file_ops_clone_skill(skill_name: String, new_name: String) -> Result<OperationResult, String> {
    let home = dirs::home_dir().ok_or("Could not find home directory")?;
    let skills_dir = home.join(".claude").join("commands");

    let source_path = skills_dir.join(format!("{}.md", skill_name));
    let dest_path = skills_dir.join(format!("{}.md", new_name));

    if !source_path.exists() {
        return Ok(OperationResult {
            success: false,
            message: format!("Skill not found: {}", skill_name),
            affected_files: vec![],
        });
    }

    if dest_path.exists() {
        return Ok(OperationResult {
            success: false,
            message: format!("Skill already exists: {}", new_name),
            affected_files: vec![],
        });
    }

    match fs::copy(&source_path, &dest_path) {
        Ok(_) => Ok(OperationResult {
            success: true,
            message: format!("Successfully cloned skill '{}' to '{}'", skill_name, new_name),
            affected_files: vec![dest_path.to_string_lossy().to_string()],
        }),
        Err(e) => Ok(OperationResult {
            success: false,
            message: format!("Failed to clone skill: {}", e),
            affected_files: vec![],
        }),
    }
}

/// Clone an agent by duplicating its database entry
#[tauri::command]
pub async fn file_ops_clone_agent(
    agent_id: i64,
    new_name: String,
    db: tauri::State<'_, crate::commands::agents::AgentDb>,
) -> Result<OperationResult, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // Get the original agent
    let agent: Result<(String, String, String, Option<String>, Option<String>, String), _> = conn.query_row(
        "SELECT name, description, system_prompt, allowed_tools, project_path, model FROM agents WHERE id = ?1",
        rusqlite::params![agent_id],
        |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
            ))
        },
    );

    match agent {
        Ok((_, description, system_prompt, allowed_tools, project_path, model)) => {
            // Insert the cloned agent
            let result = conn.execute(
                "INSERT INTO agents (name, description, system_prompt, allowed_tools, project_path, model, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now'), datetime('now'))",
                rusqlite::params![new_name, description, system_prompt, allowed_tools, project_path, model],
            );

            match result {
                Ok(_) => {
                    let new_id = conn.last_insert_rowid();
                    Ok(OperationResult {
                        success: true,
                        message: format!("Successfully cloned agent to '{}' (id: {})", new_name, new_id),
                        affected_files: vec![],
                    })
                }
                Err(e) => Ok(OperationResult {
                    success: false,
                    message: format!("Failed to clone agent: {}", e),
                    affected_files: vec![],
                }),
            }
        }
        Err(e) => Ok(OperationResult {
            success: false,
            message: format!("Agent not found: {}", e),
            affected_files: vec![],
        }),
    }
}

/// Delete a file with optional backup
#[tauri::command]
pub async fn file_ops_delete(path: String, create_backup: bool) -> Result<OperationResult, String> {
    let file_path = Path::new(&path);

    if !file_path.exists() {
        return Ok(OperationResult {
            success: false,
            message: format!("File does not exist: {}", path),
            affected_files: vec![],
        });
    }

    // Create backup if requested
    let backup_path = if create_backup {
        let backup = format!("{}.bak", path);
        fs::copy(file_path, &backup).map_err(|e| e.to_string())?;
        Some(backup)
    } else {
        None
    };

    match fs::remove_file(file_path) {
        Ok(_) => {
            let mut affected = vec![path.clone()];
            if let Some(backup) = backup_path {
                affected.push(backup);
            }
            Ok(OperationResult {
                success: true,
                message: format!("Successfully deleted {}", path),
                affected_files: affected,
            })
        }
        Err(e) => Ok(OperationResult {
            success: false,
            message: format!("Failed to delete: {}", e),
            affected_files: vec![],
        }),
    }
}

// Helper function to merge markdown content
fn merge_markdown_content(source: &str, target: &str, options: &MergeOptions) -> String {
    let separator = options
        .separator
        .clone()
        .unwrap_or_else(|| "\n\n---\n\n".to_string());

    match options.strategy {
        MergeStrategy::Append => {
            if target.is_empty() {
                source.to_string()
            } else if options.add_separator {
                format!("{}{}{}", target, separator, source)
            } else {
                format!("{}\n\n{}", target, source)
            }
        }
        MergeStrategy::Prepend => {
            if target.is_empty() {
                source.to_string()
            } else if options.add_separator {
                format!("{}{}{}", source, separator, target)
            } else {
                format!("{}\n\n{}", source, target)
            }
        }
        MergeStrategy::SectionBased => {
            // Simple section-based merge: combine unique sections
            let source_sections = extract_sections(source);
            let target_sections = extract_sections(target);

            let mut merged_sections = target_sections;
            for (header, content) in source_sections {
                if !merged_sections.contains_key(&header) {
                    merged_sections.insert(header, content);
                }
            }

            merged_sections
                .into_iter()
                .map(|(header, content)| {
                    if header.is_empty() {
                        content
                    } else {
                        format!("{}\n{}", header, content)
                    }
                })
                .collect::<Vec<_>>()
                .join("\n\n")
        }
        MergeStrategy::Custom => {
            // Custom strategy defaults to append for now
            if target.is_empty() {
                source.to_string()
            } else if options.add_separator {
                format!("{}{}{}", target, separator, source)
            } else {
                format!("{}\n\n{}", target, source)
            }
        }
    }
}

// Helper to extract markdown sections
fn extract_sections(content: &str) -> std::collections::HashMap<String, String> {
    let mut sections = std::collections::HashMap::new();
    let mut current_header = String::new();
    let mut current_content = String::new();

    for line in content.lines() {
        if line.starts_with('#') {
            if !current_header.is_empty() || !current_content.is_empty() {
                sections.insert(current_header.clone(), current_content.trim().to_string());
            }
            current_header = line.to_string();
            current_content = String::new();
        } else {
            current_content.push_str(line);
            current_content.push('\n');
        }
    }

    // Add the last section
    if !current_header.is_empty() || !current_content.is_empty() {
        sections.insert(current_header, current_content.trim().to_string());
    }

    sections
}

// Helper to detect potential conflicts
fn detect_conflicts(source: &str, target: &str) -> Vec<MergeConflict> {
    let mut conflicts = Vec::new();

    // Detect duplicate headers
    let source_headers: Vec<&str> = source
        .lines()
        .filter(|l| l.starts_with('#'))
        .collect();
    let target_headers: Vec<&str> = target
        .lines()
        .filter(|l| l.starts_with('#'))
        .collect();

    for (i, header) in source_headers.iter().enumerate() {
        if target_headers.contains(header) {
            conflicts.push(MergeConflict {
                line_number: i + 1,
                description: "Duplicate header found".to_string(),
                source_text: header.to_string(),
                target_text: header.to_string(),
            });
        }
    }

    conflicts
}
