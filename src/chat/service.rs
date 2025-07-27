use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

use crate::core::error::{AppError, AppResult};
use crate::core::types::{
    Chat, Message, CreateChatRequest, SendMessageRequest
};
use crate::crypto::service::CryptoService;
use crate::matrix::client::MatrixClient;
use crate::storage::database::Database;

pub struct ChatService {
    db: Arc<Database>,
    matrix_client: Arc<MatrixClient>,
    crypto: Arc<CryptoService>,
}

impl ChatService {
    pub fn new(
        db: Arc<Database>,
        matrix_client: Arc<MatrixClient>,
        crypto: Arc<CryptoService>,
    ) -> Self {
        Self {
            db,
            matrix_client,
            crypto,
        }
    }

    /// Create a new chat (direct message or group)
    pub async fn create_chat(&self, request: CreateChatRequest, creator_id: Uuid) -> AppResult<Chat> {
        // For direct messages, check if chat already exists
        if !request.is_group && request.participants.len() == 1 {
            let other_user_id = request.participants[0];
            
            // Check if DM already exists between these users
            let existing_chat = sqlx::query!(
                r#"
                SELECT c.id, c.name, c.matrix_room_id, c.is_group, c.is_encrypted, c.created_at, c.created_by
                FROM chats c
                JOIN chat_participants cp1 ON c.id = cp1.chat_id
                JOIN chat_participants cp2 ON c.id = cp2.chat_id
                WHERE c.is_group = false 
                AND cp1.user_id = ? AND cp2.user_id = ?
                "#,
                creator_id.to_string(),
                other_user_id.to_string()
            )
            .fetch_optional(self.db.pool())
            .await?;

            if let Some(existing) = existing_chat {
                return Ok(Chat {
                    id: Uuid::parse_str(&existing.id)
                        .map_err(|e| AppError::Internal(format!("Invalid chat ID: {}", e)))?,
                    name: existing.name,
                    matrix_room_id: existing.matrix_room_id,
                    is_group: existing.is_group,
                    is_encrypted: existing.is_encrypted,
                    created_at: chrono::DateTime::parse_from_rfc3339(&existing.created_at)
                        .map_err(|e| AppError::Internal(format!("Invalid date: {}", e)))?
                        .with_timezone(&Utc),
                    created_by: Uuid::parse_str(&existing.created_by)
                        .map_err(|e| AppError::Internal(format!("Invalid user ID: {}", e)))?,
                });
            }
        }

        // Create Matrix room
        let matrix_room_id = if request.is_group {
            let chat_name = request.name.as_deref().unwrap_or("Group Chat");
            self.matrix_client
                .create_room(chat_name, None, true) // Group chats are encrypted
                .await?
        } else {
            // For DMs, get the other user's Matrix ID and create DM
            let other_user_record = sqlx::query!(
                "SELECT matrix_user_id FROM users WHERE id = ?",
                request.participants[0].to_string()
            )
            .fetch_one(self.db.pool())
            .await
            .map_err(|_| AppError::NotFound("User not found".to_string()))?;

            self.matrix_client
                .create_dm(&other_user_record.matrix_user_id)
                .await?
        };

        let chat_id = Uuid::new_v4();
        let now = Utc::now();

        // Insert chat into database
        sqlx::query!(
            r#"
            INSERT INTO chats (id, name, matrix_room_id, is_group, is_encrypted, created_at, created_by)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
            chat_id.to_string(),
            request.name,
            matrix_room_id,
            request.is_group,
            true, // All chats are encrypted
            now.to_rfc3339(),
            creator_id.to_string()
        )
        .execute(self.db.pool())
        .await?;

        // Add creator as participant
        sqlx::query!(
            "INSERT INTO chat_participants (chat_id, user_id, is_admin) VALUES (?, ?, ?)",
            chat_id.to_string(),
            creator_id.to_string(),
            true // Creator is admin
        )
        .execute(self.db.pool())
        .await?;

        // Add other participants
        for participant_id in &request.participants {
            if *participant_id != creator_id {
                sqlx::query!(
                    "INSERT INTO chat_participants (chat_id, user_id, is_admin) VALUES (?, ?, ?)",
                    chat_id.to_string(),
                    participant_id.to_string(),
                    false
                )
                .execute(self.db.pool())
                .await?;

                // Invite user to Matrix room
                let user_record = sqlx::query!(
                    "SELECT matrix_user_id FROM users WHERE id = ?",
                    participant_id.to_string()
                )
                .fetch_one(self.db.pool())
                .await?;

                self.matrix_client
                    .invite_user(&matrix_room_id, &user_record.matrix_user_id)
                    .await?;
            }
        }

        Ok(Chat {
            id: chat_id,
            name: request.name,
            matrix_room_id,
            is_group: request.is_group,
            is_encrypted: true,
            created_at: now,
            created_by: creator_id,
        })
    }

