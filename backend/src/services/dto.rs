#![allow(unused_qualifications)]

use crate::connector::dto::{MetricsEntryModel, MetricsSnapshotModel};
use crate::models::consts::{MAX_CPU, MIN_FREE_SPACE};
pub use crate::models::models::{BobConnectionData, Credentials};
use crate::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
pub struct Good;
pub struct Bad;
pub struct Offline;

/// Physical disk definition
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Disk {
    /// Disk name
    pub name: String,

    /// Disk path
    pub path: String,

    /// Disk status
    #[serde(flatten)]
    pub status: DiskStatus,

    #[serde(rename = "totalSpace")]
    pub total_space: u64,

    #[serde(rename = "usedSpace")]
    pub used_space: u64,

    pub iops: u64,
}

/// Defines kind of problem on disk
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Serialize, Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub enum DiskProblem {
    #[serde(rename = "freeSpaceRunningOut")]
    FreeSpaceRunningOut,
}

/// Defines disk status
///
/// Variant - Disk Status
/// Content - List of problems on disk. 'null' if status != 'bad'
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
#[serde(tag = "status", content = "problems")]
pub enum DiskStatus {
    #[serde(rename = "good")]
    Good,
    #[serde(rename = "bad")]
    Bad(Vec<DiskProblem>),
    #[serde(rename = "offline")]
    Offline,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiskCount(HashMap<String, u64>);

typed_map!(DiskCount, u64, Good, "good", Bad, "bad", Offline, "offline");

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Node {
    pub name: String,

    pub hostname: String,

    pub vdisks: Vec<VDisk>,
    #[serde(flatten)]
    pub status: NodeStatus,

    #[serde(rename = "rps")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rps: Option<u64>,

    #[serde(rename = "alienCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alien_count: Option<u64>,

    #[serde(rename = "corruptedCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corrupted_count: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub space: Option<SpaceInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeCount(HashMap<String, u64>);

typed_map!(NodeCount, u64, Good, "good", Bad, "bad", Offline, "offline");

/// Defines kind of problem on Node
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Serialize, Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub enum NodeProblem {
    #[serde(rename = "aliensExists")]
    AliensExists,
    #[serde(rename = "corruptedExists")]
    CorruptedExists,
    #[serde(rename = "freeSpaceRunningOut")]
    FreeSpaceRunningOut,
    #[serde(rename = "virtualMemLargerThanRAM")]
    VirtualMemLargerThanRAM,
    #[serde(rename = "highCPULoad")]
    HighCPULoad,
}

impl NodeProblem {
    #[allow(clippy::cast_precision_loss)]
    #[must_use]
    pub fn from_metrics(node_metrics: &TypedMetrics) -> Vec<Self> {
        let mut res = vec![];
        if node_metrics[BackendAlienCount].value != 0 {
            res.push(Self::AliensExists);
        }
        if node_metrics[BackendCorruptedBlobCount].value != 0 {
            res.push(Self::CorruptedExists);
        }
        if node_metrics[HardwareBobCpuLoad].value >= MAX_CPU {
            res.push(Self::HighCPULoad);
        }
        if (1.
            - (node_metrics[HardwareTotalSpace].value - node_metrics[HardwareFreeSpace].value)
                as f64
                / node_metrics[HardwareTotalSpace].value as f64)
            < MIN_FREE_SPACE
        {
            res.push(Self::FreeSpaceRunningOut);
        }
        if node_metrics[HardwareBobVirtualRam] > node_metrics[HardwareTotalRam] {
            res.push(Self::VirtualMemLargerThanRAM);
        }

        res
    }
}

/// Defines status of node
///
/// Variants - Node status
///
/// Content - List of problems on node. 'null' if status != 'bad'
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
#[serde(tag = "status", content = "problems")]
pub enum NodeStatus {
    #[serde(rename = "good")]
    Good,
    #[serde(rename = "bad")]
    Bad(Vec<NodeProblem>),
    #[serde(rename = "offline")]
    Offline,
}

impl NodeStatus {
    #[must_use]
    pub fn from_problems(problems: Vec<NodeProblem>) -> Self {
        if problems.is_empty() {
            Self::Good
        } else {
            Self::Bad(problems)
        }
    }
}

/// [`VDisk`]'s replicas
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Replica {
    pub node: String,

    pub disk: String,

    pub path: String,

    #[serde(flatten)]
    pub status: ReplicaStatus,
}

/// Reasons why Replica is offline
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Serialize, Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub enum ReplicaProblem {
    #[serde(rename = "nodeUnavailable")]
    NodeUnavailable,
    #[serde(rename = "diskUnavailable")]
    DiskUnavailable,
}

/// Replica status. It's either good or offline with the reasons why it is offline
///
/// Variants - Replica status
///
/// Content - List of problems on replica. 'null' if status != 'offline'
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
#[serde(tag = "status", content = "problems")]
pub enum ReplicaStatus {
    #[serde(rename = "good")]
    Good,
    #[serde(rename = "offline")]
    Offline(Vec<ReplicaProblem>),
}

/// Disk space information in bytes
#[derive(Debug, Default, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct SpaceInfo {
    /// Total disk space amount
    pub total_disk: u64,

    /// The amount of free disk space
    pub free_disk: u64,

    /// Used disk space amount
    pub used_disk: u64,

    /// Disk space occupied only by BOB. occupied_disk should be lesser than used_disk
    pub occupied_disk: u64,
}

/// Virtual disk Component
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct VDisk {
    pub id: u64,

    #[serde(flatten)]
    pub status: VDiskStatus,

    #[serde(rename = "partitionCount")]
    pub partition_count: u64,

    pub replicas: Vec<Replica>,
}

/// Virtual disk status.
///
/// Variants - Virtual Disk status
/// status == 'bad' when at least one of its replicas has problems
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
#[serde(tag = "status")]
pub enum VDiskStatus {
    #[serde(rename = "good")]
    Good,
    #[serde(rename = "bad")]
    Bad,
    #[serde(rename = "offline")]
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct DetailedNode {
    pub name: String,

    pub hostname: String,

    pub vdisks: Vec<VDisk>,

    #[serde(flatten)]
    pub status: NodeStatus,

    pub metrics: DetailedNodeMetrics,

    pub disks: Vec<Disk>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
#[serde(rename_all = "camelCase")]
pub struct DetailedNodeMetrics {
    pub rps: RPS,

    pub alien_count: u64,

    pub corrupted_count: u64,

    pub space: SpaceInfo,

    pub cpu_load: u64,

    pub total_ram: u64,

    pub used_ram: u64,

    pub descr_amount: u64,
}

pub struct Put;
pub struct Get;
pub struct Exist;
pub struct Delete;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RPS(HashMap<String, u64>);

macro_rules! generate_typed_mutfield {
    // generate getter for a field
    ($map:ty, $return:ty, $($field:ty, $internal:expr),+ ) => {
        $(generate_typed_index!($map, $return, $field, $internal);
        impl std::ops::IndexMut<$field> for $map {
            fn index_mut(&mut self, _index: $field) -> &mut Self::Output {
                Option::unwrap(self.0.get_mut($internal))
            }
        })+
    };
}
macro_rules! generate_typed_index {
    // generate getter for a field
    ($map:ty, $return:ty, $($field:ty, $internal:expr),+ ) => {
        $(impl std::ops::Index<$field> for $map {
            type Output = $return;

            fn index(&self, _index: $field) -> &Self::Output {
                self.0.index($internal)
            }
        })+
    };
}
macro_rules! typed_map {
    ($map:ty, $return:ty, $($field:ty, $internal:expr),+) => {
        impl $map {
            #[must_use]
            pub fn new() -> Self {
                let mut map = HashMap::new();
                $(map.insert($internal.into(), <$return>::default());)+

                Self(map)
            }
        }
        impl Default for $map {
            fn default() -> Self {
                Self::new()
            }
        }
        $(generate_typed_mutfield!($map, $return, $field, $internal);)+
    };
}

typed_map!(RPS, u64, Put, "put", Delete, "delete", Exist, "exist", Get, "get");

pub struct PearlExistCount;
pub struct ClientDeleteCount;
pub struct PearlExistCountRate;
pub struct ClusterGrinderExistKeysCountRate;
pub struct BackendBackendState;
pub struct ClusterGrinderGetCount;
pub struct PearlDeleteCount;
pub struct BackendDisksDisk1BlobsCount;
pub struct PearlPutCountRate;
pub struct LinkManagerNodesNumber;
pub struct PearlDeleteErrorCount;
pub struct ClientPutErrorCount;
pub struct ClientExistErrorKeysCount;
pub struct BackendBlobCount;
pub struct PearlGetErrorCount;
pub struct ClientGetCount;
pub struct ClientGetErrorCount;
pub struct BackendCorruptedBlobCount;
pub struct PearlDeleteCountRate;
pub struct ClientExistErrorCount;
pub struct ClusterGrinderExistTimer;
pub struct PearlPutCount;
pub struct PearlExistErrorCount;
pub struct PearlPutErrorCountRate;
pub struct HardwareUsedRam;
pub struct PearlPutErrorCount;
pub struct PearlGetErrorCountRate;
pub struct HardwareUsedSwap;
pub struct HardwareBobRam;
pub struct HardwareBobCpuLoad;
pub struct ClusterGrinderPutCount;
pub struct ClientExistKeysCount;
pub struct BackendUsedDisk;
pub struct BackendBloomFiltersRam;
pub struct PearlGetCount;
pub struct HardwareDescrAmount;
pub struct BackendActiveDisks;
pub struct ClusterGrinderExistCountRate;
pub struct ClusterGrinderExistErrorCount;
pub struct HardwareAvailableRam;
pub struct HardwareBobVirtualRam;
pub struct HardwareFreeSpace;
pub struct BackendUsedDiskDisk1;
pub struct BackendAlienCount;
pub struct ClusterGrinderGetErrorCount;
pub struct ClusterGrinderExistKeysCount;
pub struct PearlExistErrorCountRate;
pub struct ClusterGrinderExistErrorKeysCount;
pub struct HardwareUsedSpace;
pub struct BackendDisksDisk1;
pub struct ClientDeleteErrorCount;
pub struct ClientExistCount;
pub struct BackendIndexMemory;
pub struct PearlGetCountRate;
pub struct ClusterGrinderPutErrorCount;
pub struct PearlDeleteErrorCountRate;
pub struct ClusterGrinderExistCount;
pub struct ClusterGrinderDeleteCount;
pub struct ClusterGrinderDeleteCountRate;
pub struct ClusterGrinderGetCountRate;
pub struct ClusterGrinderPutCountRate;
pub struct HardwareTotalRam;
pub struct HardwareCpuIowait;
pub struct ClientPutCount;
pub struct HardwareTotalSpace;

#[derive(Debug)]
pub struct TypedMetrics(HashMap<String, MetricsEntryModel>);

impl From<MetricsSnapshotModel> for TypedMetrics {
    fn from(value: MetricsSnapshotModel) -> Self {
        let mut map = Self::new();
        let value = value.metrics;
        for (key, entry) in &mut map.0 {
            *entry = (value.get(key).unwrap_or(&MetricsEntryModel::default())).clone();
        }

        map
    }
}

typed_map!(
    TypedMetrics,
    MetricsEntryModel,
    ClusterGrinderGetCountRate,
    "cluster_grinder.get_count_rate",
    ClusterGrinderPutCountRate,
    "cluster_grinder.put_count_rate",
    ClusterGrinderExistCountRate,
    "cluster_grinder.exist_count_rate",
    ClusterGrinderDeleteCountRate,
    "cluster_grinder.delete_count_rate",
    PearlExistCountRate,
    "pearl.exist_count_rate",
    PearlGetCountRate,
    "pearl.get_count_rate",
    PearlPutCountRate,
    "pearl.put_count_rate",
    PearlDeleteCountRate,
    "pearl.delete_count_rate",
    BackendAlienCount,
    "backend.alien_count",
    BackendCorruptedBlobCount,
    "backend.corrupted_blob_count",
    HardwareBobVirtualRam,
    "hardware.bob_virtual_ram",
    HardwareTotalRam,
    "hardware.total_ram",
    HardwareUsedRam,
    "hardware.used_ram",
    HardwareBobCpuLoad,
    "hardware.bob_cpu_load",
    HardwareFreeSpace,
    "hardware.free_space",
    HardwareTotalSpace,
    "hardware.total_space",
    HardwareDescrAmount,
    "hardware.descr_amount"
);

pub mod macros {
    pub(crate) use generate_typed_index;
    pub(crate) use generate_typed_mutfield;
    pub(crate) use typed_map;
}

/// Aliases
pub type NodeName = String;
pub type DiskName = String;
pub type NodeAddress = String;
pub type IsActive = bool;
