//! `ENGINE_THEME_GROUP` / `THING_GROUP` — C++ `CEngineThemeGroupDef`.
//!
//! Its `Transfer` body is empty: no fields are serialized (verified against the
//! spec and byte-exact against the compiled bins).

use crate::def::binary::control::{ParseControlError, SerializeControlError};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct EngineThemeGroupDef;

impl EngineThemeGroupDef {
    pub(crate) fn parse(_cur: &mut &[u8]) -> Result<Self, ParseControlError> {
        Ok(Self)
    }

    pub(crate) fn serialize(&self, _out: &mut &mut [u8]) -> Result<(), SerializeControlError> {
        Ok(())
    }

    pub(crate) fn byte_size(&self) -> usize {
        0
    }
}
