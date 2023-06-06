use std::{marker::PhantomData, sync::Arc};

use serde::{de::DeserializeOwned, Serialize};

use crate::{codec, StorageInfo};

pub struct StorageMap<K0, K1, V, INFO> {
    inner: Arc<rocksdb::DB>,
    _phantom: PhantomData<(K0, K1, V, INFO)>,
}

impl<
        K0: Serialize + DeserializeOwned,
        K1: Serialize + DeserializeOwned,
        V: Serialize + DeserializeOwned,
        INFO: StorageInfo,
    > StorageMap<K0, K1, V, INFO>
{
    pub fn new(rocks_db: &Arc<rocksdb::DB>) -> Self {
        Self {
            inner: rocks_db.clone(),
            _phantom: PhantomData,
        }
    }

    fn get_semi_key(key: &K0) -> Vec<u8> {
        let raw_key = codec::ser(key);
        [&INFO::prefix()[..], &raw_key].concat()
    }

    fn get_final_key(keys: (&K0, &K1)) -> Vec<u8> {
        let raw_key_0 = codec::ser(keys.0);
        let raw_key_1 = codec::ser(keys.1);
        [&INFO::prefix()[..], &raw_key_0, &raw_key_1].concat()
    }

    pub fn put(&self, keys: (&K0, &K1), value: &V) {
        let raw_key = Self::get_final_key(keys);
        let raw_value = codec::ser(value);

        if let Err(err) = self.inner.put(raw_key, raw_value) {
            log::error!("Cannot put value. error: {:?}", err);
        }
    }

    pub fn exists(&self, keys: (&K0, &K1)) -> bool {
        let raw_key = Self::get_final_key(keys);

        self.inner.key_may_exist(raw_key)
    }

    pub fn remove(&self, keys: (&K0, &K1)) {
        let raw_key = Self::get_final_key(keys);

        if let Err(err) = self.inner.delete(raw_key) {
            log::error!("Cannot remove value. error: {:?}", err);
        }
    }

    pub fn get(&self, keys: (&K0, &K1)) -> Option<V> {
        let raw_key = Self::get_final_key(keys);
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

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (K0, K1, V)> + 'a {
        self.inner
            .prefix_iterator(INFO::prefix())
            .filter_map(|result| {
                let (raw_key, raw_value) = result.ok()?;
                let keys = codec::de::<(K0, K1)>(&raw_key[32..]).ok()?;
                let value = codec::de::<V>(raw_value).ok()?;
                Some((keys.0, keys.1, value))
            })
    }

    pub fn iter_key<'a>(
        &'a self,
        key: &K0,
    ) -> impl Iterator<Item = (K1, V)> + 'a {
        self.inner
            .prefix_iterator(Self::get_semi_key(key))
            .filter_map(|result| {
                let (raw_key, raw_value) = result.ok()?;
                let (_, key) = codec::de::<(K0, K1)>(&raw_key[32..]).ok()?;
                let value = codec::de::<V>(raw_value).ok()?;
                Some((key, value))
            })
    }
}
