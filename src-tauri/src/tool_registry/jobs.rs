use crate::tool_registry::{
    AsyncJob, JobStatus, JobType,
};
use async_trait::async_trait;
use rusqlite::{params, Connection};
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use std::collections::HashMap;

#[async_trait]
pub trait JobManager: Send + Sync {
    async fn create_job(
        &self,
        job_type: JobType,
        params: serde_json::Value,
    ) -> Result<String, String>;

    async fn start_job(&self,
        job_id: &str,
    ) -> Result<(), String>;

    async fn update_job_progress(
        &self,
        job_id: &str,
        progress: i32,
        stage: &str,
        message: &str,
    ) -> Result<(), String>;

    async fn complete_job(
        &self,
        job_id: &str,
        result: serde_json::Value,
    ) -> Result<(), String>;

    async fn fail_job(
        &self,
        job_id: &str,
        error_message: &str,
    ) -> Result<(), String>;

    async fn cancel_job(&self,
        job_id: &str,
    ) -> Result<(), String>;

    async fn get_job(&self,
        job_id: &str,
    ) -> Result<Option<AsyncJob>, String>;

    async fn list_jobs(
        &self,
        status: Option<JobStatus>,
    ) -> Result<Vec<AsyncJob>, String>;

    async fn list_pending_jobs(&self) -> Result<Vec<AsyncJob>, String>;

    async fn cleanup_completed_jobs(
        &self,
        older_than_hours: i64,
    ) -> Result<u32, String>;
}

pub struct JobManagerImpl {
    db: Arc<Mutex<Connection>>,
    active_jobs: Arc<Mutex<HashMap<String, JoinHandle<()>>>>,
    progress_tx: mpsc::Sender<JobProgressUpdate>,
}

#[derive(Clone)]
struct JobProgressUpdate {
    job_id: String,
    progress: i32,
    stage: String,
    message: String,
}

impl JobManagerImpl {
    pub fn new(db: Arc<Mutex<Connection>>) -> Result<Self, String> {
        let (progress_tx, mut progress_rx) = mpsc::channel::<JobProgressUpdate>(100);

        let manager = Self {
            db,
            active_jobs: Arc::new(Mutex::new(HashMap::new())),
            progress_tx,
        };

        let db_clone = manager.db.clone();
        tokio::spawn(async move {
            while let Some(update) = progress_rx.recv().await {
                if let Ok(conn) = db_clone.lock() {
                    let _ = conn.execute(
                        "UPDATE async_jobs SET progress = ?1 WHERE id = ?2",
                        params![update.progress, update.job_id],
                    );
                }
            }
        });

        Ok(manager)
    }

    fn generate_job_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    fn row_to_job(row: &rusqlite::Row) -> Result<AsyncJob, rusqlite::Error> {
        let status_str: String = row.get(2)?;
        let status = match status_str.as_str() {
            "pending" => JobStatus::Pending,
            "running" => JobStatus::Running,
            "completed" => JobStatus::Completed,
            "failed" => JobStatus::Failed,
            "cancelled" => JobStatus::Cancelled,
            _ => JobStatus::Pending,
        };

        let job_type_str: String = row.get(1)?;
        let job_type = match job_type_str.as_str() {
            "skills_install" => JobType::SkillsInstall,
            "skills_uninstall" => JobType::SkillsUninstall,
            "tool_validation" => JobType::ToolValidation,
            "mcp_sync" => JobType::MCPSync,
            "config_sync" => JobType::ConfigSync,
            _ => JobType::ConfigSync,
        };

        let params_str: Option<String> = row.get(3)?;
        let params = params_str.and_then(|s| serde_json::from_str(&s).ok());

        let result_str: Option<String> = row.get(5)?;
        let result = result_str.and_then(|s| serde_json::from_str(&s).ok());

        Ok(AsyncJob {
            id: row.get(0)?,
            job_type,
            status,
            params,
            progress: row.get(4)?,
            result,
            error_message: row.get(6)?,
            created_at: row.get(7)?,
            started_at: row.get(8)?,
            completed_at: row.get(9)?,
            cancelled_at: row.get(10)?,
        })
    }
}

#[async_trait]
impl JobManager for JobManagerImpl {
    async fn create_job(
        &self,
        job_type: JobType,
        params: serde_json::Value,
    ) -> Result<String, String> {
        let job_id = Self::generate_job_id();
        let job_type_str = format!("{:?}", job_type).to_lowercase();
        let params_str = serde_json::to_string(&params).map_err(|e| e.to_string())?;

        let conn = self.db.lock().map_err(|_| "Lock error")?;

        conn.execute(
            "INSERT INTO async_jobs (id, job_type, status, params, progress, created_at)
             VALUES (?1, ?2, 'pending', ?3, 0, CURRENT_TIMESTAMP)",
            params![&job_id, &job_type_str, &params_str],
        ).map_err(|e| e.to_string())?;

        Ok(job_id)
    }

