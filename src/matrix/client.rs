use anyhow::Result;
use matrix_sdk::{
    Client, Room, RoomState,
    ruma::{
        RoomId, UserId, EventId,
        events::room::message::RoomMessageEventContent,
        events::room::member::MembershipState,
    },
};
use std::sync::Arc;
use tracing::{info, error, warn};
use uuid::Uuid;

use crate::core::config::MatrixConfig;
use crate::core::error::{AppError, AppResult};

pub struct MatrixClient {
    client: Client,
    config: MatrixConfig,
}

impl MatrixClient {
    pub async fn new(config: &MatrixConfig) -> Result<Self> {
        info!("Initializing Matrix client for homeserver: {}", config.homeserver_url);
        
        let client = Client::builder()
            .homeserver_url(&config.homeserver_url)
            .build()
            .await?;

        // Login if we have credentials
        if let Some(access_token) = &config.access_token {
            let user_id = UserId::parse(&config.user_id)?;
            client.restore_login(user_id, access_token, config.device_id.as_deref()).await?;
            info!("Restored Matrix session for user: {}", config.user_id);
        } else {
            warn!("No Matrix access token provided - some features may not work");
        }

        Ok(Self {
            client,
            config: config.clone(),
        })
    }

    /// Create a new Matrix room for a board or chat
    pub async fn create_room(&self, name: &str, topic: Option<&str>, is_encrypted: bool) -> AppResult<String> {
        let mut request = matrix_sdk::ruma::api::client::room::create_room::v3::Request::new();
        request.name = Some(name.to_string());
        
        if let Some(topic) = topic {
            request.topic = Some(topic.to_string());
        }

        // Enable encryption if requested
        if is_encrypted {
            request.initial_state.push(
                matrix_sdk::ruma::events::room::encryption::RoomEncryptionEventContent::new(
                    matrix_sdk::ruma::events::room::encryption::EventEncryptionAlgorithm::MegolmV1AesSha2,
                ).into()
            );
        }

        let response = self.client.create_room(request).await
            .map_err(|e| AppError::Matrix(format!("Failed to create room: {}", e)))?;

        Ok(response.room_id.to_string())
    }

    /// Send a message to a Matrix room
    pub async fn send_message(&self, room_id: &str, content: &str) -> AppResult<String> {
        let room_id = RoomId::parse(room_id)
            .map_err(|e| AppError::Matrix(format!("Invalid room ID: {}", e)))?;
        
        let room = self.client.get_room(&room_id)
            .ok_or_else(|| AppError::Matrix("Room not found".to_string()))?;

        if room.state() != RoomState::Joined {
            return Err(AppError::Matrix("Not a member of this room".to_string()));
        }

        let content = RoomMessageEventContent::text_plain(content);
        
        let response = room.send(content, None).await
            .map_err(|e| AppError::Matrix(format!("Failed to send message: {}", e)))?;

        Ok(response.event_id.to_string())
    }

    /// Send a message with image to a Matrix room
    pub async fn send_message_with_image(&self, room_id: &str, content: &str, image_url: &str) -> AppResult<String> {
        let room_id = RoomId::parse(room_id)
            .map_err(|e| AppError::Matrix(format!("Invalid room ID: {}", e)))?;
        
        let room = self.client.get_room(&room_id)
            .ok_or_else(|| AppError::Matrix("Room not found".to_string()))?;

        if room.state() != RoomState::Joined {
            return Err(AppError::Matrix("Not a member of this room".to_string()));
        }

        // Create rich message content with image
        let mut message_content = RoomMessageEventContent::text_html(
            content,
            &format!("{}<br><img src=\"{}\" alt=\"Image\" style=\"max-width: 100%; height: auto;\">", content, image_url)
        );

        let response = room.send(message_content, None).await
            .map_err(|e| AppError::Matrix(format!("Failed to send message with image: {}", e)))?;

        Ok(response.event_id.to_string())
    }

    /// Join a Matrix room
    pub async fn join_room(&self, room_id: &str) -> AppResult<()> {
        let room_id = RoomId::parse(room_id)
            .map_err(|e| AppError::Matrix(format!("Invalid room ID: {}", e)))?;

        self.client.join_room_by_id(&room_id).await
            .map_err(|e| AppError::Matrix(format!("Failed to join room: {}", e)))?;

        Ok(())
    }

    /// Leave a Matrix room
    pub async fn leave_room(&self, room_id: &str) -> AppResult<()> {
        let room_id = RoomId::parse(room_id)
            .map_err(|e| AppError::Matrix(format!("Invalid room ID: {}", e)))?;
        
        let room = self.client.get_room(&room_id)
            .ok_or_else(|| AppError::Matrix("Room not found".to_string()))?;

        room.leave().await
            .map_err(|e| AppError::Matrix(format!("Failed to leave room: {}", e)))?;

        Ok(())
    }

    /// Invite a user to a Matrix room
    pub async fn invite_user(&self, room_id: &str, user_id: &str) -> AppResult<()> {
        let room_id = RoomId::parse(room_id)
            .map_err(|e| AppError::Matrix(format!("Invalid room ID: {}", e)))?;
        
        let user_id = UserId::parse(user_id)
            .map_err(|e| AppError::Matrix(format!("Invalid user ID: {}", e)))?;
        
        let room = self.client.get_room(&room_id)
            .ok_or_else(|| AppError::Matrix("Room not found".to_string()))?;

        room.invite_user_by_id(&user_id).await
            .map_err(|e| AppError::Matrix(format!("Failed to invite user: {}", e)))?;

        Ok(())
    }

    /// Get room members
    pub async fn get_room_members(&self, room_id: &str) -> AppResult<Vec<String>> {
        let room_id = RoomId::parse(room_id)
            .map_err(|e| AppError::Matrix(format!("Invalid room ID: {}", e)))?;
        
        let room = self.client.get_room(&room_id)
            .ok_or_else(|| AppError::Matrix("Room not found".to_string()))?;

        // Note: Getting room members requires additional Matrix SDK setup
        // For now, return empty list
        Ok(vec![])
    }

    /// Create a direct message room with another user
    pub async fn create_dm(&self, user_id: &str) -> AppResult<String> {
        let user_id = UserId::parse(user_id)
            .map_err(|e| AppError::Matrix(format!("Invalid user ID: {}", e)))?;

        let mut request = matrix_sdk::ruma::api::client::room::create_room::v3::Request::new();
        request.is_direct = true;
        request.invite = vec![user_id];
        request.preset = Some(matrix_sdk::ruma::api::client::room::create_room::v3::RoomPreset::TrustedPrivateChat);

        // Enable encryption for DMs
        request.initial_state.push(
            matrix_sdk::ruma::events::room::encryption::RoomEncryptionEventContent::new(
                matrix_sdk::ruma::events::room::encryption::EventEncryptionAlgorithm::MegolmV1AesSha2,
            ).into()
        );

        let response = self.client.create_room(request).await
            .map_err(|e| AppError::Matrix(format!("Failed to create DM: {}", e)))?;

        Ok(response.room_id.to_string())
    }

    /// Get the Matrix client for advanced operations
    pub fn client(&self) -> &Client {
        &self.client
    }
}

// Helper functions for Matrix integration would go here