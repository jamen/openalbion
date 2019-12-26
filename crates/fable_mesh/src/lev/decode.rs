use fable_base::nom::IResult;
use fable_base::nom::number::complete::{le_u8,le_u16,le_u32,le_u64,le_f32};
use fable_base::nom::bytes::complete::{tag,take};
use fable_base::nom::sequence::tuple;
use fable_base::nom::multi::count;
use fable_base::nom::branch::alt;
use fable_base::string::decode_rle_string;

use crate::lev::{
    Lev,
    LevHeader,
    LevHeightmapCell,
    LevSoundmapCell,
    LevNavigationHeader,
    LevNavigationSection,
    LevInteractiveNode,
    LevNavigationNode,
    LevNavigationRegularNode,
    LevNavigationNavigationNode,
    LevNavigationExitNode,
    LevNavigationBlankNode,
    LevNavigationUnknown1Node,
    LevNavigationUnknown2Node,
    LevNavigationUnknown3Node,
    LevNavigationUnknownNode
};

pub fn decode_lev(input: &[u8]) -> IResult<&[u8], Lev> {
    let (maybe_input, header) = decode_header(input)?;

    let heightmap_cell_count = ((header.width + 1) * (header.height + 1)) as usize;
    let (maybe_input, heightmap_cells) = count(decode_heightmap_cell, heightmap_cell_count)(maybe_input)?;

    // fabletlcmod.com seems to have the wrong amount, using a temporary one.
    // let soundmap_cell_count = ((header.height - 1) * (header.width - 1)) as usize;
    let soundmap_cell_count = 1024usize;
    let (_maybe_input, soundmap_cells) = count(decode_soundmap_cell, soundmap_cell_count)(maybe_input)?;

    let navigation_header_data = &input[header.navigation_offset as usize..];
    let (_maybe_input, navigation_header) = decode_navigation_header(navigation_header_data)?;

    let navigation_section_data = &input[navigation_header.sections_start as usize..];
    let (maybe_input, navigation_section) = decode_navigation_section(navigation_section_data)?;

    Ok(
        (
            maybe_input,
            Lev {
                header: header,
                heightmap_cells: heightmap_cells,
                soundmap_cells: soundmap_cells,
                navigation_header: navigation_header,
                navigation_section: navigation_section,
            }
        )
    )
}

pub fn decode_header(input: &[u8]) -> IResult<&[u8], LevHeader> {
    let (input, _header_size) = le_u32(input)?;
    let (input, version) = le_u16(input)?;
    let (input, _unknown_1) = take(3usize)(input)?; // fabletlcmod.com: 3 bytes of padding. see checksum.
    let (input, _unknown_2) = le_u32(input)?;
    let (input, obsolete_offset) = le_u32(input)?;
    let (input, _unknown_3) = le_u32(input)?;
    let (input, navigation_offset) = le_u32(input)?;
    let (input, _map_header_size) = le_u8(input)?;
    let (input, map_version) = le_u32(input)?; // fabletlcmod.com:  An 8 bit integer (with 3 bytes of padding)
    let (input, unique_id_count) = le_u64(input)?;
    let (input, width) = le_u32(input)?;
    let (input, height) = le_u32(input)?;
    let (input, _always_true) = le_u8(input)?;

    let (input, _heightmap_palette) = take(33792usize)(input)?; // TODO: figure this out
    let (input, ambient_sound_version) = le_u32(input)?;
    let (input, sound_themes_count) = le_u32(input)?;
    let (input, _sound_palette) = take(33792usize)(input)?; // TODO: figure this out
    let (input, checksum) = le_u32(input)?; // fabletlcmod.com: only if the map header pad byte 2 is 9.

    let (input, sound_themes) = count(decode_rle_string, (sound_themes_count - 1) as usize)(input)?;

    Ok(
        (
            input,
            LevHeader {
                version: version,
                obsolete_offset: obsolete_offset,
                navigation_offset: navigation_offset,
                unique_id_count: unique_id_count,
                width: width,
                height: height,
                map_version: map_version,
                // heightmap_palette: heightmap_palette,
                ambient_sound_version: ambient_sound_version,
                // sound_palette: sound_palette,
                checksum: checksum,
                sound_themes: sound_themes,
            }
        )
    )
}

