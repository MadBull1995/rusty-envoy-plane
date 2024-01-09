use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
    error::Error,
    io,
    sync::{Arc},
    time::Instant,
};
use tokio::sync::{mpsc, Mutex, watch};

use crate::{
    envoy::config::core::v3 as corePb,
    cache::{status::{self, ResponseWatch}, resource::get_resource_references},
    server::stream::StreamState,
};

use super::{
    resource::{
        self, get_all_resource_references, get_response_type, get_response_type_url, ResponseType,
    },
    resources::{new_resources, ResourceType, Resources},
    simple::{ResourceSnapshot, SnapshotCache},
    status::StatusInfo,
    types::{Resource, ResourceWithTTL},
    Cache, ConfigFetcher, ConfigWatcher, NodeHash, RawResponse, Request, Response, WatchId, WatchResponder, DeltaRequest,
};

#[derive(Debug)]
pub struct Watch {
    req: Request,
    tx: Arc<WatchResponder>,
}

#[derive(Debug, Clone)]
pub struct Snapshot {
    resources: HashMap<ResponseType, Resources>,
    version_map: Arc<Mutex<HashMap<String, HashMap<String, String>>>>,
}

#[derive(Debug)]
pub struct SnapshotCacheXds<T: NodeHash> {
    /// ads flag to hold responses until all resources are named
    ads: bool,
    node_hash: T,
    inner: Arc<Mutex<Inner>>,
}

impl<T: NodeHash> SnapshotCache for SnapshotCacheXds<T> {
    async fn set_snapshot(
        &self,
        node: &str,
        snapshot: Snapshot,
    ) -> Result<(), Box<dyn std::error::Error  + Send + Sync + 'static>> {
        let mut inner = self.inner.lock().await;
        
        if let Some(status) = inner.status.get_mut(node) {
            let mut to_delete = Vec::new();
            println!("setting snapshot watches_count={:?}", status.get_num_watches());
            for (watch_id, watch) in &status.watches {
                let ver = snapshot.get_version(&watch.req.type_url);
                println!("snapshot_version = {} watch_req_version = {:?}", ver, watch.req.version_info);
                if ver != watch.req.version_info {
                    to_delete.push(*watch_id)
                }
            }
    
            // Second loop: remove the watches
            for watch_id in &to_delete {
                let watch = status.watches.remove(&watch_id).unwrap();
                // let res_type = get_response_type(&watch.req.type_url);
                let rsc = snapshot.get_resources(&watch.req.type_url);
                let ver = snapshot.get_version(&watch.req.type_url);
                println!(
                    "watch triggered version={} type_url={}",
                    ver,
                    &watch.req.type_url
                );
                // get_resource_references(&rsc.unwrap().items, &mut out);
                self.respond(&watch.req, watch.tx, &rsc, &ver, false).await?;
            }

            // let mut to_delete = Vec::new();
            // for (watch_id, watch) in &status.delta_watches {
            //     println!("delta watch triggered type_url={}", &watch.req.type_url);
            //     let responded =
            //         try_respond_delta(&watch.req, watch.tx.clone(), &watch.stream, &mut snapshot)
            //             .await;
            //     if responded {
            //         to_delete.push(watch_id)
            //     }
            // }

            // for watch_id in to_delete {
            //     status.delta_watches.remove(&watch_id);
            // }
        }

        inner.snapshots.insert(node.to_string(), snapshot.clone());

        Ok(())
    }

    async fn get_snapshot(
        &self,
        node: &str,
    ) -> Result<Snapshot, Box<dyn std::error::Error  + Send + Sync + 'static>> {
        todo!()
    }

    async fn clear_snapshot(&self, node: &str) {
        todo!()
    }

    async fn get_status_info(&self, node: &str) -> Result<StatusInfo, Box<dyn std::error::Error  + Send + Sync + 'static>> {
        todo!()
    }

    async fn get_status_keys(&self) -> Result<Vec<String>, Box<dyn std::error::Error  + Send + Sync + 'static>> {
        todo!()
    }
}

