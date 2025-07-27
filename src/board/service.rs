use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

use crate::core::error::{AppError, AppResult};
use crate::core::types::{
    Board, Thread, Post, CreateBoardRequest, CreateThreadRequest, CreatePostRequest
};
use crate::matrix::client::MatrixClient;
use crate::storage::database::Database;

pub struct BoardService {
    db: Arc<Database>,
    matrix_client: Arc<MatrixClient>,
}

impl BoardService {
    pub fn new(db: Arc<Database>, matrix_client: Arc<MatrixClient>) -> Self {
        Self { db, matrix_client }
    }

    /// Create a new board
    pub async fn create_board(&self, request: CreateBoardRequest, creator_id: Uuid) -> AppResult<Board> {
        // Check if board name is already taken
        let existing_board = sqlx::query!(
            "SELECT id FROM boards WHERE name = ?",
            request.name
        )
        .fetch_optional(self.db.pool())
        .await?;

        if existing_board.is_some() {
            return Err(AppError::InvalidRequest("Board name already taken".to_string()));
        }

        // Create Matrix room for the board
        let matrix_room_id = self.matrix_client
            .create_room(&request.title, request.description.as_deref(), false)
            .await?;

        let board_id = Uuid::new_v4();
        let now = Utc::now();

        // Insert board into database
        sqlx::query!(
            r#"
            INSERT INTO boards (id, name, title, description, matrix_room_id, is_nsfw, is_private, created_at, created_by)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            board_id.to_string(),
            request.name,
            request.title,
            request.description,
            matrix_room_id,
            request.is_nsfw,
            request.is_private,
            now.to_rfc3339(),
            creator_id.to_string()
        )
        .execute(self.db.pool())
        .await?;

        Ok(Board {
            id: board_id,
            name: request.name,
            title: request.title,
            description: request.description,
            matrix_room_id,
            is_nsfw: request.is_nsfw,
            is_private: request.is_private,
            created_at: now,
            created_by: creator_id,
        })
    }

    /// Get all boards
    pub async fn get_boards(&self) -> AppResult<Vec<Board>> {
        let board_records = sqlx::query!(
            "SELECT id, name, title, description, matrix_room_id, is_nsfw, is_private, created_at, created_by FROM boards ORDER BY created_at DESC"
        )
        .fetch_all(self.db.pool())
        .await?;

        let boards = board_records
            .into_iter()
            .map(|record| {
                Ok(Board {
                    id: Uuid::parse_str(&record.id)
                        .map_err(|e| AppError::Internal(format!("Invalid board ID: {}", e)))?,
                    name: record.name,
                    title: record.title,
                    description: record.description,
                    matrix_room_id: record.matrix_room_id,
                    is_nsfw: record.is_nsfw,
                    is_private: record.is_private,
                    created_at: chrono::DateTime::parse_from_rfc3339(&record.created_at)
                        .map_err(|e| AppError::Internal(format!("Invalid date: {}", e)))?
                        .with_timezone(&Utc),
                    created_by: Uuid::parse_str(&record.created_by)
                        .map_err(|e| AppError::Internal(format!("Invalid user ID: {}", e)))?,
                })
            })
            .collect::<AppResult<Vec<_>>>()?;

        Ok(boards)
    }

    /// Get a board by name
    pub async fn get_board(&self, name: &str) -> AppResult<Board> {
        let board_record = sqlx::query!(
            "SELECT id, name, title, description, matrix_room_id, is_nsfw, is_private, created_at, created_by FROM boards WHERE name = ?",
            name
        )
        .fetch_optional(self.db.pool())
        .await?
        .ok_or_else(|| AppError::NotFound("Board not found".to_string()))?;

        Ok(Board {
            id: Uuid::parse_str(&board_record.id)
                .map_err(|e| AppError::Internal(format!("Invalid board ID: {}", e)))?,
            name: board_record.name,
            title: board_record.title,
            description: board_record.description,
            matrix_room_id: board_record.matrix_room_id,
            is_nsfw: board_record.is_nsfw,
            is_private: board_record.is_private,
            created_at: chrono::DateTime::parse_from_rfc3339(&board_record.created_at)
                .map_err(|e| AppError::Internal(format!("Invalid date: {}", e)))?
                .with_timezone(&Utc),
            created_by: Uuid::parse_str(&board_record.created_by)
                .map_err(|e| AppError::Internal(format!("Invalid user ID: {}", e)))?,
        })
    }