pub fn decode_heightmap_cell(input: &[u8]) -> IResult<&[u8], LevHeightmapCell> {
    let (input, size) = le_u32(input)?;
    let (input, version) = le_u8(input)?;
    let (input, height) = le_f32(input)?;
    let (input, _zero) = le_u8(input)?;
    let (input, ground_theme) = tuple((le_u8, le_u8, le_u8))(input)?;
    let (input, ground_theme_strength) = tuple((le_u8, le_u8))(input)?;
    let (input, walkable) = le_u8(input)?;
    let (input, passover) = le_u8(input)?;
    let (input, sound_theme) = le_u8(input)?;
    let (input, _zero) = le_u8(input)?;
    let (input, shore) = le_u8(input)?;
    let (input, _unknown) = le_u8(input)?;

    Ok(
        (
            input,
            LevHeightmapCell {
                size: size,
                version: version,
                height: height,
                ground_theme: ground_theme,
                ground_theme_strength: ground_theme_strength,
                walkable: walkable != 0,
                passover: passover != 0,
                sound_theme: sound_theme,
                shore: shore != 0,
            }
        )
    )
}

pub fn decode_soundmap_cell(input: &[u8]) -> IResult<&[u8], LevSoundmapCell> {
    let (input, size) = le_u32(input)?;
    let (input, version) = le_u8(input)?;
    let (input, sound_theme) = tuple((le_u8, le_u8, le_u8))(input)?;
    let (input, sound_theme_strength) = tuple((le_u8, le_u8))(input)?;
    let (input, sound_index) = le_u8(input)?;

    Ok(
        (
            input,
            LevSoundmapCell {
                size: size,
                version: version,
                sound_theme: sound_theme,
                sound_theme_strength: sound_theme_strength,
                sound_index: sound_index,
            }
        )
    )
}

pub fn decode_navigation_header(input: &[u8]) -> IResult<&[u8], LevNavigationHeader> {
    let (input, sections_start) = le_u32(input)?;
    let (input, sections_count) = le_u32(input)?;

    let (input, sections) = count(decode_navigation_header_section, sections_count as usize)(input)?;

    Ok(
        (
            input,
            LevNavigationHeader {
                sections_start: sections_start,
                sections_count: sections_count,
                sections: sections,
            }
        )
    )
}

pub fn decode_navigation_header_section(input: &[u8]) -> IResult<&[u8], (String, u32)> {
    let (input, name) = decode_rle_string(input)?;
    let (input, start) = le_u32(input)?;

    Ok( (input, (name, start)) )
}


pub fn decode_navigation_section(input: &[u8]) -> IResult<&[u8], LevNavigationSection> {
    let (input, size) = le_u32(input)?;
    let (input, version) = le_u32(input)?;
    let (input, level_width) = le_u32(input)?;
    let (input, level_height) = le_u32(input)?;
    let (input, _unknown_1) = le_u32(input)?; // fabletlcmod.com: Number of levels, see navigation nodes

    let (input, interactive_nodes_count) = le_u32(input)?;
    let (input, interactive_nodes) = count(decode_navigation_interactive_node, interactive_nodes_count as usize)(input)?;

    let (input, subsets_count) = le_u32(input)?;

    // println!("size {:#?}", size);
    // println!("version {:#?}", version);
    // println!("level_width {:#?}", level_width);
    // println!("level_height {:#?}", level_height);
    // println!("interactive_nodes_count {:#?}", interactive_nodes_count);
    // println!("interactive_nodes {:#?}", interactive_nodes);
    // println!("subsets_count {:#?}", subsets_count);

    let (input, level_nodes_count) = le_u32(input)?;
    println!("level_nodes_count {:#?}", level_nodes_count);
    let (input, level_nodes) = count(decode_navigation_level_node, level_nodes_count as usize)(input)?;
    // let (input, level_nodes) = count(decode_navigation_level_node, 1usize)(input)?;
    // println!("level_nodes {:?}", level_nodes);

    Ok(
        (
            input,
            LevNavigationSection {
                size: size,
                version: version,
                level_width: level_width,
                level_height: level_height,
                interactive_nodes: interactive_nodes,
                subsets_count: subsets_count,
                level_nodes: level_nodes,
            }
        )
    )
}

