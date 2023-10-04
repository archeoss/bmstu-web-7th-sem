use super::dto::{self, DetailedNode, DiskCount, DiskProblem, IsActive, Node, ReplicaProblem, RPS};
use super::dto::{Replica, ReplicaStatus, VDisk};
use super::methods::{
    request_configuration, request_disks, request_metrics, request_nodes, request_space,
    request_status, request_vdisks,
};
use crate::connector::dto::{MetricsSnapshotModel, NodeConfiguration};
use crate::connector::{self, api as client};
use crate::connector::{ApiInterface, BobClient};
use crate::models::consts::{MAX_CPU, MIN_FREE_SPACE};
use crate::prelude::APIError;
use crate::prelude::*;
use crate::services::dto::{
    BackendAlienCount, BackendCorruptedBlobCount, ClusterGrinderDeleteCountRate,
    ClusterGrinderExistCountRate, ClusterGrinderGetCountRate, ClusterGrinderPutCountRate, Delete,
    DetailedNodeMetrics, Disk, DiskName, DiskStatus, Exist, Get, HardwareBobCpuLoad,
    HardwareBobVirtualRam, HardwareDescrAmount, HardwareFreeSpace, HardwareTotalRam,
    HardwareTotalSpace, HardwareUsedRam, NodeName, NodeProblem, NodeStatus, PearlDeleteCountRate,
    PearlExistCountRate, PearlGetCountRate, PearlPutCountRate, Put, SpaceInfo, TypedMetrics,
    VDiskStatus,
};
use crate::services::dto::{Bad, Good, NodeCount, Offline};
use axum::extract::Path;
use axum::{Extension, Json};
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use hyper::StatusCode;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Returns count of Physical Disks per status
///
pub async fn get_disks_count(Extension(client): Extension<BobClient>) -> Json<DiskCount> {
    tracing::info!("get /disks/count : {:?}", client);

    let mut space: FuturesUnordered<_> = client
        .cluster()
        .map(move |node| {
            let handle = node.clone();
            tokio::spawn(async move { (handle.get_disks().await, handle.get_space_info().await) })
        })
        .collect();

    let mut count = DiskCount::new();

    while let Some(res) = space.next().await {
        let mut disks_visited = HashSet::new();
        let (disks, space) = match res {
            Ok(d) => d,
            Err(err) => {
                tracing::warn!("couldn't finish request: tokio task failed. Err: {err}");
                continue;
            }
        };
        let space = match space {
            Ok(client::GetSpaceInfoResponse::SpaceInfo(space)) => space,
            Err(err) => {
                tracing::warn!("couldn't finish getSpace request. Err: {err}");
                continue;
            }
        };
        let disks = match disks {
            Ok(client::GetDisksResponse::AJSONArrayWithDisksAndTheirStates(disks)) => {
                let mut res = vec![];
                for disk in disks {
                    if disks_visited.insert(disk.name.clone()) {
                        res.push(disk);
                    }
                }
                res
            }
            Ok(client::GetDisksResponse::PermissionDenied(err)) => {
                count[Offline] += 1;
                tracing::warn!("Permission Denied. Err: {err:?}");
                continue;
            }
            Err(err) => {
                count[Offline] += 1;
                tracing::warn!("couldn't finish getDisks request. Err: {err}");
                continue;
            }
        };
        let active_disks = disks.iter().filter(|disk| disk.is_active);
        for disk in active_disks {
            if let Some(&occupied_space) = space.occupied_disk_space_by_disk.get(&disk.name) {
                #[allow(clippy::cast_precision_loss)]
                match disk_status_from_space(&space, occupied_space) {
                    DiskStatus::Good => count[Good] += 1,
                    _ => count[Bad] += 1,
                }
            } else {
                count[Offline] += 1;
            }
        }
    }
    tracing::info!("total disks count: {count:?}");

    Json(count)
}

