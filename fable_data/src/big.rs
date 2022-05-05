//! Big files are archives containing most of the game's assets, except the map data most notably.
//!
//! Its a binary file format with the following structure
//!
//! # Header
//!
//! | Field        | Byte Length | Type      |
//! |--------------|-------------|-----------|
//! | Magic Number | 4           | `u32`     |
//! | Version      | 4           | `u32`     |
//! | Banks Start  | 4           | `u32`     |

use nom::{
    bytes::complete::take_till,
    error::ParseError,
    number::complete::{le_u32, le_u8},
    IResult,
};

#[derive(Debug)]
pub enum BigError<'a> {
    Nom(&'a [u8], nom::error::ErrorKind),
}

impl<'a> ParseError<&'a [u8]> for BigError<'a> {
    fn from_error_kind(input: &'a [u8], kind: nom::error::ErrorKind) -> Self {
        BigError::Nom(input, kind)
    }
    fn append(input: &'a [u8], kind: nom::error::ErrorKind, _other: Self) -> Self {
        BigError::Nom(input, kind)
    }
}

#[derive(Debug)]
pub struct Big<'a> {
    pub header: BigHeader,
    pub bank: BigBank<'a>,
}

impl<'a> Big<'a> {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Big, BigError<'_>> {
        let (_, header) = BigHeader::parse(input)?;

        let bank_input = &input[header.banks_start as usize..];

        let (_, bank) = BigBank::parse(bank_input)?;

        Ok((input, Big { header, bank }))
    }
}

#[derive(Debug)]
pub struct BigHeader {
    pub magic_number: u32,
    pub version: u32,
    pub banks_start: u32,
}

impl BigHeader {
    fn parse(input: &[u8]) -> IResult<&[u8], BigHeader, BigError<'_>> {
        let (input, magic_number) = le_u32(input)?;
        let (input, version) = le_u32(input)?;
        let (input, banks_start) = le_u32(input)?;

        Ok((
            input,
            BigHeader {
                magic_number,
                version,
                banks_start,
            },
        ))
    }
}

#[derive(Debug)]
pub struct BigBank<'a> {
    pub count: u32,
    pub name: &'a [u8],
    pub id: u32,
}

impl<'a> BigBank<'a> {
    fn parse(input: &[u8]) -> IResult<&[u8], BigBank, BigError<'_>> {
        let (input, count) = le_u32(input)?;
        let (input, name) = take_till(|c| c == 0)(input)?;
        let (input, _) = le_u8(input)?;
        let (input, id) = le_u32(input)?;
        Ok((input, BigBank { count, name, id }))
    }
}