pub fn decode_navigation_interactive_node(input: &[u8]) -> IResult<&[u8], LevInteractiveNode> {
    let (input, x) = le_u32(input)?;
    let (input, y) = le_u32(input)?;
    let (input, subset) = le_u32(input)?;

    Ok(
        (
            input,
            LevInteractiveNode {
                x: x,
                y: y,
                subset: subset,
            }
        )
    )
}

pub fn decode_navigation_level_node(input: &[u8]) -> IResult<&[u8], LevNavigationNode> {
    // println!("next node {:?}", &input[..18]);

    let (input, level_node) = alt((
        decode_navigation_regular_node,
        decode_navigation_navigation_node,
        decode_navigation_exit_node,
        decode_navigation_blank_node,
        decode_navigation_unknown1_node,
        decode_navigation_unknown2_node,
        decode_navigation_unknown3_node,
        decode_navigation_unknown_node,
    ))(input)?;

    // println!("level_node {:?}", level_node);

    Ok((input, level_node))
}

pub fn decode_navigation_regular_node(input: &[u8]) -> IResult<&[u8], LevNavigationNode> {
    let (input, _node_op) = tag(&[0, 0, 0, 0, 0, 1, 0, 0])(input)?;
    let (input, _unknown_1) = le_u8(input)?;
    let (input, root) = le_u8(input)?;
    let (input, _unknown_2) = le_u8(input)?;
    let (input, end) = le_u8(input)?;
    let (input, layer) = le_u8(input)?;
    let (input, subset) = le_u8(input)?;
    let (input, x) = le_f32(input)?;
    let (input, y) = le_f32(input)?;
    let (input, node_id) = le_u32(input)?;

    println!("node_id {:?}", node_id);

    let (input, child_nodes) = tuple((le_u32, le_u32, le_u32, le_u32))(input)?;

    Ok(
        (
            input,
            LevNavigationNode::Regular(
                LevNavigationRegularNode {
                    root: root,
                    end: end,
                    layer: layer,
                    subset: subset,
                    x: x,
                    y: y,
                    node_id: node_id,
                    child_nodes: child_nodes,
                }
            )
        )
    )
}

pub fn decode_navigation_navigation_node(input: &[u8]) -> IResult<&[u8], LevNavigationNode> {
    let (input, _node_op) = tag(&[0, 0, 0, 1, 0, 1, 0, 1])(input)?;
    let (input, _unknown_1) = le_u8(input)?;
    let (input, root) = le_u8(input)?;
    let (input, _unknown_2) = le_u8(input)?;
    let (input, end) = le_u8(input)?;
    let (input, layer) = le_u8(input)?;
    let (input, subset) = le_u8(input)?;
    let (input, x) = le_f32(input)?;
    let (input, y) = le_f32(input)?;
    let (input, node_id) = le_u32(input)?;

    println!("node_id {:?}", node_id);

    let (input, node_level) = le_u32(input)?; // fabletlcmod.com: Represents some sort of z level attribute
    let (input, _unknown_3) = le_u8(input)?;  // fabletlcmod.com: So far, Subset 0 = 0 or 128, SubSet 1+ = 64

    let (input, nodes_count) = le_u32(input)?;
    let (input, nodes) = count(le_u32, nodes_count as usize)(input)?;

    Ok(
        (
            input,
            LevNavigationNode::Navigation(
                LevNavigationNavigationNode {
                    root: root,
                    end: end,
                    layer: layer,
                    subset: subset,
                    x: x,
                    y: y,
                    node_id: node_id,
                    node_level: node_level,
                    nodes: nodes,
                }
            )
        )
    )
}

pub fn decode_navigation_exit_node(input: &[u8]) -> IResult<&[u8], LevNavigationNode> {
    let (input, _node_op) = tag(&[1, 0, 0, 1, 1, 0, 1, 1])(input)?;
    let (input, _unknown_1) = le_u8(input)?;
    let (input, root) = le_u8(input)?;
    let (input, _unknown_2) = le_u8(input)?;
    let (input, end) = le_u8(input)?;
    let (input, layer) = le_u8(input)?;
    let (input, subset) = le_u8(input)?;
    let (input, x) = le_f32(input)?;
    let (input, y) = le_f32(input)?;
    let (input, node_id) = le_u32(input)?;

    println!("node_id {:?}", node_id);

    let (input, node_level) = le_u32(input)?; // fabletlcmod.com: Represents some sort of z level attribute
    let (input, _unknown_3) = le_u8(input)?;  // fabletlcmod.com: So far, Subset 0 = 0 or 128, SubSet 1+ = 64

    let (input, nodes_count) = le_u32(input)?;
    let (input, nodes) = count(le_u32, nodes_count as usize)(input)?;

    // fabletlcmod.com: Stripped UID to create the real uid add 18446741874686296064
    let (input, uids_count) = le_u32(input)?;
    let (input, uids) = count(le_u64, uids_count as usize)(input)?;

    Ok(
        (
            input,
            LevNavigationNode::Exit(
                LevNavigationExitNode {
                    root: root,
                    end: end,
                    layer: layer,
                    subset: subset,
                    x: x,
                    y: y,
                    node_id: node_id,
                    node_level: node_level,
                    nodes: nodes,
                    uids: uids,
                }
            )
        )
    )
}

