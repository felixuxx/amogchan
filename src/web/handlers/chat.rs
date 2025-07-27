use axum::{
    extract::{State, Path, Query},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::app::AppState;
use crate::core::types::{User, Chat, Message, CreateChatRequest, SendMessageRequest};
use crate::web::handlers::auth::ErrorResponse;
use crate::web::handlers::board::PaginationQuery;

#[derive(Deserialize)]
pub struct AddParticipantRequest {
    pub user_id: Uuid,
}

pub async fn list_chats(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<User>,
) -> Result<Json<Vec<Chat>>, (StatusCode, Json<ErrorResponse>)> {
    match state.chat_service.get_user_chats(user.id).await {
        Ok(chats) => Ok(Json(chats)),
        Err(e) => Err((
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ErrorResponse { error: e.to_string() }),
        )),
    }
}

pub async fn create_chat(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<User>,
    Json(request): Json<CreateChatRequest>,
) -> Result<Json<Chat>, (StatusCode, Json<ErrorResponse>)> {
    match state.chat_service.create_chat(request, user.id).await {
        Ok(chat) => Ok(Json(chat)),
        Err(e) => Err((
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ErrorResponse { error: e.to_string() }),
        )),
    }
}

pub async fn get_chat(
    State(state): State<Arc<AppState>>,
    Path(chat_id): Path<String>,
    Extension(user): Extension<User>,
) -> Result<Json<Chat>, (StatusCode, Json<ErrorResponse>)> {
    let chat_uuid = Uuid::parse_str(&chat_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "Invalid chat ID".to_string() })))?;
    
    match state.chat_service.get_chat(chat_uuid, user.id).await {
        Ok(chat) => Ok(Json(chat)),
        Err(e) => Err((
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ErrorResponse { error: e.to_string() }),
        )),
    }
}

pub async fn list_messages(
    State(state): State<Arc<AppState>>,
    Path(chat_id): Path<String>,
    Query(pagination): Query<PaginationQuery>,
    Extension(user): Extension<User>,
) -> Result<Json<Vec<Message>>, (StatusCode, Json<ErrorResponse>)> {
    let chat_uuid = Uuid::parse_str(&chat_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "Invalid chat ID".to_string() })))?;
    
    match state.chat_service.get_messages(chat_uuid, user.id, pagination.limit, pagination.offset).await {
        Ok(messages) => Ok(Json(messages)),
        Err(e) => Err((
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ErrorResponse { error: e.to_string() }),
        )),
    }
}

pub async fn send_message(
    State(state): State<Arc<AppState>>,
    Path(chat_id): Path<String>,
    Extension(user): Extension<User>,
    Json(request): Json<SendMessageRequest>,
) -> Result<Json<Message>, (StatusCode, Json<ErrorResponse>)> {
    let chat_uuid = Uuid::parse_str(&chat_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "Invalid chat ID".to_string() })))?;
    
    match state.chat_service.send_message(chat_uuid, request, user.id).await {
        Ok(message) => Ok(Json(message)),
        Err(e) => Err((
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ErrorResponse { error: e.to_string() }),
        )),
    }
}

pub async fn add_participant(
    State(state): State<Arc<AppState>>,
    Path(chat_id): Path<String>,
    Extension(user): Extension<User>,
    Json(request): Json<AddParticipantRequest>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let chat_uuid = Uuid::parse_str(&chat_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "Invalid chat ID".to_string() })))?;
    
    match state.chat_service.add_user_to_chat(chat_uuid, request.user_id, user.id).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => Err((
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ErrorResponse { error: e.to_string() }),
        )),
    }
}

pub async fn remove_participant(
    State(state): State<Arc<AppState>>,
    Path((chat_id, user_id)): Path<(String, String)>,
    Extension(admin_user): Extension<User>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let chat_uuid = Uuid::parse_str(&chat_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "Invalid chat ID".to_string() })))?;
    
    let user_uuid = Uuid::parse_str(&user_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "Invalid user ID".to_string() })))?;
    
    match state.chat_service.remove_user_from_chat(chat_uuid, user_uuid, admin_user.id).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => Err((
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ErrorResponse { error: e.to_string() }),
        )),
    }
}