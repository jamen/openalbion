use nom::IResult;
use nom::number::complete::{le_u8,le_u16,le_u32,le_u64,float};
use nom::bytes::complete::{tag,take};
use nom::sequence::tuple;
use nom::combinator::iterator;
use nom::multi::count;
use std::fs::{File,create_dir_all};
use std::io::{SeekFrom,Seek,Read,Write,Error,ErrorKind};
use std::iter::Iterator;
use std::collections::{HashMap,HashSet};
use std::path::Path;
use std::convert::TryInto;
use nom::branch::alt;

#[derive(Debug,PartialEq)]
pub struct Lev<'a> {
    header: LevHeader<'a>,
    heightmap_cells: Vec<LevHeightmapCell>,
    soundmap_cells: Vec<LevSoundmapCell>,
}

#[derive(Debug,PartialEq)]
pub struct LevHeader<'a> {
    pub version: u16,
    pub obsolete_offset: u32,
    pub navigation_offset: u32,
    pub unique_id_count: u64,
    pub width: u32,
    pub height: u32,
    pub map_version: u32,
    pub heightmap_palette: &'a [u8],
    pub ambient_sound_version: u32,
    pub sound_palette: &'a [u8],
    pub checksum: u32,
    pub sound_themes: Vec<&'a [u8]>,
}

pub fn parse_header(input: &[u8]) -> IResult<&[u8], LevHeader> {
    let (input, _header_size) = le_u32(input)?;
    let (input, version) = le_u16(input)?;
    let (input, _unknown_1) = take(3usize)(input)?;
    let (input, _unknown_2) = le_u32(input)?;
    let (input, obsolete_offset) = le_u32(input)?;
    let (input, _unknown_3) = le_u32(input)?;
    let (input, navigation_offset) = le_u32(input)?;
    let (input, _map_header_size) = le_u8(input)?;
    let (input, map_version) = le_u32(input)?;
    let (input, unique_id_count) = le_u64(input)?;
    let (input, width) = le_u32(input)?;
    let (input, height) = le_u32(input)?;
    let (input, _always_true) = le_u8(input)?;

    let (input, heightmap_palette) = take(33792usize)(input)?;
    let (input, ambient_sound_version) = le_u32(input)?;
    let (input, sound_themes_count) = le_u32(input)?;
    let (input, sound_palette) = take(33792usize)(input)?;
    let (input, checksum) = le_u32(input)?;

    let (input, sound_themes) = count(parse_header_sound_theme, (sound_themes_count - 1) as usize)(input)?;

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
                heightmap_palette: heightmap_palette,
                ambient_sound_version: ambient_sound_version,
                sound_palette: sound_palette,
                checksum: checksum,
                sound_themes: sound_themes,
            }
        )
    )
}

pub fn parse_header_sound_theme(input: &[u8]) -> IResult<&[u8], &[u8]> {
    let (input, size) = le_u32(input)?;
    let (input, sound_theme) = take(size as usize)(input)?;
    Ok((input, sound_theme))
}

#[derive(Debug,PartialEq)]
pub struct LevHeightmapCell {
    size: u32,
    version: u8,
    height: f32,
    ground_theme: (u8, u8, u8),
    ground_theme_strength: (u8, u8),
    walkable: bool,
    passover: bool,
    sound_theme: u8,
    shore: bool,
}

