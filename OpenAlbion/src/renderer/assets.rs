use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub struct Assets {
    graphics: BufReader<File>,
    textures: BufReader<File>,
}

pub enum AssetError {
    GraphicsUnavailable,
    TexturesUnavailable,
}

impl Assets {
    const GRAPHICS_PATH: &'static str = "data/graphics/graphics.big";
    const TEXTURES_PATH: &'static str = "data/graphics/pc/textures.big";

    pub fn new<P: AsRef<Path>>(fable_dir: P) -> Result<Self, AssetError> {
        let fable_dir = fable_dir.as_ref();

        let graphics_file = File::open(fable_dir.join(&Self::GRAPHICS_PATH))
            .or(Err(AssetError::GraphicsUnavailable))?;

        let graphics = BufReader::new(graphics_file);

        let textures_file = File::open(fable_dir.join(&Self::TEXTURES_PATH))
            .or(Err(AssetError::TexturesUnavailable))?;

        let textures = BufReader::new(textures_file);

        Ok(Self { graphics, textures })
    }

    pub fn read_model_by_name(&self, model_name: &str) -> fable_data::Model {
        self.graphics
    }
}
