use std::io::{Read,Seek,SeekFrom};

use views::{Look,View,Bytes,BadPos};

use crate::BytesExt;

/// Level cells and nodes.
///
/// # Format Description
///
/// WIP
#[derive(Debug,PartialEq)]
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

#[derive(Debug,PartialEq)]
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

#[derive(Debug,PartialEq)]
pub struct LevSoundCell {
    pub size: u32,
    pub version: u8,
    pub sound_theme: (u8, u8, u8),
    pub sound_theme_strength: (u8, u8),
    pub sound_index: u8,
}

//
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

#[derive(Debug,PartialEq)]
pub struct LevNavigation {
    pub sections_start: u32,
    pub sections_count: u32,
    pub sections: Vec<LevNavigationSection>,
}

#[derive(Debug,PartialEq)]
pub struct LevNavigationSection {
    pub name: String,
    pub start: u32,
    pub size: u32,
    pub version: u32,
    pub level_width: f32,
    pub level_height: f32,
    pub unknown_1: u32,
    pub interactive_nodes: Vec<LevInteractiveNode>,
    pub subsets_count: u32,
    pub level_nodes: Vec<LevLevelNode>,
}

#[derive(Debug,PartialEq)]
pub struct LevInteractiveNode {
    pub x: f32,
    pub y: f32,
    pub subset: u32,
}

#[derive(Debug,PartialEq)]
pub enum LevLevelNode {
    Regular {
        prefix: Vec<u8>,
        unknown_1: u8,
        root: u8,
        unknown_2: u8,
        end: u8,
        layer: u8,
        subset: u8,
        x: f32,
        y: f32,
        id: u32,
        // (top_right, top_left, bottom_right, bottom_left)
        children: (u32, u32, u32, u32),
    },
    Navigation {
        prefix: Vec<u8>,
        unknown_1: u8,
        root: u8,
        unknown_2: u8,
        end: u8,
        layer: u8,
        subset: u8,
        x: f32,
        y: f32,
        id: u32,
        level: u32,
        unknown_3: u8,
        adjacent: Vec<u32>,
    },
    Exit {
        prefix: Vec<u8>,
        unknown_1: u8,
        root: u8,
        unknown_2: u8,
        end: u8,
        layer: u8,
        subset: u8,
        x: f32,
        y: f32,
        id: u32,
        level: u32,
        unknown_3: u8,
        adjacent: Vec<u32>,
        uids: Vec<u64>,
    },
    Blank {
        prefix: Vec<u8>,
        unknown_1: u8,
        root: u8,
        unknown_2: u8,
    },
    Unknown1 {
        prefix: Vec<u8>,
        unknown_1: f32,
        unknown_2: f32,
    }
    // Unknown1 {
    //     prefix: Vec<u8>,
    //     unknown_1: u8,
    //     root: u8,
    //     unknown_2: u8,
    //     end: u8,
    // },
    // Unknown2 {
    //     prefix: Vec<u8>,
    //     unknown_1: u8,
    //     root: u8,
    //     unknown_2: u8,
    //     end: u8,
    // },
    // Unknown3 {
    //     prefix: Vec<u8>,
    //     unknown_1: u8,
    //     root: u8,
    //     unknown_2: u8,
    //     end: u8,
    // },
    // Unknown4 {
    //     prefix: Vec<u8>,
    //     unknown_1: u8,
    //     root: u8,
    //     unknown_2: u8,
    //     end: u8,
    // },
    // UnknownOther {
    //     prefix: Vec<u8>,
    //     unknown_1: u8,
    //     root: u8,
    //     unknown_2: u8,
    //     end: u8,
    // },
}