impl<T: NodeHash> ConfigWatcher for SnapshotCacheXds<T> {
    async fn create_delta_watch(
        &self,
        request: &super::DeltaRequest,
        response_channel: tokio::sync::mpsc::Sender<super::DeltaResponse>,
    ) -> Option<WatchId> {
        Some(WatchId { node_id: "request.node".to_string(), index: 1 })
    }

    async fn create_watch(
        &self,
        request: &Request,
        stream_state: &Arc<Mutex<StreamState>>,
        response_channel: Arc<WatchResponder>,
    ) -> Option<WatchId> {
        if let Some(n) = &request.node {
            let node_id = &n.get_id();
            let mut inner = self.inner.lock().await;
            inner.update_node_status(n.clone());
            let mut ss = stream_state.lock().await;
            if let Some(info) = inner.status.get_mut(node_id) {
                info.last_watch_request_time = Instant::now();
            } else {
                inner
                    .status
                    .insert(node_id.to_string(), StatusInfo::new(n.clone()));
            }
            // let info = inner.status.get_mut(node_id).unwrap();

            if let Some(snapshot) = inner.snapshots.get(node_id) {
                let version = snapshot.get_version(&request.type_url);
                println!("{}", version);
                if let Some(known_resources_names) =
                    ss.get_known_resource_names(&request.type_url)
                {
                    let mut diff: Vec<String> = Vec::with_capacity(known_resources_names.len());
                    let resources_names: Vec<String> = request.resource_names.clone();
                    for r in resources_names {
                        if known_resources_names.contains(&r) {
                            diff.push(r);
                        }
                    }

                    println!(
                        "node-id: {} requested {}{:?} and known {:?}. Diff {:?}",
                        node_id,
                        request.type_url,
                        request.resource_names,
                        known_resources_names.clone(),
                        diff
                    );

                    if diff.len() > 0 {
                        let resources = snapshot.get_resources(&request.type_url);
                        let _ = self.respond(
                            &request,
                            response_channel.clone(),
                            &resources,
                            &version,
                            false,
                        );
                        // for name in diff {
                        // }
                    }
                    return None
                } else if request.version_info == version {
                    inner.watch_count += 1;
                    let watch_id = inner.watch_count.clone();
                    println!("new watch id: {:?} for {}", watch_id, node_id);
                    return Some(inner.set_watch(watch_id, &node_id, request, response_channel));
                } else {
                    let resources = snapshot.get_resources(&request.type_url.clone());
                    let _ = self.respond(&request, response_channel, &resources, &version, false);
                    return None
                }
            } else {
                println!("set watch: no snapshot");
                inner.watch_count += 1;
                let watch_id = inner.watch_count.clone();
                println!("new watch id: {:?} for {}", watch_id, node_id);
                return Some(inner.set_watch(watch_id, &node_id, request, response_channel))
            }

        } else {

            None
        }
    }

}

impl<T: NodeHash> ConfigFetcher for SnapshotCacheXds<T> {
    async fn fetch<'a>(&'a self, request: &'a Request, type_url: &'static str) -> Result<Response, super::FetchError> {
        todo!()
    }
}
impl<T: NodeHash> Cache for SnapshotCacheXds<T> {
   

    async fn cancel_delta_watch(&self, watch_id: &super::WatchId) {
        todo!()
    }

    async fn cancel_watch(
        &self,
        watch_id: &WatchId,
    ) {
        let mut inner = self.inner.lock().await;
        if let Some(status) = inner.status.get_mut(&watch_id.node_id) {
            println!("canceling watch={:?} current_count={}", watch_id, status.get_num_watches());
            status.watches.remove(&watch_id.index);
        }
    }
}
impl<T: NodeHash> SnapshotCacheXds<T> {
    pub fn new(ads: bool, node_hash: T) -> Self {
        SnapshotCacheXds {
            ads,
            inner: Arc::new(Mutex::new(Inner::new())),
            node_hash,
        }
    }

