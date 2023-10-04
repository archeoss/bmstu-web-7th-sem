use std::collections::HashMap;

type StdError = dyn std::error::Error;

/// Function, used for parsing strings into DTOs
/// Accpets closures, that decides what to do with keys and values
fn parse<F>(s: &str, mut matcher: F) -> Result<(), Box<StdError>>
where
    F: FnMut(&str, &str) -> Result<(), Box<StdError>>,
{
    let mut string_iter = s.split(',');
    let mut key_result = string_iter.next();

    while key_result.is_some() {
        let Some(val) = string_iter.next() else {
                return Err("Missing value while parsing".into());
            };

        if let Some(key) = key_result {
            matcher(key, val)?;
        }

        // Get the next key
        key_result = string_iter.next();
    }
    Ok(())
}

#[derive(Default, Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Dir {
    #[serde(rename = "name")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(rename = "path")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    #[serde(rename = "children")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<Dir>>,
}

impl Dir {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            name: None,
            path: None,
            children: None,
        }
    }
}

/// Converts the Dir value to the Query Parameters representation (style=form, explode=false)
/// specified in <https://swagger.io/docs/specification/serialization/>
/// Should be implemented in a serde serializer
impl std::string::ToString for Dir {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![
            self.name
                .as_ref()
                .map(|name| vec!["name".to_string(), name.to_string()].join(",")),
            self.path
                .as_ref()
                .map(|path| vec!["path".to_string(), path.to_string()].join(",")),
            // Skipping children in query parameter serialization
        ];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Dir value
/// as specified in <https://swagger.io/docs/specification/serialization/>
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Dir {
    type Err = Box<StdError>;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub name: Vec<String>,
            pub path: Vec<String>,
            pub children: Vec<Vec<Dir>>,
        }
        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        parse(s, |key, val| {
            match key {
                "name" => intermediate_rep
                    .name
                    .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                "path" => intermediate_rep
                    .path
                    .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                "children" => {
                    return Err("Parsing a container in this style is not supported in Dir".into())
                }
                _ => return Err("Unexpected key while parsing Dir".into()),
            }
            Ok(())
        })?;

