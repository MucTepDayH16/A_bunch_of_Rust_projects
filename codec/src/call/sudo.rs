use serde::{Deserialize, Serialize};

use super::{DispatchResult, Dispatchable as _, ModuleError, Origin, Weight};
use crate::storage::StorageValue;

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum Error {
    NotSudo = 0,
}

impl ModuleError for Error {
    const MODULE: u8 = 1;

    fn errno(&self) -> u8 {
        unsafe { std::mem::transmute(std::mem::discriminant(self)) }
    }
}

pub struct Module;

pub struct SudoKey;

impl StorageValue for SudoKey {
    const STORAGE_ID: &'static [u8] = b"SudoKey";
    type Value = u64;
}

impl Module {
    fn is_sudo(who: &u64) -> bool {
        SudoKey::get().map_or(false, |sudo| &sudo == who)
    }
}

#[support::call]
impl Call for Module {
    fn sudo(origin: Origin, call: Box<super::Call>) -> DispatchResult {
        let maybe_sudo = origin.ensure_signed()?;

        if !Self::is_sudo(&maybe_sudo) {
            println!("{:?} does not have access to root operation", maybe_sudo);
            return Err(Error::NotSudo.into());
        }

        if let Err(err) = call.dispatch(Origin::Root) {
            println!("SUDO call results with error: {:?}", err);
        }

        Ok(())
    }

    fn sudo_as(origin: Origin, who: u64, call: Box<Call>) -> DispatchResult {
        let maybe_sudo = origin.ensure_signed()?;
        if !Self::is_sudo(&maybe_sudo) {
            println!("{:?} does not have access to root operation", maybe_sudo);
            return Err(Error::NotSudo.into());
        }

        if let Err(err) = call.dispatch(Origin::Signed(who)) {
            println!("SUDO call results with error: {:?}", err);
        }

        Ok(())
    }

    fn set_key(origin: Origin, new_sudo: u64) -> DispatchResult {
        origin.ensure_root()?;
        SudoKey::set(new_sudo);

        Ok(())
    }
}

impl Weight for Call {
    fn weight(&self) -> u64 {
        0
    }
}
