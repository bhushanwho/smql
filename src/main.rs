use std::sync::Arc;
use smql::api::create_api;
use smql::{config, MessageService};
use smql::storage::MemoryStorage;
use tracing::info;
use tracing_subscriber::{
    filter::LevelFilter, layer::Layer, layer::SubscriberExt, util::SubscriberInitExt,
};

#[tokio::main]
async fn main() {
    let cfg = config();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_filter(LevelFilter::from_level(cfg.tracing_level())),
        )
        .init();

    info!(
        "Starting SMQL with configuration: port={}, max_message_size={}, log_level={}",
        cfg.port, cfg.max_message_size, cfg.log_level
    );

    let store = Arc::new(MemoryStorage::new());
    let service = MessageService::new(store);

    let app = create_api(service);
    let bind_addr = format!("[::]:{}", cfg.port);
    let listener = tokio::net::TcpListener::bind(bind_addr).await.unwrap();

    info!("Listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
