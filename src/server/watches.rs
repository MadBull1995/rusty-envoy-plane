use crate::cache::{Cache, WatchId};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Watch {
    pub nonce: Option<i64>,
    pub id: WatchId,
}

#[derive(Clone)]
pub struct Watches<C: Cache + Send + 'static> {
    cache: Arc<C>,
    active: HashMap<String, Watch>,
}

impl<C: Cache> Watches<C> {
    pub fn new(cache: Arc<C>) -> Self {
        Self {
            cache,
            active: HashMap::new(),
        }
    }

    pub fn get(&self, type_url: &str) -> Option<&Watch> {
        self.active.get(type_url)
    }

    pub fn get_mut(&mut self, type_url: &str) -> Option<&mut Watch> {
        self.active.get_mut(type_url)
    }

    pub fn add(&mut self, type_url: &str, watch_id: WatchId) {
        self.active.insert(
            type_url.to_string(),
            Watch {
                nonce: None,
                id: watch_id,
            },
        );
    }
}

pub async fn cancel_all<C>(active: HashMap<String, Watch>, cache: Arc<C>)
where
    C: Cache + Send + Sync + 'static,
{
    println!("WARN: dropping all watches");
    for (_, watch) in active.iter() {
        cache.cancel_watch(&watch.id).await;
    }
}

impl<C: Cache + Send + Sync + 'static> Drop for Watches<C> {
    fn drop(&mut self) {
        let active = self.active.clone();
        let cache = self.cache.clone();
        tokio::spawn(async move { cancel_all(active, cache).await });
    }
}
