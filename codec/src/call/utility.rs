use serde::{Deserialize, Serialize};

use super::{DispatchResult, Dispatchable as _, ModuleError, Origin, Weight};

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum Error {
    LowPrivilege = 0,
    BatchInterupted { idx: usize, total: usize } = 1,
}

impl ModuleError for Error {
    const MODULE: u8 = 3;

    fn errno(&self) -> u8 {
        unsafe { std::mem::transmute(std::mem::discriminant(self)) }
    }
}

pub struct Module;

#[support::call]
impl Call for Module {
    fn batch(origin: Origin, calls: Vec<super::Call>) -> DispatchResult {
        let total = calls.len();
        for (idx, call) in calls.into_iter().enumerate() {
            if let Err(err) = call.dispatch(origin.clone()) {
                println!("DISPATCH_AS call results with error: {:?}", err);
                return Err(Error::BatchInterupted { idx, total }.into());
            }
        }
        Ok(())
    }

    fn dispatch_as(
        origin: Origin,
        as_origin: Origin,
        call: Box<super::Call>,
    ) -> DispatchResult {
        if origin <= as_origin {
            println!(
                "{:?} does not have enough privilege to dispatch as {:?}",
                origin, as_origin
            );
            return Err(Error::LowPrivilege.into());
        }

        if let Err(err) = call.dispatch(as_origin) {
            println!("DISPATCH_AS call results with error: {:?}", err);
        }

        Ok(())
    }
}

impl Weight for Call {}
