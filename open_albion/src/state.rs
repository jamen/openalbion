use winit::window::Window;

use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use hecs::World as Ec;

use egui::{FontDefinitions, Frame};
use egui_winit_platform::{Platform, PlatformDescriptor};

pub struct State {
    pub fable_dir: PathBuf,
    pub graphics_data: GraphicsData,
    pub scene: Scene,
    pub gui: Gui,
}

impl State {
    pub fn new(window: &Window, fable_dir: PathBuf) -> Self {
        let graphics_data = GraphicsData::new(fable_dir.as_path());
        let scene = Scene::new();
        let gui = Gui::new(&window);

        let wld_data =
            std::fs::read_to_string(fable_dir.join("data/Levels/FinalAlbion.wld")).unwrap();

        let wld = fable_data::Wld::parse(&wld_data).unwrap();

        println!("{:#?}", wld);

        Self {
            fable_dir,
            graphics_data,
            scene,
            gui,
        }
    }
}

pub struct GraphicsData {
    graphics_big: fable_data::BigHeader,
}

impl GraphicsData {
    // TODO: Make async, better error handling
    pub fn new(fable_dir: &Path) -> Self {
        let source = std::fs::read(fable_dir.join("data/graphics/graphics.big")).unwrap();
        let graphics_big = fable_data::BigHeader::parse(&mut &source[..]).unwrap();
        Self { graphics_big }
    }
}

pub struct Scene {
    objects: Ec,
}

impl Scene {
    pub fn new() -> Self {
        Self { objects: Ec::new() }
    }
}

pub struct Gui {
    pub platform: egui_winit_platform::Platform,
}

impl Gui {
    pub fn new(window: &Window) -> Self {
        let window_size = window.inner_size();

        let platform = Platform::new(PlatformDescriptor {
            physical_width: window_size.width as u32,
            physical_height: window_size.height as u32,
            scale_factor: window.scale_factor(),
            font_definitions: FontDefinitions::default(),
            style: Default::default(),
        });

        Self { platform }
    }
    pub fn update(state: &mut State) {
        egui::SidePanel::right("outline")
            .resizable(false)
            .min_width(280.0)
            .frame(Frame::none())
            .show(&state.gui.platform.context(), |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.collapsing("data/graphics/graphics.big", |ui| {
                        // for entry in state.graphics_data.graphics_big.entries.iter() {
                        //     for source in &entry.sources {
                        //         let btn = ui.add(egui::widgets::Button::new(source).wrap(false));
                        //         // println!("{:?}", &btn.rect);
                        //         if btn.clicked() {}
                        //     }
                        // }
                    })
                })
            });

        egui::CentralPanel::default().frame(Frame::none()).show(
            &state.gui.platform.context(),
            |ui| {
                ui.label("central panel");
            },
        );
    }
}
