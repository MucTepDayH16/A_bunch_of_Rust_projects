use bincode::{DefaultOptions, Options as _};
use call::*;
use vsdb::{KeyEnDe, VersionName, VsMgmt};

use crate::storage::{StorageValue, LOCAL_BRANCH};

mod call;
mod storage;

#[derive(Clone, Debug)]
pub struct RawDispatchable {
    pub raw: Vec<u8>,
}

impl RawDispatchable {
    pub fn new<Raw: AsRef<[u8]>>(raw: Raw) -> Self {
        Self {
            raw: raw.as_ref().to_vec(),
        }
    }

    pub fn decode(self) -> Option<Call> {
        DefaultOptions::new()
            .reject_trailing_bytes()
            .with_varint_encoding()
            .deserialize::<'_, Call>(&self.raw[..])
            .ok()
    }
}

fn main() {
    LOCAL_BRANCH.with(|branch| {
        branch
            .borrow_mut()
            .version_create(VersionName(b"0.0.0"))
            .unwrap()
    });

    println!("{:?}", sudo::SudoKey::get());
    sudo::Call::set_key(0x0123456789abcdef)
        .dispatch(Origin::Root)
        .unwrap();
    println!("{:?}", sudo::SudoKey::get());

    let new_code = vec![0x00, 0x64, 0x02, 0x03];

    let call = Call::sudo(sudo::Call::sudo(Box::new(Call::system(
        system::Call::set_code(new_code.clone()),
    ))));
    let call_enc = call.encode();
    assert_eq!(
        call_enc,
        &[0x01, 0x00, 0x00, 0x00, 0x04, 0x00, 0x64, 0x02, 0x03]
    );

    let call_raw = RawDispatchable::new(call_enc);
    println!("{:?}", call_raw);
    let result = call_raw
        .decode()
        .expect("Decode error")
        .dispatch(Origin::Signed(0x0123456789abcdef));
    println!("{:?}", result);

    assert_eq!(system::Code::get(), Some(new_code));
}
