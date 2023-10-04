#![allow(clippy::module_name_repetitions)]
use super::dto::{self};
use axum::async_trait;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::task::{Context, Poll};
use swagger::{ApiError, ContextWrapper};

pub type ServiceError = Box<dyn Error + Send + Sync + 'static>;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GetAlienResponse {
    /// Alien Node name
    AlienNodeName(String),
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[must_use]
pub enum GetAlienDirResponse {
    /// Directory
    Directory(dto::Dir),
    /// Permission denied
    PermissionDenied(dto::StatusExt),
    /// Not acceptable backend
    NotAcceptableBackend(dto::StatusExt),
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[must_use]
pub enum GetDisksResponse {
    /// A JSON array with disks and their states
    AJSONArrayWithDisksAndTheirStates(Vec<dto::DiskState>),
    /// Permission denied
    PermissionDenied(dto::StatusExt),
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GetMetricsResponse {
    /// Metrics
    Metrics(dto::MetricsSnapshotModel),
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[must_use]
pub enum GetNodesResponse {
    /// A JSON array of nodes info and vdisks on them
    AJSONArrayOfNodesInfoAndVdisksOnThem(Vec<dto::Node>),
    /// Permission denied
    PermissionDenied,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[must_use]
pub enum GetPartitionResponse {
    /// A JSON with partition info
    AJSONWithPartitionInfo(dto::Partition),
    /// Permission denied
    PermissionDenied(dto::StatusExt),
    /// Not found
    NotFound(dto::StatusExt),
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[must_use]
pub enum GetPartitionsResponse {
    /// Node info and JSON array with partitions info
    NodeInfoAndJSONArrayWithPartitionsInfo(dto::VDiskPartitions),
    /// Permission denied
    PermissionDenied(dto::StatusExt),
    /// Not found
    NotFound(dto::StatusExt),
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[must_use]
pub enum GetRecordsResponse {
    /// Records count
    RecordsCount(i32),
    /// Permission denied
    PermissionDenied(dto::StatusExt),
    /// Not found
    NotFound(dto::StatusExt),
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[must_use]
pub enum GetReplicasLocalDirsResponse {
    /// A JSON array with dirs
    AJSONArrayWithDirs(Vec<dto::Dir>),
    /// Permission denied
    PermissionDenied(dto::StatusExt),
    /// Not found
    NotFound(dto::StatusExt),
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GetSpaceInfoResponse {
    /// Space info
    SpaceInfo(dto::SpaceInfo),
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GetStatusResponse {
    /// A JSON with node info
    AJSONWithNodeInfo(dto::Node),
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[must_use]
pub enum GetVDiskResponse {
    /// A JSON with vdisk info
    AJSONWithVdiskInfo(dto::VDisk),
    /// Permission denied
    PermissionDenied(dto::StatusExt),
    /// Not found
    NotFound(dto::StatusExt),
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[must_use]
pub enum GetVDisksResponse {
    /// A JSON array of vdisks info
    AJSONArrayOfVdisksInfo(Vec<dto::VDisk>),
    /// Permission denied
    PermissionDenied,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GetVersionResponse {
    /// Version info
    VersionInfo(dto::VersionInfo),
}

/// Returns configuration of the node
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[must_use]
pub enum GetConfigurationResponse {
    /// Configuration object
    ConfigurationObject(dto::NodeConfiguration),
    /// Permission denied
    PermissionDenied,
}

/// API
#[async_trait]
#[allow(clippy::too_many_arguments, clippy::ptr_arg)]
pub trait Api<C: Send + Sync> {
    fn poll_ready(
        &self,
        _cx: &mut Context,
    ) -> Poll<Result<(), Box<dyn Error + Send + Sync + 'static>>> {
        Poll::Ready(Ok(()))
    }

    /// Return directory of alien
    async fn get_alien_dir(&self, context: &C) -> Result<GetAlienDirResponse, ApiError>;

    /// Returns the list of disks with their states
    async fn get_disks(&self, context: &C) -> Result<GetDisksResponse, ApiError>;

    /// Get metrics
    async fn get_metrics(&self, context: &C) -> Result<GetMetricsResponse, ApiError>;

    /// Returns a list of known nodes
    async fn get_nodes(&self, context: &C) -> Result<GetNodesResponse, ApiError>;

    /// Returns a partition info by ID
    async fn get_partition(
        &self,
        v_disk_id: i32,
        partition_id: String,
        context: &C,
    ) -> Result<GetPartitionResponse, ApiError>;

    /// Returns a list of partitions
    async fn get_partitions(
        &self,
        v_disk_id: i32,
        context: &C,
    ) -> Result<GetPartitionsResponse, ApiError>;

    /// Returns count of records of this on node
    async fn get_records(
        &self,
        v_disk_id: i32,
        context: &C,
    ) -> Result<GetRecordsResponse, ApiError>;

    /// Returns directories of local replicas of vdisk
    async fn get_replicas_local_dirs(
        &self,
        v_disk_id: i32,
        context: &C,
    ) -> Result<GetReplicasLocalDirsResponse, ApiError>;

    /// Get space info
    async fn get_space_info(&self, context: &C) -> Result<GetSpaceInfoResponse, ApiError>;

    /// Returns information about self
    async fn get_status(&self, context: &C) -> Result<GetStatusResponse, ApiError>;

    /// Returns a vdisk info by ID
    async fn get_v_disk(&self, v_disk_id: i32, context: &C) -> Result<GetVDiskResponse, ApiError>;

    /// Returns a list of vdisks
    async fn get_v_disks(&self, context: &C) -> Result<GetVDisksResponse, ApiError>;

    /// Returns server version
    async fn get_version(&self, context: &C) -> Result<GetVersionResponse, ApiError>;

    /// Returns configuration of the node
    async fn get_configuration(&self, context: &C) -> Result<GetConfigurationResponse, ApiError>;
}

/// API where `Context` isn't passed on every API call
#[async_trait]
#[allow(clippy::too_many_arguments, clippy::ptr_arg)]
pub trait ApiNoContext<C: Send + Sync> {
    fn poll_ready(
        &self,
        _cx: &mut Context,
    ) -> Poll<Result<(), Box<dyn Error + Send + Sync + 'static>>>;

    fn context(&self) -> &C;

    /// Return directory of alien
    async fn get_alien_dir(&self) -> Result<GetAlienDirResponse, ApiError>;

    /// Returns the list of disks with their states
    async fn get_disks(&self) -> Result<GetDisksResponse, ApiError>;

    /// Get metrics
    async fn get_metrics(&self) -> Result<GetMetricsResponse, ApiError>;

    /// Returns a list of known nodes
    async fn get_nodes(&self) -> Result<GetNodesResponse, ApiError>;

    /// Returns a partition info by ID
    async fn get_partition(
        &self,
        v_disk_id: i32,
        partition_id: String,
    ) -> Result<GetPartitionResponse, ApiError>;

    /// Returns a list of partitions
    async fn get_partitions(&self, v_disk_id: i32) -> Result<GetPartitionsResponse, ApiError>;

    /// Returns count of records of this on node
    async fn get_records(&self, v_disk_id: i32) -> Result<GetRecordsResponse, ApiError>;

    /// Returns directories of local replicas of vdisk
    async fn get_replicas_local_dirs(
        &self,
        v_disk_id: i32,
    ) -> Result<GetReplicasLocalDirsResponse, ApiError>;

    /// Get space info
    async fn get_space_info(&self) -> Result<GetSpaceInfoResponse, ApiError>;

    /// Returns information about self
    async fn get_status(&self) -> Result<GetStatusResponse, ApiError>;

    /// Returns a vdisk info by ID
    async fn get_v_disk(&self, v_disk_id: i32) -> Result<GetVDiskResponse, ApiError>;

    /// Returns a list of vdisks
    async fn get_v_disks(&self) -> Result<GetVDisksResponse, ApiError>;

    /// Returns server version
    async fn get_version(&self) -> Result<GetVersionResponse, ApiError>;

    /// Returns configuration of the node
    async fn get_configuration(&self) -> Result<GetConfigurationResponse, ApiError>;
}

/// Trait to extend an API to make it easy to bind it to a context.
pub trait ContextWrapperExt<C: Send + Sync>
where
    Self: Sized,
{
    /// Binds this API to a context.
    fn with_context(self, context: C) -> ContextWrapper<Self, C>;
}

impl<T: Api<C> + Send + Sync, C: Clone + Send + Sync> ContextWrapperExt<C> for T {
    fn with_context(self: T, context: C) -> ContextWrapper<T, C> {
        ContextWrapper::<T, C>::new(self, context)
    }
}

#[async_trait]
impl<T: Api<C> + Send + Sync, C: Clone + Send + Sync> ApiNoContext<C> for ContextWrapper<T, C> {
    fn poll_ready(&self, cx: &mut Context) -> Poll<Result<(), ServiceError>> {
        self.api().poll_ready(cx)
    }

    fn context(&self) -> &C {
        Self::context(self)
    }

    /// Return directory of alien
    async fn get_alien_dir(&self) -> Result<GetAlienDirResponse, ApiError> {
        let context = self.context().clone();
        self.api().get_alien_dir(&context).await
    }
    /// Returns the list of disks with their states
    async fn get_disks(&self) -> Result<GetDisksResponse, ApiError> {
        let context = self.context().clone();
        self.api().get_disks(&context).await
    }

    /// Get metrics
    async fn get_metrics(&self) -> Result<GetMetricsResponse, ApiError> {
        let context = self.context().clone();
        self.api().get_metrics(&context).await
    }

    /// Returns a list of known nodes
    async fn get_nodes(&self) -> Result<GetNodesResponse, ApiError> {
        let context = self.context().clone();
        self.api().get_nodes(&context).await
    }

    /// Returns a partition info by ID
    async fn get_partition(
        &self,
        v_disk_id: i32,
        partition_id: String,
    ) -> Result<GetPartitionResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .get_partition(v_disk_id, partition_id, &context)
            .await
    }

    /// Returns a list of partitions
    async fn get_partitions(&self, v_disk_id: i32) -> Result<GetPartitionsResponse, ApiError> {
        let context = self.context().clone();
        self.api().get_partitions(v_disk_id, &context).await
    }

    /// Returns count of records of this on node
    async fn get_records(&self, v_disk_id: i32) -> Result<GetRecordsResponse, ApiError> {
        let context = self.context().clone();
        self.api().get_records(v_disk_id, &context).await
    }

    /// Returns directories of local replicas of vdisk
    async fn get_replicas_local_dirs(
        &self,
        v_disk_id: i32,
    ) -> Result<GetReplicasLocalDirsResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .get_replicas_local_dirs(v_disk_id, &context)
            .await
    }

    /// Get space info
    async fn get_space_info(&self) -> Result<GetSpaceInfoResponse, ApiError> {
        let context = self.context().clone();
        self.api().get_space_info(&context).await
    }

    /// Returns information about self
    async fn get_status(&self) -> Result<GetStatusResponse, ApiError> {
        let context = self.context().clone();
        self.api().get_status(&context).await
    }

    /// Returns a vdisk info by ID
    async fn get_v_disk(&self, v_disk_id: i32) -> Result<GetVDiskResponse, ApiError> {
        let context = self.context().clone();
        self.api().get_v_disk(v_disk_id, &context).await
    }

    /// Returns a list of vdisks
    async fn get_v_disks(&self) -> Result<GetVDisksResponse, ApiError> {
        let context = self.context().clone();
        self.api().get_v_disks(&context).await
    }

    /// Returns server version
    async fn get_version(&self) -> Result<GetVersionResponse, ApiError> {
        let context = self.context().clone();
        self.api().get_version(&context).await
    }

    /// Returns configuration of the node
    async fn get_configuration(&self) -> Result<GetConfigurationResponse, ApiError> {
        let context = self.context().clone();
        self.api().get_configuration(&context).await
    }
}
