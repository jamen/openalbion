//! Parser for Fable's animation assets — the `3DAF` chunked format in `graphics.big`.
//!
//! `graphics.big` carries **3,435 animation assets, 87 MB**, alongside its 3,295 meshes;
//! `ExtraMetadata::Animation` (asset type 24) already identified them and nothing read them.
//! An animation is a skeleton's worth of per-bone tracks, sampled uniformly.
//!
//! ## The container
//!
//! The asset is **LZO-compressed whole**: a `u32` decompressed length followed by an LZO1X
//! stream. Inside is a chunked file — `CChunkedFile` (`bbblibrary/lib_chunked_file.hpp`), the
//! same family as the `C3DMeshFileX*` chunks:
//!
//! ```text
//! ">>>>"  "3DAF"  u32 version(100)  cstr "Copyright Big Blue Box Studios Ltd."
//!   ANRT  u32 size   u8 looping, f32 duration
//!     AOBJ  u32 size   cstr name, u32 …
//!       XSEQ  u32 size   one bone's tracks
//!       XSEQ …
//!     HLPR  u32 size   helper points (not parsed)
//! ```
//!
//! Every chunk is `tag(4) + size(4) + payload(size)`, and `size` covers the payload including
//! nested children. All 3,435 assets decompress and walk with no bytes left over.
//!
//! ## The tracks are compressed, and that is the whole of the format
//!
//! `C3DAnimFileSequenceChunk`'s *authoring* reader (`lib_3d_anim_file_sequence.cpp:87`) reads
//! raw 48-byte `CMatrix3x4` samples — and **no shipped track is stored that way**: 0 of 210,743
//! sequences have `data == frames × 48`, and the median is 7 bytes per sample. Retail uses
//! `C3DAnimationSequenceData::ReadCompressedFromFile` (`lib_3d_animation_2.cpp:1328`) instead,
//! whose class (`lib_3d_animation_2.hpp`) names every field:
//!
//! - rotation is a **`CQuaternion` track** (4 × f32);
//! - position is a **`C3DVectorWord` track** — 3 × `i16`, scaled by `PositionFactor`;
//! - each track carries an `ETrackMode`: `IDENTITY`, `CONSTANT`, `NORMAL` or `PALETTED`;
//! - `PALETTED` stores the distinct values once plus a `u8` index per frame, which is what
//!   takes a near-static bone down to ~1 byte per sample.
//!
//! Scaling has a mode and a `ScalingFactor` but no track array of its own.

use crate::bytes::{TakeError, UnexpectedEnd, take_bytes};
use derive_more::{Display, Error, From};
use lzo::LzoError;

