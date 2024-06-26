/*
 * Copyright (c) 2023 Stalwart Labs Ltd.
 *
 * This file is part of the Stalwart Mail Server.
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of
 * the License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 * in the LICENSE file at the top-level directory of this distribution.
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 * You can be released from the requirements of the AGPLv3 license by
 * purchasing a commercial license. Please contact licensing@stalw.art
 * for more details.
*/

use std::{
    hash::BuildHasherDefault,
    time::{Duration, Instant},
};

use lru_cache::LruCache;
use nohash::NoHashHasher;
use parking_lot::Mutex;

use super::{TokenHash, Weights};

#[derive(Debug)]
pub struct BayesTokenCache {
    positive: Mutex<LruCache<TokenHash, CacheItem, BuildHasherDefault<NoHashHasher<TokenHash>>>>,
    negative: Mutex<LruCache<TokenHash, Instant, BuildHasherDefault<NoHashHasher<TokenHash>>>>,
    ttl_negative: Duration,
    ttl_positive: Duration,
}

#[derive(Debug, Clone)]
pub struct CacheItem {
    item: Weights,
    valid_until: Instant,
}

impl BayesTokenCache {
    pub fn new(capacity: usize, ttl_positive: Duration, ttl_negative: Duration) -> Self {
        Self {
            positive: Mutex::new(LruCache::with_hasher(capacity, Default::default())),
            negative: Mutex::new(LruCache::with_hasher(capacity, Default::default())),
            ttl_negative,
            ttl_positive,
        }
    }

    pub fn get(&self, hash: &TokenHash) -> Option<Option<Weights>> {
        {
            let mut pos_cache = self.positive.lock();
            if let Some(entry) = pos_cache.get_mut(hash) {
                return if entry.valid_until >= Instant::now() {
                    Some(Some(entry.item))
                } else {
                    pos_cache.remove(hash);
                    None
                };
            }
        }
        {
            let mut neg_cache = self.negative.lock();
            if let Some(entry) = neg_cache.get_mut(hash) {
                return if *entry >= Instant::now() {
                    Some(None)
                } else {
                    neg_cache.remove(hash);
                    None
                };
            }
        }

        None
    }

    pub fn insert_positive(&self, hash: TokenHash, weights: Weights) {
        self.positive.lock().insert(
            hash,
            CacheItem {
                item: weights,
                valid_until: Instant::now() + self.ttl_positive,
            },
        );
    }

    pub fn insert_negative(&self, hash: TokenHash) {
        self.negative
            .lock()
            .insert(hash, Instant::now() + self.ttl_negative);
    }

    pub fn invalidate(&self, hash: &TokenHash) {
        if self.positive.lock().remove(hash).is_none() {
            self.negative.lock().remove(hash);
        }
    }
}

impl Default for BayesTokenCache {
    fn default() -> Self {
        Self {
            positive: Mutex::new(LruCache::with_hasher(1024, Default::default())),
            negative: Mutex::new(LruCache::with_hasher(1024, Default::default())),
            ttl_negative: Default::default(),
            ttl_positive: Default::default(),
        }
    }
}
