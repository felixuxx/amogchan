use chrono::{Duration, Utc};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::config::SecurityConfig;
use crate::core::error::{AppError, AppResult};
use crate::core::types::{User, CreateUserRequest, LoginRequest};
use crate::crypto::service::CryptoService;
use crate::storage::database::Database;

pub struct AuthService {
    db: Arc<Database>,
    crypto: Arc<CryptoService>,
    config: SecurityConfig,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub expires_at: chrono::DateTime<Utc>,
}

impl AuthService {
    pub fn new(
        db: Arc<Database>,
        crypto: Arc<CryptoService>,
        config: SecurityConfig,
    ) -> Self {
        Self { db, crypto, config }
    }

    /// Register a new user
    pub async fn register(&self, request: CreateUserRequest) -> AppResult<User> {
        // Check if username is already taken
        let existing_user = sqlx::query!(
            "SELECT id FROM users WHERE username = ?",
            request.username
        )
        .fetch_optional(self.db.pool())
        .await?;

        if existing_user.is_some() {
            return Err(AppError::InvalidRequest("Username already taken".to_string()));
        }

        // Check if email is already taken (if provided)
        if let Some(ref email) = request.email {
            let existing_email = sqlx::query!(
                "SELECT id FROM users WHERE email = ?",
                email
            )
            .fetch_optional(self.db.pool())
            .await?;

            if existing_email.is_some() {
                return Err(AppError::InvalidRequest("Email already registered".to_string()));
            }
        }

        // Hash password
        let password_hash = if !request.is_anonymous {
            Some(self.crypto.hash_password(&request.password)?)
        } else {
            None
        };

        // Generate Matrix user ID
        let matrix_user_id = if request.is_anonymous {
            format!("@anon_{}:matrix.org", Uuid::new_v4())
        } else {
            format!("@{}:matrix.org", request.username)
        };

        let user_id = Uuid::new_v4();
        let now = Utc::now();

        // Insert user into database
        sqlx::query!(
            r#"
            INSERT INTO users (id, username, email, password_hash, matrix_user_id, is_anonymous, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
            user_id.to_string(),
            request.username,
            request.email,
            password_hash,
            matrix_user_id,
            request.is_anonymous,
            now.to_rfc3339()
        )
        .execute(self.db.pool())
        .await?;

        Ok(User {
            id: user_id,
            username: request.username,
            email: request.email,
            matrix_user_id,
            avatar_url: None,
            is_anonymous: request.is_anonymous,
            created_at: now,
            last_seen: None,
        })
    }

    /// Login a user
    pub async fn login(&self, request: LoginRequest) -> AppResult<(User, Session)> {
        // Get user from database
        let user_record = sqlx::query!(
            "SELECT id, username, email, password_hash, matrix_user_id, avatar_url, is_anonymous, created_at, last_seen FROM users WHERE username = ?",
            request.username
        )
        .fetch_optional(self.db.pool())
        .await?
        .ok_or_else(|| AppError::Auth("Invalid credentials".to_string()))?;

        // Check password for non-anonymous users
        if !user_record.is_anonymous {
            let password_hash = user_record.password_hash
                .ok_or_else(|| AppError::Auth("Invalid credentials".to_string()))?;

            let is_valid = self.crypto.verify_password(&request.password, &password_hash)?;
            if !is_valid {
                return Err(AppError::Auth("Invalid credentials".to_string()));
            }
        }

        let user_id = Uuid::parse_str(&user_record.id)
            .map_err(|e| AppError::Internal(format!("Invalid user ID: {}", e)))?;

        // Create session
        let session = self.create_session(user_id).await?;

        // Update last seen
        let now = Utc::now();
        sqlx::query!(
            "UPDATE users SET last_seen = ? WHERE id = ?",
            now.to_rfc3339(),
            user_record.id
        )
        .execute(self.db.pool())
        .await?;

        let user = User {
            id: user_id,
            username: user_record.username,
            email: user_record.email,
            matrix_user_id: user_record.matrix_user_id,
            avatar_url: user_record.avatar_url,
            is_anonymous: user_record.is_anonymous,
            created_at: chrono::DateTime::parse_from_rfc3339(&user_record.created_at)
                .map_err(|e| AppError::Internal(format!("Invalid date: {}", e)))?
                .with_timezone(&Utc),
            last_seen: Some(now),
        };

        Ok((user, session))
    }

    /// Create a new session for a user
    pub async fn create_session(&self, user_id: Uuid) -> AppResult<Session> {
        let session_id = Uuid::new_v4();
        let token = self.crypto.generate_token()?;
        let token_hash = self.crypto.hash_data(&token);
        let expires_at = Utc::now() + Duration::days(30); // 30 days

        sqlx::query!(
            "INSERT INTO sessions (id, user_id, token_hash, expires_at) VALUES (?, ?, ?, ?)",
            session_id.to_string(),
            user_id.to_string(),
            token_hash,
            expires_at.to_rfc3339()
        )
        .execute(self.db.pool())
        .await?;

        Ok(Session {
            id: session_id,
            user_id,
            token,
            expires_at,
        })
    }

    /// Validate a session token
    pub async fn validate_session(&self, token: &str) -> AppResult<User> {
        let token_hash = self.crypto.hash_data(token);
        let now = Utc::now();

        let session_record = sqlx::query!(
            r#"
            SELECT s.user_id, u.username, u.email, u.matrix_user_id, u.avatar_url, u.is_anonymous, u.created_at, u.last_seen
            FROM sessions s
            JOIN users u ON s.user_id = u.id
            WHERE s.token_hash = ? AND s.expires_at > ?
            "#,
            token_hash,
            now.to_rfc3339()
        )
        .fetch_optional(self.db.pool())
        .await?
        .ok_or_else(|| AppError::Auth("Invalid or expired session".to_string()))?;

        let user_id = Uuid::parse_str(&session_record.user_id)
            .map_err(|e| AppError::Internal(format!("Invalid user ID: {}", e)))?;

        Ok(User {
            id: user_id,
            username: session_record.username,
            email: session_record.email,
            matrix_user_id: session_record.matrix_user_id,
            avatar_url: session_record.avatar_url,
            is_anonymous: session_record.is_anonymous,
            created_at: chrono::DateTime::parse_from_rfc3339(&session_record.created_at)
                .map_err(|e| AppError::Internal(format!("Invalid date: {}", e)))?
                .with_timezone(&Utc),
            last_seen: session_record.last_seen.as_ref().map(|s| {
                chrono::DateTime::parse_from_rfc3339(s)
                    .map_err(|e| AppError::Internal(format!("Invalid date: {}", e)))
                    .unwrap()
                    .with_timezone(&Utc)
            }),
        })
    }

    /// Logout a user (invalidate session)
    pub async fn logout(&self, token: &str) -> AppResult<()> {
        let token_hash = self.crypto.hash_data(token);

        sqlx::query!(
            "DELETE FROM sessions WHERE token_hash = ?",
            token_hash
        )
        .execute(self.db.pool())
        .await?;

        Ok(())
    }

    /// Get user by ID
    pub async fn get_user(&self, user_id: Uuid) -> AppResult<User> {
        let user_record = sqlx::query!(
            "SELECT username, email, matrix_user_id, avatar_url, is_anonymous, created_at, last_seen FROM users WHERE id = ?",
            user_id.to_string()
        )
        .fetch_optional(self.db.pool())
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        Ok(User {
            id: user_id,
            username: user_record.username,
            email: user_record.email,
            matrix_user_id: user_record.matrix_user_id,
            avatar_url: user_record.avatar_url,
            is_anonymous: user_record.is_anonymous,
            created_at: chrono::DateTime::parse_from_rfc3339(&user_record.created_at)
                .map_err(|e| AppError::Internal(format!("Invalid date: {}", e)))?
                .with_timezone(&Utc),
            last_seen: user_record.last_seen.as_ref().map(|s| {
                chrono::DateTime::parse_from_rfc3339(s)
                    .map_err(|e| AppError::Internal(format!("Invalid date: {}", e)))
                    .unwrap()
                    .with_timezone(&Utc)
            }),
        })
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired_sessions(&self) -> AppResult<()> {
        let now = Utc::now();
        
        sqlx::query!(
            "DELETE FROM sessions WHERE expires_at < ?",
            now.to_rfc3339()
        )
        .execute(self.db.pool())
        .await?;

        Ok(())
    }
}