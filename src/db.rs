use heapless::{LinearMap, Vec};

use crate::codec::Codec;
use crate::kv::KvStore;

pub struct Database<K, V, C, const N: usize, const B: usize, const CACH: usize>
where
    C: Codec<V>,
{
    blobs: KvStore<K, Vec<u8, B>, N>,
    cache: LinearMap<K, V, CACH>,
    _c: core::marker::PhantomData<C>,
}

impl<K, V, C, const N: usize, const B: usize, const CACH: usize> Database<K, V, C, N, B, CACH>
where
    K: Eq + Clone + core::hash::Hash,
    V: serde::Serialize + serde::de::DeserializeOwned + Clone,
    C: Codec<V>,
{
    pub const fn new() -> Self {
        Self {
            blobs: KvStore::new(),
            cache: LinearMap::new(),
            _c: core::marker::PhantomData,
        }
    }

    pub fn put(&mut self, key: K, val: V) -> Result<(), ()> {
        let mut buf = Vec::<u8, B>::new();

        let mut tmp = [0u8; B];

        let n = C::encode(&mut tmp, &val).map_err(|_| ())?;
        if buf.extend_from_slice(&tmp[..n]).is_err() {
            return Err(());
        }

        let _old = self.blobs.put(key.clone(), buf).map_err(|_| ())?;

        if self.cache.is_full() {
            if let Some((k, _)) = self.cache.iter().next() {
                let _ = self.cache.remove(k);
            }
        }

        let _ = self.cache.insert(key, val);

        Ok(())
    }

    pub fn get(&mut self, key: &K) -> Result<Option<V>, ()> {
        if let Some(v) = self.cache.get(key).cloned() {
            return Ok(Some(v));
        }
        let Some(blob) = self.blobs.get(key) else {
            return Ok(None);
        };
        let val = C::decode(blob.as_slice()).map_err(|_| ())?;

        if self.cache.is_full() {
            if let Some((k, _)) = self.cache.iter().next() {
                let _ = self.cache.remove(k);
            }
        }
    }

    pub fn get_uncached(&self, key: &K) -> Result<Option<V>, ()> {
        let Some(blob) = self.blobs.get(key) else {
            return Ok(None);
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
