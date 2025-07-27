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
use crate::core::types::{User, Board, Thread, Post, CreateBoardRequest, CreateThreadRequest, CreatePostRequest};
use crate::web::handlers::auth::ErrorResponse;

#[derive(Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_boards(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Board>>, (StatusCode, Json<ErrorResponse>)> {
    match state.board_service.get_boards().await {
        Ok(boards) => Ok(Json(boards)),
        Err(e) => Err((
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ErrorResponse { error: e.to_string() }),
        )),
    }
}

pub async fn get_board(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<Board>, (StatusCode, Json<ErrorResponse>)> {
    match state.board_service.get_board(&name).await {
        Ok(board) => Ok(Json(board)),
        Err(e) => Err((
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ErrorResponse { error: e.to_string() }),
        )),
    }
}

pub async fn create_board(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<User>,
    Json(request): Json<CreateBoardRequest>,
) -> Result<Json<Board>, (StatusCode, Json<ErrorResponse>)> {
    match state.board_service.create_board(request, user.id).await {
        Ok(board) => Ok(Json(board)),
        Err(e) => Err((
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ErrorResponse { error: e.to_string() }),
        )),
    }
}

pub async fn list_threads(
    State(state): State<Arc<AppState>>,
    Path(board_name): Path<String>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<Vec<Thread>>, (StatusCode, Json<ErrorResponse>)> {
    match state.board_service.get_threads(&board_name, pagination.limit, pagination.offset).await {
        Ok(threads) => Ok(Json(threads)),
        Err(e) => Err((
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ErrorResponse { error: e.to_string() }),
        )),
    }
}

pub async fn get_thread(
    State(state): State<Arc<AppState>>,
    Path(thread_id): Path<String>,
) -> Result<Json<Thread>, (StatusCode, Json<ErrorResponse>)> {
    let thread_uuid = Uuid::parse_str(&thread_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "Invalid thread ID".to_string() })))?;
    
    match state.board_service.get_thread(thread_uuid).await {
        Ok(thread) => Ok(Json(thread)),
        Err(e) => Err((
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ErrorResponse { error: e.to_string() }),
        )),
    }
}

pub async fn create_thread(
    State(state): State<Arc<AppState>>,
    Path(board_name): Path<String>,
    Extension(user): Extension<User>,
    Json(request): Json<CreateThreadRequest>,
) -> Result<Json<Thread>, (StatusCode, Json<ErrorResponse>)> {
    match state.board_service.create_thread(&board_name, request, user.id).await {
        Ok(thread) => Ok(Json(thread)),
        Err(e) => Err((
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ErrorResponse { error: e.to_string() }),
        )),
    }
}

pub async fn list_posts(
    State(state): State<Arc<AppState>>,
    Path(thread_id): Path<String>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<Vec<Post>>, (StatusCode, Json<ErrorResponse>)> {
    let thread_uuid = Uuid::parse_str(&thread_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "Invalid thread ID".to_string() })))?;
    
    match state.board_service.get_posts(thread_uuid, pagination.limit, pagination.offset).await {
        Ok(posts) => Ok(Json(posts)),
        Err(e) => Err((
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ErrorResponse { error: e.to_string() }),
        )),
    }
}

pub async fn create_post(
    State(state): State<Arc<AppState>>,
    Path(thread_id): Path<String>,
    Extension(user): Extension<User>,
    Json(request): Json<CreatePostRequest>,
) -> Result<Json<Post>, (StatusCode, Json<ErrorResponse>)> {
    let thread_uuid = Uuid::parse_str(&thread_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "Invalid thread ID".to_string() })))?;
    
    match state.board_service.create_post(thread_uuid, request, user.id).await {
        Ok(post) => Ok(Json(post)),
        Err(e) => Err((
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ErrorResponse { error: e.to_string() }),
        )),
    }
}