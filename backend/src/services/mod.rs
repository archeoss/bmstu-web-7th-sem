pub mod api;
pub mod auth;
pub mod dto;
mod methods;

use self::{
    api::{
        get_detailed_node_info, get_disks_count, get_nodes, get_nodes_count, get_rps, get_space,
        get_vdisks, raw_configuration_by_node, raw_metrics_by_node,
    },
    auth::{login, logout, SharedBobStore},
};
use crate::{connector::BobClient, models::models::Hostname};
use axum::routing::{get, post};
use axum::{
    response::{IntoResponse, Response},
    Router,
};
use axum_login::RequireAuthorizationLayer;
use hyper::{Body, StatusCode};
use thiserror::Error;

type RequireAuth = RequireAuthorizationLayer<Hostname, BobClient>;

/// Export all secured routes
pub fn api_router() -> Router<SharedBobStore<Hostname>, Body> {
    let router = Router::new()
        .route("/vdisks", get(get_vdisks))
        .route("/nodes", get(get_nodes))
        .route("/nodes/:node_name", get(get_detailed_node_info))
        .route("/nodes/:node_name/metrics", get(raw_metrics_by_node))
        .route(
            "/nodes/:node_name/configuration",
            get(raw_configuration_by_node),
        );
    let router = router
        .route("/nodes/count", get(get_nodes_count))
        .route("/disks/count", get(get_disks_count))
        .route("/nodes/space", get(get_space))
        .route("/nodes/rps", get(get_rps));
    let router = router.route_layer(RequireAuth::login());

    router
        .route("/login", post(login))
        .route("/logout", get(logout))
}

/// Errors that happend during API request proccessing
#[derive(Debug, Error)]
pub enum APIError {
    #[error("The request to the specified resource failed")]
    RequestFailed,
    #[error("Server received invalid status code from client: `{0}`")]
    InvalidStatusCode(StatusCode),
}

impl IntoResponse for APIError {
    fn into_response(self) -> Response {
        match self {
            Self::RequestFailed => (StatusCode::NOT_FOUND, self.to_string()).into_response(),
            Self::InvalidStatusCode(code) => code.into_response(),
        }
    }
}
