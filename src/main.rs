mod core;
mod matrix;
mod board;
mod chat;
mod auth;
mod crypto;
mod web;
mod storage;

use anyhow::Result;
use tracing::{info, error};
use tracing_subscriber;

use crate::core::app::App;
use crate::core::config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Load environment variables
    dotenv::dotenv().ok();
    
    info!("Starting Encrypted Social Platform");
    
    // Load configuration
    let config = Config::load().await?;
    
    // Initialize the application
    let app = App::new(config).await?;
    
    // Start the application
    if let Err(e) = app.run().await {
        error!("Application error: {}", e);
        return Err(e);
    }
    
    Ok(())
}