use std::fs::File;
use std::path::{Path,PathBuf};
use std::io::{Cursor,Read};

use ember::{App,Context,Config};
// use ember::winit::event::{Event,WindowEvent,KeyboardInput,VirtualKeyCode,ElementState};
// use ember::winit::event_loop::ControlFlow;

use fable_data::{Wad,Big,Decode,Entry,Lev};
// use fable_data::Lev;

pub struct OpenAlbion {
    pub fable_path: PathBuf,
}

impl App for OpenAlbion {
    fn init(self, context: Context) {
        let mut wad_path = self.fable_path.clone();

        wad_path.push("Data/Levels/FinalAlbion.wad");

        let mut wad_file = File::open(wad_path).unwrap();

        let mut wad = Wad::decode(&mut wad_file).unwrap();

        // println!("wad {:#?}", wad);

        let mut entry = wad.entries
            .into_iter()
            .find(|entry| Path::new(&entry.path).file_stem().unwrap() == "LookoutPoint")
            .unwrap();

        let mut lev_reader = entry.reader(&mut wad_file).unwrap();
        let mut lev_data = Vec::new();

        lev_reader.read_to_end(&mut lev_data);

        let mut lev_source = Cursor::new(lev_data);

        let lev = Lev::decode(&mut lev_source).unwrap();

        println!("lev entry {:#?}", entry);

        // let mut big_path = self.fable_path.clone();

        // big_path.push("Data/graphics/pc/frontend.big");

        // let mut big_file = File::open(big_path).unwrap();

        // let mut big = Big::decode(&mut big_file).unwrap();

        // let mut big_entry = big.entries.entries.get_mut(0).unwrap();

        // println!("big entry {:#?}", big_entry);

        // let mut big_entry_reader = big_entry.reader(&mut big_file).unwrap();
        // let mut big_entry_data = Vec::new();

        // big_entry_reader.read_to_end(&mut big_entry_data);

        // println!("big entry data {:?}", big_entry_data);

        loop {
            context.window_update_sender.send(ember::Message::Test);
            // std::thread::sleep_ms(1000);
        }
    }
}

fn main() {
    let mut args = pico_args::Arguments::from_env();

    let fable_path_str: Option<String> = args.opt_value_from_str("--fable").unwrap();

    let fable_path = match fable_path_str {
        Some(path) => PathBuf::from(path),
        None => std::env::current_dir().unwrap(),
    };

    let game = OpenAlbion {
        fable_path: fable_path,
    };

    game.start(
        Config {
            title: "Open Albion".to_string(),
            width: 1024,
            height: 768,
            resources: std::env::current_dir().unwrap().join("out"),
        }
    )
}

