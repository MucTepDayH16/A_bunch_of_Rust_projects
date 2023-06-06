use std::cell::RefCell;

use sha2::*;
use vsdb::{MapxRawVs, ValueEnDe};

thread_local! {
    pub static LOCAL_BRANCH: RefCell<MapxRawVs> = RefCell::new(MapxRawVs::new());
}

pub trait StorageValue {
    const STORAGE_ID: &'static [u8];
    type Value: ValueEnDe;

    fn get() -> Option<Self::Value> {
        LOCAL_BRANCH.with(|branch| {
            let key = Sha256::digest(&Self::STORAGE_ID[..]).to_vec();
            Self::Value::decode(branch.borrow().get(&key[..])?.as_ref()).ok()
        })
    }

    fn set(value: Self::Value) {
        LOCAL_BRANCH.with(|branch| {
            let key = Sha256::digest(&Self::STORAGE_ID[..]).to_vec();
            let value = value.encode();
            let _ = branch.borrow_mut().insert(key, value).unwrap();
        })
    }
}
