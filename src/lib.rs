#![cfg_attr(not(feature = "std"), no_std)]

use heapless::{LinearMap, String as HString, Vec as HVec};

/// A fixed-capacity, no-std compatible key-value database.
/// Using `heapless` and a known size, this should be stored on the stack. I need to test this.
///
/// - CAP: maximum number of key/value pairs
/// - K:   maximum key length in bytes (UTF-8; we store keys as `heapless::String`)
/// - V:   maximum value length in bytes
pub struct EmbeddedDatabase<const CAP: usize, const K: usize, const V: usize> {
    map: LinearMap<HString<K>, HVec<u8, V>, CAP>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    KeyTooLong,
    ValueTooLong,
    Full,
}

impl<const CAP: usize, const K: usize, const V: usize> EmbeddedDatabase<CAP, K, V> {
    pub const fn new() -> Self {
        Self {
            map: LinearMap::new(),
        }
    }

    /// Insert or overwrite
    pub fn put(&mut self, key: &str, value: &[u8]) -> Result<(), Error> {
        let mut k: HString<K> = HString::new();
        if k.push_str(key).is_err() {
            return Err(Error::KeyTooLong);
        }

        let mut v: HVec<u8, V> = HVec::new();
        if v.extend_from_slice(value).is_err() {
            return Err(Error::ValueTooLong);
        }

        if let Some(slot) = self.map.get_mut(&k) {
            *slot = v;
            Ok(())
        } else {
            self.map.insert(k, v).map(|_| ()).map_err(|_| Error::Full)
        }
    }

    pub fn get(&self, key: &str) -> Option<&[u8]> {
        let mut k: HString<K> = HString::new();
        if k.push_str(key).is_err() {
            return None;
        }
        self.map.get(&k).map(|v| v.as_slice())
    }

    pub fn delete(&mut self, key: &str) -> bool {
        let mut k: HString<K> = HString::new();
        if k.push_str(key).is_err() {
            return false;
        }
        self.map.remove(&k).is_some()
    }

    pub fn update(&mut self, key: &str, new_value: &[u8]) -> Result<bool, Error> {
        let mut k: HString<K> = HString::new();
        if k.push_str(key).is_err() {
            return Err(Error::KeyTooLong);
        }
        if let Some(slot) = self.map.get_mut(&k) {
            let mut v: HVec<u8, V> = HVec::new();
            if v.extend_from_slice(new_value).is_err() {
                return Err(Error::ValueTooLong);
            }
            *slot = v;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub const fn capacity(&self) -> usize {
        CAP
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}