    pub async fn node_status(&self) -> HashMap<String, Instant> {
        let inner = self.inner.lock().await;
        inner
            .status
            .iter()
            .map(|(k, v)| (k.clone(), v.get_last_watch_request_time()))
            .collect()
    }

    async fn next_watch_id(&self) -> u64 {
        let mut inner = self.inner.lock().await;
        inner.watch_count += 1;
        inner.watch_count
    }

    // async fn try_respond_delta(
    //     &self,
    //     req: &DeltaRequest,
    //     tx: DeltaWatchResponder,
    //     stream: &DeltaStreamHandle,
    //     snapshot: &mut Snapshot,
    // ) -> bool {
    //     let delta = DeltaResponse::new(req, stream, snapshot);
    //     if !delta.filtered.is_empty()
    //         || !delta.to_remove.is_empty()
    //         || (stream.is_wildcard() && stream.is_first())
    //     {
    //         info!("delta responded type_url={}", &req.type_url);
    //         tx.send((
    //             delta.to_discovery(&req.type_url),
    //             delta.next_version_map.clone(),
    //         ))
    //         .await
    //         .unwrap();
    //         true
    //     } else {
    //         info!("delta unchanged type_url={}", &req.type_url);
    //         false
    //     }
    // }

    async fn respond(
        &self,
        req: &Request,
        chan: Arc<WatchResponder>,
        resources: &Option<Resources>,
        version: &str,
        heartbeat: bool,
    ) -> Result<(), Box<dyn Error  + Send + Sync + 'static>> {
        if req.resource_names.len() != 0 && self.ads {
            // match superset(name_set(req.resource_names.clone()), resources.unwrap().items) {
            //     Err(e) => {
            //         let res_names = req.resource_names.clone();
            //         return Err(format!(
            //             "ADS mode: not responding to request {} {:?}: {:?}",
            //             req.type_url, res_names, e
            //         )
            //         .into());
            //     }
            //     Ok(_) => todo!(),
            // }
        }

        let response = build_response(&req, resources, version);
        chan.send((req.clone(), response.clone())).await;
        dbg!(response);

        Ok(())
    }
}

fn build_response(
    req: &Request,
    resources: &Option<Resources>,
    version: &str,
) -> Response {
    let mut filtered_resources = Vec::new();
    if let Some(resources) = resources {
        if req.resource_names.is_empty() {
            filtered_resources = resources
                .items
                .values()
                .map(|resource| resource.into_any())
                .collect();
        } else {
            for name in &req.resource_names {
                if let Some(resource) = resources.items.get(name) {
                    filtered_resources.push(resource.into_any())
                }
            }
        }
    }
    Response {
        type_url: req.type_url.clone(),
        nonce: String::new(),
        version_info: version.to_string(),
        resources: filtered_resources,
        control_plane: None,
        canary: false,
    }
}

fn crate_response(
    req: &Request,
    resources: HashMap<String, ResourceWithTTL>,
    version: &str,
    heartbeat: bool,
) -> RawResponse {
    let mut filterd = Vec::with_capacity(resources.len());
    if req.resource_names.len() != 0 {
        let set = name_set(req.resource_names.clone());
        for (name, res) in resources {
            if let Some(in_set) = set.get(&name) {
                if *in_set {
                    filterd.push(res);
                }
            }
        }
    } else {
        for (name, res) in resources {
            filterd.push(res);
        }
    }

    RawResponse {
        heartbeat: heartbeat,
        request: req.clone(),
        marshaled_response: Arc::new(Mutex::new(None)),
        version: version.to_owned(),
        resources: filterd,
    }
}

#[derive(Debug)]
struct Inner {
    snapshots: HashMap<String, Snapshot>,
    status: HashMap<String, StatusInfo>,
    watch_count: u64,
    delta_watch_count: u64,
}

impl Inner {
    pub fn new() -> Self {
        Self {
            delta_watch_count: 0,
            watch_count: 0,
            snapshots: HashMap::new(),
            status: HashMap::new(),
        }
    }

