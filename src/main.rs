use std::fs::File;
use std::path::PathBuf;
use std::io::Read;

use ember::{Ember,EmberConfig};
use ember::winit::event::{Event,WindowEvent,KeyboardInput,VirtualKeyCode,ElementState};
use ember::winit::event_loop::ControlFlow;

use fable_data::{Wad,Big,Decode,Entry};
// use fable_data::Lev;

pub struct Game {
    pub fable_path: PathBuf,
}

fn main() {
    let mut args = pico_args::Arguments::from_env();

    let fable_path_str: Option<String> = args.opt_value_from_str("--fable").unwrap();

    let fable_path = match fable_path_str {
        Some(path) => PathBuf::from(path),
        None => std::env::current_dir().unwrap(),
    };

    let game = Game {
        fable_path: fable_path,
    };

    // let mut wad_path = game.fable_path.clone();

    // wad_path.push("Data/Levels/FinalAlbion.wad");

    // let mut wad_file = File::open(wad_path).unwrap();

    // let wad = Wad::decode(&mut wad_file);

    // println!("wad {:#?}", wad);

    let mut big_path = game.fable_path.clone();

    big_path.push("Data/graphics/pc/frontend.big");

    let mut big_file = File::open(big_path).unwrap();

    let mut big = Big::decode(&mut big_file).unwrap();

    let mut big_entry = big.entries.entries.get_mut(0).unwrap();

    println!("big entry {:#?}", big_entry);

    let mut big_entry_reader = big_entry.reader(&mut big_file).unwrap();
    let mut big_entry_data = Vec::new();

    big_entry_reader.read_to_end(&mut big_entry_data);

    // println!("big entry data {:?}", big_entry_data);



    let config = EmberConfig {
        title: "Open Albion".to_string(),
        width: 1024,
        height: 768,
        resources: std::env::current_dir().unwrap().join("out"),
    };

    let Ember {
        winit_window,
        winit_event_loop,
        ..
    } = Ember::new(config).expect("Failed to initialize app.");

    winit_event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent { event, .. } => {
                // println!("{:?}", event);
                match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::KeyboardInput { input: KeyboardInput { virtual_keycode, state, .. }, .. } => {
                        match (virtual_keycode, state) {
                            (Some(VirtualKeyCode::Escape), ElementState::Pressed) => {
                                *control_flow = ControlFlow::Exit;
                            },
                            _key => {}
                        }
                    },
                    _window_event => {}
                }
            },
            Event::MainEventsCleared => {
                winit_window.request_redraw()
            },
            Event::RedrawRequested(_window_id) => {
                // TODO: Draw
            },
            _event => {}
        }
    })
}

