use std::io::{Read, Seek};

use common::{
    parser::{
        all_consuming, count, decode_null_terminated_string, le_f32, le_i32, le_u16, le_u32, le_u8,
        IResult,
    },
    Error,
};

use crate::{Bbm, BbmHeader, BbmHelperDummy, BbmHelperPoint};

impl Bbm {
    pub fn decode<Source>(source: &mut Source) -> Result<Bbm, Error>
    where
        Source: Read + Seek,
    {
        let mut data = Vec::new();
        source.read_to_end(&mut data)?;
        // let (_, bbm) = all_consuming(Bbm::decode_bbm)(&data)?;
        let (_, bbm) = Bbm::decode_bbm(&data)?;
        Ok(bbm)
    }

    pub fn decode_bbm(input: &[u8]) -> IResult<&[u8], Bbm, Error> {
        let (input, header) = Self::decode_header(input)?;

        Ok((input, Bbm { header: header }))
    }

    pub fn decode_header(input: &[u8]) -> IResult<&[u8], BbmHeader, Error> {
        let (input, name) = decode_null_terminated_string(input)?;
        let (input, has_skeleton) = le_u8(input)?;
        let (input, model_origin) = count(le_f32, 10usize)(input)?;
        let (input, hpnt_count) = le_u16(input)?;
        let (input, hdmy_count) = le_u16(input)?;
        let (input, hlpr_index_uncompressed) = le_u32(input)?;
        let (input, padding) = le_u16(input)?;
        let (input, hpnt_compressed) = le_u16(input)?;
        let (input, helper_points) = count(Self::decode_helper_point, hpnt_count as usize)(input)?;
        let (input, hdmy_compressed) = le_u16(input)?;
        let (input, helper_dummies) = count(Self::decode_helper_dummy, hdmy_count as usize)(input)?;
        // let (input, hlpr_index_compressed) = le_u16(input)?;
        let (input, hpnt_index_size) = le_u16(input)?;
        let (input, hpnt_index) = count(le_u8, hpnt_index_size.saturating_sub(2) as usize)(input)?;
        // let (input, hdmy_index) = count(le_u8, (hlpr_index_uncompressed.checked_sub(hpnt_index_size.into()).unwrap_or(0)) as usize)(input)?;
        // let (input, material_count) = le_u32(input)?;
        // let (input, surface_count) = le_u32(input)?;
        // let (input, bone_count) = le_u32(input)?;
        // let (input, bone_index_size) = le_u32(input)?;
        // let (input, unknown3) = le_u8(input)?;
        // let (input, unknown4) = le_u16(input)?;
        // let (input, unknown5) = le_u16(input)?;
        // let (input, compressed) = le_u16(input)?;
        // let (input, bone_index_reference) = count(le_u16, (bone_count - 1) as usize)(input)?;
        // let (input, bone_index_compressed) = le_u16(input)?;
        // let (input, bone_index) = count(le_u8, bone_index_size as usize)(input)?;
        // let (input, compressed_size) = le_u16(input)?;

        dbg!(&name);
        dbg!(&has_skeleton);
        dbg!(&model_origin);
        dbg!(&hpnt_count);
        dbg!(&hdmy_count);
        dbg!(&hlpr_index_uncompressed);
        dbg!(&padding);
        dbg!(&hpnt_compressed);
        dbg!(&helper_points);
        dbg!(&hdmy_compressed);
        dbg!(&helper_dummies);
        // dbg!(&hlpr_index_compressed);
        dbg!(&hpnt_index_size);
        dbg!(hpnt_index.len()); // dbg!(&hpnt_index);
                                // dbg!(&hdmy_index);
                                // dbg!(&material_count);
                                // dbg!(&surface_count);
                                // dbg!(&bone_count);
                                // dbg!(&bone_index_size);
                                // dbg!(&unknown3);
                                // dbg!(&unknown4);
                                // dbg!(&unknown5);
                                // dbg!(&compressed);

        hex_table::HexTable::default()
            .format(&input[..256], &mut std::io::stdout())
            .unwrap();
        dbg!(input.len());
        println!("");

        Ok((
            input,
            BbmHeader {
                name,
                has_skeleton,
                model_origin,
                hpnt_count,
                hdmy_count,
                hlpr_index_uncompressed,
                padding,
                hpnt_compressed,
                helper_points,
                hdmy_compressed,
                helper_dummies,
                // hlpr_index_compressed,
                hpnt_index_size,
                hpnt_index,
                // hdmy_index,
                // material_count,
                // surface_count,
                // bone_count,
                // bone_index_size,
                // unknown3,
                // unknown4,
                // unknown5,
                // compressed,
                // bone_index_reference,
                // bone_index_compressed,
                // bone_index,
                // compressed_size,
            },
        ))
    }

