use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    config::Config, infra::local_image_storage::LocalImageStorage,
    infra::products_inmem::InMemoryProductRepository, services::product::ProductService,
    state::AppState,
};

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env()?;
    init_tracing(&config.log_level);

    let repo = Arc::new(InMemoryProductRepository::new());
    let image_repo = Arc::new(LocalImageStorage::new()?);
    let product_service = ProductService::new(repo, image_repo);
    let state = AppState::new(config.clone(), product_service);
    let app = build_router(state);

    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    let listener = TcpListener::bind(addr).await?;

    info!("listening on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

fn init_tracing(log_level: &str) {
    tracing_subscriber::registry()
        .with(EnvFilter::new(log_level.to_string()))
        .with(tracing_subscriber::fmt::layer())
        .init();
}

fn build_router(state: AppState) -> Router {
    let middleware = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    crate::http::routes::create_router(state).layer(middleware)
}
