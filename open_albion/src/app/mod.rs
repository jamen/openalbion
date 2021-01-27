use crate::renderer::Renderer;

use std::io::BufReader;
use std::fs::File;
use std::path::PathBuf;
// use std::num::NonZeroUsize;
// use std::time::SystemTime;
use std::collections::HashMap;
use std::env;

use winit::window::{Window,WindowBuilder};
use winit::event_loop::{EventLoop,ControlFlow};
use winit::event::{Event,WindowEvent,KeyboardInput,VirtualKeyCode,ElementState,DeviceEvent,ModifiersState};

use fable_data::{Decode,Wad,Lev};

pub struct App {
    pub target_directory: PathBuf,
    pub window: Window,
    pub event_loop: EventLoop<()>,
    pub state: State,
    pub renderer: Renderer,
}

pub struct State {
    pub wad: Wad,
    pub lev: Lev,
    // pub graphics: Big,
    // pub selected_mesh: Option<Bba>,
    pub errors: Vec<String>,
    pub paused: bool,
    pub console: bool,
    pub keys: HashMap<VirtualKeyCode, usize>,
    pub key_modifiers: ModifiersState,
}

impl App {
    pub fn init() -> Self {
        let errors: Vec<String> = Vec::new();

        let args: Vec<String> = env::args().skip(1).collect();

        let target_directory =
            args.get(0)
            .map(|x| PathBuf::from(x))
            .unwrap_or_else(|| {
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
            });

        let event_loop = EventLoop::new();

        let window = WindowBuilder::new()
            .with_title("Open Albion")
            .with_inner_size(winit::dpi::LogicalSize::new(1024.0, 768.0))
            // .with_fullscreen(Some(Fullscreen::Borderless(event_loop.primary_monitor()))) // TODO: Allow windowed later.
            .with_resizable(false) // FIXME
            .with_visible(false) // NOTE: Revealed later.
            .build(&event_loop)
            .unwrap();

        // let mut graphics_file = BufReader::new(File::open(fable_directory.join("data/graphics/graphics.big")).unwrap());

        // let graphics = Big::decode(&mut graphics_file).unwrap();

        let mut wad_file = BufReader::new(File::open(target_directory.join("data/Levels/FinalAlbion.wad")).unwrap());

        let wad = Wad::decode(&mut wad_file).unwrap();

        // println!("{:#?}", wad);

        let lev_entry = wad.entries.iter().find(|x| x.path.contains("LookoutPoint.lev")).unwrap();

        let mut lev_source = lev_entry.source(&mut wad_file).unwrap();

        let lev = Lev::decode(&mut lev_source).unwrap();

        let state = State {
            wad,
            lev,
            // graphics,
            // selected_mesh: None,
            errors,
            paused: false,
            console: false,
            keys: HashMap::default(),
            key_modifiers: ModifiersState::empty(),
        };

        let renderer = Renderer::new(&window);

        App {
            target_directory,
            window,
            event_loop,
            state,
            renderer,
        }
    }

    pub fn run(self) -> ! {
        let Self { event_loop, window, mut state, .. } = self;

        // TODO: Better error handling lol
        window.set_cursor_grab(true).unwrap();
        window.set_cursor_visible(false);

        // let graphics = Resource::new(graphics_file, Some(graphics_data));

        // TODO: Audio?

        // println!("{:#?}", big.entries.entries.iter().enumerate().map(|(i, x)| (i, x.symbol_name.as_str())).collect::<Vec<(usize, &str)>>());

        // MESH_OBJECT_BARREL
        // let x = big.entries.entries.get(168);

        // println!("{:?}", x);

        // renderer.camera.position = Vec3::new(60.0, 80.0, 60.0);
        // renderer.camera.pitch = -90f32.to_radians();
        // renderer.camera.angle = Quat::from_rotation_ypr((-90.0f32).to_radians(), (-90.0f32).to_radians(), 0.0);

        // renderer.render(&app);

        window.set_visible(true);

        event_loop.run(move |event, _, control_flow| {
            // state.keys.iter_mut().for_each(|x| *x.1 += 1);

            match event {
                Event::WindowEvent { event: window_event, .. } => {
                    match window_event {
                        WindowEvent::KeyboardInput { input: KeyboardInput { virtual_keycode: Some(virtual_keycode), state: element_state, .. }, .. } => {
                            match element_state {
                                ElementState::Pressed => {
                                    // if !state.keys.contains_key(&virtual_keycode) {
                                    //     state.keys.insert(virtual_keycode, 0usize);
                                    // }

                                    // match self.state.keys.get_mut(&virtual_keycode) {
                                    //     Some(frame_count) => {
                                    //         *frame_count += 1;
                                    //     },
                                    //     None => {
                                    //         self.state.keys.insert(virtual_keycode, 0);
                                    //     },
                                    // };
                                },
                                ElementState::Released => {
                                    state.keys.remove(&virtual_keycode);
                                },
                            }
                        },
                        WindowEvent::ModifiersChanged(modifiers) => {
                        },
                        WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::Exit;
                        },
                        _ => {}
                    }
                },
                Event::DeviceEvent { event: device_event, .. } => {
                    match device_event {
                        DeviceEvent::MouseMotion { delta } => {
                            if !state.paused {
                                // let sens = 0.003;

                                // renderer.camera.yaw =
                                    // (renderer.camera.yaw + 2.0 * PI * delta.0 as f32 * sens) % (2.0 * PI);

                                // renderer.camera.pitch =
                                    // (renderer.camera.pitch + 2.0 * PI * delta.1 as f32 * sens)
                                    // .min(PI)
                                    // .max(FRAC_PI_2);

                                // TODO: BEtter error handling
                                // window.set_cursor_position(Position::new(PhysicalPosition::new(renderer.size.width / 2, renderer.size.height / 2))).unwrap();
                            }
                        },
                        _ => {}
                    }
                }
                Event::MainEventsCleared => {
                    window.request_redraw()
                },
                Event::RedrawEventsCleared => {
                    // renderer.render(&app);
                },
                _ => {}
            }

            // self.update(&mut renderer, &mut window);
        })
    }

    // fn update(&mut self, renderer: &mut Renderer, window: &mut Window) {
    //     // println!("{:?}", self.keys);

    //     if self.keys.get(&VirtualKeyCode::Escape) == Some(&1usize) {
    //         self.paused = !self.paused;
    //         window.set_cursor_grab(!self.paused).unwrap();
    //         window.set_cursor_visible(self.paused);
    //     }

    //     if !self.paused {
    //         // renderer.camera.velocity.y =
    //         //     (self.keys.contains_key(&VirtualKeyCode::Space) && self.key_modifiers.shift()) as u8 as f32 +
    //         //     (self.keys.contains_key(&VirtualKeyCode::Space) && !self.key_modifiers.shift()) as u8 as f32 * -1.0;

    //         // renderer.camera.velocity.x =
    //         //     self.keys.contains_key(&VirtualKeyCode::A) as u8 as f32 +
    //         //     self.keys.contains_key(&VirtualKeyCode::D) as u8 as f32 * -1.0;

    //         // renderer.camera.velocity.z =
    //         //     self.keys.contains_key(&VirtualKeyCode::W) as u8 as f32 +
    //         //     self.keys.contains_key(&VirtualKeyCode::S) as u8 as f32 * -1.0;
    //     }
    // }
}
