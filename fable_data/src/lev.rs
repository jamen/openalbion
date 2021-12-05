use crate::Bytes;

use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, PartialEq)]
pub struct Lev {
    pub version: u16,
    pub obsolete_offset: u32,
    pub navigation_start: u32,
    pub unique_id_count: u64,
    pub width: u32,
    pub height: u32,
    pub map_version: u32,
    pub heightmap_palette: Vec<u8>,
    pub ambient_sound_version: u32,
    pub sound_palette: Vec<u8>,
    pub checksum: u32,
    pub sound_themes: Vec<String>,
    pub heightmap: Vec<LevHeightCell>,
    pub soundmap: Vec<LevSoundCell>,
    pub unknown_1: u32,
    pub unknown_2: u8,
    pub navigation: LevNavigation,
}

#[derive(Debug, PartialEq)]
pub struct LevHeightCell {
    pub size: u32,
    pub version: u8,
    pub height: f32,
    pub ground_theme: (u8, u8, u8),
    pub ground_theme_strength: (u8, u8),
    pub walkable: bool,
    pub passover: bool,
    pub sound_theme: u8,
    pub shore: bool,
}

#[derive(Debug, PartialEq)]
pub struct LevSoundCell {
    pub size: u32,
    pub version: u8,
    pub sound_theme: (u8, u8, u8),
    pub sound_theme_strength: (u8, u8),
    pub sound_index: u8,
}

// From fabletlcmod.com:
//
// A Subset has 7 Layers (0-6), each defining blocks of walkable area.
// Layer 0 = 32 X 32
// Layer 1 = 16 X 16
// Layer 2 = 8 X 8
// Layer 3 = 4 X 4
// Layer 4 = 2 X 2
// Layer 5 = 1 X 1
// Layer 6 = 0.5 X 0.5
//

#[derive(Debug, PartialEq)]
pub struct LevNavigation {
    pub sections_start: u32,
    pub sections_count: u32,
    pub sections: Vec<LevNavigationSection>,
}

#[derive(Debug, PartialEq)]
pub struct LevNavigationSection {
    pub name: String,
    pub start: u32,
    pub size: u32,
    pub version: u32,
    pub width: f32,
    pub height: f32,
    pub unknown_1: u32,
    pub interactive_nodes: Vec<LevInteractiveNode>,
    pub subsets_count: u32,
    pub nodes: Vec<LevLevelNode>,
}

#[derive(Debug, PartialEq)]
pub struct LevInteractiveNode {
    pub x: f32,
    pub y: f32,
    pub subset: u32,
}

#[derive(Debug, PartialEq)]
pub enum LevLevelNode {
    Short {
        unknown_1: bool,
        unknown_2: bool,
        unknown_3: bool,
    },
    Regular {
        unknown_1: bool,
        unknown_2: bool,
        unknown_3: bool,
        unknown_4: bool,
        subset: u16,
        x: f32,
        y: f32,
        id: u32,
        children: (u32, u32, u32, u32),
    },
    Extended1 {
        unknown_1: bool,
        unknown_2: bool,
        unknown_3: bool,
        unknown_4: bool,
        subset: u16,
        x: f32,
        y: f32,
        id: u32,
        unknown_5: u32,
        unknown_6: u8,
        children: Vec<u32>,
    },
    Extended2 {
        unknown_1: bool,
        unknown_2: bool,
        unknown_3: bool,
        unknown_4: bool,
        subset: u16,
        x: f32,
        y: f32,
        id: u32,
        unknown_5: u32,
        unknown_6: u8,
        children_1: Vec<u32>,
        children_2: Vec<u64>,
    },
}

