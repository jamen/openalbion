use crate::{BigTextureInfo, Bytes};

#[derive(Debug)]
pub struct Texture {
    pub width: u16,
    pub height: u16,
    pub dxt_compression: u16,
    pub frames: Vec<Vec<u8>>,
}

impl Texture {
    pub fn decode(mut data: &[u8], info: &BigTextureInfo) -> Option<Self> {
        // println!("{:#?}", info);

        let mut frames = Vec::new();

        if info.first_mipmap_compressed_size > 0 {
            let size = info.first_mipmap_compressed_size & 0xffff0000 | data.parse_u16_le()? as u32;

            let size = if size as i16 == -1 {
                data.parse_u32_le()?
            } else {
                size
            };

            let first_mipmap_compressed = data.advance(size as usize)?;

            let mut texture_data =
                minilzo::decompress(first_mipmap_compressed, info.first_mipmap_size as usize)
                    .ok()?;

            let trail = data.advance(3)?;

            texture_data.extend_from_slice(&trail);

            // let bc_format = match info.dxt_compression {
            //     31 => { bcndecode::BcnEncoding::Bc1 },
            //     32 => { bcndecode::BcnEncoding::Bc2 },
            //     _ => { bcndecode::BcnEncoding::Bc2 }, // Idk but default to this
            // };

            // let data = bcndecode::decode(&data, info.width as usize, info.height as usize,
            // bc_format, bcndecode::BcnDecoderFormat::RGBA).unwrap();

            // let stdout = std::io::stdout();
            // let mut stdout_writer = stdout.lock();

            // let mut png_encoder = png::Encoder::new(&mut stdout_writer, info.width as u32,
            // info.height as u32);

            // png_encoder.set_color(png::ColorType::RGBA);
            // png_encoder.set_depth(png::BitDepth::Eight);

            // let mut writer = png_encoder.write_header().unwrap();

            // writer.write_image_data(&data);

            frames.push(texture_data);
        }

        // let data = &data[..252];

        // let bc_format = match info.dxt_compression {
        //     31 => bcndecode::BcnEncoding::Bc1,
        //     32 => bcndecode::BcnEncoding::Bc2,
        //     _ => bcndecode::BcnEncoding::Bc2, // Idk but default to this
        // };

        // let data = bcndecode::decode(
        //     &data,
        //     info.width as usize / 2,
        //     info.height as usize / 2,
        //     bc_format,
        //     bcndecode::BcnDecoderFormat::RGBA,
        // )
        // .unwrap();

        // let stdout = std::io::stdout();
        // let mut stdout_writer = stdout.lock();

        // let mut png_encoder = png::Encoder::new(
        //     &mut stdout_writer,
        //     info.width as u32 / 2,
        //     info.height as u32 / 2,
        // );

        // png_encoder.set_color(png::ColorType::RGBA);
        // png_encoder.set_depth(png::BitDepth::Eight);

        // let mut writer = png_encoder.write_header().unwrap();

        // writer.write_image_data(&data);

        // if info.mipmaps > 1 {

        //     // for _ in 1..info.mipmaps {

        //     // }
        // }

        Some(Texture {
            width: info.width,
            height: info.height,
            dxt_compression: info.dxt_compression,
            frames,
        })
    }
}