pub fn parse_heightmap_cell(input: &[u8]) -> IResult<&[u8], LevHeightmapCell> {
    let (input, size) = le_u32(input)?;
    let (input, version) = le_u8(input)?;
    let (input, height) = float(input)?;
    let (input, _zero) = le_u32(input)?;
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

#[derive(Debug,PartialEq)]
pub struct LevSoundmapCell {
    size: u32,
    version: u8,
    sound_theme: (u8, u8, u8),
    sound_theme_strength: (u8, u8),
    sound_index: u8,
}

pub fn parse_soundmap_cell(input: &[u8]) -> IResult<&[u8], LevSoundmapCell> {
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

#[derive(Debug,PartialEq)]
pub struct LevNavigationHeader<'a> {
    sections_start: u32,
    sections_count: u32,
    sections: Vec<(&'a [u8], u32)>,
}

fn parse_navigation_header(input: &[u8]) -> IResult<&[u8], LevNavigationHeader> {
    let (input, sections_start) = le_u32(input)?;
    let (input, sections_count) = le_u32(input)?;

    let (input, sections) = count(parse_navigation_header_section, sections_count as usize)(input)?;

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

fn parse_navigation_header_section(input: &[u8]) -> IResult<&[u8], (&[u8], u32)> {
    let (input, len) = le_u32(input)?;
    let (input, name) = take(len as usize)(input)?;
    let (input, start) = le_u32(input)?;

    Ok( (input, (name, start)) )
}

#[derive(Debug,PartialEq)]
pub struct LevNavigationSection {
    size: u32,
    version: u32,
    level_width: u32,
    level_height: u32,
    interactive_nodes: Vec<LevInteractiveNode>,
    subsets_count: u32,
    level_nodes: Vec<LevNavigationNode>,
}

fn parse_navigation_section(input: &[u8]) -> IResult<&[u8], LevNavigationSection> {
    let (input, size) = le_u32(input)?;
    let (input, version) = le_u32(input)?;
    let (input, level_width) = le_u32(input)?;
    let (input, level_height) = le_u32(input)?;
    let (input, _unknown_1) = le_u32(input)?;

    let (input, interactive_nodes_count) = le_u32(input)?;
    let (input, interactive_nodes) = count(parse_navigation_interactive_node, interactive_nodes_count as usize)(input)?;

    let (input, subsets_count) = le_u32(input)?;

    let (input, level_nodes_count) = le_u32(input)?;
    let (input, level_nodes) = count(parse_navigation_level_node, level_nodes_count as usize)(input)?;

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

#[derive(Debug,PartialEq)]
struct LevInteractiveNode {
    x: u32,
    y: u32,
    subset: u32,
}

fn parse_navigation_interactive_node(input: &[u8]) -> IResult<&[u8], LevInteractiveNode> {
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

#[derive(Debug,PartialEq)]
enum LevNavigationNode {
    Regular(LevNavigationRegularNode),
    Navigation(LevNavigationNavigationNode),
    Exit(LevNavigationExitNode),
    Blank(LevNavigationBlankNode),
}

fn parse_navigation_level_node(input: &[u8]) -> IResult<&[u8], LevNavigationNode> {
    alt((
        parse_navigation_regular_node,
        parse_navigation_navigation_node,
        parse_navigation_exit_node,
        parse_navigation_blank_node
    ))(input)
}

#[derive(Debug,PartialEq)]
struct LevNavigationRegularNode {
    root: u8,
    end: u8,
    layer: u8,
    subset: u8,
    x: f32,
    y: f32,
    node_id: u32,
    child_nodes: (u32, u32, u32, u32)
}

fn parse_navigation_regular_node(input: &[u8]) -> IResult<&[u8], LevNavigationNode> {
    let (input, _node_op) = tag(&[0, 0, 0, 0, 0, 1, 0, 0])(input)?;
    let (input, _unknown_1) = le_u8(input)?;
    let (input, root) = le_u8(input)?;
    let (input, _unknown_2) = le_u8(input)?;
    let (input, end) = le_u8(input)?;
    let (input, layer) = le_u8(input)?;
    let (input, subset) = le_u8(input)?;
    let (input, x) = float(input)?;
    let (input, y) = float(input)?;
    let (input, node_id) = le_u32(input)?;
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

#[derive(Debug,PartialEq)]
struct LevNavigationNavigationNode {
    root: u8,
    end: u8,
    layer: u8,
    subset: u8,
    x: f32,
    y: f32,
    node_id: u32,
    node_level: u32,
    nodes: Vec<u32>,
}

fn parse_navigation_navigation_node(input: &[u8]) -> IResult<&[u8], LevNavigationNode> {
    let (input, _node_op) = tag(&[0, 0, 0, 1, 0, 1, 0, 1])(input)?;
    let (input, _unknown_1) = le_u8(input)?;
    let (input, root) = le_u8(input)?;
    let (input, _unknown_2) = le_u8(input)?;
    let (input, end) = le_u8(input)?;
    let (input, layer) = le_u8(input)?;
    let (input, subset) = le_u8(input)?;
    let (input, x) = float(input)?;
    let (input, y) = float(input)?;
    let (input, node_id) = le_u32(input)?;
    let (input, node_level) = le_u32(input)?;
    let (input, _unknown_3) = le_u8(input)?;

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

#[derive(Debug,PartialEq)]
struct LevNavigationExitNode {
    root: u8,
    end: u8,
    layer: u8,
    subset: u8,
    x: f32,
    y: f32,
    node_id: u32,
    node_level: u32,
    nodes: Vec<u32>,
    uids: Vec<u64>,
}

fn parse_navigation_exit_node(input: &[u8]) -> IResult<&[u8], LevNavigationNode> {
    let (input, _node_op) = tag(&[1, 0, 0, 1, 1, 0, 1, 1])(input)?;
    let (input, _unknown_1) = le_u8(input)?;
    let (input, root) = le_u8(input)?;
    let (input, _unknown_2) = le_u8(input)?;
    let (input, end) = le_u8(input)?;
    let (input, layer) = le_u8(input)?;
    let (input, subset) = le_u8(input)?;
    let (input, x) = float(input)?;
    let (input, y) = float(input)?;
    let (input, node_id) = le_u32(input)?;
    let (input, node_level) = le_u32(input)?;
    let (input, _unknown_3) = le_u8(input)?;

    let (input, nodes_count) = le_u32(input)?;
    let (input, nodes) = count(le_u32, nodes_count as usize)(input)?;

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

#[derive(Debug,PartialEq)]
struct LevNavigationBlankNode {
    root: u8
}

fn parse_navigation_blank_node(input: &[u8]) -> IResult<&[u8], LevNavigationNode> {
    let (input, _node_op) = tag(&[0, 1, 1])(input)?;
    let (input, _unknown_1) = le_u8(input)?;
    let (input, root) = le_u8(input)?;
    let (input, _unknown_2) = le_u8(input)?;

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

// #![cfg(test)]
// mod tests {
//     use super::*;

//     fn test_
// }