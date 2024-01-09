use prost::Message;
use std::{time::Duration, any::Any};

use super::resources::ResourceType;

pub type MarsheledResource = Vec<u8>;

pub trait Resource: Message + Any {
}
pub trait ResourceWithName: Resource {
    fn get_name(&self) -> String;
}
#[derive(Debug, Clone)]
pub struct ResourceWithTTL {
    pub resource: ResourceType,
    pub ttl: Option<Duration>,
}