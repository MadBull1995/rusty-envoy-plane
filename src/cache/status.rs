use std::{
    collections::HashMap,
    time::Instant
};
use tokio::sync::mpsc;
use crate::envoy::{
    config::core::v3 as core,
    service::discovery::v3 as discovery,
};

use super::{order::Keys, snapshot::Watch};

/// StatusInfo tracks the server state for the remote Envoy node.
#[derive(Debug)]
pub struct StatusInfo  {
	/// node is the constant Envoy node metadata.
	node: core::Node,

	/// watches are indexed channels for the response watches and the original requests.
	pub watches: HashMap<u64, Watch>,
	ordered_watches: Keys,

	/// deltaWatches are indexed channels for the delta response watches and the original requests
	pub delta_watches:        HashMap<u64, DeltaResponseWatch>,
	ordered_delta_watches: Keys,

	/// the timestamp of the last watch request
	pub last_watch_request_time: Instant,

	/// the timestamp of the last delta watch request
	pub last_delta_watch_request_time: Instant
}

pub type Request = discovery::DiscoveryRequest;
pub type ResponseChannel = mpsc::Sender<discovery::DiscoveryResponse>;

#[derive(Debug)]
pub struct ResponseWatch {
    pub(crate) request: Request,
    pub(crate) response: ResponseChannel, // Define the channel type appropriately
}

pub type DeltaRequest = discovery::DeltaDiscoveryRequest;
pub type DeltaResponseChannel = mpsc::Sender<discovery::DeltaDiscoveryResponse>;

#[derive(Debug)]
pub struct DeltaResponseWatch {
    request: DeltaRequest,
    response: DeltaResponseChannel, // Define the channel type appropriately
}

impl StatusInfo {

    pub fn new(node: core::Node) -> Self {
        Self {
            node,
            watches: HashMap::new(),
            ordered_watches: Vec::new(),
            delta_watches: HashMap::new(),
            ordered_delta_watches: Vec::new(),
            last_delta_watch_request_time: Instant::now(),
            last_watch_request_time: Instant::now(),
        }
    }

    pub fn get_node(&self) -> core::Node {
        self.node.clone()
    }

    pub fn get_num_watches(&self) -> usize {
        self.watches.len()
    }

    pub fn get_num_delta_watches(&self) -> usize {
        self.delta_watches.len()
    }

    pub fn get_last_watch_request_time(&self) -> Instant {
        self.last_watch_request_time
    }

    pub fn get_last_delta_watch_request_time(&self) -> Instant {
        self.last_delta_watch_request_time
    }
}