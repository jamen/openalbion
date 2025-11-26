use derive_more::Display;
use fable_data::{BigFile, BigFileError};
use std::{fs::File, io, path::Path};

pub struct Files {
    textures: Textures,
}

impl Files {
    pub fn new(fable_directory: &Path) -> Result<Self, NewFilesError> {
        use NewFilesError as E;
        let textures = Textures::new(fable_directory).map_err(E::NewTextures)?;
        Ok(Self { textures })
    }
}

#[derive(Display, Debug)]
pub enum NewFilesError {
    NewTextures(NewTexturesError),
}

pub struct Textures {
    big_file: BigFile,
}

impl Textures {
    pub fn new(fable_directory: &Path) -> Result<Self, NewTexturesError> {
        use NewTexturesError as E;
        let big_file = fable_directory.join("data/graphics/pc/textures.big");
        let big_file = File::open(big_file).map_err(E::Open)?;
        let big_file = BigFile::new(big_file).map_err(E::Parse)?;
        Ok(Self { big_file })
    }
}

#[derive(Display, Debug)]
pub enum NewTexturesError {
    Open(io::Error),
    Parse(BigFileError),
}
