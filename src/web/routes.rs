use axum::{
    Router,
    routing::{get, post, put, delete},
    middleware::from_fn_with_state,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

use crate::core::app::AppState;
use crate::web::handlers::{auth, board, chat, user};
use crate::web::middleware::auth_middleware;

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        // Root route
        .route("/", get(serve_index))
        
        // Static files
        .nest_service("/static", ServeDir::new("static"))
        
        // Public routes (no auth required)
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/login", post(auth::login))
        .route("/api/boards", get(board::list_boards))
        .route("/api/boards/:name", get(board::get_board))
        .route("/api/boards/:name/threads", get(board::list_threads))
        .route("/api/threads/:id", get(board::get_thread))
        .route("/api/threads/:id/posts", get(board::list_posts))
        
        // Protected routes (auth required) - Apply middleware to specific routes
        .route("/api/auth/logout", post(auth::logout).layer(from_fn_with_state(state.clone(), auth_middleware)))
        .route("/api/auth/me", get(auth::me).layer(from_fn_with_state(state.clone(), auth_middleware)))
        .route("/api/boards", post(board::create_board).layer(from_fn_with_state(state.clone(), auth_middleware)))
        .route("/api/boards/:name/threads", post(board::create_thread).layer(from_fn_with_state(state.clone(), auth_middleware)))
        .route("/api/threads/:id/posts", post(board::create_post).layer(from_fn_with_state(state.clone(), auth_middleware)))
        .route("/api/chats", get(chat::list_chats).layer(from_fn_with_state(state.clone(), auth_middleware)))
        .route("/api/chats", post(chat::create_chat).layer(from_fn_with_state(state.clone(), auth_middleware)))
        .route("/api/chats/:id", get(chat::get_chat).layer(from_fn_with_state(state.clone(), auth_middleware)))
        .route("/api/chats/:id/messages", get(chat::list_messages).layer(from_fn_with_state(state.clone(), auth_middleware)))
        .route("/api/chats/:id/messages", post(chat::send_message).layer(from_fn_with_state(state.clone(), auth_middleware)))
        .route("/api/chats/:id/participants", post(chat::add_participant).layer(from_fn_with_state(state.clone(), auth_middleware)))
        .route("/api/chats/:id/participants/:user_id", delete(chat::remove_participant).layer(from_fn_with_state(state.clone(), auth_middleware)))
        .route("/api/users/:id", get(user::get_user).layer(from_fn_with_state(state.clone(), auth_middleware)))
        
        // Health check
        .route("/health", get(health_check))
        
        // Add CORS middleware
        .layer(CorsLayer::permissive())
        
        .with_state(state)
}

async fn health_check() -> &'static str {
    "OK"
}

async fn serve_index() -> axum::response::Html<&'static str> {
    axum::response::Html(include_str!("../../static/index.html"))
}