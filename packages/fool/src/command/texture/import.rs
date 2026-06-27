use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct TextureImportArgs {}

pub fn handler(_args: TextureImportArgs) -> anyhow::Result<()> {
    // TODO: Encode an image (e.g. PNG) into Fable's texture format and insert it back into a
    // .big bank. Requires a texture encoder, which does not exist in fable-data yet.
    anyhow::bail!("`texture import` is not implemented yet")
}
