use serde::{Deserialize, Serialize};

use super::{DispatchResult, Origin, Weight};
use crate::storage::StorageValue;

pub struct Code;

impl StorageValue for Code {
    const STORAGE_ID: &'static [u8] = b"Code";
    type Value = Vec<u8>;
}

pub struct Module;

#[support::call]
impl Call for Module {
    fn set_code(origin: Origin, code: Vec<u8>) -> DispatchResult {
        origin.ensure_root()?;

        println!("New code {:02x?}", code);
        Code::set(code);

        Ok(())
    }

    fn set_storage(
        origin: Origin,
        key: Vec<u8>,
        value: Vec<u8>,
    ) -> DispatchResult {
        origin.ensure_root()?;

        crate::storage::LOCAL_BRANCH.with(move |branch| {
            branch.borrow_mut().insert(&key[..], &value[..]).unwrap();
        });

        Ok(())
    }
}

impl Weight for Call {
    fn weight(&self) -> u64 {
        match self {
            Call::set_code(code) => code.len() as _,
            Call::set_storage(key, value) => (key.len() + value.len()) as _,
        }
    }
}
