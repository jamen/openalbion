mod compile;
mod parse;

pub use compile::*;
pub use parse::*;

/// Level cells and nodes.
///
/// ## Format
///
/// WIP
#[derive(Debug, PartialEq)]
pub struct Lev {
    pub header: LevHeader,
    // pub heightmap_cells: Vec<LevHeightCell>,
    // pub soundmap_cells: Vec<LevSoundCell>,
    // pub navigation_header: LevNavigationHeader,
    // pub navigation_section: LevNavigationSection
}

#[derive(Debug, PartialEq)]
pub struct LevHeader {
    pub version: u16,
    pub obsolete_offset: u32,
    pub navigation_offset: u32,
    pub unique_id_count: u64,
    pub width: u32,
    pub height: u32,
    pub map_version: u32,
    // pub heightmap_palette: &'a [u8],
    pub ambient_sound_version: u32,
    // pub sound_palette: &'a [u8],
    pub checksum: u32,
    pub sound_themes: Vec<String>,
}

// #[derive(Debug, PartialEq)]
// pub struct LevHeightCell {
//     pub size: u32,
//     pub version: u8,
//     pub height: f32,
//     pub ground_theme: (u8, u8, u8),
//     pub ground_theme_strength: (u8, u8),
//     pub walkable: bool,
//     pub passover: bool,
//     pub sound_theme: u8,
//     pub shore: bool,
// }

// #[derive(Debug, PartialEq)]
// pub struct LevSoundCell {
//     pub size: u32,
//     pub version: u8,
//     pub sound_theme: (u8, u8, u8),
//     pub sound_theme_strength: (u8, u8),
//     pub sound_index: u8,
// }

// #[derive(Debug, PartialEq)]
// pub struct LevNavigationHeader {
//     pub sections_start: u32,
//     pub sections_count: u32,
//     pub sections: Vec<(String, u32)>,
// }

// //
// // From fabletlcmod.com:
// //
// // A Subset has 7 Layers (0-6), each defining blocks of walkable area.
// // Layer 0 = 32 X 32
// // Layer 1 = 16 X 16
// // Layer 2 = 8 X 8
// // Layer 3 = 4 X 4
// // Layer 4 = 2 X 2
// // Layer 5 = 1 X 1
// // Layer 6 = 0.5 X 0.5
// //

// #[derive(Debug, PartialEq)]
// pub struct LevNavigationSection {
//     pub size: u32,
//     pub version: u32,
//     pub level_width: u32,
//     pub level_height: u32,
//     pub interactive_nodes: Vec<LevInteractiveNode>,
//     pub subsets_count: u32,
//     pub level_nodes: Vec<LevNavigationNode>,
// }

// #[derive(Debug, PartialEq)]
// pub struct LevInteractiveNode {
//     pub x: u32,
//     pub y: u32,
//     pub subset: u32,
// }

// #[derive(Debug, PartialEq)]
// pub enum LevNavigationNode {
//     Regular(LevNavigationRegularNode),
//     Navigation(LevNavigationNavigationNode),
//     Exit(LevNavigationExitNode),
//     Blank(LevNavigationBlankNode),
//     Unknown1(LevNavigationUnknownNode1),
//     Unknown2(LevNavigationUnknownNode2),
//     Unknown3(LevNavigationUnknownNode3),
//     Unknown4(LevNavigationUnknownNode4),
// }

// #[derive(Debug, PartialEq)]
// pub struct LevNavigationRegularNode {
//     pub root: u8,
//     pub end: u8,
//     pub layer: u8,
//     pub subset: u8,
//     pub x: f32,
//     pub y: f32,
//     pub node_id: u32,
//     pub child_nodes: (u32, u32, u32, u32), // (top_right, top_left, bottom_right, bottom_left)
// }

// #[derive(Debug, PartialEq)]
// pub struct LevNavigationNavigationNode {
//     pub root: u8,
//     pub end: u8,
//     pub layer: u8,
//     pub subset: u8,
//     pub x: f32,
//     pub y: f32,
//     pub node_id: u32,
//     pub node_level: u32,
//     pub nodes: Vec<u32>,
// }

// #[derive(Debug, PartialEq)]
// pub struct LevNavigationExitNode {
//     pub root: u8,
//     pub end: u8,
//     pub layer: u8,
//     pub subset: u8,
//     pub x: f32,
//     pub y: f32,
//     pub node_id: u32,
//     pub node_level: u32,
//     pub nodes: Vec<u32>,
//     pub uids: Vec<u64>,
// }

// #[derive(Debug, PartialEq)]
// pub struct LevNavigationUnknownNode1 {
//     pub node_op: Vec<u8>,
//     pub end: u8,
// }

// #[derive(Debug, PartialEq)]
// pub struct LevNavigationUnknownNode2 {
//     pub end: u8,
// }

// #[derive(Debug, PartialEq)]
// pub struct LevNavigationUnknownNode3 {
//     pub end: u8,
// }

// #[derive(Debug, PartialEq)]
// pub struct LevNavigationUnknownNode4 {
//     pub end: u8,
// }

// #[derive(Debug, PartialEq)]
// pub struct LevNavigationBlankNode {
//     pub root: u8,
// }
