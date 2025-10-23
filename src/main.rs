use axum::{
    routing::{get, post},
    Router,
};
use std::{net::SocketAddr, sync::Arc};
use tower_http::{cors::CorsLayer, services::ServeDir};

mod config;
mod error;
mod font;
mod handlers;
mod service;
mod utils;

use config::AppConfig;

use service::FontService;

pub type AppState = Arc<FontService>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    
    let config = AppConfig::load()?;
    let font_service = Arc::new(FontService::new(config).await?);
    
    let app = Router::new()
        .route("/api/v1/list", get(handlers::list_fonts))
        .route("/api/v1/font", get(handlers::get_font))
        .route("/api/v1/generate", post(handlers::generate_font))
        .nest_service("/static", ServeDir::new("data/static"))
        .layer(CorsLayer::permissive())
        .with_state(font_service);

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8000".to_string())
        .parse::<u16>()
        .unwrap_or(8000);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    log::info!("服务器启动在 {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}