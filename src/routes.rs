use axum::routing::{delete, post, put};
use axum::{Router, routing::get};
use axum_login::{AuthManagerLayerBuilder, login_required};
use std::path::Path;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tower_sessions::{MemoryStore, SessionManagerLayer};
use tracing::info;

use crate::auth::login_datastar;
use crate::{auth, layer};
use crate::service::{OidcAuthBackend, OidcConfig};
use crate::state::AppState;
use crate::utils;
use crate::view;

pub async fn router() -> Router<AppState> {
    let server_dir = utils::server_directory();
    let dist_dir = Path::new(&server_dir).join("dist");

    info!("Serve Path: {}", dist_dir.display());

    let serve_dir = ServeDir::new(dist_dir)
        .precompressed_br()
        .precompressed_gzip();

    // Session layer.
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store);

    // Auth service.
    let backend = OidcAuthBackend::new(OidcConfig {
        client_id: std::env::var("OIDC_CLIENT_ID").expect("OIDC_CLIENT_ID must be set."),
        client_secret: std::env::var("OIDC_CLIENT_SECRET")
            .expect("OIDC_CLIENT_SECRET must be set."),
        issuer_url: std::env::var("OIDC_ISSUER_URL").expect("OIDC_ISSUER_URL must be set."),
        redirect_url: "http://127.0.0.1:3000/login/authorization/callback".to_string(),
        scopes: vec![
            "openid".to_string(),
            "email".to_string(),
            "profile".to_string(),
            // offline access requires explicit consent
            //"offline_access".to_string(),
        ],
    })
    .await
    .expect("Failed to create OIDC backend");

    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

    Router::new()
        .without_v07_checks()
        .route("/", get(view::index::index))
        .route("/note", post(view::note::new_note))
        .route("/note/{id}", get(view::note::get_note))
        .route("/note/{id}", put(view::note::update_note))
        .route("/note/{id}", delete(view::note::delete_note))
        .route("/note/{id}/:edit", get(view::note::edit_note_view))
        .route("/note/{id}/:check", put(view::note::check_note))
        .route("/note/{id}/:uncheck", put(view::note::uncheck_note))
        .route_layer(login_required!(OidcAuthBackend, login_url = "/login"))
        .route("/login", get(auth::login))
        // Data-Star related routes for redirection
        .route("/login", put(login_datastar))
        .route("/login", post(login_datastar))
        .route("/login", delete(login_datastar))
        .route("/login/authorization/callback", get(auth::login_callback))
        .route("/login/error", get(auth::login_error))
        .layer(auth_layer)
        .fallback_service(serve_dir)
        .layer(layer::default_http_headers())
        .layer(TraceLayer::new_for_http())
}