#[derive(Debug, Display, Error, From)]
pub enum AnimError {
    Take(TakeError),
    Bytes(UnexpectedEnd),
    Decompress(LzoError),
    #[from(skip)]
    #[display("not a 3DAF animation (magic {_0:?})")]
    BadMagic(#[error(not(source))] [u8; 4]),
    #[from(skip)]
    #[display("chunk {_0} claims {_1} bytes but only {_2} remain")]
    ChunkOverrun(#[error(not(source))] String, usize, usize),
    #[from(skip)]
    #[display("no ANRT root chunk")]
    NoRoot,
    #[from(skip)]
    #[display("invalid UTF-8 in an animation string")]
    Utf8,
}

/// How one track is stored — `C3DAnimationSequenceData::ETrackMode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackMode {
    /// No data: the track is the identity for every frame.
    Identity,
    /// One value, held for every frame.
    Constant,
    /// One value per frame.
    Normal,
    /// Distinct values plus a `u8` index per frame.
    Paletted,
}

impl TrackMode {
    fn from_bits(b: u8) -> TrackMode {
        match b & 3 {
            0 => TrackMode::Identity,
            1 => TrackMode::Constant,
            2 => TrackMode::Normal,
            _ => TrackMode::Paletted,
        }
    }
}

/// A quantised position — `C3DAnimationSequenceData::C3DVectorWord`, scaled by
/// [`Sequence::position_factor`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VectorWord {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

/// One bone's animation — a `XSEQ` chunk.
#[derive(Debug, Clone, PartialEq)]
pub struct Sequence {
    /// The bone this drives, by name. Retargeting is by name, not index: Fable's skeletons are
    /// 3ds Max Biped (`Bip01 Pelvis`, `Bip01 L Calf`, …), and the mesh carries the same names.
    pub bone_name: String,
    /// Index of the parent *sequence* in the owning object, or `-1` for the root.
    pub parent_index: i32,
    pub bone_type: u8,
    pub enabled: bool,
    pub samples_per_second: f32,
    pub frame_count: u32,
    pub rotation_mode: TrackMode,
    pub position_mode: TrackMode,
    pub scaling_mode: TrackMode,
    /// Multiplies [`VectorWord`] components to give world units.
    pub position_factor: f32,
    pub scaling_factor: f32,
    /// `(x, y, z, w)`, in the file's order.
    pub rotations: Vec<[f32; 4]>,
    /// Per-frame indices into [`Sequence::rotations`], when `rotation_mode` is `Paletted`.
    pub rotation_palette: Vec<u8>,
    pub positions: Vec<VectorWord>,
    /// Per-frame indices into [`Sequence::positions`], when `position_mode` is `Paletted`.
    pub position_palette: Vec<u8>,
}

impl Sequence {
    /// The rotation at a frame, resolved through this track's mode.
    ///
    /// Returns `None` for an `Identity` track — the caller substitutes the identity rotation
    /// rather than this inventing one.
    pub fn rotation_at(&self, frame: usize) -> Option<[f32; 4]> {
        match self.rotation_mode {
            TrackMode::Identity => None,
            TrackMode::Constant => self.rotations.first().copied(),
            TrackMode::Normal => self.rotations.get(frame).copied(),
            TrackMode::Paletted => self
                .rotation_palette
                .get(frame)
                .and_then(|&i| self.rotations.get(i as usize))
                .copied(),
        }
    }

    /// The position at a frame, in world units (the stored `i16`s times `position_factor`).
    pub fn position_at(&self, frame: usize) -> Option<[f32; 3]> {
        let word = match self.position_mode {
            TrackMode::Identity => None,
            TrackMode::Constant => self.positions.first().copied(),
            TrackMode::Normal => self.positions.get(frame).copied(),
            TrackMode::Paletted => self
                .position_palette
                .get(frame)
                .and_then(|&i| self.positions.get(i as usize))
                .copied(),
        }?;
        let f = self.position_factor;
        Some([word.x as f32 * f, word.y as f32 * f, word.z as f32 * f])
    }
}

/// One animated skeleton — an `AOBJ` chunk. An animation file usually holds exactly one.
#[derive(Debug, Clone, PartialEq)]
pub struct AnimObject {
    pub name: String,
    pub sequences: Vec<Sequence>,
}

/// A whole animation asset — the `ANRT` root.
#[derive(Debug, Clone, PartialEq)]
pub struct Animation {
    pub looping: bool,
    /// Seconds. `duration × samples_per_second` is the frame count, exactly.
    pub duration: f32,
    pub objects: Vec<AnimObject>,
}

impl Animation {
    /// Decode an animation from a `graphics.big` asset's bytes.
    pub fn decode(data: &[u8]) -> Result<Animation, AnimError> {
        let raw = decompress_asset(data)?;
        Self::decode_uncompressed(&raw)
    }

    /// Decode from already-decompressed bytes — the shape `fool` and the tests want.
    pub fn decode_uncompressed(d: &[u8]) -> Result<Animation, AnimError> {
        // ">>>>" then the magic. The leading marker is constant across every shipped asset.
        let magic_at = 4;
        let magic: [u8; 4] = take_at(d, magic_at, 4)?.try_into().unwrap();
        if &magic != b"3DAF" {
            return Err(AnimError::BadMagic(magic));
        }
        // u32 version, then a nul-terminated copyright string.
        let (_copyright, n) = read_cstr(d, 12)?;
        let mut off = 12 + n;

        let mut animation = None;
        while let Some((tag, payload, size)) = read_chunk(d, off, d.len())? {
            if &tag == b"ANRT" {
                animation = Some(read_root(d, payload, payload + size)?);
                break;
            }
            off = payload + size;
        }
        animation.ok_or(AnimError::NoRoot)
    }

