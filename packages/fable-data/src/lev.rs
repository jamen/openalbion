//! Parser for Fable's `.lev` level files.
//!
//! A `.lev` file describes a level's landscape: a header (with map dimensions and sound themes),
//! a grid of heightmap cells (the terrain elevation + ground/sound theme blending), a run of
//! soundmap cells filling the space up to the navigation graph, and finally the navigation graph.
//!
//! This parser currently covers the header, heightmap, and soundmap — everything needed to build
//! a terrain mesh. The navigation graph (after `header.navigation_offset`) is not parsed yet.
//!
//! Field layout is derived from the historical OpenAlbion parsers and fabletlcmod.com docs.

use crate::bytes::{TakeError, UnexpectedEnd, take, take_bytes};
use derive_more::{Display, Error, From};

/// Size in bytes of the heightmap and sound palettes embedded in the header.
const PALETTE_SIZE: usize = 33792;

#[derive(Debug, Display, Error, From)]
pub enum LevError {
    Take(TakeError),
    Bytes(UnexpectedEnd),
    #[from(skip)]
    #[display("invalid UTF-8 in sound theme name")]
    SoundThemeUtf8,
    #[from(skip)]
    #[display("invalid UTF-8 in navigation section name")]
    NavSectionNameUtf8,
    #[from(skip)]
    #[display("cell reported size ({size}) smaller than its {consumed} parsed bytes")]
    CellSizeUnderflow { size: u32, consumed: usize },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Lev {
    pub header: LevHeader,
    pub heightmap_cells: Vec<LevHeightCell>,
    pub soundmap_cells: Vec<LevSoundCell>,
    pub navigation: LevNavigation,
}

impl Lev {
    /// Parse a level from a cursor, consuming it to the end of the file.
    ///
    /// The cursor must begin at the start of the file (offset 0) so that the absolute offsets in
    /// the header (`navigation_offset`) and navigation sections can be used to delimit sections.
    pub fn parse(i: &mut &[u8]) -> Result<Lev, LevError> {
        let total_len = i.len();
        let abs = |i: &&[u8]| total_len - i.len();

        let header = LevHeader::parse(i)?;
        let navigation_offset = header.navigation_offset as usize;

        // The heightmap is a (width + 1) by (height + 1) grid of cells.
        let heightmap_cell_count = ((header.width + 1) * (header.height + 1)) as usize;

        let mut heightmap_cells = Vec::with_capacity(heightmap_cell_count);
        for _ in 0..heightmap_cell_count {
            heightmap_cells.push(LevHeightCell::parse(i)?);
        }

        // The soundmap is a variable number of cells filling the space up to the navigation graph.
        // Each cell is size-prefixed, so we read cells until the cursor reaches navigation_offset.
        let mut soundmap_cells = Vec::new();
        while abs(i) < navigation_offset {
            soundmap_cells.push(LevSoundCell::parse(i)?);
        }

        let navigation = LevNavigation::parse(i)?;

        Ok(Lev {
            header,
            heightmap_cells,
            soundmap_cells,
            navigation,
        })
    }

