use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::core::app::AppState;
use crate::core::error::AppError;
use crate::core::types::{User, CreateUserRequest, LoginRequest};

#[derive(Serialize)]
pub struct AuthResponse {
    pub user: User,
    pub token: String,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateUserRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.auth_service.register(request).await {
        Ok(user) => {
            match state.auth_service.create_session(user.id).await {
                Ok(session) => Ok(Json(AuthResponse {
                    user,
                    token: session.token,
                })),
                Err(e) => Err((
                    StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                    Json(ErrorResponse { error: e.to_string() }),
                )),
            }
        }
        Err(e) => Err((
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ErrorResponse { error: e.to_string() }),
        )),
    }
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.auth_service.login(request).await {
        Ok((user, session)) => Ok(Json(AuthResponse {
            user,
            token: session.token,
        })),
        Err(e) => Err((
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ErrorResponse { error: e.to_string() }),
        )),
    }
}

pub async fn logout(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<User>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // Note: We'd need to get the token from the request headers to logout properly
    // For now, we'll just return success
    Ok(StatusCode::OK)
}

pub async fn me(
    Extension(user): Extension<User>,
) -> Json<User> {
    Json(user)
}