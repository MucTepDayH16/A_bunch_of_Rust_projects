use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

#[versioned::versioned]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageValue<Timestamp: Default> {
    pub msg: String,
    #[version(1)]
    pub timestamp: Timestamp,
    pub _phantom: PhantomData<(Timestamp,)>,
}

impl<Timestamp: Default> TryFrom<MessageValueV0<Timestamp>>
    for MessageValueV1<Timestamp>
{
    type Error = ();

    fn try_from(value: MessageValueV0<Timestamp>) -> Result<Self, Self::Error> {
        Ok(Self {
            msg: value.msg,
            timestamp: Default::default(),
            _phantom: Default::default(),
        })
    }
}