    /// Create a new thread in a board
    pub async fn create_thread(&self, board_name: &str, request: CreateThreadRequest, creator_id: Uuid) -> AppResult<Thread> {
        // Get board
        let board = self.get_board(board_name).await?;

        // Post to Matrix room
        let matrix_event_id = if let Some(ref image_url) = request.image_url {
            self.matrix_client
                .send_message_with_image(&board.matrix_room_id, &request.content, image_url)
                .await?
        } else {
            self.matrix_client
                .send_message(&board.matrix_room_id, &request.content)
                .await?
        };

        let thread_id = Uuid::new_v4();
        let now = Utc::now();

        // Insert thread into database
        sqlx::query!(
            r#"
            INSERT INTO threads (id, board_id, title, content, image_url, matrix_event_id, created_at, created_by)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            thread_id.to_string(),
            board.id.to_string(),
            request.title,
            request.content,
            request.image_url,
            matrix_event_id,
            now.to_rfc3339(),
            creator_id.to_string()
        )
        .execute(self.db.pool())
        .await?;

        Ok(Thread {
            id: thread_id,
            board_id: board.id,
            title: request.title,
            content: request.content,
            image_url: request.image_url,
            matrix_event_id,
            is_pinned: false,
            is_locked: false,
            created_at: now,
            created_by: creator_id,
            reply_count: 0,
            last_reply_at: None,
        })
    }

    /// Get threads in a board
    pub async fn get_threads(&self, board_name: &str, limit: Option<i64>, offset: Option<i64>) -> AppResult<Vec<Thread>> {
        let board = self.get_board(board_name).await?;
        let limit = limit.unwrap_or(50).min(100); // Max 100 threads per request
        let offset = offset.unwrap_or(0);

        let thread_records = sqlx::query!(
            r#"
            SELECT id, board_id, title, content, image_url, matrix_event_id, is_pinned, is_locked, 
                   created_at, created_by, reply_count, last_reply_at
            FROM threads 
            WHERE board_id = ? 
            ORDER BY is_pinned DESC, COALESCE(last_reply_at, created_at) DESC
            LIMIT ? OFFSET ?
            "#,
            board.id.to_string(),
            limit,
            offset
        )
        .fetch_all(self.db.pool())
        .await?;

        let threads = thread_records
            .into_iter()
            .map(|record| {
                Ok(Thread {
                    id: Uuid::parse_str(&record.id)
                        .map_err(|e| AppError::Internal(format!("Invalid thread ID: {}", e)))?,
                    board_id: Uuid::parse_str(&record.board_id)
                        .map_err(|e| AppError::Internal(format!("Invalid board ID: {}", e)))?,
                    title: record.title,
                    content: record.content,
                    image_url: record.image_url,
                    matrix_event_id: record.matrix_event_id,
                    is_pinned: record.is_pinned,
                    is_locked: record.is_locked,
                    created_at: chrono::DateTime::parse_from_rfc3339(&record.created_at)
                        .map_err(|e| AppError::Internal(format!("Invalid date: {}", e)))?
                        .with_timezone(&Utc),
                    created_by: Uuid::parse_str(&record.created_by)
                        .map_err(|e| AppError::Internal(format!("Invalid user ID: {}", e)))?,
                    reply_count: record.reply_count,
                    last_reply_at: record.last_reply_at.as_ref().map(|s| {
                        chrono::DateTime::parse_from_rfc3339(s)
                            .map_err(|e| AppError::Internal(format!("Invalid date: {}", e)))
                            .unwrap()
                            .with_timezone(&Utc)
                    }),
                })
            })
            .collect::<AppResult<Vec<_>>>()?;

        Ok(threads)
    }

    /// Get a specific thread
    pub async fn get_thread(&self, thread_id: Uuid) -> AppResult<Thread> {
        let thread_record = sqlx::query!(
            r#"
            SELECT id, board_id, title, content, image_url, matrix_event_id, is_pinned, is_locked, 
                   created_at, created_by, reply_count, last_reply_at
            FROM threads WHERE id = ?
            "#,
            thread_id.to_string()
        )
        .fetch_optional(self.db.pool())
        .await?
        .ok_or_else(|| AppError::NotFound("Thread not found".to_string()))?;

        Ok(Thread {
            id: thread_id,
            board_id: Uuid::parse_str(&thread_record.board_id)
                .map_err(|e| AppError::Internal(format!("Invalid board ID: {}", e)))?,
            title: thread_record.title,
            content: thread_record.content,
            image_url: thread_record.image_url,
            matrix_event_id: thread_record.matrix_event_id,
            is_pinned: thread_record.is_pinned,
            is_locked: thread_record.is_locked,
            created_at: chrono::DateTime::parse_from_rfc3339(&thread_record.created_at)
                .map_err(|e| AppError::Internal(format!("Invalid date: {}", e)))?
                .with_timezone(&Utc),
            created_by: Uuid::parse_str(&thread_record.created_by)
                .map_err(|e| AppError::Internal(format!("Invalid user ID: {}", e)))?,
            reply_count: thread_record.reply_count,
            last_reply_at: thread_record.last_reply_at.as_ref().map(|s| {
                chrono::DateTime::parse_from_rfc3339(s)
                    .map_err(|e| AppError::Internal(format!("Invalid date: {}", e)))
                    .unwrap()
                    .with_timezone(&Utc)
            }),
        })
    }

