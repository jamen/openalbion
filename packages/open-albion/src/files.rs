use derive_more::{Display, Error};
use fable_data::big::{BigReader, BigReaderError};
use std::{fs::File, io, path::Path};

pub struct Files {
    pub textures: BigReader<File>,
}

#[derive(Debug, Display, Error)]
pub enum NewFilesError {
    OpenTextures(io::Error),
    LoadTextures(BigReaderError),
}

impl Files {
    pub fn new(fable_directory: &Path) -> Result<Self, NewFilesError> {
        use NewFilesError as E;

        let textures_path = fable_directory.join("data/graphics/pc/textures.big");
        let textures_file = File::open(&textures_path).map_err(E::OpenTextures)?;
        let textures = BigReader::new(textures_file).map_err(E::LoadTextures)?;

        Ok(Self { textures })
    }
}
