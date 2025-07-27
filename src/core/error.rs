use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Matrix error: {0}")]
    Matrix(String),
    
    #[error("Crypto error: {0}")]
    Crypto(String),
    
    #[error("Authentication error: {0}")]
    Auth(String),
    
    #[error("Authorization error: {0}")]
    Authorization(String),
    
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Rate limit exceeded")]
    RateLimit,
    
    #[error("Internal server error: {0}")]
    Internal(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl AppError {
    pub fn status_code(&self) -> u16 {
        match self {
            AppError::InvalidRequest(_) => 400,
            AppError::Auth(_) => 401,
            AppError::Authorization(_) => 403,
            AppError::NotFound(_) => 404,
            AppError::RateLimit => 429,
            _ => 500,
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;