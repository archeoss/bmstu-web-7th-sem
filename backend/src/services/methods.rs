use crate::connector::api;
use crate::connector::dto;
use crate::{connector::ApiInterface, prelude::*};
use hyper::StatusCode;
use std::sync::Arc;

pub async fn request_metrics(client: &Arc<ApiInterface>) -> AxumResult<dto::MetricsSnapshotModel> {
    let api::GetMetricsResponse::Metrics(metrics) = client.get_metrics().await.map_err(|err| {
        tracing::error!("{err}");
        APIError::RequestFailed
    })?;

    Ok(metrics)
}

pub async fn request_vdisks(client: &Arc<ApiInterface>) -> AxumResult<Vec<dto::VDisk>> {
    let api::GetVDisksResponse::AJSONArrayOfVdisksInfo(virt_disks) =
        client.get_v_disks().await.map_err(|err| {
            tracing::error!("{err}");
            APIError::RequestFailed
        })?
    else {
        return Err(APIError::InvalidStatusCode(StatusCode::FORBIDDEN).into());
    };

    Ok(virt_disks)
}

pub async fn request_space(client: &Arc<ApiInterface>) -> AxumResult<dto::SpaceInfo> {
    let api::GetSpaceInfoResponse::SpaceInfo(space) =
        client.get_space_info().await.map_err(|err| {
            tracing::error!("{err}");
            APIError::RequestFailed
        })?;

    Ok(space)
}

pub async fn request_status(client: &Arc<ApiInterface>) -> AxumResult<dto::Node> {
    let api::GetStatusResponse::AJSONWithNodeInfo(node_status) =
        client.get_status().await.map_err(|err| {
            tracing::error!("{err}");
            APIError::RequestFailed
        })?;

    Ok(node_status)
}

pub async fn request_disks(client: &Arc<ApiInterface>) -> AxumResult<Vec<dto::DiskState>> {
    let api::GetDisksResponse::AJSONArrayWithDisksAndTheirStates(disks) =
        client.get_disks().await.map_err(|err| {
            tracing::error!("{err}");
            APIError::RequestFailed
        })?
    else {
        tracing::error!(
            "client received invalid status code: {}",
            StatusCode::FORBIDDEN
        );
        return Err(APIError::InvalidStatusCode(StatusCode::FORBIDDEN).into());
    };

    Ok(disks)
}

pub async fn request_configuration(
    client: &Arc<ApiInterface>,
) -> AxumResult<dto::NodeConfiguration> {
    let api::GetConfigurationResponse::ConfigurationObject(configuration) =
        client.get_configuration().await.map_err(|err| {
            tracing::error!("couldn't get node's configuration: {err}");
            APIError::RequestFailed
        })?
    else {
        tracing::error!("received invalid ststus code: {}", StatusCode::FORBIDDEN);
        return Err(APIError::InvalidStatusCode(StatusCode::FORBIDDEN).into());
    };

    Ok(configuration)
}

pub async fn request_nodes(client: &Arc<ApiInterface>) -> AxumResult<Vec<dto::Node>> {
    let api::GetNodesResponse::AJSONArrayOfNodesInfoAndVdisksOnThem(nodes) =
        client.get_nodes().await.map_err(|err| {
            tracing::error!("couldn't get nodes list from bob: {err}");
            APIError::RequestFailed
        })?
    else {
        tracing::error!("received invalid ststus code: {}", StatusCode::FORBIDDEN);
        return Err(APIError::InvalidStatusCode(StatusCode::FORBIDDEN).into());
    };

    Ok(nodes)
}
