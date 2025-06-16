use ::tracing::info;
use dotenv::dotenv;
use datastar_axum_todolist::{db, routes, state::AppState, tracing, utils};

#[tokio::main]
async fn main() {
    tracing::init_tracing();

    info!("Starting server...");
    info!("  Server directory: {}", utils::server_directory());
    info!("  Version: {}", env!("CARGO_PKG_VERSION"));

    let _ = dotenv().ok();

    let database = db::create_pool().await;
    let app_state = AppState::from_database(database).await;

    let router = routes::router().await.with_state(app_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    info!("Listening on http://{}", listener.local_addr().unwrap());

    axum::serve(listener, router).await.unwrap();
}
