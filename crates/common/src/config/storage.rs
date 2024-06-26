use std::sync::Arc;

use ahash::AHashMap;
use directory::Directory;
use store::{BlobStore, FtsStore, LookupStore, Store};

pub struct Storage {
    pub data: Store,
    pub blob: BlobStore,
    pub fts: FtsStore,
    pub lookup: LookupStore,
    pub lookups: AHashMap<String, LookupStore>,
    pub directory: Arc<Directory>,
    pub directories: AHashMap<String, Arc<Directory>>,
}
