use core::borrow::Borrow;
use heapless::FnvIndexMap;

type BackingMap<K, V, const N: usize> = FnvIndexMap<K, V, N>;

pub struct KvStore<K, V, const N: usize>
where
    K: Eq,
{
    map: BackingMap<K, V, N>,
}

impl<K, V, const N: usize> KvStore<K, V, N>
where
    K: Eq,
{
    pub const fn new() -> Self {
        Self {
            map: BackingMap::new(),
        }
    }

    pub fn capacity(&self) -> usize {
        N
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_full(&self) -> bool {
        self.map.is_full()
    }

    pub fn clear(&mut self) {
        self.map.clear()
    }

    pub fn put(&mut self, k: K, v: V) -> Result<Option<V>, (K, V)> {
        self.map.insert(k, v).map_err(|e| match e {
            heapless::insert::Error::Full(kv) => kv,
        })
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        self.map.get(key)
    }

    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        self.map.get_mut(key)
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        self.map.remove(key)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.map.iter()
    }
}