impl Lev {
    pub fn decode<T: Read + Seek>(source: &mut T) -> Result<Lev, BadPos> {
        let mut data = Vec::new();
        source.read_to_end(&mut data).or(Err(BadPos))?;
        let mut map_data = &data[..];

        let _header_size = map_data.take_u32_le()?;
        let version = map_data.take_u16_le()?;
        let _unknown_1 = Bytes::take(&mut map_data, 3usize)?; // fabletlcmod.com: 3 bytes of padding? see checksum.
        let _unknown_2 = map_data.take_u32_le()?;
        let obsolete_offset = map_data.take_u32_le()?;
        let _unknown_3 = map_data.take_u32_le()?;
        let navigation_start = map_data.take_u32_le()?;
        let _map_header_size = map_data.take_u8()?;
        let map_version = map_data.take_u32_le()?; // fabletlcmod.com:  An 8 bit integer (with 3 bytes of padding)?
        let unique_id_count = map_data.take_u64_le()?;
        let width = map_data.take_u32_le()?;
        let height = map_data.take_u32_le()?;
        let _always_true = map_data.take_u8()?;

        let heightmap_palette = Bytes::take(&mut map_data, 33792usize)?.to_owned(); // TODO: figure this out?
        let ambient_sound_version = map_data.take_u32_le()?;
        let sound_themes_count = map_data.take_u32_le()?;
        let sound_palette = Bytes::take(&mut map_data, 33792usize)?.to_owned(); // TODO: figure this out?
        let checksum = map_data.take_u32_le()?; // fabletlcmod.com: only if the map header pad byte 2 is 9?

        let sound_themes = Self::decode_sound_themes(&mut map_data, sound_themes_count.saturating_sub(1) as usize)?;

        let height_cell_count = ((width + 1) * (height + 1)) as usize;
        let heightmap = Self::decode_heightmap(&mut map_data, height_cell_count)?;

        let sound_cell_count = ((height / 4) * (width / 4)) as usize;
        let soundmap = Self::decode_soundmap(&mut map_data, sound_cell_count)?;

        let unknown_1 = map_data.take_u32_le()?;
        let unknown_2 = map_data.take_u8()?;

        let mut nav_data = &data[..];

        let navigation = Self::decode_navigation(&mut nav_data, navigation_start as usize)?;

        Ok(Lev {
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

    fn decode_sound_themes(data: &mut &[u8], sound_themes_count: usize) -> Result<Vec<String>, BadPos> {
        let mut sound_themes = Vec::new();
        while sound_themes.len() < sound_themes_count {
            sound_themes.push(data.take_as_str_with_u32_le_prefix()?.to_owned())
        }
        Ok(sound_themes)
    }

    fn decode_heightmap(data: &mut &[u8], height_cell_count: usize) -> Result<Vec<LevHeightCell>, BadPos> {
        let mut heightmap = Vec::new();

        while heightmap.len() < height_cell_count {
            let size = data.take_u32_le()?;
            let version = data.take_u8()?;
            let height = data.take_f32_le()?;
            let _zero = data.take_u8()?;
            let ground_theme = (data.take_u8()?, data.take_u8()?, data.take_u8()?);
            let ground_theme_strength = (data.take_u8()?, data.take_u8()?);
            let walkable = data.take_u8()?;
            let passover = data.take_u8()?;
            let sound_theme = data.take_u8()?;
            let _zero = data.take_u8()?;
            let shore = data.take_u8()?;
            let _unknown = data.take_u8()?;

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

        Ok(heightmap)
    }

    fn decode_soundmap(data: &mut &[u8], sound_cell_count: usize) -> Result<Vec<LevSoundCell>, BadPos> {
        let mut soundmap = Vec::new();

        while soundmap.len() < sound_cell_count {
            let size = data.take_u32_le()?;
            let version = data.take_u8()?;
            let sound_theme = (data.take_u8()?, data.take_u8()?, data.take_u8()?);
            let sound_theme_strength = (data.take_u8()?, data.take_u8()?);
            let sound_index = data.take_u8()?;

            soundmap.push(LevSoundCell {
                size: size,
                version: version,
                sound_theme: sound_theme,
                sound_theme_strength: sound_theme_strength,
                sound_index: sound_index,
            });
        }

        Ok(soundmap)
    }

    fn decode_navigation(mut data: &mut &[u8], navigation_start: usize) -> Result<LevNavigation, BadPos> {
        let header_data = &mut data.get(navigation_start..).ok_or(BadPos)?;

        let sections_start = header_data.take_u32_le()?;
        let sections_count = header_data.take_u32_le()?;

        let mut sections = Vec::new();

        // println!("sections count {:?}", sections_count);

        while sections.len() < sections_count as usize {
            let name = header_data.take_as_str_with_u32_le_prefix()?.to_owned();
            let start = header_data.take_u32_le()?;

            // println!("{:?} {:?}", name, start);

            let mut section_data = &mut data.get(start as usize ..).ok_or(BadPos)?;

            let size = section_data.take_u32_le()?;
            let version = section_data.take_u32_le()?;
            let level_width = section_data.take_f32_le()?;
            let level_height = section_data.take_f32_le()?;
            let unknown_1 = section_data.take_u32_le()?; // fabletlcmod.com: Number of levels, see navigation nodes
            let interactive_nodes = Self::decode_interactive_nodes(&mut section_data)?;
            let subsets_count = section_data.take_u32_le()?;
            let level_nodes = Self::decode_level_nodes(&mut section_data)?;

            sections.push(LevNavigationSection {
                name,
                start,
                size,
                version,
                level_width,
                level_height,
                unknown_1,
                interactive_nodes,
                subsets_count,
                level_nodes,
            });
        }

        Ok(LevNavigation {
            sections_start,
            sections_count,
            sections,
        })
    }

    fn decode_interactive_nodes(data: &mut &[u8]) -> Result<Vec<LevInteractiveNode>, BadPos> {
        let nodes_count = data.take_u32_le()?;
        let mut interactive_nodes = Vec::new();

        while interactive_nodes.len() < nodes_count as usize {
            let x = data.take_f32_le()?;
            let y = data.take_f32_le()?;
            let subset = data.take_u32_le()?;
            interactive_nodes.push(LevInteractiveNode { x, y, subset });
        }

        Ok(interactive_nodes)
    }

    fn decode_level_nodes(mut data: &mut &[u8]) -> Result<Vec<LevLevelNode>, BadPos> {
        let nodes_count = data.take_u32_le()?;
        let mut level_nodes = Vec::new();

        while level_nodes.len() < nodes_count as usize {
            level_nodes.push(Self::decode_level_node(&mut data)?);
        }

        Ok(level_nodes)
    }

    fn decode_level_node(mut data: &mut &[u8]) -> Result<LevLevelNode, BadPos> {
        if data.get(..8) == Some(&[0,0,0,0,0,1,0,0]) {
            Self::decode_regular_node(data)
        }
        else if data.get(..8) == Some(&[0,0,0,1,0,1,0,1]) {
            Self::decode_navigation_node(data)
        }
        else if data.get(..8) == Some(&[1,0,0,1,1,0,1,1]) {
            Self::decode_exit_node(data)
        }
        else if data.get(..3) == Some(&[0,1,1]) {
            Self::decode_blank_node(data)
        }
        // else if data.get(..10) == Some(&[11,0,0,0,0,0,0,0,0,0]) {
        //     Self::decode_unknown1_node(data)
        // }
        else if data.get(..8) == Some(&[0,1,0,0,0,0,0,0]) {
            Self::decode_unknown1_node(data)
        }
        // else if data.get(..8) == Some(&[0,1,0,0,0,0,0,0]) {
        //     Self::decode_unknown3_node(data)
        // }
        // else if data.get(..8) == Some(&[0,1,0,0,0,0,0,0]) {
        //     Self::decode_unknown4_node(data)
        // }
        else {
            return Err(BadPos)
            // Self::decode_unknown_other_node(data)
        }
    }

    fn decode_regular_node(data: &mut &[u8]) -> Result<LevLevelNode, BadPos> {
        let prefix = Bytes::take(data, 8)?.to_owned();
        let unknown_1 = data.take_u8()?;
        let root = data.take_u8()?;
        let unknown_2 = data.take_u8()?;
        let end = data.take_u8()?;
        let layer = data.take_u8()?;
        let subset = data.take_u8()?;
        let x = data.take_f32_le()?;
        let y = data.take_f32_le()?;
        let id = data.take_u32_le()?;

        let tr = data.take_u32_le()?;
        let tl = data.take_u32_le()?;
        let br = data.take_u32_le()?;
        let bl = data.take_u32_le()?;

        let children = (tr, tl, br, bl);

        Ok(LevLevelNode::Regular {
            prefix,
            unknown_1,
            root,
            unknown_2,
            end,
            layer,
            subset,
            x,
            y,
            id,
            children,
        })
    }

    fn decode_navigation_node(data: &mut &[u8]) -> Result<LevLevelNode, BadPos> {
        let prefix = Bytes::take(data, 8)?.to_owned();
        let unknown_1 = data.take_u8()?;
        let root = data.take_u8()?;
        let unknown_2 = data.take_u8()?;
        let end = data.take_u8()?;
        let layer = data.take_u8()?;
        let subset = data.take_u8()?;
        let x = data.take_f32_le()?;
        let y = data.take_f32_le()?;
        let id = data.take_u32_le()?;
        let level = data.take_u32_le()?;
        let unknown_3 = data.take_u8()?;

        let adjacent_count = data.take_u32_le()? as usize;
        let mut adjacent = Vec::new();

        while adjacent.len() < adjacent_count {
            adjacent.push(data.take_u32_le()?);
        }

        Ok(LevLevelNode::Navigation {
            prefix,
            unknown_1,
            root,
            unknown_2,
            end,
            layer,
            subset,
            x,
            y,
            id,
            level,
            unknown_3,
            adjacent,
        })
    }

    fn decode_exit_node(data: &mut &[u8]) -> Result<LevLevelNode, BadPos> {
        let prefix = Bytes::take(data, 8)?.to_owned();
        let unknown_1 = data.take_u8()?;
        let root = data.take_u8()?;
        let unknown_2 = data.take_u8()?;
        let end = data.take_u8()?;
        let layer = data.take_u8()?;
        let subset = data.take_u8()?;
        let x = data.take_f32_le()?;
        let y = data.take_f32_le()?;
        let id = data.take_u32_le()?;
        let level = data.take_u32_le()?;
        let unknown_3 = data.take_u8()?;

        let adjacent_count = data.take_u32_le()? as usize;
        let mut adjacent = Vec::new();

        while adjacent.len() < adjacent_count {
            adjacent.push(data.take_u32_le()?);
        }

        let uids_count = data.take_u32_le()? as usize;
        let mut uids = Vec::new();

        while uids.len() < uids_count {
            uids.push(data.take_u64_le()?);
        }

        Ok(LevLevelNode::Exit {
            prefix,
            unknown_1,
            root,
            unknown_2,
            end,
            layer,
            subset,
            x,
            y,
            id,
            level,
            unknown_3,
            adjacent,
            uids,
        })
    }

    fn decode_blank_node(data: &mut &[u8]) -> Result<LevLevelNode, BadPos> {
        let prefix = Bytes::take(data, 3)?.to_owned();
        let unknown_1 = data.take_u8()?;
        let root = data.take_u8()?;
        let unknown_2 = data.take_u8()?;

        Ok(LevLevelNode::Blank {
            prefix,
            unknown_1,
            root,
            unknown_2,
        })
    }

    fn decode_unknown1_node(data: &mut &[u8]) -> Result<LevLevelNode, BadPos> {
        let prefix = Bytes::take(data, 8)?.to_owned();
        let unknown_1 = data.take_f32_le()?;
        let unknown_2 = data.take_f32_le()?;
        Ok(LevLevelNode::Unknown1 {
            prefix,
            unknown_1,
            unknown_2,
        })
    }
}