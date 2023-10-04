use super::dto::BobConnectionData;
use crate::models::models::RequestTimeout;
use crate::prelude::*;
use crate::{connector::BobClient, models::models::Hostname};
use axum::Extension;
use axum::{
    async_trait,
    extract::State,
    headers::{authorization::Basic, Authorization},
    http::StatusCode,
    Json,
};
use axum_login::{
    extractors::AuthContext, memory_store::MemoryStore, secrecy::SecretVec, AuthUser, UserStore,
};
use serde::Deserialize;
use std::sync::Arc;
use std::{collections::HashMap, fmt::Display, hash::Hash};
use swagger::Has;
use tokio::sync::RwLock;

pub type SocketBobMemoryStore = MemoryStore<Hostname, BobClient>;
pub type BobStore<Id> = HashMap<Id, BobClient>;
pub type SharedBobStore<Id> = Arc<RwLock<BobStore<Id>>>;

/// Optional credentials for a BOB cluster
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Deserialize)]
pub struct Credentials {
    pub login: String,
    pub password: String,
}

/// Data needed to login to a BOB cluster
impl AuthUser<Hostname> for BobClient {
    fn get_id(&self) -> Hostname {
        self.hostname().clone()
    }

    fn get_password_hash(&self) -> SecretVec<u8> {
        Has::<Option<Authorization<Basic>>>::get(self.context())
            .as_ref()
            .map_or_else(
                || SecretVec::new("".into()),
                |cred| SecretVec::new(cred.0.password().as_bytes().to_vec()),
            )
    }
}

/// Login to a BOB cluster
///
/// # Errors
/// This function can return the following errors
///
/// 1. [`StatusCode::BAD_REQUEST`]
/// The function failed to parse hostname of the request
///
/// 2. [`StatusCode::NOT_FOUND`]
/// The client was unbale to reach the host
///
/// 3. [`StatusCode::UNAUTHORIZED`]
/// The client could.t authorize on the host
///
pub async fn login(
    State(store): State<Arc<RwLock<BobStore<Hostname>>>>,
    mut auth: AuthContext<Hostname, BobClient, SocketBobMemoryStore>,
    Extension(request_timeout): Extension<RequestTimeout>,
    Json(bob): Json<BobConnectionData>,
) -> AxumResult<StatusCode> {
    tracing::info!("post /login : {:?}", &bob);
    let hostname = bob.hostname.clone();

    let Ok(connector) = BobClient::try_new(bob, request_timeout).await else {
        tracing::warn!("Couldn't create client");
        return Err(StatusCode::UNAUTHORIZED.into());
    };
    let Ok(res) = connector.probe().await else {
        return Err(StatusCode::UNAUTHORIZED.into());
    };
    tracing::info!("received {res} from BOB");

    if res == StatusCode::OK {
        if let Err(err) = auth.login(&connector).await {
            tracing::warn!("Couldn't login the user. Err: {err}, User: {connector:?}");
            return Err(StatusCode::UNAUTHORIZED.into());
        };
        store.write().await.insert(hostname, connector);
        tracing::info!("AUTHORIZATION SUCCESSFUL");
        tracing::info!("Logged in as {:?}", &auth.current_user);
    }

    Ok(res)
}

/// Logout from a BOB cluster
pub async fn logout(mut auth: AuthContext<Hostname, BobClient, SocketBobMemoryStore>) {
    tracing::info!("get /logout : {:?}", &auth.current_user);
    auth.logout().await;
}

/// An ephemeral in-memory store, since we don't need to store any users.
#[derive(Clone, Debug, Default)]
pub struct BobMemoryStore<UserId> {
    inner: Arc<RwLock<BobStore<UserId>>>,
}

impl<UserId> BobMemoryStore<UserId> {
    /// Creates a new in-memory store.
    pub fn new(inner: &Arc<RwLock<HashMap<UserId, BobClient>>>) -> Self {
        Self {
            inner: inner.clone(),
        }
    }
}

#[async_trait]
impl<UserId, Role> UserStore<UserId, Role> for BobMemoryStore<UserId>
where
    Role: PartialOrd + PartialEq + Clone + Send + Sync + 'static,
    UserId: Display + Eq + Clone + Send + Sync + Hash + 'static,
    BobClient: AuthUser<UserId, Role>,
{
    type User = BobClient;
    type Error = APIError;

    async fn load_user(&self, user_id: &UserId) -> Result<Option<Self::User>, Self::Error> {
        tracing::debug!("load_user: {}", user_id);

        Ok(self.inner.read().await.get(user_id).cloned())
    }
}
