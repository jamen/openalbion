use std::path::{PathBuf,Component};
use std::fs::{read,write,create_dir_all,File};
use std::io::BufReader;
use std::ffi::OsStr;

use pico_args::Arguments;

use fable_data::{Wad,Big,Lev};

fn main() {
    let mut args = Arguments::from_env();

    let source: String = args.free_from_str().unwrap();
    let source_path = PathBuf::from(source);
    let source_ext = source_path.extension().expect("source has no extension").to_str().unwrap();

    match source_ext {
        "wad" => wad(&mut args, &source_path),
        "big" => big(&mut args, &source_path),
        "lev" => lev(&mut args, &source_path),
        x => panic!("Unknown file given {:?}", x),
    }
}

fn wad(args: &mut Arguments, wad_path: &PathBuf) {
    let mut wad_file = BufReader::new(File::open(wad_path).unwrap());

    let wad = Wad::decode(&mut wad_file).unwrap();

    if args.contains(["-d","--debug"]) {
        println!("{:#?}", wad);
        return
    }

    let wad_name = wad_path.file_stem().unwrap();

    for entry in wad.entries {
        let mut file_data = vec![0; entry.data_size as usize];

        entry.read_from(&mut wad_file, &mut file_data).unwrap();

        let file_path = PathBuf::from(entry.path);

        let file_path_components: Vec<Component<'_>> = file_path
            .components()
            .skip_while(|x| *x != Component::Normal(OsStr::new(wad_name)))
            .skip(1)
            .collect();

        let file_path = if file_path_components.len() > 0 {
            let file_path_relevant: PathBuf = file_path_components.iter().collect();
            wad_path.with_extension("").join(file_path_relevant)
        } else {
            wad_path.parent().unwrap().join(file_path.file_name().unwrap())
        };

        match create_dir_all(file_path.parent().unwrap()) {
            Ok(_) => {}
            Err(x) if x.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(x) => panic!("{}", x),
        };

        write(file_path, file_data).unwrap();
    }
}

fn big(args: &mut Arguments, big_path: &PathBuf) {
    let mut big_file = BufReader::new(File::open(big_path).unwrap());
    let big = Big::decode(&mut big_file).unwrap();

    if args.contains(["-d","--debug"]) {
        println!("{:#?}", big);
        return
    }

    for bank in big.banks {
        for entry in bank.index.entries {
            let mut file_data = vec![0; entry.data_size as usize];

            entry.read_from(&mut big_file, &mut file_data).unwrap();

            let file_path = big_path.with_extension("").join(&bank.path).join(&entry.symbol);

            match create_dir_all(file_path.parent().unwrap()) {
                Ok(_) => {}
                Err(x) if x.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(x) => panic!("{}", x),
            };

            write(&file_path, file_data).unwrap();
        }
    }
}

fn lev(args: &mut Arguments, lev_path: &PathBuf) {
    println!("{:?}", lev_path.file_stem().unwrap());
    let mut lev_file = BufReader::new(File::open(lev_path).unwrap());
    let lev = Lev::decode(&mut lev_file).unwrap();
    // if args.contains(["-d","--debug"]) {
        println!("{:#?}", lev);
    //     return
    // }
}