        // Use the intermediate representation to return the struct
        Ok(Self {
            name: intermediate_rep.name.into_iter().next(),
            path: intermediate_rep.path.into_iter().next(),
            children: intermediate_rep.children.into_iter().next(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct DiskState {
    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "path")]
    pub path: String,

    #[serde(rename = "is_active")]
    pub is_active: bool,
}

impl DiskState {
    /// Creates a new [`DiskState`].
    #[must_use]
    pub const fn new(name: String, path: String, is_active: bool) -> Self {
        Self {
            name,
            path,
            is_active,
        }
    }
}

/// Converts the [`DiskState`] value to the Query Parameters representation (style=form, explode=false)
/// specified in <https://swagger.io/docs/specification/serialization/>
/// Should be implemented in a serde serializer
impl std::string::ToString for DiskState {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![
            Some("name".to_string()),
            Some(self.name.to_string()),
            Some("path".to_string()),
            Some(self.path.to_string()),
            Some("is_active".to_string()),
            Some(self.is_active.to_string()),
        ];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a [`DiskState`] value
/// as specified in <https://swagger.io/docs/specification/serialization/>
/// Should be implemented in a serde deserializer
impl std::str::FromStr for DiskState {
    type Err = Box<StdError>;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub name: Vec<String>,
            pub path: Vec<String>,
            pub is_active: Vec<bool>,
        }
        let mut intermediate_rep = IntermediateRep::default();

        parse(s, |key, val| {
            match key {
                "name" => intermediate_rep
                    .name
                    .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                "path" => intermediate_rep
                    .path
                    .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                "is_active" => intermediate_rep
                    .is_active
                    .push(<bool as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                _ => return Err("Unexpected key while parsing DiskState".into()),
            }
            Ok(())
        })?;

        // Use the intermediate representation to return the struct
        Ok(Self {
            name: intermediate_rep
                .name
                .into_iter()
                .next()
                .ok_or_else(|| "name missing in DiskState".to_string())?,
            path: intermediate_rep
                .path
                .into_iter()
                .next()
                .ok_or_else(|| "path missing in DiskState".to_string())?,
            is_active: intermediate_rep
                .is_active
                .into_iter()
                .next()
                .ok_or_else(|| "is_active missing in DiskState".to_string())?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct DistrFunc {
    // Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "func")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub func: Option<String>,
}

impl DistrFunc {
    #[must_use]
    pub const fn new() -> Self {
        Self { func: None }
    }
}

/// Converts the [`DistrFunc`] value to the Query Parameters representation (style=form, explode=false)
/// specified in <https://swagger.io/docs/specification/serialization/>
/// Should be implemented in a serde serializer
impl std::string::ToString for DistrFunc {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![self
            .func
            .as_ref()
            .map(|func| vec!["func".to_string(), func.to_string()].join(","))];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a [`DistrFunc`] value
/// as specified in <https://swagger.io/docs/specification/serialization/>
/// Should be implemented in a serde deserializer
impl std::str::FromStr for DistrFunc {
    type Err = Box<StdError>;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub func: Vec<String>,
        }
        let mut intermediate_rep = IntermediateRep::default();
        // Parse into intermediate representation
        parse(s, |key, val| {
            match key {
                "func" => intermediate_rep
                    .func
                    .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                _ => return Err("Unexpected key while parsing DistrFunc".into()),
            }
            Ok(())
        })?;

        // Use the intermediate representation to return the struct
        Ok(Self {
            func: intermediate_rep.func.into_iter().next(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Error {
    #[serde(rename = "code")]
    pub code: String,

    #[serde(rename = "message")]
    pub message: String,
}

impl Error {
    #[must_use]
    pub const fn new(code: String, message: String) -> Self {
        Self { code, message }
    }
}

/// Converts the [`Error`] value to the Query Parameters representation (style=form, explode=false)
/// specified in <https://swagger.io/docs/specification/serialization/>
/// Should be implemented in a serde serializer
impl std::string::ToString for Error {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![
            Some("code".to_string()),
            Some(self.code.to_string()),
            Some("message".to_string()),
            Some(self.message.to_string()),
        ];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a [`Error`] value
/// as specified in <https://swagger.io/docs/specification/serialization/>
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Error {
    type Err = Box<StdError>;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub code: Vec<String>,
            pub message: Vec<String>,
        }
        let mut intermediate_rep = IntermediateRep::default();
        parse(s, |key, val| {
            match key {
                "code" => intermediate_rep
                    .code
                    .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                "message" => intermediate_rep
                    .message
                    .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                _ => return Err("Unexpected key while parsing Error".into()),
            }
            Ok(())
        })?;

        // Use the intermediate representation to return the struct
        Ok(Self {
            code: intermediate_rep
                .code
                .into_iter()
                .next()
                .ok_or_else(|| "code missing in Error".to_string())?,
            message: intermediate_rep
                .message
                .into_iter()
                .next()
                .ok_or_else(|| "message missing in Error".to_string())?,
        })
    }
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct MetricsEntryModel {
    #[serde(rename = "value")]
    pub value: u64,

    #[serde(rename = "timestamp")]
    pub timestamp: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct MetricsSnapshotModel {
    #[serde(rename = "metrics")]
    pub metrics: HashMap<String, MetricsEntryModel>,
}

impl PartialEq for MetricsEntryModel {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl PartialOrd for MetricsEntryModel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl Ord for MetricsEntryModel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

impl Eq for MetricsEntryModel {}
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Node {
    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "address")]
    pub address: String,

    #[serde(rename = "vdisks")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vdisks: Option<Vec<VDisk>>,
}

impl Node {
    #[must_use]
    pub const fn new(name: String, address: String) -> Self {
        Self {
            name,
            address,
            vdisks: None,
        }
    }
}

/// Converts the [`Node`] value to the Query Parameters representation (style=form, explode=false)
/// specified in <https://swagger.io/docs/specification/serialization/>
/// Should be implemented in a serde serializer
impl std::string::ToString for Node {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![
            Some("name".to_string()),
            Some(self.name.to_string()),
            Some("address".to_string()),
            Some(self.address.to_string()),
            // Skipping vdisks in query parameter serialization
        ];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a [`Node`] value
/// as specified in <https://swagger.io/docs/specification/serialization/>
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Node {
    type Err = Box<StdError>;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub name: Vec<String>,
            pub address: Vec<String>,
            pub vdisks: Vec<Vec<VDisk>>,
        }

        let mut intermediate_rep = IntermediateRep::default();
        parse(s, |key, val| {
            match key {
                "name" => intermediate_rep
                    .name
                    .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                "address" => intermediate_rep
                    .address
                    .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                "vdisks" => {
                    return Err("Parsing a container in this style is not supported in Node".into())
                }
                _ => return Err("Unexpected key while parsing Node".into()),
            }
            Ok(())
        })?;

        Ok(Self {
            name: intermediate_rep
                .name
                .into_iter()
                .next()
                .ok_or_else(|| "name missing in Node".to_string())?,
            address: intermediate_rep
                .address
                .into_iter()
                .next()
                .ok_or_else(|| "address missing in Node".to_string())?,
            vdisks: intermediate_rep.vdisks.into_iter().next(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct NodeConfiguration {
    #[serde(rename = "blob_file_name_prefix")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob_file_name_prefix: Option<String>,

    #[serde(rename = "root_dir_name")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_dir_name: Option<String>,
}

impl NodeConfiguration {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            blob_file_name_prefix: None,
            root_dir_name: None,
        }
    }
}

/// Converts the [`NodeConfiguration`] value to the Query Parameters representation (style=form, explode=false)
/// specified in <https://swagger.io/docs/specification/serialization/>
/// Should be implemented in a serde serializer
impl std::string::ToString for NodeConfiguration {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![
            self.blob_file_name_prefix
                .as_ref()
                .map(|blob_file_name_prefix| {
                    vec![
                        "blob_file_name_prefix".to_string(),
                        blob_file_name_prefix.to_string(),
                    ]
                    .join(",")
                }),
            self.root_dir_name.as_ref().map(|root_dir_name| {
                vec!["root_dir_name".to_string(), root_dir_name.to_string()].join(",")
            }),
        ];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a [`NodeConfiguration`] value
/// as specified in <https://swagger.io/docs/specification/serialization/>
/// Should be implemented in a serde deserializer
impl std::str::FromStr for NodeConfiguration {
    type Err = Box<StdError>;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub blob_file_name_prefix: Vec<String>,
            pub root_dir_name: Vec<String>,
        }
        let mut intermediate_rep = IntermediateRep::default();

        parse(s, |key, val| {
            match key {
                "blob_file_name_prefix" => intermediate_rep
                    .blob_file_name_prefix
                    .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                "root_dir_name" => intermediate_rep
                    .root_dir_name
                    .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                _ => return Err("Unexpected key while parsing NodeConfiguration".into()),
            }
            Ok(())
        })?;

        // Use the intermediate representation to return the struct
        Ok(Self {
            blob_file_name_prefix: intermediate_rep.blob_file_name_prefix.into_iter().next(),
            root_dir_name: intermediate_rep.root_dir_name.into_iter().next(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Partition {
    #[serde(rename = "vdisk_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vdisk_id: Option<i32>,

    #[serde(rename = "node_name")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_name: Option<String>,

    #[serde(rename = "disk_name")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disk_name: Option<String>,

    #[serde(rename = "timestamp")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<i32>,

    #[serde(rename = "records_count")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub records_count: Option<i32>,
}

impl Partition {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            vdisk_id: None,
            node_name: None,
            disk_name: None,
            timestamp: None,
            records_count: None,
        }
    }
}

/// Converts the [`Partition`] value to the Query Parameters representation (style=form, explode=false)
/// specified in <https://swagger.io/docs/specification/serialization/>
/// Should be implemented in a serde serializer
impl std::string::ToString for Partition {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![
            self.vdisk_id
                .as_ref()
                .map(|vdisk_id| vec!["vdisk_id".to_string(), vdisk_id.to_string()].join(",")),
            self.node_name
                .as_ref()
                .map(|node_name| vec!["node_name".to_string(), node_name.to_string()].join(",")),
            self.disk_name
                .as_ref()
                .map(|disk_name| vec!["disk_name".to_string(), disk_name.to_string()].join(",")),
            self.timestamp
                .as_ref()
                .map(|timestamp| vec!["timestamp".to_string(), timestamp.to_string()].join(",")),
            self.records_count.as_ref().map(|records_count| {
                vec!["records_count".to_string(), records_count.to_string()].join(",")
            }),
        ];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a [`Partition`] value
/// as specified in <https://swagger.io/docs/specification/serialization/>
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Partition {
    type Err = Box<StdError>;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub vdisk_id: Vec<i32>,
            pub node_name: Vec<String>,
            pub disk_name: Vec<String>,
            pub timestamp: Vec<i32>,
            pub records_count: Vec<i32>,
        }
        let mut intermediate_rep = IntermediateRep::default();

        parse(s, |key, val| {
            match key {
                "vdisk_id" => intermediate_rep
                    .vdisk_id
                    .push(<i32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                "node_name" => intermediate_rep
                    .node_name
                    .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                "disk_name" => intermediate_rep
                    .disk_name
                    .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                "timestamp" => intermediate_rep
                    .timestamp
                    .push(<i32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                "records_count" => intermediate_rep
                    .records_count
                    .push(<i32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                _ => return Err("Unexpected key while parsing Partition".into()),
            }
            Ok(())
        })?;

        // Use the intermediate representation to return the struct
        Ok(Self {
            vdisk_id: intermediate_rep.vdisk_id.into_iter().next(),
            node_name: intermediate_rep.node_name.into_iter().next(),
            disk_name: intermediate_rep.disk_name.into_iter().next(),
            timestamp: intermediate_rep.timestamp.into_iter().next(),
            records_count: intermediate_rep.records_count.into_iter().next(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Replica {
    #[serde(rename = "node")]
    pub node: String,

    #[serde(rename = "disk")]
    pub disk: String,

    #[serde(rename = "path")]
    pub path: String,
}

impl Replica {
    #[must_use]
    pub const fn new(node: String, disk: String, path: String) -> Self {
        Self { node, disk, path }
    }
}

/// Converts the [`Replica`] value to the Query Parameters representation (style=form, explode=false)
/// specified in <https://swagger.io/docs/specification/serialization/>
/// Should be implemented in a serde serializer
impl std::string::ToString for Replica {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![
            Some("node".to_string()),
            Some(self.node.to_string()),
            Some("disk".to_string()),
            Some(self.disk.to_string()),
            Some("path".to_string()),
            Some(self.path.to_string()),
        ];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a [`Replica`] value
/// as specified in <https://swagger.io/docs/specification/serialization/>
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Replica {
    type Err = Box<StdError>;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub node: Vec<String>,
            pub disk: Vec<String>,
            pub path: Vec<String>,
        }
        let mut intermediate_rep = IntermediateRep::default();

        parse(s, |key, val| {
            match key {
                "node" => intermediate_rep
                    .node
                    .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                "disk" => intermediate_rep
                    .disk
                    .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                "path" => intermediate_rep
                    .path
                    .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                _ => return Err("Unexpected key while parsing Replica".into()),
            }
            Ok(())
        })?;

        // Use the intermediate representation to return the struct
        Ok(Self {
            node: intermediate_rep
                .node
                .into_iter()
                .next()
                .ok_or_else(|| "node missing in Replica".to_string())?,
            disk: intermediate_rep
                .disk
                .into_iter()
                .next()
                .ok_or_else(|| "disk missing in Replica".to_string())?,
            path: intermediate_rep
                .path
                .into_iter()
                .next()
                .ok_or_else(|| "path missing in Replica".to_string())?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct SpaceInfo {
    #[serde(rename = "total_disk_space_bytes")]
    pub total_disk_space_bytes: u64,

    #[serde(rename = "free_disk_space_bytes")]
    pub free_disk_space_bytes: u64,

    #[serde(rename = "used_disk_space_bytes")]
    pub used_disk_space_bytes: u64,

    #[serde(rename = "occupied_disk_space_bytes")]
    pub occupied_disk_space_bytes: u64,

    #[serde(rename = "occupied_disk_space_by_disk")]
    pub occupied_disk_space_by_disk: std::collections::HashMap<String, u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct StatusExt {
    #[serde(rename = "status")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<i32>,

    #[serde(rename = "ok")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ok: Option<bool>,

    #[serde(rename = "msg")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg: Option<String>,
}

impl StatusExt {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            status: None,
            ok: None,
            msg: None,
        }
    }
}

/// Converts the [`StatusExt`] value to the Query Parameters representation (style=form, explode=false)
/// specified in <https://swagger.io/docs/specification/serialization/>
/// Should be implemented in a serde serializer
impl std::string::ToString for StatusExt {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![
            self.status
                .as_ref()
                .map(|status| vec!["status".to_string(), status.to_string()].join(",")),
            self.ok
                .as_ref()
                .map(|ok| vec!["ok".to_string(), ok.to_string()].join(",")),
            self.msg
                .as_ref()
                .map(|msg| vec!["msg".to_string(), msg.to_string()].join(",")),
        ];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a [`StatusExt`] value
/// as specified in <https://swagger.io/docs/specification/serialization/>
/// Should be implemented in a serde deserializer
impl std::str::FromStr for StatusExt {
    type Err = Box<StdError>;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub status: Vec<i32>,
            pub ok: Vec<bool>,
            pub msg: Vec<String>,
        }
        let mut intermediate_rep = IntermediateRep::default();

        parse(s, |key, val| {
            match key {
                "status" => intermediate_rep
                    .status
                    .push(<i32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                "ok" => intermediate_rep
                    .ok
                    .push(<bool as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                "msg" => intermediate_rep
                    .msg
                    .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                _ => return Err("Unexpected key while parsing StatusExt".into()),
            }
            Ok(())
        })?;

        // Use the intermediate representation to return the struct
        Ok(Self {
            status: intermediate_rep.status.into_iter().next(),
            ok: intermediate_rep.ok.into_iter().next(),
            msg: intermediate_rep.msg.into_iter().next(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct VDisk {
    #[serde(rename = "id")]
    pub id: i32,

    #[serde(rename = "replicas")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replicas: Option<Vec<Replica>>,
}

impl VDisk {
    #[must_use]
    pub const fn new(id: i32) -> Self {
        Self { id, replicas: None }
    }
}

/// Converts the [`VDisk`] value to the Query Parameters representation (style=form, explode=false)
/// specified in <https://swagger.io/docs/specification/serialization/>
/// Should be implemented in a serde serializer
impl std::string::ToString for VDisk {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![
            Some("id".to_string()),
            Some(self.id.to_string()),
            // Skipping replicas in query parameter serialization
        ];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a [`VDisk`] value
/// as specified in <https://swagger.io/docs/specification/serialization/>
/// Should be implemented in a serde deserializer
impl std::str::FromStr for VDisk {
    type Err = Box<StdError>;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub id: Vec<i32>,
            pub replicas: Vec<Vec<Replica>>,
        }
        let mut intermediate_rep = IntermediateRep::default();
        parse(s, |key, val| {
            match key {
                "id" => intermediate_rep
                    .id
                    .push(<i32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                "replicas" => {
                    return Err(
                        "Parsing a container in this style is not supported in VDisk".into(),
                    )
                }
                _ => return Err("Unexpected key while parsing VDisk".into()),
            }
            Ok(())
        })?;

        // Use the intermediate representation to return the struct
        Ok(Self {
            id: intermediate_rep
                .id
                .into_iter()
                .next()
                .ok_or_else(|| "id missing in VDisk".to_string())?,
            replicas: intermediate_rep.replicas.into_iter().next(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct VDiskPartitions {
    #[serde(rename = "vdisk")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vdisk: Option<i32>,

    #[serde(rename = "node")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node: Option<String>,

    #[serde(rename = "disk")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disk: Option<String>,

    #[serde(rename = "partitions")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partitions: Option<Vec<String>>,
}

impl VDiskPartitions {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            vdisk: None,
            node: None,
            disk: None,
            partitions: None,
        }
    }
}

/// Converts the [`VDiskPartitions`] value to the Query Parameters representation (style=form, explode=false)
/// specified in <https://swagger.io/docs/specification/serialization/>
/// Should be implemented in a serde serializer
impl std::string::ToString for VDiskPartitions {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![
            self.vdisk
                .as_ref()
                .map(|vdisk| vec!["vdisk".to_string(), vdisk.to_string()].join(",")),
            self.node
                .as_ref()
                .map(|node| vec!["node".to_string(), node.to_string()].join(",")),
            self.disk
                .as_ref()
                .map(|disk| vec!["disk".to_string(), disk.to_string()].join(",")),
            self.partitions.as_ref().map(|partitions| {
                vec![
                    "partitions".to_string(),
                    partitions
                        .iter()
                        .map(std::string::ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(","),
                ]
                .join(",")
            }),
        ];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a [`VDiskPartitions`] value
/// as specified in <https://swagger.io/docs/specification/serialization/>
/// Should be implemented in a serde deserializer
impl std::str::FromStr for VDiskPartitions {
    type Err = Box<StdError>;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub vdisk: Vec<i32>,
            pub node: Vec<String>,
            pub disk: Vec<String>,
            pub partitions: Vec<Vec<String>>,
        }
        let mut intermediate_rep = IntermediateRep::default();

        parse(s, |key, val| {
            match key {
                "vdisk" => intermediate_rep
                    .vdisk
                    .push(<i32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                "node" => intermediate_rep
                    .node
                    .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                "disk" => intermediate_rep
                    .disk
                    .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                "partitions" => {
                    return Err(
                        "Parsing a container in this style is not supported in VDiskPartitions"
                            .into(),
                    )
                }
                _ => return Err("Unexpected key while parsing VDiskPartitions".into()),
            }
            Ok(())
        })?;

        // Use the intermediate representation to return the struct
        Ok(Self {
            vdisk: intermediate_rep.vdisk.into_iter().next(),
            node: intermediate_rep.node.into_iter().next(),
            disk: intermediate_rep.disk.into_iter().next(),
            partitions: intermediate_rep.partitions.into_iter().next(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Version {
    #[serde(rename = "version")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    #[serde(rename = "build_time")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_time: Option<String>,
}

impl Version {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            version: None,
            build_time: None,
        }
    }
}

/// Converts the [`Version`] value to the Query Parameters representation (style=form, explode=false)
/// specified in <https://swagger.io/docs/specification/serialization/>
/// Should be implemented in a serde serializer
impl std::string::ToString for Version {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![
            self.version
                .as_ref()
                .map(|version| vec!["version".to_string(), version.to_string()].join(",")),
            self.build_time
                .as_ref()
                .map(|build_time| vec!["build_time".to_string(), build_time.to_string()].join(",")),
        ];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a [`Version`] value
/// as specified in <https://swagger.io/docs/specification/serialization/>
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Version {
    type Err = Box<StdError>;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub version: Vec<String>,
            pub build_time: Vec<String>,
        }
        let mut intermediate_rep = IntermediateRep::default();
        parse(s, |key, val| {
            match key {
                "version" => intermediate_rep
                    .version
                    .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                "build_time" => intermediate_rep
                    .build_time
                    .push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                _ => return Err("Unexpected key while parsing Version".into()),
            }
            Ok(())
        })?;

        // Use the intermediate representation to return the struct
        Ok(Self {
            version: intermediate_rep.version.into_iter().next(),
            build_time: intermediate_rep.build_time.into_iter().next(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct VersionInfo {
    #[serde(rename = "bobversion")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bobversion: Option<Version>,

    #[serde(rename = "pearlversion")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pearlversion: Option<Version>,
}

impl VersionInfo {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            bobversion: None,
            pearlversion: None,
        }
    }
}

/// Converts the [`VersionInfo`] value to the Query Parameters representation (style=form, explode=false)
/// specified in <https://swagger.io/docs/specification/serialization/>
/// Should be implemented in a serde serializer
impl std::string::ToString for VersionInfo {
    fn to_string(&self) -> String {
        let params: Vec<Option<String>> = vec![
            // Skipping bobversion in query parameter serialization

            // Skipping pearlversion in query parameter serialization

        ];

        params.into_iter().flatten().collect::<Vec<_>>().join(",")
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a [`VersionInfo`] value
/// as specified in <https://swagger.io/docs/specification/serialization/>
/// Should be implemented in a serde deserializer
impl std::str::FromStr for VersionInfo {
    type Err = Box<StdError>;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub bobversion: Vec<Version>,
            pub pearlversion: Vec<Version>,
        }
        let mut intermediate_rep = IntermediateRep::default();
        parse(s, |key, val| {
            match key {
                "bobversion" => intermediate_rep.bobversion.push(
                    <Version as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                ),
                "pearlversion" => intermediate_rep.pearlversion.push(
                    <Version as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?,
                ),
                _ => return Err("Unexpected key while parsing VersionInfo".into()),
            }
            Ok(())
        })?;

        // Use the intermediate representation to return the struct
        Ok(Self {
            bobversion: intermediate_rep.bobversion.into_iter().next(),
            pearlversion: intermediate_rep.pearlversion.into_iter().next(),
        })
    }
}