    /// Parse a level from a byte slice.
    pub fn from_bytes(bytes: &[u8]) -> Result<Lev, LevError> {
        Self::parse(&mut &bytes[..])
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LevHeader {
    pub version: u16,
    pub obsolete_offset: u32,
    pub navigation_offset: u32,
    pub unique_id_count: u64,
    pub width: u32,
    pub height: u32,
    pub map_version: u32,
    pub ambient_sound_version: u32,
    pub checksum: u32,
    pub sound_themes: Vec<String>,
}

impl LevHeader {
    pub fn parse(i: &mut &[u8]) -> Result<LevHeader, LevError> {
        let _header_size = take::<u32>(i)?.to_le();
        let version = take::<u16>(i)?.to_le();
        let _unknown_1 = take_bytes(i, 3)?; // fabletlcmod.com: 3 bytes of padding
        let _unknown_2 = take::<u32>(i)?.to_le();
        let obsolete_offset = take::<u32>(i)?.to_le();
        let _unknown_3 = take::<u32>(i)?.to_le();
        let navigation_offset = take::<u32>(i)?.to_le();
        let _map_header_size = take::<u8>(i)?;
        let map_version = take::<u32>(i)?.to_le();
        let unique_id_count = take::<u64>(i)?.to_le();
        let width = take::<u32>(i)?.to_le();
        let height = take::<u32>(i)?.to_le();
        let _always_true = take::<u8>(i)?;
        let _heightmap_palette = take_bytes(i, PALETTE_SIZE)?;
        let ambient_sound_version = take::<u32>(i)?.to_le();
        let sound_themes_count = take::<u32>(i)?.to_le();
        let _sound_palette = take_bytes(i, PALETTE_SIZE)?;
        let checksum = take::<u32>(i)?.to_le();

        // The first sound theme slot is empty, so there are `count - 1` named themes.
        let sound_theme_count = sound_themes_count.saturating_sub(1);
        let mut sound_themes = Vec::with_capacity(sound_theme_count as usize);
        for _ in 0..sound_theme_count {
            let len = take::<u32>(i)?.to_le() as usize;
            let bytes = take_bytes(i, len)?;
            let theme = str::from_utf8(bytes).map_err(|_| LevError::SoundThemeUtf8)?;
            sound_themes.push(theme.to_owned());
        }

        Ok(LevHeader {
            version,
            obsolete_offset,
            navigation_offset,
            unique_id_count,
            width,
            height,
            map_version,
            ambient_sound_version,
            checksum,
            sound_themes,
        })
    }
}

/// A single heightmap cell: terrain elevation and ground/sound theme blending at one grid point.
#[derive(Debug, Clone, PartialEq)]
pub struct LevHeightCell {
    pub size: u32,
    pub version: u8,
    pub height: f32,
    /// Up to three blended ground textures (theme palette indices).
    pub ground_theme: (u8, u8, u8),
    /// Blend strengths between the three ground themes.
    pub ground_theme_strength: (u8, u8),
    pub walkable: bool,
    pub passover: bool,
    pub sound_theme: u8,
    pub shore: bool,
}

impl LevHeightCell {
    pub fn parse(i: &mut &[u8]) -> Result<LevHeightCell, LevError> {
        // `size` is the total byte length of this cell, including the size field itself. Cells are
        // variable-length (larger cells carry extra blend data after the fields below), so we read
        // the fields we understand and then skip to the end of the cell.
        let start_len = i.len();
        let size = take::<u32>(i)?.to_le();
        let version = take::<u8>(i)?;
        let height = f32::from_bits(take::<u32>(i)?.to_le());
        let _zero = take::<u8>(i)?;
        let ground_theme = (take::<u8>(i)?, take::<u8>(i)?, take::<u8>(i)?);
        let ground_theme_strength = (take::<u8>(i)?, take::<u8>(i)?);
        let walkable = take::<u8>(i)? != 0;
        let passover = take::<u8>(i)? != 0;
        let sound_theme = take::<u8>(i)?;
        let _zero = take::<u8>(i)?;
        let shore = take::<u8>(i)? != 0;
        let _unknown = take::<u8>(i)?;

        let consumed = start_len - i.len();
        let remaining = (size as usize)
            .checked_sub(consumed)
            .ok_or(LevError::CellSizeUnderflow { size, consumed })?;
        take_bytes(i, remaining)?;

        Ok(LevHeightCell {
            size,
            version,
            height,
            ground_theme,
            ground_theme_strength,
            walkable,
            passover,
            sound_theme,
            shore,
        })
    }
}

/// A single soundmap cell. Cells run consecutively from the end of the heightmap up to the
/// navigation graph at `header.navigation_offset`.
#[derive(Debug, Clone, PartialEq)]
pub struct LevSoundCell {
    pub size: u32,
    pub version: u8,
    pub sound_theme: (u8, u8, u8),
    pub sound_theme_strength: (u8, u8),
    pub sound_index: u8,
}

impl LevSoundCell {
    pub fn parse(i: &mut &[u8]) -> Result<LevSoundCell, LevError> {
        let start_len = i.len();
        let size = take::<u32>(i)?.to_le();
        let version = take::<u8>(i)?;
        let sound_theme = (take::<u8>(i)?, take::<u8>(i)?, take::<u8>(i)?);
        let sound_theme_strength = (take::<u8>(i)?, take::<u8>(i)?);
        let sound_index = take::<u8>(i)?;

        let consumed = start_len - i.len();
        let remaining = (size as usize)
            .checked_sub(consumed)
            .ok_or(LevError::CellSizeUnderflow { size, consumed })?;
        take_bytes(i, remaining)?;

        Ok(LevSoundCell {
            size,
            version,
            sound_theme,
            sound_theme_strength,
            sound_index,
        })
    }
}

/// The navigation graph, beginning at `header.navigation_offset` and running to the end of file.
///
/// Layout: a small header listing named sections (each with an absolute start offset), followed by
/// the sections themselves. Each section is the engine's `CNavQuadTree` serialization: map
/// dimensions, action points, and one or more navigation layers whose nodes form a quad-tree.
///
/// The structure was reverse-engineered from `CNavQuadTree::LoadFromFile` in the Fable decompile
/// and verified against retail data (every section's nodes consume exactly up to its end offset).
#[derive(Debug, Clone, PartialEq)]
pub struct LevNavigation {
    pub sections_start: u32,
    pub section_names: Vec<LevNavigationName>,
    pub sections: Vec<LevNavigationSection>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LevNavigationName {
    pub name: String,
    pub start: u32,
}

impl LevNavigation {
    pub fn parse(i: &mut &[u8]) -> Result<LevNavigation, LevError> {
        let sections_start = take::<u32>(i)?.to_le();
        let sections_count = take::<u32>(i)?.to_le();

        let mut section_names = Vec::with_capacity(sections_count as usize);
        for _ in 0..sections_count {
            let len = take::<u32>(i)?.to_le() as usize;
            let name_bytes = take_bytes(i, len)?;
            let name = str::from_utf8(name_bytes)
                .map_err(|_| LevError::NavSectionNameUtf8)?
                .to_owned();
            let start = take::<u32>(i)?.to_le();
            section_names.push(LevNavigationName { name, start });
        }

        let mut sections = Vec::with_capacity(sections_count as usize);
        for _ in 0..sections_count {
            sections.push(LevNavigationSection::parse(i)?);
        }

        Ok(LevNavigation {
            sections_start,
            section_names,
            sections,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LevNavigationSection {
    /// Absolute offset of the end of this section.
    pub end_offset: u32,
    pub version: u32,
    pub map_width: f32,
    pub map_height: f32,
    /// Read as `local_34` in the engine, which only tests it for non-zero; exact meaning unclear.
    pub region_flag: u32,
    pub action_points: Vec<LevActionPoint>,
    /// Number of navigation layers the nodes are distributed across.
    pub layer_count: u32,
    /// The flat master list of quad-tree nodes, referenced internally by id.
    pub nodes: Vec<LevNavigationNode>,
}

impl LevNavigationSection {
    fn parse(i: &mut &[u8]) -> Result<LevNavigationSection, LevError> {
        let end_offset = take::<u32>(i)?.to_le();
        let version = take::<u32>(i)?.to_le();
        let map_width = f32::from_bits(take::<u32>(i)?.to_le());
        let map_height = f32::from_bits(take::<u32>(i)?.to_le());
        let region_flag = take::<u32>(i)?.to_le();

        let action_point_count = take::<u32>(i)?.to_le();
        let mut action_points = Vec::with_capacity(action_point_count as usize);
        for _ in 0..action_point_count {
            action_points.push(LevActionPoint::parse(i)?);
        }

        let layer_count = take::<u32>(i)?.to_le();
        let node_count = take::<u32>(i)?.to_le();
        let mut nodes = Vec::with_capacity(node_count as usize);
        for _ in 0..node_count {
            nodes.push(LevNavigationNode::parse(i, version)?);
        }

        // A trailing word follows the node list (read after the nodes in LoadFromFile).
        let _trailer = take::<u32>(i)?.to_le();

        Ok(LevNavigationSection {
            end_offset,
            version,
            map_width,
            map_height,
            region_flag,
            action_points,
            layer_count,
            nodes,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LevActionPoint {
    pub x: f32,
    pub y: f32,
    pub subset: u32,
}

impl LevActionPoint {
    pub fn parse(i: &mut &[u8]) -> Result<LevActionPoint, LevError> {
        let x = f32::from_bits(take::<u32>(i)?.to_le());
        let y = f32::from_bits(take::<u32>(i)?.to_le());
        let subset = take::<u32>(i)?.to_le();
        Ok(LevActionPoint { x, y, subset })
    }
}

/// A node in a navigation layer's quad-tree.
///
/// Nodes are stored in a flat list and reference each other (children, neighbours) by `id`. Every
/// node carries an `in_layer` flag controlling its placement into a layer's node array.
#[derive(Debug, Clone, PartialEq)]
pub enum LevNavigationNode {
    /// An impassable node. Stored as the shared blocked sentinel, so it has no geometry.
    Blocked { in_layer: bool },
    /// An interior node subdividing into four children (top-left, top-right, bottom-left,
    /// bottom-right), each referenced by node `id`.
    Interior {
        in_layer: bool,
        header: LevNodeHeader,
        children: [u32; 4],
    },
    /// A walkable leaf node.
    Navigable {
        in_layer: bool,
        header: LevNodeHeader,
        leaf: LevNavigationLeaf,
    },
    /// A leaf node that is only walkable under certain conditions, keyed by `switch_keys`.
    Switchable {
        in_layer: bool,
        header: LevNodeHeader,
        leaf: LevNavigationLeaf,
        switch_keys: Vec<u64>,
    },
}

impl LevNavigationNode {
    pub fn parse(i: &mut &[u8], version: u32) -> Result<LevNavigationNode, LevError> {
        // Version 6+ prefixes each node with a "switchable" flag.
        let switchable = if version > 5 {
            take::<u8>(i)? != 0
        } else {
            false
        };
        let in_layer = take::<u8>(i)? != 0;
        let blocked = take::<u8>(i)? != 0;

        // A non-switchable blocked node is the shared sentinel and stores nothing further.
        if !switchable && blocked {
            return Ok(LevNavigationNode::Blocked { in_layer });
        }

        let leaf = take::<u8>(i)? != 0;
        let header = LevNodeHeader::parse(i)?;

        if !leaf {
            let children = [
                take::<u32>(i)?.to_le(),
                take::<u32>(i)?.to_le(),
                take::<u32>(i)?.to_le(),
                take::<u32>(i)?.to_le(),
            ];
            return Ok(LevNavigationNode::Interior {
                in_layer,
                header,
                children,
            });
        }

        let leaf_data = LevNavigationLeaf::parse(i)?;

        if !switchable {
            Ok(LevNavigationNode::Navigable {
                in_layer,
                header,
                leaf: leaf_data,
            })
        } else {
            let switch_count = take::<u32>(i)?.to_le();
            let mut switch_keys = Vec::with_capacity(switch_count as usize);
            for _ in 0..switch_count {
                switch_keys.push(take::<u64>(i)?.to_le());
            }
            Ok(LevNavigationNode::Switchable {
                in_layer,
                header,
                leaf: leaf_data,
                switch_keys,
            })
        }
    }
}

/// Geometry shared by every non-blocked navigation node.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LevNodeHeader {
    /// Quad-tree subdivision level (0 = largest).
    pub quad_tree_level: u8,
    pub nav_layer: u8,
    /// Node centre, relative to the map position.
    pub position: (f32, f32),
    /// Identifier referenced by parents' `children` and leaves' `neighbours`.
    pub id: u32,
}

impl LevNodeHeader {
    pub fn parse(i: &mut &[u8]) -> Result<LevNodeHeader, LevError> {
        let quad_tree_level = take::<u8>(i)?;
        let nav_layer = take::<u8>(i)?;
        let position = (
            f32::from_bits(take::<u32>(i)?.to_le()),
            f32::from_bits(take::<u32>(i)?.to_le()),
        );
        let id = take::<u32>(i)?.to_le();
        Ok(LevNodeHeader {
            quad_tree_level,
            nav_layer,
            position,
            id,
        })
    }
}

/// Region membership and adjacency for a leaf node.
#[derive(Debug, Clone, PartialEq)]
pub struct LevNavigationLeaf {
    /// Index of the navigation region this leaf belongs to.
    pub region_id: u32,
    /// Path-finding cost preference for this leaf.
    pub preferability: u8,
    /// Ids of neighbouring leaf nodes.
    pub neighbours: Vec<u32>,
}

impl LevNavigationLeaf {
    pub fn parse(i: &mut &[u8]) -> Result<LevNavigationLeaf, LevError> {
        let region_id = take::<u32>(i)?.to_le();
        let preferability = take::<u8>(i)?;
        let neighbour_count = take::<u32>(i)?.to_le();
        let mut neighbours = Vec::with_capacity(neighbour_count as usize);
        for _ in 0..neighbour_count {
            neighbours.push(take::<u32>(i)?.to_le());
        }
        Ok(LevNavigationLeaf {
            region_id,
            preferability,
            neighbours,
        })
    }
}
