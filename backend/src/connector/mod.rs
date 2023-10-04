use self::{api::ApiNoContext, api::ContextWrapperExt, client::Client};
use crate::models::models::{BobConnectionData, Hostname, RequestTimeout};
use crate::prelude::*;
use crate::services::dto::NodeName;
use axum::headers::{authorization::Basic, Authorization};
use hyper::{HeaderMap, StatusCode};
use std::{collections::HashMap, sync::Arc};
use swagger::{new_context_type, Has, Push, XSpanIdString};

pub mod api;
pub mod client;
pub mod dto;

new_context_type!(
    BobContext,
    EmptyBobContext,
    RequestTimeout,
    Option<Authorization<Basic>>,
    XSpanIdString
);
type ClientContext = swagger::make_context_ty!(
    BobContext,
    EmptyBobContext,
    RequestTimeout,
    Option<Authorization<Basic>>,
    XSpanIdString
);

pub type ApiInterface = dyn ApiNoContext<ClientContext> + Send + Sync; // hide ugly type

#[derive(Clone)]
pub struct BobClient {
    /// Bob's hostname
    hostname: Hostname,
    // Can (and should) the client mutate?..
    /// Underlying hyper client
    main: Arc<ApiInterface>,
    /// Clients for all known nodes
    cluster: HashMap<NodeName, Arc<ApiInterface>>,
}

impl std::fmt::Debug for BobClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let user = Has::<Option<Authorization<Basic>>>::get(self.context())
            .as_ref()
            .map_or("Unknown", |cred| cred.username());
        f.debug_struct("BobClient")
            .field("hostname", &self.hostname)
            .field("user", &user)
            .finish()
    }
}

impl BobClient {
    /// Creates new [`BobClient`] from [`BobConnectionData`]
    ///
    /// # Errors
    /// The function will fail if a hostname isn't a valid url
    pub async fn try_new(bob_data: BobConnectionData, timeout: RequestTimeout) -> Result<Self> {
        let auth = if let Some(creds) = bob_data.credentials {
            Some(Authorization::basic(&creds.login, &creds.password))
        } else {
            None
        };
        let hostname = bob_data.hostname.clone();

        let context: ClientContext = swagger::make_context!(
            BobContext,
            EmptyBobContext,
            timeout,
            auth,
            XSpanIdString::default()
        );

        let client = Box::new(Client::try_new_http(
            &hostname.to_string(),
            HeaderMap::new(),
        )?);
        let nodes = client
            .clone()
            .with_context(context.clone())
            .get_nodes()
            .await
            .map_err(|_| APIError::RequestFailed)?;
        let api::GetNodesResponse::AJSONArrayOfNodesInfoAndVdisksOnThem(nodes) = nodes else {
            return Err(APIError::InvalidStatusCode(StatusCode::FORBIDDEN).into());
        };

        let cluster: HashMap<NodeName, Arc<ApiInterface>> = nodes
            .iter()
            .map(|node| {
                (
                    &node.name,
                    Self::change_port(&node.address, &bob_data.hostname),
                )
            })
            .filter_map(|(name, hostname)| {
                if hostname.is_err() {
                    tracing::warn!("couldn't change port for {name}. Client won't be created");
                }
                hostname.ok().map(|hostname| {
                    (
                        name,
                        Client::try_new_http(&hostname.to_string(), HeaderMap::new()),
                    )
                })
            })
            .filter_map(|(name, client)| {
                if client.is_err() {
                    tracing::warn!("couldn't create client for {hostname}");
                }
                client.ok().map(|client| {
                    (
                        name.clone(),
                        Arc::new(client.with_context(context.clone())) as Arc<ApiInterface>,
                    )
                })
            })
            .collect();

        Ok(Self {
            hostname: bob_data.hostname,
            main: Arc::new(client.with_context(context)),
            cluster,
        })
    }

    /// Probes connection to the Bob cluster
    ///
    /// Returns [`StatusCode`], that it received from Bob
    ///
    /// # Errors
    ///
    /// The function fails if there was an error during creation of request
    /// It shouldn't happen on normal behaviour
    ///
    pub async fn probe(&self) -> Result<StatusCode> {
        match self.main.get_nodes().await? {
            api::GetNodesResponse::AJSONArrayOfNodesInfoAndVdisksOnThem(_) => Ok(StatusCode::OK),
            api::GetNodesResponse::PermissionDenied => Ok(StatusCode::UNAUTHORIZED),
        }
    }

    /// Probes connection to the `hostname`
    ///
    /// # Errors
    ///
    /// This function will return an error if no client present for the specified `hostname` or
    /// the client was unable to send request
    pub async fn probe_socket(&self, name: &NodeName) -> Result<StatusCode> {
        if let Some(client) = self.cluster.get(name) {
            match client.get_nodes().await? {
                api::GetNodesResponse::AJSONArrayOfNodesInfoAndVdisksOnThem(_) => {
                    Ok(StatusCode::OK)
                }
                api::GetNodesResponse::PermissionDenied => Ok(StatusCode::UNAUTHORIZED),
            }
        } else {
            Err(eyre!("no client present for node"))
        }
    }

    #[must_use]
    pub fn context(&self) -> &ClientContext {
        self.main.context()
    }

    #[must_use]
    pub fn api(&self) -> &Arc<ApiInterface> {
        &self.main
    }

    pub fn cluster(&self) -> impl Iterator<Item = &Arc<ApiInterface>> {
        self.cluster.values()
    }

    #[must_use]
    pub fn cluster_with_addr(&self) -> &HashMap<NodeName, Arc<ApiInterface>> {
        &self.cluster
    }

    /// Changes port of the address with specified [`Hostname`]
    ///
    /// # Errors
    ///
    /// This function will return an error if address couldn't be converted into [`SocketAddr`]
    pub fn change_port(address: &str, src_hostname: &Hostname) -> Result<Hostname> {
        let (body, _) = address.rsplit_once(':').ok_or(eyre!("bad address"))?;
        let mut body = body.to_string();
        body.push_str(&format!(":{:?}", src_hostname.port()));

        Ok(Hostname::new(body)?)
    }

    #[must_use]
    pub const fn hostname(&self) -> &Hostname {
        &self.hostname
    }
}