/// Get Nodes count per Status
///
pub async fn get_nodes_count(Extension(client): Extension<BobClient>) -> Json<NodeCount> {
    tracing::info!("get /nodes/count : {:?}", client);

    let mut metrics: FuturesUnordered<_> = client
        .cluster()
        .map(move |node| {
            let handle = node.clone();
            tokio::spawn(async move { handle.get_metrics().await })
        })
        .collect();

    let mut count = NodeCount::new();

    let mut counter = 0;
    while let Some(res) = metrics.next().await {
        if let Ok(Ok(client::GetMetricsResponse::Metrics(metrics))) = res {
            tracing::info!("#{counter}: metrics received successfully");
            let metrics = Into::<TypedMetrics>::into(metrics);
            if is_bad_node(&metrics) {
                count[Bad] += 1;
            } else {
                count[Good] += 1;
            }
        } else {
            tracing::warn!("#{counter}: couldn't receive metrics from node");
            count[Offline] += 1;
        }
        counter += 1;
    }
    tracing::info!("total nodes per status count: {count:?}");

    Json(count)
}

/// Returns Total RPS on cluster
///
pub async fn get_rps(Extension(client): Extension<BobClient>) -> Json<RPS> {
    tracing::info!("get /nodes/rps : {:?}", client);

    let mut metrics: FuturesUnordered<_> = client
        .cluster()
        .map(move |node| {
            let handle = node.clone();
            tokio::spawn(async move { handle.get_metrics().await })
        })
        .collect();

    let mut rps = RPS::new();
    let mut counter = 0;
    while let Some(res) = metrics.next().await {
        if let Ok(Ok(metrics)) = res {
            tracing::info!("#{counter}: metrics received successfully");

            let client::GetMetricsResponse::Metrics(metrics) = metrics;
            let metrics = Into::<TypedMetrics>::into(metrics);
            rps[Get] += metrics[ClusterGrinderGetCountRate].value;
            rps[Delete] += metrics[ClusterGrinderDeleteCountRate].value;
            rps[Exist] += metrics[ClusterGrinderExistCountRate].value;
            rps[Put] += metrics[ClusterGrinderPutCountRate].value;
        } else {
            tracing::warn!("#{counter}: couldn't receive metrics from node");
        }
        counter += 1;
    }
    tracing::info!("total rps: {rps:?}");

    Json(rps)
}

/// Return inforamtion about space on cluster
///
pub async fn get_space(Extension(client): Extension<BobClient>) -> Json<dto::SpaceInfo> {
    tracing::info!("get /space : {:?}", client);
    let mut spaces: FuturesUnordered<_> = client
        .cluster()
        .map(move |node| {
            let handle = node.clone();
            tokio::spawn(async move { handle.get_space_info().await })
        })
        .collect();

    let mut total_space = dto::SpaceInfo::default();
    let mut counter = 0;
    while let Some(res) = spaces.next().await {
        if let Ok(Ok(space)) = res {
            tracing::info!("#{counter}: space info received successfully");

            let client::GetSpaceInfoResponse::SpaceInfo(space) = space;
            total_space.total_disk += space.total_disk_space_bytes;
            total_space.free_disk += space.free_disk_space_bytes;
            total_space.used_disk += space.total_disk_space_bytes - space.free_disk_space_bytes;
            total_space.occupied_disk += space.occupied_disk_space_bytes;
        } else {
            tracing::warn!("#{counter}: couldn't receive space info from node");
        }
        counter += 1;
    }
    tracing::trace!("send response: {total_space:?}");

    Json(total_space)
}

#[allow(clippy::cast_precision_loss)]
fn is_bad_node(node_metrics: &TypedMetrics) -> bool {
    node_metrics[BackendAlienCount].value != 0
        || node_metrics[BackendCorruptedBlobCount].value != 0
        || node_metrics[HardwareBobCpuLoad].value >= MAX_CPU
        || (1.
            - (node_metrics[HardwareTotalSpace].value - node_metrics[HardwareFreeSpace].value)
                as f64
                / node_metrics[HardwareTotalSpace].value as f64)
            < MIN_FREE_SPACE
        || node_metrics[HardwareBobVirtualRam] > node_metrics[HardwareTotalRam]
}

