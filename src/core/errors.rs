use thiserror::Error;

#[derive(Debug, Error)]
pub enum LuDockError {
    #[error("Luau analysis failed with {0} errors")]
    AnalysisError(usize),

    #[error("DSL parsing failed: {0}")]
    DslError(String),

    #[error("World validation failed: {0}")]
    WorldError(String),

    #[error("Renderer failed: {0}")]
    RendererError(String),

    #[error("Configuration/Environment error: {0}")]
    ConfigError(String),

    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl LuDockError {
    pub fn exit_code(&self) -> i32 {
        match self {
            LuDockError::AnalysisError(_) => 1,
            LuDockError::DslError(_) | LuDockError::WorldError(_) => 2,
            LuDockError::RendererError(_) => 3,
            LuDockError::ConfigError(_) | LuDockError::IoError(_) => 4,
            LuDockError::Unknown(_) => 5,
        }
    }
}
