// This Database is a wrapper around the KvStore
// It provides a more user-friendly interface
// and adds a cache to the KvStore
// It also allows us to encode and decode data
// using the Codec trait

use crate::codec::Codec;
use crate::kv::KvStore;
use embedded_storage::nor_flash::{NorFlash, ReadNorFlash};
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

    /// Save the database to flash storage
    /// This writes to flash with a simple format:
    /// [num_entries: u32][key1_len: u32][key1_data][val1_len: u32][val1_data]...
    ///
    /// flash_offset: The offset in flash where to write (must be aligned)
    /// flash: The flash storage device
    pub fn save_to_flash<F>(
        &self,
        flash: &mut F,
        flash_size: usize,
        flash_offset: u32,
    ) -> Result<(), FlashError>
    where
        F: NorFlash,
        K: serde::Serialize,
    {
        const MAX_SERIALIZED_SIZE: usize = 8192; // 8KB buffer
        let mut buffer = [0u8; MAX_SERIALIZED_SIZE];
        let mut pos = 0;

        // Write number of entries
        let num_entries = self.len() as u32;
        buffer[pos..pos + flash_size].copy_from_slice(&num_entries.to_le_bytes());
        pos += flash_size;

        // Iterate through all entries and serialize them
        for (key, blob) in self.blobs.iter() {
            // Serialize the key
            // using postcard because it is a compact format
            let key_bytes = postcard::to_slice(key, &mut buffer[pos + flash_size..])
                .map_err(|_| FlashError::SerializationError)?;
            let key_len = key_bytes.len() as u32;

            // Write key length
            buffer[pos..pos + 4].copy_from_slice(&key_len.to_le_bytes());
            pos += 4 + key_len as usize;

            // Write value length and data
            let val_len = blob.len() as u32;
            if pos + 4 + val_len as usize > MAX_SERIALIZED_SIZE {
                return Err(FlashError::BufferTooSmall);
            }

            buffer[pos..pos + 4].copy_from_slice(&val_len.to_le_bytes());
            pos += 4;
            buffer[pos..pos + val_len as usize].copy_from_slice(blob.as_slice());
            pos += val_len as usize;
        }

        // Pad to word alignment (4 bytes)
        let aligned_size = (pos + 3) & !3;

        // Erase the flash region first
        let page_size = F::ERASE_SIZE;
        let pages_needed = (aligned_size + page_size - 1) / page_size;
        let erase_end = flash_offset + (pages_needed * page_size) as u32;

        flash
            .erase(flash_offset, erase_end)
            .map_err(|_| FlashError::EraseError)?;

        // Write to flash
        flash
            .write(flash_offset, &buffer[..aligned_size])
            .map_err(|_| FlashError::WriteError)?;

        Ok(())
    }

    /// Load the database from flash storage
    /// Reads data saved by save_to_flash and populates the database
    pub fn load_from_flash<F>(&mut self, flash: &mut F, flash_offset: u32) -> Result<(), FlashError>
    where
        F: ReadNorFlash,
        K: serde::de::DeserializeOwned,
    {
        const MAX_READ_SIZE: usize = 8192;
        let mut buffer = [0u8; MAX_READ_SIZE];

        // Read from flash
        flash
            .read(flash_offset, &mut buffer)
            .map_err(|_| FlashError::ReadError)?;

        let mut pos = 0;

        // Read number of entries
        let num_entries = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
        pos += 4;

        // Check if flash is empty (all 0xFF)
        if num_entries == 0xFFFFFFFF {
            // Flash is erased, nothing to load
            return Ok(());
        }

        // Clear existing data
        self.blobs.clear();
        self.cache.clear();

        // Read each entry
        for _ in 0..num_entries {
            // Read key length
            if pos + 4 > MAX_READ_SIZE {
                return Err(FlashError::BufferTooSmall);
            }
            let key_len = u32::from_le_bytes([
                buffer[pos],
                buffer[pos + 1],
                buffer[pos + 2],
                buffer[pos + 3],
            ]) as usize;
            pos += 4;

            // Read key
            if pos + key_len > MAX_READ_SIZE {
                return Err(FlashError::BufferTooSmall);
            }
            let key: K = postcard::from_bytes(&buffer[pos..pos + key_len])
                .map_err(|_| FlashError::DeserializationError)?;
            pos += key_len;

            // Read value length
            if pos + 4 > MAX_READ_SIZE {
                return Err(FlashError::BufferTooSmall);
            }
            let val_len = u32::from_le_bytes([
                buffer[pos],
                buffer[pos + 1],
                buffer[pos + 2],
                buffer[pos + 3],
            ]) as usize;
            pos += 4;

            // Read value
            if pos + val_len > MAX_READ_SIZE {
                return Err(FlashError::BufferTooSmall);
            }
            let mut blob = Vec::<u8, B>::new();
            blob.extend_from_slice(&buffer[pos..pos + val_len])
                .map_err(|_| FlashError::BufferTooSmall)?;
            pos += val_len;

            // Insert into store
            self.blobs
                .put(key, blob)
                .map_err(|_| FlashError::DatabaseFull)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, defmt::Format)]
pub enum FlashError {
    SerializationError,
    DeserializationError,
    BufferTooSmall,
    EraseError,
    WriteError,
    ReadError,
    DatabaseFull,
}