/// Returns list of all known nodes
///
/// # Errors
///
/// This function will return an error if one of the requests to get list of virtual disks or nodes fails
pub async fn get_nodes(Extension(client): Extension<BobClient>) -> AxumResult<Json<Vec<Node>>> {
    tracing::info!("get /nodes : {client:?}");

    let mut metrics: FuturesUnordered<_> = client
        .cluster()
        .map(move |node| {
            let handle = node.clone();
            tokio::spawn(async move {
                (
                    handle.get_status().await,
                    handle.get_metrics().await,
                    handle.get_space_info().await,
                )
            })
        })
        .collect();

    let vdisks = get_vdisks(Extension(client.clone())).await.map_err(|err| {
        tracing::error!("{err:?}");
        APIError::RequestFailed
    })?;

    let nodes = request_nodes(client.api()).await?;
    let vdisks: HashMap<u64, &VDisk> = vdisks.iter().map(|vdisk| (vdisk.id, vdisk)).collect();

    let nodes: HashMap<&NodeName, &crate::connector::dto::Node> =
        nodes.iter().map(move |node| (&node.name, node)).collect();

    let mut res = nodes
        .iter()
        .map(|(&name, node)| {
            let vdisks = node
                .vdisks
                .as_ref()
                .map_or_else(std::vec::Vec::new, |node_vdisks| {
                    node_vdisks
                        .iter()
                        .filter_map(|vdisk| vdisks.get(&(vdisk.id as u64)))
                        .map(|vdisk| (*vdisk).clone())
                        .collect()
                });
            (
                name,
                Node {
                    name: name.clone(),
                    hostname: node.address.clone(),
                    vdisks,
                    status: NodeStatus::Offline,
                    rps: None,
                    alien_count: None,
                    corrupted_count: None,
                    space: None,
                },
            )
        })
        .collect::<HashMap<&NodeName, Node>>();

    let mut counter = 0;
    while let Some(fut) = metrics.next().await {
        let Ok((
            Ok(client::GetStatusResponse::AJSONWithNodeInfo(status)),
            Ok(client::GetMetricsResponse::Metrics(metric)),
            Ok(client::GetSpaceInfoResponse::SpaceInfo(space)),
        )) = fut
        else {
            tracing::warn!("couldn't finish task: tokio task failed.");
            continue;
        };
        if let Some(node) = res.get_mut(&status.name.to_string()) {
            let metric = Into::<TypedMetrics>::into(metric);
            tracing::info!("#{counter}: received metrics successfully.");
            node.status = NodeStatus::from_problems(NodeProblem::from_metrics(&metric));
            node.rps = Some(
                metric[PearlGetCountRate].value
                    + metric[PearlPutCountRate].value
                    + metric[PearlExistCountRate].value
                    + metric[PearlDeleteCountRate].value,
            );
            node.alien_count = Some(metric[BackendAlienCount].value);
            node.corrupted_count = Some(metric[BackendCorruptedBlobCount].value);
            node.space = Some(SpaceInfo {
                total_disk: space.total_disk_space_bytes,
                free_disk: space.total_disk_space_bytes - space.used_disk_space_bytes,
                used_disk: space.used_disk_space_bytes,
                occupied_disk: space.occupied_disk_space_bytes,
            });
        }
        counter += 1;
    }
    tracing::trace!("send response: {res:?}");

    Ok(Json(res.values().cloned().collect()))
}

