use bincode::Options;
use serde::{de::DeserializeOwned, Serialize};

fn options() -> impl Options {
    bincode::options()
        .allow_trailing_bytes()
        .with_no_limit()
        .with_varint_encoding()
        .with_big_endian()
}

pub fn ser<T: Serialize>(t: &T) -> Vec<u8> {
    options().serialize(t).expect("Should always serialize")
}

pub fn de<T: DeserializeOwned>(
    raw_t: impl AsRef<[u8]>,
) -> Result<T, bincode::Error> {
    options().deserialize(raw_t.as_ref())
}
