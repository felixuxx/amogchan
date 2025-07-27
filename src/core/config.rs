use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub matrix: MatrixConfig,
    pub database: DatabaseConfig,
    pub crypto: CryptoConfig,
    pub security: SecurityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatrixConfig {
    pub homeserver_url: String,
    pub user_id: String,
    pub access_token: Option<String>,
    pub device_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoConfig {
    pub encryption_key: String,
    pub signing_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub session_secret: String,
    pub bcrypt_cost: u32,
    pub rate_limit_per_minute: u32,
}

impl Config {
    pub async fn load() -> Result<Self> {
        let config = Config {
            server: ServerConfig {
                host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("PORT")
                    .unwrap_or_else(|_| "3000".to_string())
                    .parse()
                    .unwrap_or(3000),
                base_url: env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string()),
            },
            matrix: MatrixConfig {
                homeserver_url: env::var("MATRIX_HOMESERVER_URL")
                    .unwrap_or_else(|_| "https://matrix.org".to_string()),
                user_id: env::var("MATRIX_USER_ID")
                    .unwrap_or_else(|_| "@bot:matrix.org".to_string()),
                access_token: env::var("MATRIX_ACCESS_TOKEN").ok(),
                device_id: env::var("MATRIX_DEVICE_ID").ok(),
            },
            database: DatabaseConfig {
                url: env::var("DATABASE_URL")
                    .unwrap_or_else(|_| "sqlite:./data.db".to_string()),
                max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap_or(10),
            },
            crypto: CryptoConfig {
                encryption_key: env::var("ENCRYPTION_KEY")
                    .unwrap_or_else(|_| "your-32-char-encryption-key-here".to_string()),
                signing_key: env::var("SIGNING_KEY")
                    .unwrap_or_else(|_| "your-signing-key-here".to_string()),
            },
            security: SecurityConfig {
                session_secret: env::var("SESSION_SECRET")
                    .unwrap_or_else(|_| "your-session-secret-here".to_string()),
                bcrypt_cost: env::var("BCRYPT_COST")
                    .unwrap_or_else(|_| "12".to_string())
                    .parse()
                    .unwrap_or(12),
                rate_limit_per_minute: env::var("RATE_LIMIT_PER_MINUTE")
                    .unwrap_or_else(|_| "60".to_string())
                    .parse()
                    .unwrap_or(60),
            },
        };

        Ok(config)
    }
}