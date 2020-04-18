use crate::nom::IResult;

use crate::{
    DefBin,
    DefBinHeader,
    DefBinNameLookup,
    DefSecondTableHeader,
    DefSecondTableRow,
    DefSecondTableRowDecompressed,
    Error,
};

// impl DefBin {
//     fn decode_def_bin(input: &[u8]) -> IResult<&[u8], DefBin, Error> {
//     }
// }