    /// Frames at `samples_per_second`, taken from the first sequence that has any.
    ///
    /// The root chunk's `duration` and a sequence's `frame_count` agree exactly across the
    /// shipped data, so either can be used; this prefers the sequence because a still object
    /// stores `duration == 0` with one frame.
    pub fn frame_count(&self) -> u32 {
        self.objects
            .iter()
            .flat_map(|o| &o.sequences)
            .map(|s| s.frame_count)
            .max()
            .unwrap_or(0)
    }

    pub fn samples_per_second(&self) -> f32 {
        self.objects
            .iter()
            .flat_map(|o| &o.sequences)
            .map(|s| s.samples_per_second)
            .next()
            .unwrap_or(30.0)
    }
}

/// `u32` decompressed length, then an LZO1X stream.
///
/// Unlike `mesh.rs`'s `decompress_section`, which stores the *compressed* length, an animation
/// asset stores the decompressed one — verified on all 3,435 assets, every one of which
/// produces exactly the declared byte count.
pub fn decompress_asset(data: &[u8]) -> Result<Vec<u8>, AnimError> {
    let len_bytes = take_at(data, 0, 4)?;
    let size = u32::from_le_bytes(len_bytes.try_into().unwrap()) as usize;
    Ok(lzo::decompress(&data[4..], size)?)
}

fn take_at(d: &[u8], off: usize, len: usize) -> Result<&[u8], AnimError> {
    let mut cur = d.get(off..).ok_or(UnexpectedEnd)?;
    Ok(take_bytes(&mut cur, len)?)
}

fn read_u32(d: &[u8], off: usize) -> Result<u32, AnimError> {
    Ok(u32::from_le_bytes(take_at(d, off, 4)?.try_into().unwrap()))
}

fn read_f32(d: &[u8], off: usize) -> Result<f32, AnimError> {
    Ok(f32::from_le_bytes(take_at(d, off, 4)?.try_into().unwrap()))
}

fn read_u16(d: &[u8], off: usize) -> Result<u16, AnimError> {
    Ok(u16::from_le_bytes(take_at(d, off, 2)?.try_into().unwrap()))
}

fn read_u8(d: &[u8], off: usize) -> Result<u8, AnimError> {
    Ok(take_at(d, off, 1)?[0])
}

/// A nul-terminated string; returns it and the bytes consumed *including* the terminator.
fn read_cstr(d: &[u8], off: usize) -> Result<(String, usize), AnimError> {
    let rest = d.get(off..).ok_or(UnexpectedEnd)?;
    let end = rest.iter().position(|&b| b == 0).ok_or(UnexpectedEnd)?;
    let s = std::str::from_utf8(&rest[..end])
        .map_err(|_| AnimError::Utf8)?
        .to_string();
    Ok((s, end + 1))
}

/// `tag(4) + size(4)`, returning `(tag, payload_offset, size)`, or `None` at the end.
fn read_chunk(d: &[u8], off: usize, end: usize) -> Result<Option<([u8; 4], usize, usize)>, AnimError> {
    if off + 8 > end {
        return Ok(None);
    }
    let tag: [u8; 4] = take_at(d, off, 4)?.try_into().unwrap();
    let size = read_u32(d, off + 4)? as usize;
    let payload = off + 8;
    if payload + size > end {
        return Err(AnimError::ChunkOverrun(
            String::from_utf8_lossy(&tag).to_string(),
            size,
            end.saturating_sub(payload),
        ));
    }
    Ok(Some((tag, payload, size)))
}

/// `ANRT` — `bool LoopFlag`, `float Duration`, then child chunks.
fn read_root(d: &[u8], payload: usize, end: usize) -> Result<Animation, AnimError> {
    let looping = read_u8(d, payload)? != 0;
    let duration = read_f32(d, payload + 1)?;

    let mut objects = Vec::new();
    let mut off = payload + 5;
    while let Some((tag, p, size)) = read_chunk(d, off, end)? {
        if &tag == b"AOBJ" {
            objects.push(read_object(d, p, p + size)?);
        }
        // HLPR (helper points) and anything else are skipped deliberately — nothing draws them.
        off = p + size;
    }

    Ok(Animation {
        looping,
        duration,
        objects,
    })
}

/// `AOBJ` — a name, one long, then the bone sequences.
///
/// `C3DAnimFileObjectChunk` declares `SubMeshIndex`, `ParentIndex`, `FirstChildIndex` and
/// `NextSiblingIndex`, but only **one** long is present between the name and the first child
/// chunk in every shipped asset; the tree indices are rebuilt from the sequences' own
/// `parent_index` on load. Reading four would overrun into the first `XSEQ` tag.
fn read_object(d: &[u8], payload: usize, end: usize) -> Result<AnimObject, AnimError> {
    let (name, n) = read_cstr(d, payload)?;
    let mut off = payload + n + 4;

    let mut sequences = Vec::new();
    while let Some((tag, p, size)) = read_chunk(d, off, end)? {
        if &tag == b"XSEQ" {
            sequences.push(read_sequence(d, p, p + size)?);
        }
        off = p + size;
    }
    Ok(AnimObject { name, sequences })
}

/// `XSEQ` — one bone's compressed tracks.
///
/// Field order is `C3DAnimationSequenceData::ReadCompressedFromFile`
/// (`lib_3d_animation_2.cpp:1328`) behind a two-field chunk prefix:
///
/// ```text
/// u32  version (0x7ADA on every shipped sequence)
/// i32  parent index
/// cstr bone name
/// u8   bone type          (the low 5 bits of a packed byte)
/// f32  samples per second
/// u32  frame count
/// u8   enabled
/// u8   rotation track mode      \
/// u8   position track mode       > two bits each, written as three separate bytes
/// u8   scaling  track mode      /
/// f32  position factor
/// f32  scaling factor
/// u16 n, [CQuaternion; n]       rotation values
/// u16 n, [u8; n]                per-frame rotation palette indices
/// u16 n, [C3DVectorWord; n]     position values
/// u16 n, [u8; n]                per-frame position palette indices
/// ```
fn read_sequence(d: &[u8], payload: usize, end: usize) -> Result<Sequence, AnimError> {
    let _version = read_u32(d, payload)?;
    let parent_index = read_u32(d, payload + 4)? as i32;
    let (bone_name, n) = read_cstr(d, payload + 8)?;
    let mut off = payload + 8 + n;

    let bone_type = read_u8(d, off)? & 0x1f;
    let samples_per_second = read_f32(d, off + 1)?;
    let frame_count = read_u32(d, off + 5)?;
    let enabled = read_u8(d, off + 9)? != 0;
    let rotation_mode = TrackMode::from_bits(read_u8(d, off + 10)?);
    let position_mode = TrackMode::from_bits(read_u8(d, off + 11)?);
    let scaling_mode = TrackMode::from_bits(read_u8(d, off + 12)?);
    let position_factor = read_f32(d, off + 13)?;
    let scaling_factor = read_f32(d, off + 17)?;
    off += 21;

    let rot_count = read_u16(d, off)? as usize;
    off += 2;
    let mut rotations = Vec::with_capacity(rot_count);
    for _ in 0..rot_count {
        rotations.push([
            read_f32(d, off)?,
            read_f32(d, off + 4)?,
            read_f32(d, off + 8)?,
            read_f32(d, off + 12)?,
        ]);
        off += 16;
    }

    let rot_palette_count = read_u16(d, off)? as usize;
    off += 2;
    let rotation_palette = take_at(d, off, rot_palette_count)?.to_vec();
    off += rot_palette_count;

    let pos_count = read_u16(d, off)? as usize;
    off += 2;
    let mut positions = Vec::with_capacity(pos_count);
    for _ in 0..pos_count {
        positions.push(VectorWord {
            x: i16::from_le_bytes(take_at(d, off, 2)?.try_into().unwrap()),
            y: i16::from_le_bytes(take_at(d, off + 2, 2)?.try_into().unwrap()),
            z: i16::from_le_bytes(take_at(d, off + 4, 2)?.try_into().unwrap()),
        });
        off += 6;
    }

    let pos_palette_count = read_u16(d, off)? as usize;
    off += 2;
    let position_palette = take_at(d, off, pos_palette_count)?.to_vec();
    off += pos_palette_count;

    debug_assert!(off <= end, "sequence {bone_name} read past its chunk");
    let _ = end;

    Ok(Sequence {
        bone_name,
        parent_index,
        bone_type,
        enabled,
        samples_per_second,
        frame_count,
        rotation_mode,
        position_mode,
        scaling_mode,
        position_factor,
        scaling_factor,
        rotations,
        rotation_palette,
        positions,
        position_palette,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn track_modes_come_from_the_low_two_bits() {
        assert_eq!(TrackMode::from_bits(0), TrackMode::Identity);
        assert_eq!(TrackMode::from_bits(1), TrackMode::Constant);
        assert_eq!(TrackMode::from_bits(2), TrackMode::Normal);
        assert_eq!(TrackMode::from_bits(3), TrackMode::Paletted);
        // Only the low two bits participate; the byte carries other fields in the engine.
        assert_eq!(TrackMode::from_bits(0xfc), TrackMode::Identity);
    }

    fn seq(mode: TrackMode, values: Vec<[f32; 4]>, palette: Vec<u8>) -> Sequence {
        Sequence {
            bone_name: "Bip01".into(),
            parent_index: -1,
            bone_type: 0,
            enabled: true,
            samples_per_second: 30.0,
            frame_count: 4,
            rotation_mode: mode,
            position_mode: TrackMode::Identity,
            scaling_mode: TrackMode::Identity,
            position_factor: 1.0,
            scaling_factor: 1.0,
            rotations: values,
            rotation_palette: palette,
            positions: Vec::new(),
            position_palette: Vec::new(),
        }
    }

    /// An identity track yields nothing, so the caller substitutes the identity rather than
    /// this module inventing a value.
    #[test]
    fn identity_tracks_yield_nothing() {
        let s = seq(TrackMode::Identity, vec![], vec![]);
        assert_eq!(s.rotation_at(0), None);
        assert_eq!(s.rotation_at(3), None);
    }

    /// A constant track holds its single value for every frame — including frames past the
    /// stored count, which is the whole point of the mode.
    #[test]
    fn constant_tracks_hold_one_value() {
        let s = seq(TrackMode::Constant, vec![[0.0, 0.0, 0.0, 1.0]], vec![]);
        assert_eq!(s.rotation_at(0), Some([0.0, 0.0, 0.0, 1.0]));
        assert_eq!(s.rotation_at(99), Some([0.0, 0.0, 0.0, 1.0]));
    }

    #[test]
    fn normal_tracks_index_by_frame() {
        let s = seq(
            TrackMode::Normal,
            vec![[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0]],
            vec![],
        );
        assert_eq!(s.rotation_at(0), Some([1.0, 0.0, 0.0, 0.0]));
        assert_eq!(s.rotation_at(1), Some([0.0, 1.0, 0.0, 0.0]));
        assert_eq!(s.rotation_at(2), None, "past the end rather than wrapping");
    }

    /// The paletted mode is the one that earns the format its size: distinct values once, a
    /// byte per frame. Frame 2 and frame 0 sharing a value must give the same rotation.
    #[test]
    fn paletted_tracks_index_through_the_palette() {
        let s = seq(
            TrackMode::Paletted,
            vec![[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0]],
            vec![0, 1, 0, 1],
        );
        assert_eq!(s.rotation_at(0), Some([1.0, 0.0, 0.0, 0.0]));
        assert_eq!(s.rotation_at(1), Some([0.0, 1.0, 0.0, 0.0]));
        assert_eq!(s.rotation_at(2), s.rotation_at(0));
    }

    /// Positions are `i16`s times `position_factor`; forgetting the factor would put every
    /// bone thousands of units from the origin.
    #[test]
    fn positions_scale_by_the_factor() {
        let mut s = seq(TrackMode::Identity, vec![], vec![]);
        s.position_mode = TrackMode::Constant;
        s.position_factor = 0.5;
        s.positions = vec![VectorWord { x: 2, y: -4, z: 100 }];
        assert_eq!(s.position_at(0), Some([1.0, -2.0, 50.0]));
    }
}
