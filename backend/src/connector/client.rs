#![allow(missing_docs, clippy::module_name_repetitions)]
use super::api::GetConfigurationResponse;
use super::dto;
use crate::models::models::RequestTimeout;
use axum::async_trait;
use axum::headers::authorization::Credentials;
use axum::headers::Authorization;
use axum::headers::HeaderMapExt;
use futures::{future::TryFutureExt, stream::StreamExt};
use hyper::header::{HeaderName, HeaderValue};
use hyper::HeaderMap;
use hyper::{service::Service, Body, Request, Response, Uri};
use percent_encoding::{utf8_percent_encode, AsciiSet};
use serde::de::DeserializeOwned;
use std::convert::TryInto;
use std::error::Error;
use std::fmt;
use std::marker::PhantomData;
use std::str;
use std::str::FromStr;
use std::string::ToString;
use std::task::{Context, Poll};
use swagger::{ApiError, BodyExt, Connector, DropContextService, Has, XSpanIdString};

/// <https://url.spec.whatwg.org/#fragment-percent-encode-set>
#[allow(dead_code)]
const FRAGMENT_ENCODE_SET: &AsciiSet = &percent_encoding::CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'<')
    .add(b'>')
    .add(b'`');

/// This encode set is used for object IDs
///
/// Aside from the special characters defined in the `PATH_SEGMENT_ENCODE_SET`,
/// the vertical bar (|) is encoded.
#[allow(dead_code)]
const ID_ENCODE_SET: &AsciiSet = &FRAGMENT_ENCODE_SET.add(b'|');

use super::api::{
    Api, GetAlienDirResponse, GetDisksResponse, GetMetricsResponse, GetNodesResponse,
    GetPartitionResponse, GetPartitionsResponse, GetRecordsResponse, GetReplicasLocalDirsResponse,
    GetSpaceInfoResponse, GetStatusResponse, GetVDiskResponse, GetVDisksResponse,
    GetVersionResponse,
};

/// Convert input into a base path, e.g. <http://example:123>. Also checks the scheme as it goes.
fn into_base_path(
    input: impl TryInto<Uri, Error = hyper::http::uri::InvalidUri>,
    _correct_scheme: Option<&'static str>,
) -> Result<String, ClientInitError> {
    // First convert to Uri, since a base path is a subset of Uri.
    let uri = input.try_into()?;

    // let scheme = uri.scheme_str().ok_or(ClientInitError::InvalidScheme)?;

    let scheme = "http"; // Hardcode for convenience

    // Check the scheme if necessary
    // if let Some(correct_scheme) = correct_scheme {
    //     if scheme != correct_scheme {
    //         return Err(ClientInitError::InvalidScheme);
    //     }
    // }

    let host = uri.host().ok_or(ClientInitError::MissingHost)?;
    let port = uri.port_u16().map(|x| format!(":{x}")).unwrap_or_default();
    Ok(format!(
        "{}://{}{}{}",
        scheme,
        host,
        port,
        uri.path().trim_end_matches('/')
    ))
}

/// A client that implements the API by making HTTP calls out to a server.
#[derive(Clone)]
pub struct Client<S, C, Cr>
where
    S: Service<(Request<Body>, C), Response = Response<Body>> + Clone + Sync + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<super::api::ServiceError> + fmt::Display,
    C: Clone + Send + Sync + 'static,
{
    /// Inner service
    client_service: S,

    /// Base path of the API
    base_path: String,

    /// Headers to include in requests
    headers: HeaderMap,

    /// Context Marker
    con_marker: PhantomData<fn(C)>,

    /// Credentials Marker
    cred_marker: PhantomData<fn(Cr)>,
}

impl<S, C, Cr> fmt::Debug for Client<S, C, Cr>
where
    S: Service<(Request<Body>, C), Response = Response<Body>> + Clone + Sync + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<super::api::ServiceError> + fmt::Display,
    C: Clone + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Client {{ base_path: {} }}", self.base_path)
    }
}