    /// Get user's chats
    pub async fn get_user_chats(&self, user_id: Uuid) -> AppResult<Vec<Chat>> {
        let chat_records = sqlx::query!(
            r#"
            SELECT c.id, c.name, c.matrix_room_id, c.is_group, c.is_encrypted, c.created_at, c.created_by
            FROM chats c
            JOIN chat_participants cp ON c.id = cp.chat_id
            WHERE cp.user_id = ?
            ORDER BY c.created_at DESC
            "#,
            user_id.to_string()
        )
        .fetch_all(self.db.pool())
        .await?;

        let chats = chat_records
            .into_iter()
            .map(|record| {
                Ok(Chat {
                    id: Uuid::parse_str(&record.id)
                        .map_err(|e| AppError::Internal(format!("Invalid chat ID: {}", e)))?,
                    name: record.name,
                    matrix_room_id: record.matrix_room_id,
                    is_group: record.is_group,
                    is_encrypted: record.is_encrypted,
                    created_at: chrono::DateTime::parse_from_rfc3339(&record.created_at)
                        .map_err(|e| AppError::Internal(format!("Invalid date: {}", e)))?
                        .with_timezone(&Utc),
                    created_by: Uuid::parse_str(&record.created_by)
                        .map_err(|e| AppError::Internal(format!("Invalid user ID: {}", e)))?,
                })
            })
            .collect::<AppResult<Vec<_>>>()?;

        Ok(chats)
    }

    /// Get a specific chat
    pub async fn get_chat(&self, chat_id: Uuid, user_id: Uuid) -> AppResult<Chat> {
        // Verify user is a participant
        let participant = sqlx::query!(
            "SELECT user_id FROM chat_participants WHERE chat_id = ? AND user_id = ?",
            chat_id.to_string(),
            user_id.to_string()
        )
        .fetch_optional(self.db.pool())
        .await?;

        if participant.is_none() {
            return Err(AppError::Authorization("Not a member of this chat".to_string()));
        }

        let chat_record = sqlx::query!(
            "SELECT id, name, matrix_room_id, is_group, is_encrypted, created_at, created_by FROM chats WHERE id = ?",
            chat_id.to_string()
        )
        .fetch_optional(self.db.pool())
        .await?
        .ok_or_else(|| AppError::NotFound("Chat not found".to_string()))?;

        Ok(Chat {
            id: chat_id,
            name: chat_record.name,
            matrix_room_id: chat_record.matrix_room_id,
            is_group: chat_record.is_group,
            is_encrypted: chat_record.is_encrypted,
            created_at: chrono::DateTime::parse_from_rfc3339(&chat_record.created_at)
                .map_err(|e| AppError::Internal(format!("Invalid date: {}", e)))?
                .with_timezone(&Utc),
            created_by: Uuid::parse_str(&chat_record.created_by)
                .map_err(|e| AppError::Internal(format!("Invalid user ID: {}", e)))?,
        })
    }

    /// Send a message to a chat
    pub async fn send_message(&self, chat_id: Uuid, request: SendMessageRequest, sender_id: Uuid) -> AppResult<Message> {
        // Get chat and verify user is a participant
        let chat = self.get_chat(chat_id, sender_id).await?;

        // Encrypt message content if it's an encrypted chat
        let content = if chat.is_encrypted {
            self.crypto.encrypt(&request.content)?
        } else {
            request.content.clone()
        };

        // Send to Matrix room
        let matrix_event_id = self.matrix_client
            .send_message(&chat.matrix_room_id, &request.content) // Send unencrypted to Matrix (Matrix handles its own encryption)
            .await?;

        let message_id = Uuid::new_v4();
        let now = Utc::now();

        // Insert message into database (store encrypted content)
        sqlx::query!(
            r#"
            INSERT INTO messages (id, chat_id, content, message_type, matrix_event_id, reply_to, is_encrypted, created_at, created_by)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            message_id.to_string(),
            chat_id.to_string(),
            content, // This is encrypted if chat.is_encrypted is true
            format!("{:?}", request.message_type).to_lowercase(),
            matrix_event_id,
            request.reply_to.map(|id| id.to_string()),
            chat.is_encrypted,
            now.to_rfc3339(),
            sender_id.to_string()
        )
        .execute(self.db.pool())
        .await?;

        Ok(Message {
            id: message_id,
            chat_id,
            content: request.content, // Return original unencrypted content
            message_type: request.message_type,
            matrix_event_id,
            reply_to: request.reply_to,
            is_encrypted: chat.is_encrypted,
            created_at: now,
            created_by: sender_id,
        })
    }

