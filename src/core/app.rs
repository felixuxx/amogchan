use anyhow::Result;
use axum::Router;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;

use crate::core::config::Config;
use crate::storage::database::Database;
use crate::matrix::client::MatrixClient;
use crate::auth::service::AuthService;
use crate::board::service::BoardService;
use crate::chat::service::ChatService;
use crate::crypto::service::CryptoService;
use crate::web::routes;

pub struct App {
    config: Config,
    db: Arc<Database>,
    matrix_client: Arc<MatrixClient>,
    auth_service: Arc<AuthService>,
    board_service: Arc<BoardService>,
    chat_service: Arc<ChatService>,
    crypto_service: Arc<CryptoService>,
}

impl App {
    pub async fn new(config: Config) -> Result<Self> {
        info!("Initializing application components");

        // Initialize database
        let db = Arc::new(Database::new(&config.database).await?);
        
        // Run migrations
        db.migrate().await?;

        // Initialize crypto service
        let crypto_service = Arc::new(CryptoService::new(&config.crypto)?);

        // Initialize Matrix client
        let matrix_client = Arc::new(MatrixClient::new(&config.matrix).await?);

        // Initialize services
        let auth_service = Arc::new(AuthService::new(
            Arc::clone(&db),
            Arc::clone(&crypto_service),
            config.security.clone(),
        ));

        let board_service = Arc::new(BoardService::new(
            Arc::clone(&db),
            Arc::clone(&matrix_client),
        ));

        let chat_service = Arc::new(ChatService::new(
            Arc::clone(&db),
            Arc::clone(&matrix_client),
            Arc::clone(&crypto_service),
        ));

        Ok(Self {
            config,
            db,
            matrix_client,
            auth_service,
            board_service,
            chat_service,
            crypto_service,
        })
    }

    pub async fn run(self) -> Result<()> {
        let app_state = AppState {
            db: self.db,
            matrix_client: self.matrix_client,
            auth_service: self.auth_service,
            board_service: self.board_service,
            chat_service: self.chat_service,
            crypto_service: self.crypto_service,
            config: self.config.clone(),
        };

        let app = routes::create_router(Arc::new(app_state));

        let addr = SocketAddr::from(([0, 0, 0, 0], self.config.server.port));
        info!("Server listening on {}", addr);

        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub matrix_client: Arc<MatrixClient>,
    pub auth_service: Arc<AuthService>,
    pub board_service: Arc<BoardService>,
    pub chat_service: Arc<ChatService>,
    pub crypto_service: Arc<CryptoService>,
    pub config: Config,
}