impl<Connector, C, Cr> Client<DropContextService<hyper::client::Client<Connector, Body>, C>, C, Cr>
where
    Connector: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    C: Clone + Send + Sync + 'static,
{
    /// Create a client with a custom implementation of [`hyper::client::Connect`].
    ///
    /// Intended for use with custom implementations of connect for e.g. protocol logging
    /// or similar functionality which requires wrapping the transport layer. When wrapping a TCP connection,
    /// this function should be used in conjunction with `swagger::Connector::builder()`.
    ///
    /// # Arguments
    ///
    /// * `base_path` - base path of the client API, i.e. <http://www.my-api-implementation.com>
    /// * `protocol` - Which protocol to use when constructing the request url, e.g. `Some("http")`
    /// * `connector` - Implementation of `hyper::client::Connect` to use for the client
    ///
    /// # Errors
    ///
    ///  The function will fail if a hostname isn't a valid url
    ///
    pub fn try_new_with_connector(
        base_path: &str,
        protocol: Option<&'static str>,
        connector: Connector,
        headers: HeaderMap,
    ) -> Result<Self, ClientInitError> {
        let client_service = hyper::client::Client::builder().build(connector);
        let client_service = DropContextService::new(client_service);

        Ok(Self {
            client_service,
            base_path: into_base_path(base_path, protocol)?,
            con_marker: PhantomData,
            headers,
            cred_marker: PhantomData,
        })
    }
}

#[derive(Debug, Clone)]
pub enum HyperClient {
    Http(hyper::client::Client<hyper::client::HttpConnector, Body>),
}

impl Service<Request<Body>> for HyperClient {
    type Response = Response<Body>;
    type Error = hyper::Error;
    type Future = hyper::client::ResponseFuture;

    fn poll_ready(&mut self, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        match self {
            Self::Http(client) => client.poll_ready(cx),
        }
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        match self {
            Self::Http(client) => client.call(req),
        }
    }
}

impl<C, Cr> Client<DropContextService<HyperClient, C>, C, Cr>
where
    C: Clone + Send + Sync + 'static,
{
    /// Create an HTTP client.
    ///
    /// # Arguments
    /// * `base_path` - base path of the client API, i.e. <http://0.0.0.0:8000>
    pub fn try_new(base_path: &str) -> Result<Self, ClientInitError> {
        let uri = Uri::from_str(base_path)?;

        let scheme = uri.scheme_str().ok_or(ClientInitError::InvalidScheme)?;
        let scheme = scheme.to_ascii_lowercase();

        let connector = Connector::builder();

        let client_service = match scheme.as_str() {
            "http" => HyperClient::Http(hyper::client::Client::builder().build(connector.build())),

            _ => {
                return Err(ClientInitError::InvalidScheme);
            }
        };

        let client_service = DropContextService::new(client_service);

        Ok(Self {
            client_service,
            base_path: into_base_path(base_path, None)?,
            con_marker: PhantomData,
            headers: HeaderMap::new(),
            cred_marker: PhantomData,
        })
    }
}

impl<C, Cr>
    Client<DropContextService<hyper::client::Client<hyper::client::HttpConnector, Body>, C>, C, Cr>
where
    C: Clone + Send + Sync + 'static,
{
    /// Create an HTTP client.
    ///
    /// # Arguments
    /// * `base_path` - base path of the client API, i.e. <http://www.example.com>
    pub fn try_new_http(base_path: &str, headers: HeaderMap) -> Result<Self, ClientInitError> {
        let http_connector = Connector::builder().build();

        Self::try_new_with_connector(base_path, Some("http"), http_connector, headers)
    }
}