    /// Get messages from a chat
    pub async fn get_messages(&self, chat_id: Uuid, user_id: Uuid, limit: Option<i64>, offset: Option<i64>) -> AppResult<Vec<Message>> {
        // Verify user is a participant
        self.get_chat(chat_id, user_id).await?;

        let limit = limit.unwrap_or(50).min(100); // Max 100 messages per request
        let offset = offset.unwrap_or(0);

        let message_records = sqlx::query!(
            r#"
            SELECT id, chat_id, content, message_type, matrix_event_id, reply_to, is_encrypted, created_at, created_by
            FROM messages 
            WHERE chat_id = ? 
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
            "#,
            chat_id.to_string(),
            limit,
            offset
        )
        .fetch_all(self.db.pool())
        .await?;

        let messages = message_records
            .into_iter()
            .map(|record| {
                // Decrypt content if it's encrypted
                let content = if record.is_encrypted {
                    self.crypto.decrypt(&record.content).unwrap_or_else(|_| "[Encrypted]".to_string())
                } else {
                    record.content
                };

                let message_type = match record.message_type.as_str() {
                    "text" => crate::core::types::MessageType::Text,
                    "image" => crate::core::types::MessageType::Image,
                    "file" => crate::core::types::MessageType::File,
                    "audio" => crate::core::types::MessageType::Audio,
                    "video" => crate::core::types::MessageType::Video,
                    _ => crate::core::types::MessageType::Text,
                };

                Ok(Message {
                    id: Uuid::parse_str(&record.id)
                        .map_err(|e| AppError::Internal(format!("Invalid message ID: {}", e)))?,
                    chat_id: Uuid::parse_str(&record.chat_id)
                        .map_err(|e| AppError::Internal(format!("Invalid chat ID: {}", e)))?,
                    content,
                    message_type,
                    matrix_event_id: record.matrix_event_id,
                    reply_to: record.reply_to.as_ref().map(|id| {
                        Uuid::parse_str(id)
                            .map_err(|e| AppError::Internal(format!("Invalid reply_to ID: {}", e)))
                            .unwrap()
                    }),
                    is_encrypted: record.is_encrypted,
                    created_at: chrono::DateTime::parse_from_rfc3339(&record.created_at)
                        .map_err(|e| AppError::Internal(format!("Invalid date: {}", e)))?
                        .with_timezone(&Utc),
                    created_by: Uuid::parse_str(&record.created_by)
                        .map_err(|e| AppError::Internal(format!("Invalid user ID: {}", e)))?,
                })
            })
            .collect::<AppResult<Vec<_>>>()?;

        Ok(messages)
    }

    /// Add a user to a group chat
    pub async fn add_user_to_chat(&self, chat_id: Uuid, user_id: Uuid, admin_id: Uuid) -> AppResult<()> {
        // Verify admin is a member and has admin privileges
        let admin_participant = sqlx::query!(
            "SELECT is_admin FROM chat_participants WHERE chat_id = ? AND user_id = ?",
            chat_id.to_string(),
            admin_id.to_string()
        )
        .fetch_optional(self.db.pool())
        .await?
        .ok_or_else(|| AppError::Authorization("Not a member of this chat".to_string()))?;

        if !admin_participant.is_admin {
            return Err(AppError::Authorization("Admin privileges required".to_string()));
        }

        // Check if user is already a member
        let existing_participant = sqlx::query!(
            "SELECT user_id FROM chat_participants WHERE chat_id = ? AND user_id = ?",
            chat_id.to_string(),
            user_id.to_string()
        )
        .fetch_optional(self.db.pool())
        .await?;

        if existing_participant.is_some() {
            return Err(AppError::InvalidRequest("User is already a member".to_string()));
        }

        // Get chat info
        let chat = self.get_chat(chat_id, admin_id).await?;

        // Add user to database
        sqlx::query!(
            "INSERT INTO chat_participants (chat_id, user_id, is_admin) VALUES (?, ?, ?)",
            chat_id.to_string(),
            user_id.to_string(),
            false
        )
        .execute(self.db.pool())
        .await?;

        // Invite user to Matrix room
        let user_record = sqlx::query!(
            "SELECT matrix_user_id FROM users WHERE id = ?",
            user_id.to_string()
        )
        .fetch_one(self.db.pool())
        .await?;

        self.matrix_client
            .invite_user(&chat.matrix_room_id, &user_record.matrix_user_id)
            .await?;

        Ok(())
    }

    /// Remove a user from a group chat
    pub async fn remove_user_from_chat(&self, chat_id: Uuid, user_id: Uuid, admin_id: Uuid) -> AppResult<()> {
        // Verify admin is a member and has admin privileges
        let admin_participant = sqlx::query!(
            "SELECT is_admin FROM chat_participants WHERE chat_id = ? AND user_id = ?",
            chat_id.to_string(),
            admin_id.to_string()
        )
        .fetch_optional(self.db.pool())
        .await?
        .ok_or_else(|| AppError::Authorization("Not a member of this chat".to_string()))?;

        if !admin_participant.is_admin {
            return Err(AppError::Authorization("Admin privileges required".to_string()));
        }

        // Remove user from database
        sqlx::query!(
            "DELETE FROM chat_participants WHERE chat_id = ? AND user_id = ?",
            chat_id.to_string(),
            user_id.to_string()
        )
        .execute(self.db.pool())
        .await?;

        // Note: Matrix room removal would require additional Matrix SDK calls
        // For now, we just remove from our database

        Ok(())
    }
}