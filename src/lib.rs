pub mod auth;
pub mod db;
pub mod fragments;
pub mod model;
pub mod repository;
pub mod routes;
pub mod service;
pub mod state;
pub mod view;

pub mod tracing {
    use tracing_subscriber::fmt::format::FmtSpan;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{filter::EnvFilter, fmt};

    pub fn init_tracing() {
        let filter_layer = EnvFilter::try_from_default_env()
            .or_else(|_| EnvFilter::try_new("info"))
            .unwrap();

        let stdout_layer = fmt::layer()
            .with_span_events(FmtSpan::CLOSE)
            .with_target(true);

        tracing_subscriber::registry()
            .with(filter_layer)
            .with(stdout_layer)
            .init();
    }
}

pub mod layer {
    use axum::http::HeaderMap;
    use axum::http::HeaderValue;
    use axum::http::header::{CONTENT_SECURITY_POLICY, X_FRAME_OPTIONS};
    use tower_default_headers::DefaultHeadersLayer;

    pub fn default_http_headers() -> tower_default_headers::DefaultHeadersLayer {
        let csp_header_value = format!(
            r#"base-uri 'none'
            object-src 'none'
            script-src 'self' 'unsafe-eval'
            style-src 'self' 'unsafe-inline'
            default-src 'self'
            img-src 'self' data:
            frame-ancestors 'self'
            form-action 'self'
            report-uri /csp-report"#
        )
        .replace("\n", ";");

        let mut default_headers = HeaderMap::new();
        default_headers.insert(X_FRAME_OPTIONS, HeaderValue::from_static("deny"));
        default_headers.insert(
            CONTENT_SECURITY_POLICY,
            HeaderValue::from_str(csp_header_value.as_str()).unwrap(),
        );

        DefaultHeadersLayer::new(default_headers)
    }
}

pub mod utils {

    pub fn server_directory() -> String {
        let cwd = std::env::current_dir().unwrap();
        let cwd_str = String::from(cwd.to_str().unwrap());

        std::env::var("CARGO_MANIFEST_DIR").unwrap_or(cwd_str)
    }
}