impl<S, C, Cr> Client<S, C, Cr>
where
    S: Service<(Request<Body>, C), Response = Response<Body>> + Clone + Sync + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<super::api::ServiceError> + fmt::Display,
    C: Clone + Send + Sync + 'static,
{
    /// Constructor for creating a `Client` by passing in a pre-made `hyper::service::Service` /
    /// `tower::Service`
    ///
    /// This allows adding custom wrappers around the underlying transport, for example for logging.
    pub fn try_new_with_client_service(
        client_service: S,
        base_path: &str,
        headers: HeaderMap,
    ) -> Result<Self, ClientInitError> {
        Ok(Self {
            client_service,
            base_path: into_base_path(base_path, None)?,
            con_marker: PhantomData,
            headers,
            cred_marker: PhantomData,
        })
    }
}

/// Error type failing to create a Client
#[derive(Debug)]
pub enum ClientInitError {
    /// Invalid URL Scheme
    InvalidScheme,

    /// Invalid URI
    InvalidUri(hyper::http::uri::InvalidUri),

    /// Missing Hostname
    MissingHost,

    /// Couldn't connect to all cluster
    NoCluster,
}

impl From<hyper::http::uri::InvalidUri> for ClientInitError {
    fn from(err: hyper::http::uri::InvalidUri) -> Self {
        Self::InvalidUri(err)
    }
}

impl fmt::Display for ClientInitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s: &dyn fmt::Debug = self;
        s.fmt(f)
    }
}

impl Error for ClientInitError {
    fn description(&self) -> &str {
        "Failed to produce a hyper client."
    }
}

impl<S, C, Cr> Client<S, C, Cr>
where
    Cr: Credentials + Clone,
    S: Service<(Request<Body>, C), Response = Response<Body>> + Clone + Sync + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<super::api::ServiceError> + fmt::Display,
    C: Has<RequestTimeout>
        + Has<XSpanIdString>
        + Has<Option<Authorization<Cr>>>
        + Clone
        + Send
        + Sync
        + 'static,
{
    fn form_get_request(&self, endpoint: &str, context: &C) -> Result<Request<Body>, ApiError> {
        let uri = format!("{}{endpoint}", self.base_path);

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {err}"))),
        };

        let mut request = match Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(err) => return Err(ApiError(format!("Unable to create request: {err}"))),
        };

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str())
            .map_err(|err| ApiError(format!("Unable to create X-Span ID header value: {err}")))?;

        request
            .headers_mut()
            .insert(HeaderName::from_static("x-span-id"), header);

        if let Some(auth) = Has::<Option<Authorization<Cr>>>::get(context) {
            request
                .headers_mut()
                .typed_insert::<Authorization<Cr>>(auth.clone());
        }
        for (key, value) in self.headers.iter() {
            request.headers_mut().insert(key.clone(), value.clone());
        }

        Ok(request)
    }

    async fn handle_response_json<R: DeserializeOwned, T>(
        &self,
        response: Response<Body>,
        body_handler: impl Fn(R) -> T + Send,
    ) -> Result<T, ApiError> {
        let body = response.into_body();
        let body = body
            .into_raw()
            .map_err(|e| ApiError(format!("Failed to read response: {e}")))
            .await?;
        let body = str::from_utf8(&body)
            .map_err(|e| ApiError(format!("Response was not valid UTF8: {e}")))?;

        let body = serde_json::from_str::<R>(body)
            .map_err(|e| ApiError(format!("Response body did not match the schema: {e}")))?;

        Ok(body_handler(body))
    }

    async fn err_from_code(&self, response: Response<Body>) -> ApiError {
        let code = response.status().as_u16();
        let headers = response.headers().clone();
        let body = response.into_body().take(100).into_raw().await;
        ApiError(format!(
            "Unexpected response code {}:\n{:?}\n\n{}",
            code,
            headers,
            match body {
                Ok(body) => match String::from_utf8(body) {
                    Ok(body) => body,
                    Err(e) => format!("<Body was not UTF8: {e:?}>"),
                },
                Err(e) => format!("<Failed to read body: {e}>"),
            }
        ))
    }

    async fn call(&self, req: Request<Body>, cx: &C) -> Result<Response<Body>, ApiError> {
        let duration = Has::<RequestTimeout>::get(cx).clone().into_inner();
        tokio::time::timeout(
            duration,
            self.client_service.clone().call((req, cx.clone())),
        )
        .map_err(|err| ApiError(format!("Hyper - No response received: {err}")))
        .await?
        .map_err(|_| ApiError(format!("No response received")))
    }
}