    pub fn decode_helper_point(input: &[u8]) -> IResult<&[u8], BbmHelperPoint, Error> {
        let (input, matrix) = count(le_f32, 4usize)(input)?;
        let (input, hierarchy) = le_i32(input)?;

        Ok((
            input,
            BbmHelperPoint {
                matrix: matrix,
                hierarchy: hierarchy,
            },
        ))
    }

    pub fn decode_helper_dummy(input: &[u8]) -> IResult<&[u8], BbmHelperDummy, Error> {
        let (input, matrix) = count(le_f32, 13usize)(input)?;
        let (input, hierarchy) = le_i32(input)?;

        Ok((
            input,
            BbmHelperDummy {
                matrix: matrix,
                hierarchy: hierarchy,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Entry;
    use std::env;
    use std::fs::File;
    use std::path::PathBuf;

    #[test]
    fn test_bbm_print_meshes() {
        let file_path = PathBuf::from(env::var("FABLE_DIR").expect("Missing FABLE_DIR"))
            .join("Data/graphics/graphics.big");

        let mut file = File::open(&file_path).unwrap();
        let big = crate::Big::decode(&mut file).unwrap();

        let entries: Vec<crate::big::BigFileEntry> = big
            .entries
            .entries
            .into_iter()
            .filter(|x| {
                [
                    "MESH_CREATURE_NEW_CHICKEN_01",
                    "MESH_OBJECT_BARREL",
                    "MESH_OBJECT_BRAZIER_TORCH_LIT",
                    "MESH_OBJECT_STATUE_BEAR",
                    "MESH_SWORD_BLAST_07",
                    "MESH_HERO_WEAPON_SMALL",
                ]
                .contains(&x.symbol_name.as_str())
            })
            .collect();

        for entry in &entries {
            let mut reader = entry.to_sub_source(&mut file).unwrap();

            let bbm = Bbm::decode(&mut reader).unwrap();
        }
    }

    #[test]
    fn test_bbm_print_all_meshes() {
        let file_path = PathBuf::from(env::var("FABLE_DIR").expect("Missing FABLE_DIR"))
            .join("Data/graphics/graphics.big");

        let mut file = File::open(&file_path).unwrap();
        let big = crate::Big::decode(&mut file).unwrap();

        let mesh_entries: Vec<crate::BigFileEntry> = big
            .entries
            .entries
            .into_iter()
            .filter(|x| x.symbol_name.starts_with("MESH_"))
            .collect();

        for entry in mesh_entries {
            let mut barrel_reader = entry.to_sub_source(&mut file).unwrap();

            let bbm = Bbm::decode(&mut barrel_reader).unwrap();

            println!("");
            // println!("{:#?}", bbm);
        }
    }

    #[test]
    fn test_bbm_print_non_physics_meshes() {
        let file_path = PathBuf::from(env::var("FABLE_DIR").expect("Missing FABLE_DIR"))
            .join("Data/graphics/graphics.big");

        let mut file = File::open(&file_path).unwrap();
        let big = crate::Big::decode(&mut file).unwrap();

        let mesh_entries: Vec<crate::BigFileEntry> = big
            .entries
            .entries
            .into_iter()
            .filter(|x| x.symbol_name.starts_with("MESH_") && !x.symbol_name.ends_with("[PHYSICS]"))
            .collect();

        for entry in mesh_entries {
            let mut barrel_reader = entry.to_sub_source(&mut file).unwrap();

            let bbm = Bbm::decode(&mut barrel_reader).unwrap();

            println!("");
            // println!("{:#?}", bbm);
        }
    }

    #[test]
    fn test_bbm_print_20_random_meshes() {
        use rand::Rng;

        let file_path = PathBuf::from(env::var("FABLE_DIR").expect("Missing FABLE_DIR"))
            .join("Data/graphics/graphics.big");

        let mut file = File::open(&file_path).unwrap();
        let big = crate::Big::decode(&mut file).unwrap();

        let mesh_entries: Vec<crate::BigFileEntry> = big
            .entries
            .entries
            .into_iter()
            .filter(|x| x.symbol_name.starts_with("MESH_"))
            .collect();

        let mut rng = rand::thread_rng();

        let mut rand_mesh_entries: Vec<&crate::BigFileEntry> = Vec::new();

        while rand_mesh_entries.len() != 20 {
            let rand_mesh_idx = rng.gen_range(0, mesh_entries.len());
            let rand_mesh_entry = &mesh_entries[rand_mesh_idx];

            if !rand_mesh_entries.as_slice().contains(&rand_mesh_entry) {
                rand_mesh_entries.push(rand_mesh_entry);
            }
        }

        for entry in rand_mesh_entries {
            let mut reader = entry.to_sub_source(&mut file).unwrap();

            dbg!(entry.id);
            dbg!(&entry.symbol_name);
            let bbm = Bbm::decode(&mut reader).unwrap();

            println!("");
            // println!("{:#?}", bbm);
        }
    }

    #[test]
    fn test_bbm_print_20_random_non_physics_meshes() {
        use rand::Rng;

        let file_path = PathBuf::from(env::var("FABLE_DIR").expect("Missing FABLE_DIR"))
            .join("Data/graphics/graphics.big");

        let mut file = File::open(&file_path).unwrap();
        let big = crate::Big::decode(&mut file).unwrap();

        let mesh_entries: Vec<crate::BigFileEntry> = big
            .entries
            .entries
            .into_iter()
            .filter(|x| x.symbol_name.starts_with("MESH_") && !x.symbol_name.ends_with("[PHYSICS]"))
            .collect();

        let mut rng = rand::thread_rng();

        let mut rand_mesh_entries: Vec<&crate::BigFileEntry> = Vec::new();

        while rand_mesh_entries.len() != 20 {
            let rand_mesh_idx = rng.gen_range(0, mesh_entries.len());
            let rand_mesh_entry = &mesh_entries[rand_mesh_idx];

            if !rand_mesh_entries.as_slice().contains(&rand_mesh_entry) {
                rand_mesh_entries.push(rand_mesh_entry);
            }
        }

        for entry in rand_mesh_entries {
            let mut reader = entry.to_sub_source(&mut file).unwrap();

            dbg!(entry.id);
            dbg!(&entry.symbol_name);
            let bbm = Bbm::decode(&mut reader).unwrap();

            println!("");
            // println!("{:#?}", bbm);
        }
    }

    // NOTE: This entry ends in [PHYSICS] and breaks the parser. In fact all entries ending in [PHYSICS] seem to break the parser. Look into more thoroughly.
    #[test]
    fn test_bbm_print_physics_mesh() {
        let file_path = PathBuf::from(env::var("FABLE_DIR").expect("Missing FABLE_DIR"))
            .join("Data/graphics/graphics.big");

        let mut file = File::open(&file_path).unwrap();
        let big = crate::Big::decode(&mut file).unwrap();

        let entry = big.entries.entries.get(306).unwrap(); // MESH_OBJECT_STEPS_SMALL_DOUBLE_01

        println!("{:#?}", entry);

        let mut reader = entry.to_sub_source(&mut file).unwrap();

        let bbm = Bbm::decode(&mut reader).unwrap();

        println!("{:#?}", bbm);

        let entry = big.entries.entries.get(305).unwrap(); // MESH_OBJECT_STEPS_SMALL_DOUBLE_01[PHYSICS]

        println!("{:#?}", entry);

        let mut reader = entry.to_sub_source(&mut file).unwrap();

        let bbm = Bbm::decode(&mut reader).unwrap();

        println!("{:#?}", bbm);
    }
}