/// Get Virtual Disks
///
/// # Errors
///
/// This function will return an error if one of the requests to get list of vdisks or nodes fails
pub async fn get_vdisks(Extension(client): Extension<BobClient>) -> AxumResult<Json<Vec<VDisk>>> {
    tracing::info!("get /vdisks : {client:?}");

    let mut disks: FuturesUnordered<_> = client
        .cluster()
        .map(move |node| {
            let handle = node.clone();
            tokio::spawn(async move { (handle.get_status().await, handle.get_disks().await) })
        })
        .collect();

    let api = client.api();
    let nodes = request_nodes(api).await?;
    let virt_disks = request_vdisks(api).await?;

    let nodes: HashMap<&NodeName, &connector::dto::Node> =
        nodes.iter().map(|node| (&node.name, node)).collect();

    let mut res_disks = HashMap::new();
    while let Some(res) = disks.next().await {
        if let Ok((
            Ok(client::GetStatusResponse::AJSONWithNodeInfo(status)),
            Ok(client::GetDisksResponse::AJSONArrayWithDisksAndTheirStates(disks)),
        )) = res
        {
            let mut map = HashMap::new();
            for disk in disks {
                map.insert(disk.name, disk.is_active);
            }
            res_disks.insert(status.name, map);
        } else {
            tracing::warn!("couldn't receive node's space info");
        }
    }

    let mut res = vec![];

    for vdisk in virt_disks {
        let replicas = if let Some(replicas) = vdisk.replicas {
            let mut res = vec![];
            for replica in replicas {
                res.push(if let Some(disks) = res_disks.get(&replica.node) {
                    process_replica(&client, replica, disks, &nodes).await
                } else {
                    Replica {
                        node: replica.node,
                        disk: replica.disk,
                        path: replica.path,
                        status: ReplicaStatus::Offline(vec![ReplicaProblem::NodeUnavailable]),
                    }
                });
            }
            res
        } else {
            vec![]
        };
        let count = replicas
            .iter()
            .filter(|replica| matches!(replica.status, ReplicaStatus::Offline(_)))
            .count();
        let status = if count == 0 {
            VDiskStatus::Good
        } else if count == replicas.len() {
            VDiskStatus::Offline
        } else {
            VDiskStatus::Bad
        };
        let part = client.api().get_partitions(vdisk.id).await.ok();
        let partition_count = if let Some(
            client::GetPartitionsResponse::NodeInfoAndJSONArrayWithPartitionsInfo(part),
        ) = part
        {
            part.partitions.unwrap_or_default().len()
        } else {
            0
        } as u64;
        res.push(VDisk {
            id: vdisk.id as u64,
            status,
            partition_count,
            replicas,
        });
    }
    tracing::trace!("send response: {res:?}");

    Ok(Json(res))
}

async fn process_replica(
    client: &BobClient,
    replica: connector::dto::Replica,
    disks: &HashMap<DiskName, IsActive>,
    nodes: &HashMap<&NodeName, &connector::dto::Node>,
) -> Replica {
    let mut status = ReplicaStatus::Good;
    if let Some(disk_state) = disks.get(&replica.disk) {
        if !disk_state {
            status = ReplicaStatus::Offline(vec![ReplicaProblem::DiskUnavailable]);
        }
    } else {
        status = ReplicaStatus::Offline(vec![ReplicaProblem::DiskUnavailable]);
    }

    if let Some(node) = nodes.get(&replica.node) {
        if !is_node_online(client, node).await {
            status = match status {
                ReplicaStatus::Good => {
                    ReplicaStatus::Offline(vec![ReplicaProblem::DiskUnavailable])
                }
                ReplicaStatus::Offline(mut problems) => {
                    problems.push(ReplicaProblem::NodeUnavailable);
                    ReplicaStatus::Offline(problems)
                }
            }
        }
    } else {
        status = match status {
            ReplicaStatus::Good => ReplicaStatus::Offline(vec![ReplicaProblem::DiskUnavailable]),
            ReplicaStatus::Offline(mut problems) => {
                problems.push(ReplicaProblem::NodeUnavailable);
                ReplicaStatus::Offline(problems)
            }
        }
    }

    Replica {
        node: replica.node,
        disk: replica.disk,
        path: replica.path,
        status,
    }
}

async fn is_node_online(client: &BobClient, node: &connector::dto::Node) -> bool {
    (client.probe_socket(&node.name).await).map_or(false, |code| code == StatusCode::OK)
}

