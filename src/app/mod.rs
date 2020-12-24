mod renderer;

pub use renderer::*;

use std::io::BufReader;
use std::fs::File;
use std::path::PathBuf;
// use std::num::NonZeroUsize;
// use std::time::SystemTime;

use winit::window::WindowBuilder;
use winit::event_loop::{EventLoop,ControlFlow};
use winit::event::{Event,WindowEvent,KeyboardInput,VirtualKeyCode,ElementState};

use crate::format::{Decode,Big,Bba};

pub struct App {
    pub fable_directory: PathBuf,
    pub graphics: Big,
    pub selected_mesh: Option<Bba>,
    pub errors: Vec<String>,
}

impl App {
    pub fn run(fable_directory: Option<PathBuf>) -> ! {
        let errors: Vec<String> = Vec::new();

        let fable_directory = match fable_directory {
            Some(x) => x,
            None => {
                match nfd::open_dialog(None, None, nfd::DialogType::PickFolder) {
                    Ok(response) => {
                        match response {
                            nfd::Response::Okay(path) => {
                                PathBuf::from(path)
                            }
                            nfd::Response::OkayMultiple(_) => {
                                panic!("Selected multiple files");
                            }
                            nfd::Response::Cancel => {
                                panic!("Did not select file");
                            }
                        }
                    },
                    Err(err) => {
                        panic!("{:?}", err);
                    }
                }
            }
        };

        let event_loop = EventLoop::new();

        let window = WindowBuilder::new()
            .with_title("Open Albion")
            .with_inner_size(winit::dpi::LogicalSize::new(1024.0, 768.0))
            // .with_fullscreen(Some(Fullscreen::Borderless(event_loop.primary_monitor()))) // TODO: Allow windowed later.
            .with_resizable(false) // FIXME
            .with_visible(false) // NOTE: Revealed later.
            .build(&event_loop)
            .unwrap();

        let mut graphics_file = BufReader::new(File::open(fable_directory.join("data/graphics/graphics.big")).unwrap());

        let graphics = Big::decode(&mut graphics_file).unwrap();

        let app = App {
            fable_directory,
            graphics,
            selected_mesh: None,
            errors,
        };

        // let graphics = Resource::new(graphics_file, Some(graphics_data));

        let mut renderer = Renderer::new(window);

        // TODO: Audio?

        // println!("{:#?}", big.entries.entries.iter().enumerate().map(|(i, x)| (i, x.symbol_name.as_str())).collect::<Vec<(usize, &str)>>());

        // MESH_OBJECT_BARREL
        // let x = big.entries.entries.get(168);

        // println!("{:?}", x);

        renderer.render(&app);

        renderer.window.set_visible(true);

        event_loop.run(move |event, _, control_flow| {
            match event {
                Event::WindowEvent { event: WindowEvent::KeyboardInput { input: KeyboardInput { virtual_keycode: Some(VirtualKeyCode::Escape), state: ElementState::Pressed, .. }, .. }, .. } |
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                    *control_flow = ControlFlow::Exit;
                },
                Event::MainEventsCleared => {
                    renderer.window.request_redraw()
                },
                Event::RedrawEventsCleared => {
                    renderer.render(&app);
                },
                _ => {}
            }
        })
    }
}