impl Lev {
    pub fn decode(data: &[u8]) -> Option<Lev> {
        let mut map_data = data.clone();

        let _header_size = map_data.parse_u32_le()?;
        let version = map_data.parse_u16_le()?;
        let _unknown_1 = map_data.advance(3usize)?; // fabletlcmod.com: 3 bytes of padding? see checksum.
        let _unknown_2 = map_data.parse_u32_le()?;
        let obsolete_offset = map_data.parse_u32_le()?;
        let _unknown_3 = map_data.parse_u32_le()?;
        let navigation_start = map_data.parse_u32_le()?;
        let _map_header_size = map_data.parse_u8()?;
        let map_version = map_data.parse_u32_le()?; // fabletlcmod.com:  An 8 bit integer (with 3 bytes of padding)?
        let unique_id_count = map_data.parse_u64_le()?;
        let width = map_data.parse_u32_le()?;
        let height = map_data.parse_u32_le()?;
        let _always_true = map_data.parse_u8()?;

        let heightmap_palette = map_data.advance(33792usize)?.to_owned(); // TODO: figure this out?
        let ambient_sound_version = map_data.parse_u32_le()?;
        let sound_themes_count = map_data.parse_u32_le()?;
        let sound_palette = map_data.advance(33792usize)?.to_owned(); // TODO: figure this out?
        let checksum = map_data.parse_u32_le()?; // fabletlcmod.com: only if the map header pad byte 2 is 9?

        let sound_themes = Self::decode_sound_themes(
            &mut map_data,
            sound_themes_count.saturating_sub(1) as usize,
        )?;

        let height_cell_count = ((width + 1) * (height + 1)) as usize;
        let heightmap = Self::decode_heightmap(&mut map_data, height_cell_count)?;

        let sound_cell_count = ((height / 4) * (width / 4)) as usize;
        let soundmap = Self::decode_soundmap(&mut map_data, sound_cell_count)?;

        let unknown_1 = map_data.parse_u32_le()?;
        let unknown_2 = map_data.parse_u8()?;

        let mut nav_data = &data[..];

        let navigation = Self::decode_navigation(&mut nav_data, navigation_start as usize)?;

        Some(Lev {
            version: version,
            obsolete_offset: obsolete_offset,
            navigation_start: navigation_start,
            unique_id_count: unique_id_count,
            width: width,
            height: height,
            map_version: map_version,
            heightmap_palette,
            ambient_sound_version: ambient_sound_version,
            sound_palette,
            checksum: checksum,
            sound_themes: sound_themes,
            heightmap,
            soundmap,
            unknown_1,
            unknown_2,
            navigation,
        })
    }

    fn decode_sound_themes(data: &mut &[u8], sound_themes_count: usize) -> Option<Vec<String>> {
        let mut sound_themes = Vec::new();
        while sound_themes.len() < sound_themes_count {
            sound_themes.push(data.parse_str_with_u32_le_prefix()?.to_owned())
        }
        Some(sound_themes)
    }

    fn decode_heightmap(data: &mut &[u8], height_cell_count: usize) -> Option<Vec<LevHeightCell>> {
        let mut heightmap = Vec::new();

        while heightmap.len() < height_cell_count {
            let size = data.parse_u32_le()?;
            let version = data.parse_u8()?;
            let height = data.parse_f32_le()?;
            let _zero = data.parse_u8()?;
            let ground_theme = (data.parse_u8()?, data.parse_u8()?, data.parse_u8()?);
            let ground_theme_strength = (data.parse_u8()?, data.parse_u8()?);
            let walkable = data.parse_u8()?;
            let passover = data.parse_u8()?;
            let sound_theme = data.parse_u8()?;
            let _zero = data.parse_u8()?;
            let shore = data.parse_u8()?;
            let _unknown = data.parse_u8()?;

            heightmap.push(LevHeightCell {
                size: size,
                version: version,
                height: height,
                ground_theme: ground_theme,
                ground_theme_strength: ground_theme_strength,
                walkable: walkable != 0,
                passover: passover != 0,
                sound_theme: sound_theme,
                shore: shore != 0,
            });
        }

        Some(heightmap)
    }

    fn decode_soundmap(data: &mut &[u8], sound_cell_count: usize) -> Option<Vec<LevSoundCell>> {
        let mut soundmap = Vec::new();

        while soundmap.len() < sound_cell_count {
            let size = data.parse_u32_le()?;
            let version = data.parse_u8()?;
            let sound_theme = (data.parse_u8()?, data.parse_u8()?, data.parse_u8()?);
            let sound_theme_strength = (data.parse_u8()?, data.parse_u8()?);
            let sound_index = data.parse_u8()?;

            soundmap.push(LevSoundCell {
                size: size,
                version: version,
                sound_theme: sound_theme,
                sound_theme_strength: sound_theme_strength,
                sound_index: sound_index,
            });
        }

        Some(soundmap)
    }

