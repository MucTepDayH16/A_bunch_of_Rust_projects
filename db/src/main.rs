use std::sync::Arc;

use rocksdb::{DBCompressionType, Options};

pub mod codec;
pub mod message_format;
pub mod storage_double_map;
pub mod storage_map;

use message_format::*;

pub trait StorageInfo {
    const STORAGE_NAME: &'static str;

    fn prefix() -> [u8; 32] {
        <sha3::Sha3_256 as sha3::Digest>::digest(Self::STORAGE_NAME).into()
    }
}

pub struct MessagesInfo;
impl StorageInfo for MessagesInfo {
    const STORAGE_NAME: &'static str = "Messages";
}

pub type MessageId = u64;

pub type Messages = storage_map::StorageMap<
    MessageId,
    message_format::MessageValue<u64>,
    MessagesInfo,
>;

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .try_init()?;

    let path = "./base";
    let mut open_options = Options::default();
    open_options.create_if_missing(true);
    open_options.enable_statistics();
    open_options.set_atomic_flush(true);
    open_options.set_compression_type(DBCompressionType::Zlib);

    let db = Arc::new(rocksdb::DB::open(&open_options, path)?);
    let messages = Messages::new(&db);

    messages.put(
        &1,
        &MessageValueV1 {
            msg: "abc".into(),
            timestamp: 1,
            _phantom: Default::default(),
        }
        .into(),
    );
    messages.put(
        &2,
        &MessageValueV1 {
            msg: "def".into(),
            timestamp: 2,
            _phantom: Default::default(),
        }
        .into(),
    );

    messages.iter().for_each(|(key, value)| {
        println!("{:?} => {:?}", key, value);
    });

    messages.remove(&3);

    println!("{:?}", messages.get(&2));

    Ok(())
}
