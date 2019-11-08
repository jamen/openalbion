// use nom::IResult;
// use nom::number::complete::{le_u8,le_u16,le_u32,le_f32};
// use nom::multi::count;

// use crate::shared::string::decode_null_terminated_string;

// use crate::bbm::{
//     Bbm,
//     BbmHeader,
//     BbmHelperPoint,
//     BbmHelperDummy,
// };

// pub fn decode_bbm(input: &[u8]) -> IResult<&[u8], Bbm> {
//     let (input, header) = decode_header(input)?;

//     Ok(
//         (
//             input,
//             Bbm {
//                 header: header,
//             }
//         )
//     )
// }

// pub fn decode_header(input: &[u8]) -> IResult<&[u8], BbmHeader> {
//     let (input, unknown1) = decode_null_terminated_string(input)?;
//     let (input, selection_present) = le_u8(input)?;
//     let (input, unknown2) = count(le_f32, 10usize)(input)?;
//     let (input, hpnt_count) = le_u16(input)?;
//     let (input, hdmy_count) = le_u16(input)?;
//     let (input, hlpr_index_uncompressed) = le_u32(input)?;
//     let (input, padding) = le_u16(input)?;
//     let (input, hpnt_compressed) = le_u16(input)?;
//     let (input, helper_points) = count(decode_helper_point, hpnt_count as usize)(input)?;
//     let (input, hdmy_compressed) = le_u16(input)?;
//     let (input, helper_dummies) = count(decode_helper_dummy, hdmy_count as usize)(input)?;
//     let (input, hlpr_index_compressed) = le_u16(input)?;
//     let (input, hpnt_index_size) = le_u16(input)?;

//     Ok(
//         (
//             input,
//             BbmHeader {
//                 unknown1: unknown1,
//                 selection_present: selection_present,
//                 unknown2: unknown2,
//                 hpnt_count: hpnt_count,
//                 hdmy_count: hdmy_count,
//                 hlpr_index_uncompressed: hlpr_index_uncompressed,
//                 padding: padding,
//                 hpnt_compressed: hpnt_compressed,
//                 helper_points: helper_points,
//                 hdmy_compressed: hdmy_compressed,
//                 helper_dummies: helper_dummies,
//                 hlpr_index_compressed: hlpr_index_compressed,
//                 hpnt_index_size: hpnt_index_size,
//             }
//         )
//     )
// }

// pub fn decode_helper_point(input: &[u8]) -> IResult<&[u8], BbmHelperPoint> {
//     Ok(
//         (
//             input,
//             BbmHelperPoint {}
//         )
//     )
// }

// pub fn decode_helper_dummy(input: &[u8]) -> IResult<&[u8], BbmHelperDummy> {
//     Ok(
//         (
//             input,
//             BbmHelperDummy {}
//         )
//     )
// }