    fn decode_navigation(data: &mut &[u8], navigation_start: usize) -> Option<LevNavigation> {
        let header_data = &mut data.get(navigation_start..)?;

        let sections_start = header_data.parse_u32_le()?;
        let sections_count = header_data.parse_u32_le()?;

        let mut sections = Vec::new();

        // println!("sections count {:?}", sections_count);

        while sections.len() < sections_count as usize {
            let name = header_data.parse_str_with_u32_le_prefix()?.to_owned();
            let start = header_data.parse_u32_le()?;

            // println!("{:?} {:?}", name, start);

            let mut section_data = &mut data.get(start as usize..)?;

            let size = section_data.parse_u32_le()?;
            let version = section_data.parse_u32_le()?;
            let width = section_data.parse_f32_le()?;
            let height = section_data.parse_f32_le()?;
            let unknown_1 = section_data.parse_u32_le()?; // fabletlcmod.com: Number of Levels, see navigation nodes
            let interactive_nodes = Self::decode_interactive_nodes(&mut section_data)?;
            let subsets_count = section_data.parse_u32_le()?;
            let nodes = Self::decode_nodes(&mut section_data)?;

            sections.push(LevNavigationSection {
                name,
                start,
                size,
                version,
                width,
                height,
                unknown_1,
                interactive_nodes,
                subsets_count,
                nodes,
            });
        }

        Some(LevNavigation {
            sections_start,
            sections_count,
            sections,
        })
    }

    fn decode_interactive_nodes(data: &mut &[u8]) -> Option<Vec<LevInteractiveNode>> {
        let nodes_count = data.parse_u32_le()?;
        let mut interactive_nodes = Vec::new();

        while interactive_nodes.len() < nodes_count as usize {
            let x = data.parse_f32_le()?;
            let y = data.parse_f32_le()?;
            let subset = data.parse_u32_le()?;
            interactive_nodes.push(LevInteractiveNode { x, y, subset });
        }

        Some(interactive_nodes)
    }

    fn decode_nodes(mut data: &mut &[u8]) -> Option<Vec<LevLevelNode>> {
        let nodes_count = data.parse_u32_le()?;
        let mut nodes = Vec::new();

        while nodes.len() < nodes_count as usize {
            nodes.push(Self::decode_node(&mut data)?);
        }

        Some(nodes)
    }

    fn decode_node(data: &mut &[u8]) -> Option<LevLevelNode> {
        let unknown_1 = data.parse_u8()? == 1;
        let unknown_2 = data.parse_u8()? == 1;
        let unknown_3 = data.parse_u8()? == 1;

        if (unknown_1, unknown_2, unknown_3) == (false, true, true) {
            return Some(LevLevelNode::Short {
                unknown_1,
                unknown_2,
                unknown_3,
            });
        }

        let unknown_4 = data.parse_u8()? == 1;
        let subset = data.parse_u16_le()?;
        let x = data.parse_f32_le()?;
        let y = data.parse_f32_le()?;
        let id = data.parse_u32_le()?;

        match (unknown_1, unknown_2, unknown_3, unknown_4) {
            (false, _, _, false) => {
                let children = (
                    data.parse_u32_le()?,
                    data.parse_u32_le()?,
                    data.parse_u32_le()?,
                    data.parse_u32_le()?,
                );

                Some(LevLevelNode::Regular {
                    unknown_1,
                    unknown_2,
                    unknown_3,
                    unknown_4,
                    subset,
                    x,
                    y,
                    id,
                    children,
                })
            }
            (false, _, _, true) => {
                let unknown_5 = data.parse_u32_le()?;
                let unknown_6 = data.parse_u8()?;

                let children_count = data.parse_u32_le()? as usize;
                let mut children = Vec::new();

                while children_count > children.len() {
                    children.push(data.parse_u32_le()?);
                }

                Some(LevLevelNode::Extended1 {
                    unknown_1,
                    unknown_2,
                    unknown_3,
                    unknown_4,
                    subset,
                    x,
                    y,
                    id,
                    unknown_5,
                    unknown_6,
                    children,
                })
            }
            (true, _, _, true) => {
                let unknown_5 = data.parse_u32_le()?;
                let unknown_6 = data.parse_u8()?;

                let children_1_count = data.parse_u32_le()? as usize;
                let mut children_1 = Vec::new();

                while children_1_count > children_1.len() {
                    children_1.push(data.parse_u32_le()?);
                }

                let children_2_count = data.parse_u32_le()? as usize;
                let mut children_2 = Vec::new();

                while children_2_count > children_2.len() {
                    children_2.push(data.parse_u64_le()?);
                }

                Some(LevLevelNode::Extended2 {
                    unknown_1,
                    unknown_2,
                    unknown_3,
                    unknown_4,
                    subset,
                    x,
                    y,
                    id,
                    unknown_5,
                    unknown_6,
                    children_1,
                    children_2,
                })
            }
            flags => {
                // eprintln!("unhandled node with flags {:?}", flags);
                None
            }
        }
    }
}