fn proccess_disks(
    disks: &[connector::dto::DiskState],
    space: &connector::dto::SpaceInfo,
    metrics: &connector::dto::MetricsSnapshotModel,
) -> Vec<Disk> {
    let mut res_disks = vec![];
    let mut visited_disks = HashSet::new();
    for disk in disks {
        if !visited_disks.insert(disk.name.clone()) {
            tracing::warn!(
                "disk {} with path {} duplicated, skipping...",
                disk.name,
                disk.path
            );
            continue;
        }
        let status =
            if let Some(&occupied_space) = space.occupied_disk_space_by_disk.get(&disk.name) {
                disk_status_from_space(space, occupied_space)
            } else {
                DiskStatus::Offline
            };
        res_disks.push(Disk {
            name: disk.name.clone(),
            path: disk.path.clone(),
            status,
            total_space: space.total_disk_space_bytes,
            used_space: space
                .occupied_disk_space_by_disk
                .get(&disk.name.clone())
                .copied()
                .unwrap_or_default(),
            iops: metrics
                .metrics
                .get(&format!("hardware.disks.{:?}_iops", disk.name))
                .cloned()
                .unwrap_or_default()
                .value,
        });
    }

    res_disks
}

#[allow(clippy::cast_precision_loss)]
fn disk_status_from_space(space: &connector::dto::SpaceInfo, occupied_space: u64) -> DiskStatus {
    if ((space.total_disk_space_bytes - occupied_space) as f64
        / space.total_disk_space_bytes as f64)
        < MIN_FREE_SPACE
    {
        DiskStatus::Bad(vec![DiskProblem::FreeSpaceRunningOut])
    } else {
        DiskStatus::Good
    }
}

// Bad function, 6 args :(
async fn process_vdisks_for_node(
    client: BobClient,
    virt_disks: &[connector::dto::VDisk],
    node_name: NodeName,
    all_disks: &HashMap<DiskName, IsActive>,
    nodes: &HashMap<&NodeName, &connector::dto::Node>,
    partitions_count_on_vdisk: &HashMap<i32, usize>,
) -> Vec<VDisk> {
    let mut res_replicas = vec![];
    let mut res_vdisks = vec![];
    for (vdisk, replicas) in virt_disks
        .iter()
        .filter_map(|vdisk| vdisk.replicas.as_ref().map(|repl| (vdisk, repl)))
        .filter(|(_, replicas)| replicas.iter().any(|replica| replica.node == node_name))
    {
        for replica in replicas {
            res_replicas.push((
                vdisk.id,
                process_replica(&client, replica.clone(), all_disks, nodes).await,
            ));
        }
        res_vdisks.push(VDisk {
            id: vdisk.id as u64,
            status: if res_replicas
                .iter()
                .any(|(_, replica)| matches!(replica.status, ReplicaStatus::Offline(_)))
            {
                VDiskStatus::Bad
            } else {
                VDiskStatus::Good
            },
            partition_count: partitions_count_on_vdisk
                .get(&vdisk.id)
                .copied()
                .unwrap_or_default() as u64,
            replicas: res_replicas
                .iter()
                .filter(|(id, _)| id == &vdisk.id)
                .map(|(_, replica)| replica.clone())
                .collect(),
        });
    }

    res_vdisks
}