pub fn decode_navigation_unknown1_node(input: &[u8]) -> IResult<&[u8], LevNavigationNode> {
    let (maybe_input, _node_op) = tag(&[11, 0, 0, 0, 0, 0, 0, 0, 0, 0])(input)?;
    let (maybe_input, _unknown_1) = le_u8(maybe_input)?;
    let (maybe_input, _root) = le_u8(maybe_input)?;
    let (maybe_input, _unknown_2) = le_u8(maybe_input)?;
    let (_maybe_input, end) = le_u8(maybe_input)?;

    let unknown = LevNavigationUnknown1Node {
        end: end,
    };

    let maybe_input = &input[end as usize..];

    Ok((maybe_input, LevNavigationNode::Unknown1(unknown)))
}

pub fn decode_navigation_unknown2_node(input: &[u8]) -> IResult<&[u8], LevNavigationNode> {
    let (maybe_input, _node_op) = tag(&[0, 1, 0, 0, 0, 0, 0, 0])(input)?;
    let (maybe_input, _unknown_1) = le_u8(maybe_input)?;
    let (maybe_input, _root) = le_u8(maybe_input)?;
    let (maybe_input, _unknown_2) = le_u8(maybe_input)?;
    let (_maybe_input, end) = le_u8(maybe_input)?;

    let unknown = LevNavigationUnknown2Node {
        end: end,
    };

    let maybe_input = &input[end as usize..];

    Ok((maybe_input, LevNavigationNode::Unknown2(unknown)))
}

pub fn decode_navigation_unknown3_node(input: &[u8]) -> IResult<&[u8], LevNavigationNode> {
    let (maybe_input, _node_op) = tag(&[0, 0, 0, 0, 0, 0, 0, 0])(input)?;
    let (maybe_input, _unknown_1) = le_u8(maybe_input)?;
    let (maybe_input, _root) = le_u8(maybe_input)?;
    let (maybe_input, _unknown_2) = le_u8(maybe_input)?;
    let (_maybe_input, end) = le_u8(maybe_input)?;

    let unknown = LevNavigationUnknown3Node {
        end: end,
    };

    let maybe_input = &input[end as usize..];

    Ok((maybe_input, LevNavigationNode::Unknown3(unknown)))
}

pub fn decode_navigation_unknown_node(input: &[u8]) -> IResult<&[u8], LevNavigationNode> {
    let (maybe_input, node_op) = take(8usize)(input)?;
    let (maybe_input, _unknown_1) = le_u8(maybe_input)?;
    let (maybe_input, _root) = le_u8(maybe_input)?;
    let (maybe_input, _unknown_2) = le_u8(maybe_input)?;
    let (_maybe_input, end) = le_u8(maybe_input)?;

    let unknown = LevNavigationUnknownNode {
        node_op: node_op.to_vec(),
        end: end,
    };

    let maybe_input = &input[end as usize..];

    Ok((maybe_input, LevNavigationNode::Unknown(unknown)))
}

pub fn decode_navigation_blank_node(input: &[u8]) -> IResult<&[u8], LevNavigationNode> {
    let (input, _node_op) = tag(&[0, 1, 1])(input)?;
    let (input, _unknown_1) = le_u8(input)?;
    let (input, root) = le_u8(input)?;
    let (input, _unknown_2) = le_u8(input)?;

    println!("blank {:?}", root);

    Ok(
        (
            input,
            LevNavigationNode::Blank(
                LevNavigationBlankNode {
                    root: root,
                }
            )
        )
    )
}