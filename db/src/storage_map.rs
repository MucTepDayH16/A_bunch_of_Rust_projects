use std::{marker::PhantomData, sync::Arc};

use serde::{de::DeserializeOwned, Serialize};

use crate::{codec, StorageInfo};

pub struct StorageMap<K, V, INFO> {
    inner: Arc<rocksdb::DB>,
    _phantom: PhantomData<(K, V, INFO)>,
}

impl<
        K: Serialize + DeserializeOwned,
        V: Serialize + DeserializeOwned,
        INFO: StorageInfo,
    > StorageMap<K, V, INFO>
{
    pub fn new(rocks_db: &Arc<rocksdb::DB>) -> Self {
        Self {
            inner: rocks_db.clone(),
            _phantom: PhantomData,
        }
    }

    fn get_final_key(key: &K) -> Vec<u8> {
        let raw_key = codec::ser(key);
        [&INFO::prefix()[..], &raw_key].concat()
    }

    pub fn put(&self, key: &K, value: &V) {
        let raw_key = Self::get_final_key(key);
        let raw_value = codec::ser(value);

        if let Err(err) = self.inner.put(raw_key, raw_value) {
            log::error!("Cannot put value. error: {:?}", err);
        }
    }

    pub fn exists(&self, key: &K) -> bool {
        let raw_key = Self::get_final_key(key);

        self.inner.key_may_exist(raw_key)
    }

    pub fn remove(&self, key: &K) {
        let raw_key = Self::get_final_key(key);

        if let Err(err) = self.inner.delete(raw_key) {
            log::error!("Cannot remove value. error: {:?}", err);
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let raw_key = Self::get_final_key(key);
        let raw_value = match self.inner.get_pinned(&raw_key).ok().flatten() {
            Some(raw_value) => raw_value,
            None => {
                log::error!("Value not found. key: {:?}", raw_key);
                return None;
            }
        };

        match codec::de::<V>(&raw_value) {
            Ok(value) => Some(value),
            Err(err) => {
                log::error!(
                    "Cannot deserialize. raw_value: {:?}, error: {:?}",
                    &raw_value[..],
                    err
                );
                None
            }
        }
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (K, V)> + 'a {
        self.inner
            .prefix_iterator(INFO::prefix())
            .filter_map(|result| {
                let (raw_key, raw_value) = result.ok()?;
                let key = codec::de::<K>(&raw_key[32..]).ok()?;
                let value = codec::de::<V>(raw_value).ok()?;
                Some((key, value))
            })
    }
}
