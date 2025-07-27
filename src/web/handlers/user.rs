use axum::{
    extract::{State, Path},
    http::StatusCode,
    response::Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::app::AppState;
use crate::core::types::User;
use crate::web::handlers::auth::ErrorResponse;

pub async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> Result<Json<User>, (StatusCode, Json<ErrorResponse>)> {
    let user_uuid = Uuid::parse_str(&user_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "Invalid user ID".to_string() })))?;
    
    match state.auth_service.get_user(user_uuid).await {
        Ok(user) => Ok(Json(user)),
        Err(e) => Err((
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ErrorResponse { error: e.to_string() }),
        )),
    }
}