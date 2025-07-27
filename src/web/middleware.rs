use axum::{
    extract::{Request, State},
    http::{StatusCode, HeaderMap},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

use crate::core::app::AppState;
use crate::core::types::User;

pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract authorization header
    let headers = request.headers();
    let auth_header = headers
        .get("Authorization")
        .and_then(|header| header.to_str().ok());

    if let Some(auth_header) = auth_header {
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            // Validate the token
            match state.auth_service.validate_session(token).await {
                Ok(user) => {
                    // Add user to request extensions
                    request.extensions_mut().insert(user);
                    Ok(next.run(request).await)
                }
                Err(_) => Err(StatusCode::UNAUTHORIZED),
            }
        } else {
            Err(StatusCode::UNAUTHORIZED)
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}