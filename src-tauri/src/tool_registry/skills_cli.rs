use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsSearchResult {
    pub skills: Vec<SkillInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInfo {
    pub name: String,
    pub description: String,
    pub source: String,
    pub version: String,
    pub author: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInstallProgress {
    pub stage: String,
    pub message: String,
    pub progress: i32,
}

pub struct SkillsCLI;

impl SkillsCLI {
    pub fn new() -> Self {
        Self
    }

    pub async fn search(&self, query: &str) -> Result<SkillsSearchResult, String> {
        let output = Command::new("npx")
            .args(["skills", "search", query, "--json"
            ])
            .output()
            .await
            .map_err(|e| format!("Failed to execute skills search: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("skills search failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let skills: Vec<SkillInfo> = serde_json::from_str(&stdout)
            .map_err(|e| format!("Failed to parse skills output: {}", e))?;

        Ok(SkillsSearchResult { skills })
    }

    pub async fn install(
        &self,
        source: &str,
        name: &str,
        scope: &str,
        progress_tx: mpsc::Sender<SkillInstallProgress>,
    ) -> Result<String, String> {
        let scope_flag = if scope == "global" {
            "-g"
        } else {
            "--project"
        };

        progress_tx
            .send(SkillInstallProgress {
                stage: "prepare".to_string(),
                message: format!("Installing {} from {}", name, source),
                progress: 0,
            })
            .await
            .map_err(|_| "Failed to send progress")?;

        let mut cmd = Command::new("npx");
        cmd.args(["skills", "install", &format!("{}/{}", source, name), scope_flag]);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn skills install: {}", e))?;

        progress_tx
            .send(SkillInstallProgress {
                stage: "installing".to_string(),
                message: "Downloading and installing skill...".to_string(),
                progress: 25,
            })
            .await
            .map_err(|_| "Failed to send progress")?;

        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                progress_tx
                    .send(SkillInstallProgress {
                        stage: "installing".to_string(),
                        message: line.clone(),
                        progress: 50,
                    })
                    .await
                    .map_err(|_| "Failed to send progress")?;
            }
        }

        progress_tx
            .send(SkillInstallProgress {
                stage: "finalizing".to_string(),
                message: "Finalizing installation...".to_string(),
                progress: 75,
            })
            .await
            .map_err(|_| "Failed to send progress")?;

        let exit_status = child
            .wait()
            .await
            .map_err(|e| format!("Failed to wait for skills install: {}", e))?;

        if !exit_status.success() {
            return Err(format!("Skills install failed with exit code: {:?}",
                exit_status.code()
            ));
        }

        progress_tx
            .send(SkillInstallProgress {
                stage: "complete".to_string(),
                message: format!("Successfully installed {}", name),
                progress: 100,
            })
            .await
            .map_err(|_| "Failed to send progress")?;

        Ok(format!("Skill {} installed successfully", name))
    }

    pub async fn uninstall(
        &self,
        name: &str,
        scope: &str,
        progress_tx: mpsc::Sender<SkillInstallProgress>,
    ) -> Result<String, String> {
        let scope_flag = if scope == "global" {
            "-g"
        } else {
            "--project"
        };

        progress_tx
            .send(SkillInstallProgress {
                stage: "prepare".to_string(),
                message: format!("Uninstalling {}", name),
                progress: 0,
            })
            .await
            .map_err(|_| "Failed to send progress")?;

        let output = Command::new("npx")
            .args(["skills", "uninstall", name, scope_flag
            ])
            .output()
            .await
            .map_err(|e| format!("Failed to execute skills uninstall: {}", e))?;

        progress_tx
            .send(SkillInstallProgress {
                stage: "uninstalling".to_string(),
                message: "Removing skill...".to_string(),
                progress: 50,
            })
            .await
            .map_err(|_| "Failed to send progress")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Skills uninstall failed: {}", stderr));
        }

        progress_tx
            .send(SkillInstallProgress {
                stage: "complete".to_string(),
                message: format!("Successfully uninstalled {}", name),
                progress: 100,
            })
            .await
            .map_err(|_| "Failed to send progress")?;

        Ok(format!("Skill {} uninstalled successfully", name))
    }

    pub async fn list_installed(&self, scope: Option<&str>) -> Result<Vec<SkillInfo>, String> {
        let mut args = vec!["skills", "list", "--json"];

        if let Some(s) = scope {
            if s == "global" {
                args.push("-g");
            } else if s == "project" {
                args.push("--project");
            }
        }

        let output = Command::new("npx")
            .args(&args)
            .output()
            .await
            .map_err(|e| format!("Failed to execute skills list: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Skills list failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let skills: Vec<SkillInfo> = serde_json::from_str(&stdout)
            .map_err(|e| format!("Failed to parse skills list output: {}", e))?;

        Ok(skills)
    }

    pub async fn check_updates(&self) -> Result<Vec<SkillInfo>, String> {
        let output = Command::new("npx")
            .args(["skills", "outdated", "--json"
            ])
            .output()
            .await
            .map_err(|e| format!("Failed to execute skills outdated: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Skills outdated check failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let skills: Vec<SkillInfo> = serde_json::from_str(&stdout)
            .map_err(|e| format!("Failed to parse skills outdated output: {}", e))?;

        Ok(skills)
    }
}
