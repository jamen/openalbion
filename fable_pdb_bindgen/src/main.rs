use clap::{App,Arg};

use pdb::{PDB,TypeIndex,TypeData,PrimitiveKind};

use std::fs::File;
use std::collections::BTreeSet;

fn main() {
    let matches = App::new("pdb_bindgen")
        .author("Jamen Marz <me@jamen.dev>")
        .about("This tool generates Rust bindings from a Microsoft PDB file")
        .arg(
            Arg::with_name("pdb")
            .takes_value(true)
            .required(true)
        )
        .get_matches();

    let pdb_path = matches.value_of("pdb").unwrap();

    //
    // PDB data
    //

    let pdb_file = File::open(pdb_path).expect("Failed to open file.");
    let mut pdb = PDB::open(file).expect("Failed to parse PDB.");

    let type_information = pdb.type_information().expect("Failed to get type information.");
    let type_finder = type_information.finder();

    let needed_types: BTreeSet<TypeIndex> = Default::default();

    let forward_references: Vec<ForwardReference> = Vec::new();
    let classes: Vec<Class> = Vec::new();
    let enums: Vec<Enum> = Vec::new();

    //
    // Build PDB data
    //

    let type_iter = type_information.iter();

    let add_component = move |type_index: TypeIndex| -> pdb::Result<()> {
        let type_string =
            match type_finder.find(type_index)?.parse()? {
                // From pdb2hpp: https://github.com/willglynn/pdb/blob/b72a5903422d66aeb3a1da6017830bd0d9b0b4e3/examples/pdb2hpp.rs#L14
                TypeData::Primitive(data) => {
                    let mut type_ = match data.kind {
                        pdb::PrimitiveKind::Void => "::std::ffi::c_void".to_string(),
                        pdb::PrimitiveKind::Char => "::std::os::raw::c_char".to_string(),
                        pdb::PrimitiveKind::UChar => "::std::os::raw::c_uchar".to_string(),

                        pdb::PrimitiveKind::I8 => "i8".to_string(),
                        pdb::PrimitiveKind::U8 => "u8".to_string(),
                        pdb::PrimitiveKind::I16 => "i16".to_string(),
                        pdb::PrimitiveKind::U16 => "u16".to_string(),
                        pdb::PrimitiveKind::I32 => "i32".to_string(),
                        pdb::PrimitiveKind::U32 => "u32".to_string(),
                        pdb::PrimitiveKind::I64 => "i64".to_string(),
                        pdb::PrimitiveKind::U64 => "u64".to_string(),

                        pdb::PrimitiveKind::F32 => "f32".to_string(),
                        pdb::PrimitiveKind::F64 => "f64".to_string(),

                        pdb::PrimitiveKind::Bool8 => "bool".to_string(),
                    };

                    if data.indirection.is_some() {
                        type_ = "*mut " + &type_;
                    }

                    type_
                }
                TypeData::Class(data) => {

                }
            }
    };

    while let Some(type_) = type_iter().next().expect("Failed type iteration") {
        type_finder.update(&type_iter);

        if let Ok(TypeData::Class(class)) = type_.parse() {
            if !class.properties.forward_reference() {
                add_component(type_.index()).expect("Failed to add component.");
            }
        }
    }

    let needed_types_iter = needed_types.iter();

    while let Some(type_index) = needed_types_iter.next_back() {
        needed_types.remove(&type_index);
        add_component(type_index).expect("Failed to add component.");
    }

    //
    // Generate bindings from data
    //
}