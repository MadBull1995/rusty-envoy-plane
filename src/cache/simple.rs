use std::collections::HashMap;
use std::error::Error;

use super::Cache;
use super::resources::{ResourceType, Resources};
use super::snapshot::Snapshot;
use super::status::StatusInfo;
use super::types::{ResourceWithTTL, Resource};

pub trait SnapshotCache: Cache + Send + Sync {
    async fn set_snapshot(&self, node: &str, snapshot: Snapshot) -> Result<(), Box<dyn std::error::Error  + Send + Sync + 'static>>;
    async fn get_snapshot(&self, node: &str) -> Result<Snapshot, Box<dyn std::error::Error  + Send + Sync + 'static>>;
    async fn clear_snapshot(&self, node: &str);
    async fn get_status_info(&self, node: &str) -> Result<StatusInfo, Box<dyn std::error::Error  + Send + Sync + 'static>>;
    async fn get_status_keys(&self) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync + 'static>>;
}

pub trait ResourceSnapshot {
    // Returns the current version of the resource indicated by `type_url`.
    fn get_version(&self, type_url: &str) -> String;

    // Returns all resources of the type indicated by `type_url`, together with their TTL.
    fn get_resources_and_ttl(&self, type_url: &str) -> HashMap<String, ResourceWithTTL>;

    // Returns all resources of the type indicated by `type_url`.
    fn get_resources(&self, type_url: &str) -> Option<Resources>;

    // A hint that a delta watch will soon make a call to `get_version_map`.
    // The snapshot should construct an internal opaque version string for each collection of resource types.
    fn construct_version_map(&mut self) -> Result<(), Box<dyn Error + Send + Sync + 'static>>;

    // Returns a map of resource name to resource version for all the resources of type indicated by `type_url`.
    fn get_version_map(&self, type_url: &str) -> HashMap<String, String>;
}