    /// Create a post (reply to thread)
    pub async fn create_post(&self, thread_id: Uuid, request: CreatePostRequest, creator_id: Uuid) -> AppResult<Post> {
        // Get thread and board
        let thread = self.get_thread(thread_id).await?;
        let board_record = sqlx::query!(
            "SELECT matrix_room_id FROM boards WHERE id = ?",
            thread.board_id.to_string()
        )
        .fetch_one(self.db.pool())
        .await?;

        // Check if thread is locked
        if thread.is_locked {
            return Err(AppError::InvalidRequest("Thread is locked".to_string()));
        }

        // Post to Matrix room
        let matrix_event_id = if let Some(ref image_url) = request.image_url {
            self.matrix_client
                .send_message_with_image(&board_record.matrix_room_id, &request.content, image_url)
                .await?
        } else {
            self.matrix_client
                .send_message(&board_record.matrix_room_id, &request.content)
                .await?
        };

        let post_id = Uuid::new_v4();
        let now = Utc::now();

        // Insert post into database
        sqlx::query!(
            r#"
            INSERT INTO posts (id, thread_id, board_id, content, image_url, matrix_event_id, reply_to, created_at, created_by)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            post_id.to_string(),
            thread_id.to_string(),
            thread.board_id.to_string(),
            request.content,
            request.image_url,
            matrix_event_id,
            request.reply_to.map(|id| id.to_string()),
            now.to_rfc3339(),
            creator_id.to_string()
        )
        .execute(self.db.pool())
        .await?;

        // Update thread reply count and last reply time
        sqlx::query!(
            "UPDATE threads SET reply_count = reply_count + 1, last_reply_at = ? WHERE id = ?",
            now.to_rfc3339(),
            thread_id.to_string()
        )
        .execute(self.db.pool())
        .await?;

        Ok(Post {
            id: post_id,
            thread_id: Some(thread_id),
            board_id: thread.board_id,
            content: request.content,
            image_url: request.image_url,
            matrix_event_id,
            reply_to: request.reply_to,
            created_at: now,
            created_by: creator_id,
        })
    }

    /// Get posts in a thread
    pub async fn get_posts(&self, thread_id: Uuid, limit: Option<i64>, offset: Option<i64>) -> AppResult<Vec<Post>> {
        let limit = limit.unwrap_or(50).min(100); // Max 100 posts per request
        let offset = offset.unwrap_or(0);

        let post_records = sqlx::query!(
            r#"
            SELECT id, thread_id, board_id, content, image_url, matrix_event_id, reply_to, created_at, created_by
            FROM posts 
            WHERE thread_id = ? 
            ORDER BY created_at ASC
            LIMIT ? OFFSET ?
            "#,
            thread_id.to_string(),
            limit,
            offset
        )
        .fetch_all(self.db.pool())
        .await?;

        let posts = post_records
            .into_iter()
            .map(|record| {
                Ok(Post {
                    id: Uuid::parse_str(&record.id)
                        .map_err(|e| AppError::Internal(format!("Invalid post ID: {}", e)))?,
                    thread_id: record.thread_id.as_ref().map(|id| {
                        Uuid::parse_str(id)
                            .map_err(|e| AppError::Internal(format!("Invalid thread ID: {}", e)))
                            .unwrap()
                    }),
                    board_id: Uuid::parse_str(&record.board_id)
                        .map_err(|e| AppError::Internal(format!("Invalid board ID: {}", e)))?,
                    content: record.content,
                    image_url: record.image_url,
                    matrix_event_id: record.matrix_event_id,
                    reply_to: record.reply_to.as_ref().map(|id| {
                        Uuid::parse_str(id)
                            .map_err(|e| AppError::Internal(format!("Invalid reply_to ID: {}", e)))
                            .unwrap()
                    }),
                    created_at: chrono::DateTime::parse_from_rfc3339(&record.created_at)
                        .map_err(|e| AppError::Internal(format!("Invalid date: {}", e)))?
                        .with_timezone(&Utc),
                    created_by: Uuid::parse_str(&record.created_by)
                        .map_err(|e| AppError::Internal(format!("Invalid user ID: {}", e)))?,
                })
            })
            .collect::<AppResult<Vec<_>>>()?;

        Ok(posts)
    }
}