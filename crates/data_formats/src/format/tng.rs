use thiserror::Error;

use crate::util::kv::{Kv, KvError};

#[derive(Clone, Debug)]
pub struct Tng {
    // raw_tng: RawTng<'a>,
}

#[derive(Clone, Debug, Error)]
#[error(transparent)]
pub struct TngError(#[from] KvError);

impl Tng {
    pub fn parse(source: &str) -> Result<Self, TngError> {
        let kv = Kv::parse(source)?;
        Ok(Self {})
    }
}
