// #![allow(clippy::module_name_repetitions)]
//
// use axum::response::{IntoResponse, Response};
// use hyper::StatusCode;
// use thiserror::Error;
//
// /// Server start up errors
// #[derive(Debug, Error)]
// pub enum AppError {
//     #[error("Server initialization failed")]
//     InitializationError,
//     #[error("Server start up failed")]
//     StartUpError,
// }
//
// impl IntoResponse for AppError {
//     fn into_response(self) -> Response {
//         tracing::error!("{}", self);
//         (
//             StatusCode::INTERNAL_SERVER_ERROR,
//             "Something went wrong".to_string(),
//         )
//             .into_response()
//     }
// }
#![allow(clippy::module_name_repetitions)]
use axum::response::{IntoResponse, Response};
use color_eyre::eyre::Error;
use hyper::StatusCode;
use thiserror::Error;

/// Generic error that wraps `eyre::Error`.
///
/// For the internal server errors
pub struct AppError(Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        tracing::error!("{}", self.0);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Something went wrong".to_string(),
        )
            .into_response()
    }
}

/// Enables using `?` on functions that return `Result<_, eyre::Error>` to turn them into
/// `Result<_, AppError>`.
impl<E> From<E> for AppError
where
    E: Into<Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

/// Errors that happend during API request proccessing
///
/// For errors that should be known on the client
#[derive(Error, Debug)]
pub enum APIError {
    #[error("the request to the specified resource failed")]
    RequestFailed,
    #[error("server received invalid status code from client")]
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