    async fn start_job(&self, job_id: &str) -> Result<(), String> {
        let conn = self.db.lock().map_err(|_| "Lock error")?;

        let rows_affected = conn.execute(
            "UPDATE async_jobs SET status = 'running', started_at = CURRENT_TIMESTAMP
             WHERE id = ?1 AND status = 'pending'",
            [job_id],
        ).map_err(|e| e.to_string())?;

        if rows_affected == 0 {
            return Err("Job not found or not in pending state".to_string());
        }

        Ok(())
    }

    async fn update_job_progress(
        &self,
        job_id: &str,
        progress: i32,
        stage: &str,
        message: &str,
    ) -> Result<(), String> {
        let update = JobProgressUpdate {
            job_id: job_id.to_string(),
            progress,
            stage: stage.to_string(),
            message: message.to_string(),
        };

        self.progress_tx.send(update).await.map_err(|_| "Failed to send progress update")?;

        Ok(())
    }

    async fn complete_job(
        &self,
        job_id: &str,
        result: serde_json::Value,
    ) -> Result<(), String> {
        let result_str = serde_json::to_string(&result).map_err(|e| e.to_string())?;

        let conn = self.db.lock().map_err(|_| "Lock error")?;

        conn.execute(
            "UPDATE async_jobs SET status = 'completed', result = ?1, completed_at = CURRENT_TIMESTAMP, progress = 100
             WHERE id = ?2",
            params![&result_str, job_id],
        ).map_err(|e| e.to_string())?;

        Ok(())
    }

    async fn fail_job(
        &self,
        job_id: &str,
        error_message: &str,
    ) -> Result<(), String> {
        let conn = self.db.lock().map_err(|_| "Lock error")?;

        conn.execute(
            "UPDATE async_jobs SET status = 'failed', error_message = ?1, completed_at = CURRENT_TIMESTAMP
             WHERE id = ?2",
            params![error_message, job_id],
        ).map_err(|e| e.to_string())?;

        Ok(())
    }

    async fn cancel_job(&self, job_id: &str) -> Result<(), String> {
        let conn = self.db.lock().map_err(|_| "Lock error")?;

        conn.execute(
            "UPDATE async_jobs SET status = 'cancelled', cancelled_at = CURRENT_TIMESTAMP
             WHERE id = ?1 AND status IN ('pending', 'running')",
            [job_id],
        ).map_err(|e| e.to_string())?;

        if let Ok(mut jobs) = self.active_jobs.lock() {
            if let Some(handle) = jobs.remove(job_id) {
                handle.abort();
            }
        }

        Ok(())
    }

    async fn get_job(&self, job_id: &str) -> Result<Option<AsyncJob>, String> {
        let conn = self.db.lock().map_err(|_| "Lock error")?;

        let result = conn.query_row(
            "SELECT id, job_type, status, params, progress, result, error_message, created_at, started_at, completed_at, cancelled_at
             FROM async_jobs WHERE id = ?1",
            [job_id],
            Self::row_to_job,
        );

        match result {
            Ok(job) => Ok(Some(job)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn list_jobs(
        &self,
        status: Option<JobStatus>,
    ) -> Result<Vec<AsyncJob>, String> {
        let conn = self.db.lock().map_err(|_| "Lock error")?;

        let query = if let Some(s) = status {
            let status_str = format!("{:?}", s).to_lowercase();
            format!(
                "SELECT id, job_type, status, params, progress, result, error_message, created_at, started_at, completed_at, cancelled_at
                 FROM async_jobs WHERE status = '{}' ORDER BY created_at DESC",
                status_str
            )
        } else {
            "SELECT id, job_type, status, params, progress, result, error_message, created_at, started_at, completed_at, cancelled_at
             FROM async_jobs ORDER BY created_at DESC".to_string()
        };

        let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;

        let jobs = stmt
            .query_map([], Self::row_to_job)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(jobs)
    }

    async fn list_pending_jobs(&self) -> Result<Vec<AsyncJob>, String> {
        self.list_jobs(Some(JobStatus::Pending)).await
    }

    async fn cleanup_completed_jobs(
        &self,
        older_than_hours: i64,
    ) -> Result<u32, String> {
        let conn = self.db.lock().map_err(|_| "Lock error")?;

        let rows_affected = conn.execute(
            "DELETE FROM async_jobs
             WHERE status IN ('completed', 'failed', 'cancelled')
             AND datetime(created_at) < datetime('now', ?1 || ' hours')",
            params![format!("-{}", older_than_hours)],
        ).map_err(|e| e.to_string())?;

        Ok(rows_affected as u32)
    }
}