/// Get Detailed Information on Node
///
/// # Errors
///
/// This function will return an error if the server was unable to get node'a client
/// or one of the requests to get information from the node fails
pub async fn get_detailed_node_info(
    Extension(client): Extension<BobClient>,
    Path(node_name): Path<NodeName>,
) -> AxumResult<Json<DetailedNode>> {
    let mut all_disks: FuturesUnordered<_> = client
        .cluster()
        .map(move |node| {
            let handle = node.clone();
            tokio::spawn(async move { handle.get_disks().await })
        })
        .collect();

    let node_client = get_client_by_node(&client, node_name.clone()).await?;

    let virt_disks = request_vdisks(&node_client).await?;

    let mut all_partitions: FuturesUnordered<_> = virt_disks
        .iter()
        .map(|vdisk| {
            let id = vdisk.id;
            let handle = client.api().clone();
            tokio::spawn(async move { (id, handle.get_partitions(id).await) })
        })
        .collect();

    let metrics = request_metrics(&node_client).await?;
    let typed_metrics: TypedMetrics = metrics.clone().into();

    let space = request_space(&node_client).await?;
    let node_status = request_status(&node_client).await?;
    let disks = request_disks(&node_client).await?;

    let res_disks = proccess_disks(&disks, &space, &metrics);
    let nodes = request_nodes(&node_client).await?;

    let nodes = nodes
        .iter()
        .map(|node| (&node.name, node))
        .collect::<HashMap<&NodeName, &connector::dto::Node>>();

    let mut proc_disks = HashMap::new();
    while let Some(disks) = all_disks.next().await {
        let Ok(Ok(client::GetDisksResponse::AJSONArrayWithDisksAndTheirStates(disks))) = disks
        else {
            tracing::error!("couldn't get disk inforamtion from node");
            continue;
        };
        for disk in disks {
            proc_disks.insert(disk.name, disk.is_active);
        }
    }

    let mut res_partitions = HashMap::new();
    while let Some(partitions) = all_partitions.next().await {
        let Ok((
            id,
            Ok(client::GetPartitionsResponse::NodeInfoAndJSONArrayWithPartitionsInfo(partitions)),
        )) = partitions
        else {
            // tracing::error!("couldn't get Partition inforamtion from node"); // Too noisy
            continue;
        };
        if let Some(partitions) = partitions.partitions {
            res_partitions.insert(id, partitions.len());
        }
    }

    let vdisks = process_vdisks_for_node(
        client,
        &virt_disks,
        node_name,
        &proc_disks,
        &nodes,
        &res_partitions,
    )
    .await;

    let mut rps = RPS::new();

    let status = NodeStatus::from_problems(NodeProblem::from_metrics(&typed_metrics));

    rps[Get] = typed_metrics[PearlGetCountRate].value;
    rps[Put] = typed_metrics[PearlPutCountRate].value;
    rps[Exist] = typed_metrics[PearlExistCountRate].value;
    rps[Delete] = typed_metrics[PearlDeleteCountRate].value;

    let res = Json(DetailedNode {
        name: node_status.name,
        hostname: node_status.address,
        vdisks,
        status,
        metrics: DetailedNodeMetrics {
            rps,
            alien_count: typed_metrics[BackendAlienCount].value,
            corrupted_count: typed_metrics[BackendCorruptedBlobCount].value,
            space: SpaceInfo {
                total_disk: space.total_disk_space_bytes,
                free_disk: space.total_disk_space_bytes - space.used_disk_space_bytes,
                used_disk: space.used_disk_space_bytes,
                occupied_disk: space.occupied_disk_space_bytes,
            },
            cpu_load: typed_metrics[HardwareBobCpuLoad].value,
            total_ram: typed_metrics[HardwareTotalRam].value,
            used_ram: typed_metrics[HardwareUsedRam].value,
            descr_amount: typed_metrics[HardwareDescrAmount].value,
        },
        disks: res_disks,
    });
    tracing::trace!("send response: {res:?}");

    Ok(res)
}

/// Get Raw Metrics from Node
///
/// # Errors
///
/// This function will return an error if the server was unable to get node'a client or the request to get metrics fails
pub async fn raw_metrics_by_node(
    Extension(client): Extension<BobClient>,
    Path(node_name): Path<NodeName>,
) -> AxumResult<Json<MetricsSnapshotModel>> {
    let client = get_client_by_node(&client, node_name).await?;

    Ok(Json(request_metrics(&client).await?))
}

/// Get Configuration from Node
///
/// # Errors
///
/// This function will return an error if the server was unable to get node'a client or the request to get configuration fails
pub async fn raw_configuration_by_node(
    Extension(client): Extension<BobClient>,
    Path(node_name): Path<NodeName>,
) -> AxumResult<Json<NodeConfiguration>> {
    let client = get_client_by_node(&client, node_name).await?;

    Ok(Json(request_configuration(&client).await?))
}

async fn get_client_by_node(
    client: &BobClient,
    node_name: NodeName,
) -> AxumResult<Arc<ApiInterface>> {
    let nodes = request_nodes(client.api()).await?;

    let node = nodes
        .iter()
        .find(|node| node.name == node_name)
        .ok_or_else(|| {
            tracing::error!("Couldn't find specified node");
            APIError::RequestFailed
        })?;

    client
        .cluster_with_addr()
        .get(&node.name)
        .ok_or_else(|| {
            tracing::error!("Couldn't find specified node");
            APIError::RequestFailed.into()
        })
        .cloned()
}
