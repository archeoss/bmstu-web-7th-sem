// #![allow(
//     clippy::multiple_crate_versions,
//     clippy::unwrap_used,
//     clippy::expect_used
// )]
//
// use axum::{routing::get, Router};
// use backend::{
//     config::{ConfigExt, LoggerExt},
//     prelude::*,
//     root,
//     services::api_router,
// };
// use cli::Parser;
// use error_stack::{Result, ResultExt};
// use std::env;
// use tower::ServiceBuilder;
// use tower_http::{cors::CorsLayer, services::ServeDir};
//
// #[tokio::main]
// async fn main() -> Result<(), AppError> {
//     let config = cli::Config::try_from(cli::Args::parse())
//         .change_context(AppError::InitializationError)
//         .attach_printable("Couldn't get config file.")?;
//
//     let logger = &config.logger;
//
//     let _guard = logger.init_logger().unwrap();
//     tracing::info!("Logger: {logger:?}");
//
//     let cors: CorsLayer = config.get_cors_configuration();
//     tracing::info!("CORS: {cors:?}");
//
//     let addr = config.address;
//     tracing::info!("Listening on {addr}");
//
//     let app = router(cors);
//     #[cfg(feature = "swagger")]
//     let app = app.merge(backend::openapi_doc());
//
//     axum::Server::bind(&addr)
//         .serve(app.into_make_service())
//         .await
//         .change_context(AppError::StartUpError)
//         .attach_printable("Failed to start axum server")?;
//
//     Ok(())
// }
//
// fn router(cors: CorsLayer) -> Router {
//     let mut frontend = env::current_exe().expect("Couldn't get current executable path.");
//     frontend.pop();
//     frontend.push("dist");
//     tracing::info!("serving frontend at: {frontend:?}");
//     // Add api
//     Router::new()
//         // Frontend
//         .nest_service("/", ServeDir::new(frontend))
//         // Unsecured Routes
//         .route("/root", get(root))
//         .nest("/api", api_router())
//         .layer(ServiceBuilder::new().layer(cors))
// }
#![allow(clippy::multiple_crate_versions)]
use axum::Router;
use axum::{routing::get, Extension};
use axum_login::{
    axum_sessions::{async_session::MemoryStore as SessionMemoryStore, SessionLayer},
    memory_store::MemoryStore as AuthMemoryStore,
    AuthLayer,
};
use backend::config::{ConfigExt, LoggerExt};
use backend::services::api_router;
use backend::{models::models::RequestTimeout, services::auth::SocketBobMemoryStore};
use backend::{prelude::*, root};
use cli::Parser;
use rand::Rng;
use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing::Level;

#[tokio::main]
#[allow(clippy::unwrap_used, clippy::expect_used)]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let config = cli::Config::try_from(cli::Args::parse()).unwrap();

    let logger = &config.logger;

    let _guard = logger.init_logger().unwrap();
    tracing::info!("Logger: {logger:?}");

    let cors: CorsLayer = config.get_cors_configuration();
    tracing::info!("CORS: {cors:?}");

    let addr = config.address;
    tracing::info!("Listening on {addr}");

    //     let app = router(cors);
    //     #[cfg(feature = "swagger")]
    //     let app = app.merge(backend::openapi_doc());
    //
    //     axum::Server::bind(&addr)
    //         .serve(app.into_make_service())
    //         .await
    //         .change_context(AppError::StartUpError)
    //         .attach_printable("Failed to start axum server")?;
    //

    axum::Server::bind(&addr)
        .serve(
            router(cors)
                .layer(Extension(RequestTimeout::from_millis(
                    config.request_timeout.as_millis() as u64,
                )))
                .into_make_service(),
        )
        .await
        .unwrap();

    Ok(())
}

fn init_tracer(log_file: Option<PathBuf>, trace_level: Level) -> Result<(), Box<dyn Error>> {
    let subscriber = tracing_subscriber::fmt().with_max_level(trace_level);
    subscriber.init();

    Ok(())
}

fn router(cors: CorsLayer) -> Router {
    let secret = rand::thread_rng().gen::<[u8; 64]>();

    let session_store = SessionMemoryStore::new();
    let session_layer = SessionLayer::new(session_store, &secret);

    // We can use in-memory database since we don't need to actually store users
    let store = Arc::new(RwLock::new(HashMap::default()));

    let bob_store: SocketBobMemoryStore = AuthMemoryStore::new(&store);
    let auth_layer = AuthLayer::new(bob_store, &secret);

    // Add api
    Router::new()
        // Unsecured Routes
        .route("/", get(root))
        .nest(
            "/api",
            api_router().layer(
                ServiceBuilder::new()
                    .layer(cors)
                    .layer(session_layer)
                    .layer(auth_layer),
            ),
        )
        .with_state(store)
}
