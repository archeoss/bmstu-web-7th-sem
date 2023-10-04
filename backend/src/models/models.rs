use nutype::nutype;
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, time::Duration};

/// Time limit for waiting a responce, in milliseconds.
#[derive(Clone, Debug)]
pub struct Timeout(pub u64);

#[nutype(sanitize(trim, lowercase) validate(not_empty, regex = r"(https?)?(?::[\/]{2})?([^\:]+):(([1-9][0-9]{0,3}|[1-5][0-9]{4}|6[0-4][0-9]{3}|65[0-4][0-9]{2}|655[0-2][0-9]|6553[0-5])$)"))]
#[derive(Display, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Hostname(String);

impl Hostname {
    /// Can be safely unwraped thanks to regex
    #[must_use]
    #[allow(clippy::pedantic, clippy::unwrap_used)]
    pub fn port(&self) -> u16 {
        self.clone()
            .into_inner()
            .rsplit(':')
            .next()
            .unwrap()
            .parse::<u16>()
            .unwrap()
    }
}

impl TryFrom<SocketAddr> for Hostname {
    type Error = HostnameError;

    fn try_from(value: SocketAddr) -> Result<Self, Self::Error> {
        Self::new(value.to_string())
    }
}

impl TryFrom<Hostname> for SocketAddr {
    type Error = std::net::AddrParseError;

    fn try_from(value: Hostname) -> Result<Self, Self::Error> {
        value.to_string().parse()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RequestTimeout(Duration);

impl RequestTimeout {
    #[must_use]
    pub const fn from_millis(millis: u64) -> Self {
        Self(Duration::from_millis(millis))
    }

    #[must_use]
    pub const fn into_inner(self) -> Duration {
        self.0
    }
}
/// Data needed to connect to a BOB cluster
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct BobConnectionData {
    /// Address to connect to
    pub hostname: Hostname,

    /// [Optional] Credentials used for BOB authentication
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credentials: Option<Credentials>,
}

/// Optional auth credentials for a BOB cluster
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub struct Credentials {
    /// Login used during auth
    pub login: String,

    /// Password used during auth
    pub password: String,
}
