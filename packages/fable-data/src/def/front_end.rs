use super::binary::control::{
    ID_BYTE_SIZE, ParseControlError, ParseControlErrorReason, SerializeControlError,
    SerializeControlErrorReason, parse_id, parse_scalar, serialize_id, serialize_scalar,
};
use crate::bytes::{put, put_null_terminated_utf8, take, take_null_terminated_utf8};

#[derive(Debug)]
pub struct FrontEndDef {
    pub attract_mode_movie: Vec<String>,
    pub error_message_background_graphic: u32,
    pub button_a_big_graphic: u32,
    pub button_b_big_graphic: u32,
}

impl FrontEndDef {
    pub(crate) fn parse(cur: &mut &[u8]) -> Result<Self, ParseControlError> {
        let (_, attract_mode_movie) = Self::parse_attract_mode_movie(cur)?;

        Ok(Self {
            attract_mode_movie,
            error_message_background_graphic: parse_scalar::<u32>(
                cur,
                "ErrorMessageBackgroundGraphic",
            )?,
            button_a_big_graphic: parse_scalar::<u32>(cur, "ButtonABigGraphic")?,
            button_b_big_graphic: parse_scalar::<u32>(cur, "ButtonBBigGraphic")?,
        })
    }

    pub(crate) fn serialize(&self, out: &mut &mut [u8]) -> Result<(), SerializeControlError> {
        self.serialize_attract_mode_movie(out)?;
        serialize_scalar::<u32>(
            out,
            "ErrorMessageBackgroundGraphic",
            self.error_message_background_graphic,
        )?;
        serialize_scalar::<u32>(out, "ButtonABigGraphic", self.button_a_big_graphic)?;
        serialize_scalar::<u32>(out, "ButtonBBigGraphic", self.button_b_big_graphic)?;
        Ok(())
    }

    pub(crate) fn byte_size(&self) -> usize {
        self.attract_mode_movie_byte_size() + (ID_BYTE_SIZE + size_of::<u32>()) * 3
    }

    fn parse_attract_mode_movie(cur: &mut &[u8]) -> Result<(u32, Vec<String>), ParseControlError> {
        let name = "vAttractModeMovie";
        let id = parse_id(cur, name)?;

        let element_count = take::<u32>(cur)
            .map_err(|error| ParseControlError {
                name,
                reason: ParseControlErrorReason::ListCount(error),
            })?
            .to_le();

        let mut paths = Vec::with_capacity(element_count as usize);
        for _ in 0..element_count {
            let path = take_null_terminated_utf8(cur)
                .map_err(|error| ParseControlError {
                    name,
                    reason: ParseControlErrorReason::Utf8(error),
                })?
                .to_owned();
            paths.push(path);
        }

        Ok((id, paths))
    }

    fn serialize_attract_mode_movie(&self, out: &mut &mut [u8]) -> Result<(), SerializeControlError> {
        let name = "vAttractModeMovie";
        serialize_id(out, name)?;

        put(out, &(self.attract_mode_movie.len() as u32).to_le()).map_err(|error| {
            SerializeControlError {
                name,
                reason: SerializeControlErrorReason::ListCount(error),
            }
        })?;

        for (i, string) in self.attract_mode_movie.iter().enumerate() {
            put_null_terminated_utf8(out, string).map_err(|error| SerializeControlError {
                name,
                reason: SerializeControlErrorReason::ListItem(i, error),
            })?;
        }

        Ok(())
    }

    fn attract_mode_movie_byte_size(&self) -> usize {
        ID_BYTE_SIZE
            + size_of::<u32>()
            + self
                .attract_mode_movie
                .iter()
                .map(|x| x.len() + size_of::<u8>())
                .sum::<usize>()
    }

}