    fn set_watch(&mut self, watch_id: u64, node_id: &str, req: &Request, tx: Arc<WatchResponder>) -> WatchId {
        let watch = Watch {
            req: req.clone(),
            tx,
        };
        let status = self.status.get_mut(node_id).unwrap();
        status.watches.insert(watch_id, watch);
        WatchId {
            node_id: node_id.to_string(),
            index: watch_id,
        }
    }

    fn update_node_status(&mut self, node: corePb::Node) {
        self.status
            .entry(node.id.to_string())
            .and_modify(|entry| entry.last_watch_request_time = Instant::now())
            .or_insert_with(|| StatusInfo::new(node));
    }
}

impl ResourceSnapshot for Snapshot {
    fn get_version(&self, type_url: &str) -> String {
        let typ = resource::get_response_type(type_url);
        match typ {
            ResponseType::UnknownType => "".to_string(),
            t => match self.resources.get(&t) {
                Some(d) => d.version.clone(),
                None => "".to_string(),
            },
        }
    }

    fn get_resources_and_ttl(&self, type_url: &str) -> HashMap<String, ResourceWithTTL> {
        todo!()
    }

    fn get_resources(&self, type_url: &str) -> Option<Resources> {
        self.resources.get(&get_response_type(type_url)).cloned()
    }

    fn construct_version_map(&mut self) -> Result<(), Box<dyn Error  + Send + Sync + 'static>> {
        todo!()
    }

    fn get_version_map(&self, type_url: &str) -> HashMap<String, String> {
        todo!()
    }
}

impl Snapshot {
    pub fn new(
        version: String,
        resources: HashMap<ResponseType, Vec<ResourceType>>,
    ) -> Result<Self, Box<dyn Error  + Send + Sync + 'static>> {
        let mut out = Self {
            resources: HashMap::new(),
            version_map: Arc::new(Mutex::new(HashMap::new())),
        };

        for (typ, resource) in resources.into_iter() {
            if typ == ResponseType::UnknownType {
                return Err(format!("unknown resource type: {:?}", typ).into());
            }
            out.resources
                .insert(typ, new_resources(version.clone(), resource));
        }

        Ok(out)
    }

    pub fn consistent(&self) -> Result<(), Box<dyn Error  + Send + Sync + 'static>> {
        let referenced_resources = get_all_resource_references(&self.resources);
        // Define the resource types that you expect to have references
        let referenced_response_types = vec![ResponseType::Endpoint, ResponseType::Route]
            .into_iter()
            .collect::<HashSet<_>>();
        for (idx, items) in self.resources.iter().enumerate() {
            if referenced_response_types.contains(&items.0) {
                let response_type = ResponseType::from_idx(idx);
                let type_url = get_response_type_url(response_type)?;
                let reference_set = referenced_resources
                    .get(&items.0)
                    .ok_or_else(|| "Reference set not found")?;

                if reference_set.len() != items.1.items.len() {
                    return Err(format!(
                        "Mismatched reference and resource lengths for type: {}",
                        type_url
                    )
                    .into());
                }

                // Check superset
                // if !self.is_superset(reference_set, items.1.items.keys()) {
                //     return Err(format!("Inconsistent reference for type: {}", type_url).into());
                // }
            }
        }

        Ok(())
    }

    // Implement this function to check if one set is a superset of another
    fn is_superset(
        &self,
        reference_set: &HashMap<String, bool>,
        items: impl Iterator<Item = &'static String>,
    ) -> bool {
        // Logic to check if reference_set is a superset of items
        // ...
        true // Placeholder
    }
}

fn superset(
    names: HashMap<String, bool>,
    resources: HashMap<String, ResourceWithTTL>,
) -> Result<(), Box<dyn Error  + Send + Sync + 'static>> {
    for (name, res) in resources {
        if !names.contains_key(&name) {
            return Err(format!("{} not listed", name).into());
        }
    }

    Ok(())
}

fn name_set(names: Vec<String>) -> HashMap<String, bool> {
    let mut set = HashMap::with_capacity(names.len());
    for name in names {
        set.insert(name, true);
    }

    set
}
