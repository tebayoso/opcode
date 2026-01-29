use thiserror::Error;

#[derive(Error, Debug)]
pub enum RegistryError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
    
    #[error("Tool already exists: {0}")]
    ToolAlreadyExists(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Lock error")]
    LockError,
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("MCP server not found: {0}")]
    MCPServerNotFound(String),
    
    #[error("Job not found: {0}")]
    JobNotFound(String),
    
    #[error("Invalid tool specification: {0}")]
    InvalidSpec(String),
}

impl serde::Serialize for RegistryError {
    fn serialize<S>(&self,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