#[async_trait]
impl<S, C, Cr> Api<C> for Client<S, C, Cr>
where
    Cr: Credentials + Clone,
    S: Service<(Request<Body>, C), Response = Response<Body>> + Clone + Sync + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<super::api::ServiceError> + fmt::Display,
    C: Has<RequestTimeout>
        + Has<XSpanIdString>
        + Has<Option<Authorization<Cr>>>
        + Clone
        + Send
        + Sync
        + 'static,
{
    fn poll_ready(&self, cx: &mut Context) -> Poll<Result<(), super::api::ServiceError>> {
        match self.client_service.clone().poll_ready(cx) {
            Poll::Ready(Err(e)) => Poll::Ready(Err(e.into())),
            Poll::Ready(Ok(o)) => Poll::Ready(Ok(o)),
            Poll::Pending => Poll::Pending,
        }
    }

    async fn get_alien_dir(&self, context: &C) -> Result<GetAlienDirResponse, ApiError> {
        let request = self.form_get_request("/alien/dir", context)?;
        let response = self.call(request, context).await?;

        match response.status().as_u16() {
            200 => Ok(self
                .handle_response_json(response, |body: dto::Dir| {
                    GetAlienDirResponse::Directory(body)
                })
                .await?),
            403 => Ok(self
                .handle_response_json(response, |body: dto::StatusExt| {
                    GetAlienDirResponse::PermissionDenied(body)
                })
                .await?),
            406 => Ok(self
                .handle_response_json(response, |body: dto::StatusExt| {
                    GetAlienDirResponse::NotAcceptableBackend(body)
                })
                .await?),
            _ => Err(self.err_from_code(response).await),
        }
    }

    async fn get_disks(&self, context: &C) -> Result<GetDisksResponse, ApiError> {
        let request = self.form_get_request("/disks/list", context)?;
        let response = self.call(request, context).await?;

        match response.status().as_u16() {
            200 => Ok(self
                .handle_response_json(response, |body: Vec<dto::DiskState>| {
                    GetDisksResponse::AJSONArrayWithDisksAndTheirStates(body)
                })
                .await?),
            403 => Ok(self
                .handle_response_json(response, |body: dto::StatusExt| {
                    GetDisksResponse::PermissionDenied(body)
                })
                .await?),
            _ => Err(self.err_from_code(response).await),
        }
    }

    async fn get_metrics(&self, context: &C) -> Result<GetMetricsResponse, ApiError> {
        let request = self.form_get_request("/metrics", context)?;
        let response = self.call(request, context).await?;

        match response.status().as_u16() {
            200 => Ok(self
                .handle_response_json(response, |body: dto::MetricsSnapshotModel| {
                    GetMetricsResponse::Metrics(body)
                })
                .await?),
            _ => Err(self.err_from_code(response).await),
        }
    }

    async fn get_nodes(&self, context: &C) -> Result<GetNodesResponse, ApiError> {
        let request = self.form_get_request("/nodes", context)?;
        let response = self.call(request, context).await?;

        match response.status().as_u16() {
            200 => Ok(self
                .handle_response_json(response, |body: Vec<dto::Node>| {
                    GetNodesResponse::AJSONArrayOfNodesInfoAndVdisksOnThem(body)
                })
                .await?),
            403 => Ok(GetNodesResponse::PermissionDenied),
            _ => Err(self.err_from_code(response).await),
        }
    }

    async fn get_partition(
        &self,
        param_v_disk_id: i32,
        param_partition_id: String,
        context: &C,
    ) -> Result<GetPartitionResponse, ApiError> {
        let request = self.form_get_request(
            &format!(
                "/vdisks/{v_disk_id}/partitions/{partition_id}",
                v_disk_id = utf8_percent_encode(&param_v_disk_id.to_string(), ID_ENCODE_SET),
                partition_id = utf8_percent_encode(&param_partition_id.to_string(), ID_ENCODE_SET)
            ),
            context,
        )?;
        let response = self.call(request, context).await?;

        match response.status().as_u16() {
            200 => Ok(self
                .handle_response_json(response, |body: dto::Partition| {
                    GetPartitionResponse::AJSONWithPartitionInfo(body)
                })
                .await?),
            403 => Ok(self
                .handle_response_json(response, |body: dto::StatusExt| {
                    GetPartitionResponse::PermissionDenied(body)
                })
                .await?),
            404 => Ok(self
                .handle_response_json(response, |body: dto::StatusExt| {
                    GetPartitionResponse::NotFound(body)
                })
                .await?),
            _ => Err(self.err_from_code(response).await),
        }
    }

    async fn get_partitions(
        &self,
        param_v_disk_id: i32,
        context: &C,
    ) -> Result<GetPartitionsResponse, ApiError> {
        let request = self.form_get_request(
            &format!(
                "/vdisks/{v_disk_id}/partitions",
                v_disk_id = utf8_percent_encode(&param_v_disk_id.to_string(), ID_ENCODE_SET)
            ),
            context,
        )?;
        let response = self.call(request, context).await?;

        match response.status().as_u16() {
            200 => Ok(self
                .handle_response_json(response, |body: dto::VDiskPartitions| {
                    GetPartitionsResponse::NodeInfoAndJSONArrayWithPartitionsInfo(body)
                })
                .await?),
            403 => Ok(self
                .handle_response_json(response, |body: dto::StatusExt| {
                    GetPartitionsResponse::PermissionDenied(body)
                })
                .await?),

            404 => Ok(self
                .handle_response_json(response, |body: dto::StatusExt| {
                    GetPartitionsResponse::NotFound(body)
                })
                .await?),
            _ => Err(self.err_from_code(response).await),
        }
    }

    async fn get_records(
        &self,
        param_v_disk_id: i32,
        context: &C,
    ) -> Result<GetRecordsResponse, ApiError> {
        let request = self.form_get_request(
            &format!(
                "/vdisks/{v_disk_id}/records/count",
                v_disk_id = utf8_percent_encode(&param_v_disk_id.to_string(), ID_ENCODE_SET)
            ),
            context,
        )?;
        let response = self.call(request, context).await?;

        match response.status().as_u16() {
            200 => Ok(self
                .handle_response_json(response, GetRecordsResponse::RecordsCount)
                .await?),
            403 => Ok(self
                .handle_response_json(response, |body: dto::StatusExt| {
                    GetRecordsResponse::PermissionDenied(body)
                })
                .await?),
            404 => Ok(self
                .handle_response_json(response, |body: dto::StatusExt| {
                    GetRecordsResponse::NotFound(body)
                })
                .await?),
            _ => Err(self.err_from_code(response).await),
        }
    }

    async fn get_replicas_local_dirs(
        &self,
        param_v_disk_id: i32,
        context: &C,
    ) -> Result<GetReplicasLocalDirsResponse, ApiError> {
        let request = self.form_get_request(
            &format!(
                "/vdisks/{v_disk_id}/replicas/local/dirs",
                v_disk_id = utf8_percent_encode(&param_v_disk_id.to_string(), ID_ENCODE_SET)
            ),
            context,
        )?;
        let response = self.call(request, context).await?;

        match response.status().as_u16() {
            200 => Ok(self
                .handle_response_json(response, |body: Vec<dto::Dir>| {
                    GetReplicasLocalDirsResponse::AJSONArrayWithDirs(body)
                })
                .await?),
            403 => Ok(self
                .handle_response_json(response, |body: dto::StatusExt| {
                    GetReplicasLocalDirsResponse::PermissionDenied(body)
                })
                .await?),
            404 => Ok(self
                .handle_response_json(response, |body: dto::StatusExt| {
                    GetReplicasLocalDirsResponse::NotFound(body)
                })
                .await?),
            _ => Err(self.err_from_code(response).await),
        }
    }

    async fn get_space_info(&self, context: &C) -> Result<GetSpaceInfoResponse, ApiError> {
        let request = self.form_get_request("/status/space", context)?;
        let response = self.call(request, context).await?;

        match response.status().as_u16() {
            200 => Ok(self
                .handle_response_json(response, |body: dto::SpaceInfo| {
                    GetSpaceInfoResponse::SpaceInfo(body)
                })
                .await?),
            _ => Err(self.err_from_code(response).await),
        }
    }

    async fn get_status(&self, context: &C) -> Result<GetStatusResponse, ApiError> {
        let request = self.form_get_request("/status", context)?;
        let response = self.call(request, context).await?;

        match response.status().as_u16() {
            200 => Ok(self
                .handle_response_json(response, |body: dto::Node| {
                    GetStatusResponse::AJSONWithNodeInfo(body)
                })
                .await?),
            _ => Err(self.err_from_code(response).await),
        }
    }

    async fn get_v_disk(
        &self,
        param_v_disk_id: i32,
        context: &C,
    ) -> Result<GetVDiskResponse, ApiError> {
        let request = self.form_get_request(
            &format!(
                "/vdisks/{v_disk_id}",
                v_disk_id = utf8_percent_encode(&param_v_disk_id.to_string(), ID_ENCODE_SET)
            ),
            context,
        )?;
        let response = self.call(request, context).await?;

        match response.status().as_u16() {
            200 => Ok(self
                .handle_response_json(response, |body: dto::VDisk| {
                    GetVDiskResponse::AJSONWithVdiskInfo(body)
                })
                .await?),
            403 => Ok(self
                .handle_response_json(response, |body: dto::StatusExt| {
                    GetVDiskResponse::PermissionDenied(body)
                })
                .await?),
            404 => Ok(self
                .handle_response_json(response, |body: dto::StatusExt| {
                    GetVDiskResponse::NotFound(body)
                })
                .await?),
            _ => Err(self.err_from_code(response).await),
        }
    }

    async fn get_v_disks(&self, context: &C) -> Result<GetVDisksResponse, ApiError> {
        let request = self.form_get_request("/vdisks", context)?;
        let response = self.call(request, context).await?;

        match response.status().as_u16() {
            200 => Ok(self
                .handle_response_json(response, |body: Vec<dto::VDisk>| {
                    GetVDisksResponse::AJSONArrayOfVdisksInfo(body)
                })
                .await?),
            403 => Ok(GetVDisksResponse::PermissionDenied),

            _ => Err(self.err_from_code(response).await),
        }
    }

    async fn get_version(&self, context: &C) -> Result<GetVersionResponse, ApiError> {
        let request = self.form_get_request("/version", context)?;
        let response = self.call(request, context).await?;

        match response.status().as_u16() {
            200 => Ok(self
                .handle_response_json(response, |body: dto::VersionInfo| {
                    GetVersionResponse::VersionInfo(body)
                })
                .await?),

            _ => Err(self.err_from_code(response).await),
        }
    }

    async fn get_configuration(&self, context: &C) -> Result<GetConfigurationResponse, ApiError> {
        let request = self.form_get_request("/configuration", context)?;
        let response = self.call(request, context).await?;

        match response.status().as_u16() {
            200 => Ok(self
                .handle_response_json(response, |body: dto::NodeConfiguration| {
                    GetConfigurationResponse::ConfigurationObject(body)
                })
                .await?),
            403 => Ok(GetConfigurationResponse::PermissionDenied),

            _ => Err(self.err_from_code(response).await),
        }
    }
}
