// This Database is a wrapper around the KvStore
// It provides a more user-friendly interface
// and adds a cache to the KvStore
// It also allows us to encode and decode data
// using the Codec trait

use crate::codec::Codec;
use crate::kv::KvStore;
use heapless::{LinearMap, Vec};

pub struct Database<K, V, C, const N: usize, const B: usize, const CACH: usize>
where
    C: Codec<V>,
    K: Eq + core::hash::Hash + Clone,
    V: serde::Serialize + serde::de::DeserializeOwned + Clone,
{
    blobs: KvStore<K, Vec<u8, B>, N>,
    // This cache is a small hot cache to speed up operations
    // The LinearMap is a fixed-size map that is used to store the data
    // When the cache is full, the oldest entry is evicted
    // I'm not sure if this is the best way to do this
    cache: LinearMap<K, V, CACH>,
    _c: core::marker::PhantomData<C>,
}

impl<K, V, C, const N: usize, const B: usize, const CACH: usize> Database<K, V, C, N, B, CACH>
where
    C: Codec<V>,
    K: Eq + core::hash::Hash + Clone,
    V: serde::Serialize + serde::de::DeserializeOwned + Clone,
{
    pub const fn new() -> Self {
        Self {
            blobs: KvStore::<K, Vec<u8, B>, N>::new(),
            cache: LinearMap::new(),
            _c: core::marker::PhantomData,
        }
    }

    pub fn put(&mut self, key: K, val: V) -> Result<(), ()> {
        let mut tmp = [0u8; B];
        let used = C::encode(&mut tmp, &val).map_err(|_| ())?;

        let mut blob = Vec::<u8, B>::new();
        blob.extend_from_slice(&tmp[..used]).map_err(|_| ())?;

        let _ = self.blobs.put(key.clone(), blob).map_err(|_| ())?;

        if self.cache.is_full() {
            if let Some((k0, _)) = self.cache.iter().next() {
                let victim = k0.clone();
                let _ = self.cache.remove(&victim);
            }
        }
        let _ = self.cache.insert(key, val);
        Ok(())
    }

    pub fn get(&mut self, key: &K) -> Result<Option<V>, ()> {
        if let Some(v) = self.cache.get(key).cloned() {
            return Ok(Some(v));
        }
        let blob_opt: Option<&Vec<u8, B>> = self.blobs.get(key);
        let blob = match blob_opt {
            Some(b) => b,
            None => return Ok(None),
        };

        let val = C::decode(blob.as_slice()).map_err(|_| ())?;

        if self.cache.is_full() {
            if let Some((k0, _)) = self.cache.iter().next() {
                let victim = k0.clone();
                let _ = self.cache.remove(&victim);
            }
        }
        let _ = self.cache.insert(key.clone(), val.clone());

        Ok(Some(val))
    }

    pub fn get_uncached(&self, key: &K) -> Result<Option<V>, ()> {
        let blob_opt: Option<&Vec<u8, B>> = self.blobs.get(key);
        let blob = match blob_opt {
            Some(b) => b,
            None => return Ok(None),
        };
        C::decode(blob.as_slice()).map(Some).map_err(|_| ())
    }

    pub fn delete(&mut self, key: &K) -> bool {
        let removed = self.blobs.remove(key).is_some();
        let _ = self.cache.remove(key);
        removed
    }

    pub fn len(&self) -> usize {
        self.blobs.len()
    }
    pub fn capacity(&self) -> usize {
        self.blobs.capacity()
    }
}
