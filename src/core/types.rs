use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub matrix_user_id: String,
    pub avatar_url: Option<String>,
    pub is_anonymous: bool,
    pub created_at: DateTime<Utc>,
    pub last_seen: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Board {
    pub id: Uuid,
    pub name: String,
    pub title: String,
    pub description: Option<String>,
    pub matrix_room_id: String,
    pub is_nsfw: bool,
    pub is_private: bool,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Thread {
    pub id: Uuid,
    pub board_id: Uuid,
    pub title: Option<String>,
    pub content: String,
    pub image_url: Option<String>,
    pub matrix_event_id: String,
    pub is_pinned: bool,
    pub is_locked: bool,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub reply_count: i32,
    pub last_reply_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Post {
    pub id: Uuid,
    pub thread_id: Option<Uuid>,
    pub board_id: Uuid,
    pub content: String,
    pub image_url: Option<String>,
    pub matrix_event_id: String,
    pub reply_to: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Chat {
    pub id: Uuid,
    pub name: Option<String>,
    pub matrix_room_id: String,
    pub is_group: bool,
    pub is_encrypted: bool,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Message {
    pub id: Uuid,
    pub chat_id: Uuid,
    pub content: String,
    pub message_type: MessageType,
    pub matrix_event_id: String,
    pub reply_to: Option<Uuid>,
    pub is_encrypted: bool,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "message_type", rename_all = "lowercase")]
pub enum MessageType {
    Text,
    Image,
    File,
    Audio,
    Video,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: Option<String>,
    pub password: String,
    pub is_anonymous: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBoardRequest {
    pub name: String,
    pub title: String,
    pub description: Option<String>,
    pub is_nsfw: bool,
    pub is_private: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateThreadRequest {
    pub title: Option<String>,
    pub content: String,
    pub image_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePostRequest {
    pub content: String,
    pub image_url: Option<String>,
    pub reply_to: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChatRequest {
    pub name: Option<String>,
    pub is_group: bool,
    pub participants: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
    pub message_type: MessageType,
    pub reply_to: Option<Uuid>,
}