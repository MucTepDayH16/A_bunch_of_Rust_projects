use std::{cmp::Ordering, panic::catch_unwind};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Origin {
    Root,
    Signed(u64),
    None,
}

impl Origin {
    pub fn ensure_root(self) -> Result<(), DispatchError> {
        match self {
            Origin::Root => Ok(()),
            _ => Err(DispatchError::BadOrigin),
        }
    }

    pub fn ensure_signed(self) -> Result<u64, DispatchError> {
        match self {
            Origin::Signed(who) => Ok(who),
            _ => Err(DispatchError::BadOrigin),
        }
    }
}

impl PartialOrd for Origin {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Origin {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Origin::Root, Origin::Root) => Ordering::Equal,
            (Origin::Root, _) => Ordering::Greater,
            (Origin::Signed(_), Origin::Root) => Ordering::Less,
            (Origin::Signed(_), Origin::Signed(_)) => Ordering::Equal,
            (Origin::Signed(_), Origin::None) => Ordering::Greater,
            (Origin::None, Origin::None) => Ordering::Equal,
            (Origin::None, _) => Ordering::Less,
        }
    }
}

pub type DispatchResult<R = ()> = Result<R, DispatchError>;

#[derive(Debug)]
pub enum DispatchError {
    BadOrigin,
    ModuleError { module: u8, index: u8 },
    Panic(Box<dyn std::any::Any + Send>),
    NotImplemented,
}

impl std::error::Error for DispatchError {}

impl std::fmt::Display for DispatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as std::fmt::Debug>::fmt(self, f)
    }
}

trait ModuleError {
    const MODULE: u8;

    fn errno(&self) -> u8;
}

impl<E: ModuleError> From<E> for DispatchError {
    fn from(e: E) -> Self {
        Self::ModuleError {
            module: E::MODULE,
            index: e.errno(),
        }
    }
}

pub trait Dispatchable: Sized {
    fn dispatch(self, origin: Origin) -> Result<(), DispatchError> {
        let _ = origin;
        Err(DispatchError::NotImplemented)
    }
}

impl<D: Dispatchable> Dispatchable for Box<D> {
    fn dispatch(self, origin: Origin) -> Result<(), DispatchError> {
        (*self).dispatch(origin)
    }
}

pub trait Weight {
    fn weight(&self) -> u64 {
        0
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Call {
    system(system::Call),
    sudo(sudo::Call),
    utility(utility::Call),
}

pub mod sudo;
pub mod system;
pub mod utility;

impl Dispatchable for Call {
    fn dispatch(self, origin: Origin) -> Result<(), DispatchError> {
        let maybe_panic = catch_unwind(|| match self {
            Call::system(system) => system.dispatch(origin),
            Call::sudo(sudo) => sudo.dispatch(origin),
            Call::utility(utility) => utility.dispatch(origin),
        });

        match maybe_panic {
            Ok(dispatch_result) => dispatch_result,
            Err(catched_panic) => Err(DispatchError::Panic(catched_panic)),
        }
    }
}
