use derive_more::Display;
use fable_data::{BigFile, BigFileError};
use std::{fs::File, io, path::Path};

pub struct Files {
    textures: TextureFiles,
}

#[derive(Display, Debug)]
pub enum NewFilesError {
    NewTextureFiles(NewTextureFilesError),
}

impl Files {
    pub fn new<P: AsRef<Path>>(fable_directory: P) -> Result<Self, NewFilesError> {
        use NewFilesError as E;
        let fable_directory = fable_directory.as_ref();
        let textures = TextureFiles::new(fable_directory).map_err(E::NewTextureFiles)?;
        Ok(Self { textures })
    }
}

pub struct TextureFiles {
    big_file: BigFile,
}

#[derive(Display, Debug)]
pub enum NewTextureFilesError {
    Open(io::Error),
    Parse(BigFileError),
}

impl TextureFiles {
    fn new(fable_directory: &Path) -> Result<Self, NewTextureFilesError> {
        use NewTextureFilesError as E;

        let big_file = fable_directory.join("data/graphics/pc/textures.big");
        let big_file = File::open(big_file).map_err(E::Open)?;
        let mut big_file = BigFile::new(big_file).map_err(E::Parse)?;

        tracing::info!("huh");

        // TODO: Log bank contents
        for bank_info in big_file.read_bank_infos().unwrap() {
            tracing::info!("{:#?}", bank_info);

            let (asset_infos_header, asset_infos) = big_file.read_asset_infos(&bank_info).unwrap();

            tracing::info!("{:#?}", asset_infos_header);

            for asset_info in asset_infos {
                tracing::info!("{:#?}", asset_info);
            }
        }

        Ok(Self { big_file })
    }
}
