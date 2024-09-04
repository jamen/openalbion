use crate::util::kv::{Kv, KvError, KvField};
use thiserror::Error;

#[derive(Clone, Debug)]
pub struct Tng {
    sections: Vec<TngSection>,
}

#[derive(Clone, Debug, Error)]
pub enum TngError {
    #[error(transparent)]
    Kv(#[from] KvError),

    #[error(transparent)]
    Section(#[from] TngSectionError),
}

impl Tng {
    pub fn parse(source: &str) -> Result<Self, TngError> {
        let kv = Kv::parse(source)?;
        let mut fields = &kv.fields[..];
        let mut sections = Vec::new();

        while !kv.fields.is_empty() {
            sections.push(TngSection::parse(&mut fields)?);
        }

        Ok(Self { sections })
    }
}

#[derive(Clone, Debug)]
pub struct TngSection {
    things: Vec<TngThing>,
}

#[derive(Copy, Clone, Debug, Error)]
pub enum TngSectionError {
    #[error(transparent)]
    Thing(#[from] TngThingItemError),

    #[error(transparent)]
    Marker(#[from] TngMarkerError),

    #[error(transparent)]
    Object(#[from] TngObjectError),

    #[error(transparent)]
    HolySite(#[from] TngHolySiteError),
}

impl TngSection {
    fn parse(fields: &mut &[KvField]) -> Result<Self, TngSectionError> {}
}

#[derive(Clone, Debug)]
pub enum TngThing {
    Thing(TngThingItem),
    Marker(TngMarker),
    Object(TngObject),
    HolySite(TngHolySite),
}

#[derive(Copy, Clone, Debug, Error)]
pub enum TngThingError {}

impl TngThing {
    fn parse(fields: &mut &[KvField]) -> Result<Self, TngThingError> {}
}

#[derive(Clone, Debug)]
pub struct TngThingItem {}

#[derive(Copy, Clone, Debug, Error)]
pub enum TngThingItemError {}

impl TngThingItem {
    fn parse(fields: &mut &[KvField]) -> Result<Self, TngThingItemError> {}
}

#[derive(Clone, Debug)]
pub struct TngMarker {}

#[derive(Copy, Clone, Debug, Error)]
pub enum TngMarkerError {}

impl TngMarker {
    fn parse(fields: &mut &[KvField]) -> Result<Self, TngMarkerError> {}
}

#[derive(Clone, Debug)]
pub struct TngObject {}

#[derive(Copy, Clone, Debug, Error)]
pub enum TngObjectError {}

impl TngObject {
    fn parse(fields: &mut &[KvField]) -> Result<Self, TngObjectError> {}
}

#[derive(Clone, Debug)]
pub struct TngHolySite {}

#[derive(Copy, Clone, Debug, Error)]
pub enum TngHolySiteError {}

impl TngHolySite {
    fn parse(fields: &mut &[KvField]) -> Result<Self, TngHolySiteError> {}
}
