use winit::window::Window;

use std::path::PathBuf;

use hecs::World as Ec;

use egui::{FontDefinitions, Frame, Style};
use egui_winit_platform::{Platform, PlatformDescriptor};

pub struct State {
    pub fable_dir: PathBuf,
    pub scene: Scene,
    pub gui: Gui,
}

impl State {
    pub fn new(window: &Window, fable_dir: PathBuf) -> Self {
        Self {
            fable_dir,
            scene: Scene::new(),
            gui: Gui::new(&window),
        }
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
    pub fn update(&mut self) {
        egui::SidePanel::right("outline")
            .resizable(false)
            .default_width(280.0)
            .frame(Frame::none())
            .show(&self.platform.context(), |ui| {
                ui.add(egui::Label::new("Hello World!"));
                ui.label("A shorter and more convenient way to add a label.");
                ui.horizontal(|ui| {
                    ui.label("Add widgets");
                    if ui.button("on the same row!").clicked() {
                        println!("aaa")
                    }
                });
            });

        egui::CentralPanel::default()
            .frame(Frame::none())
            .show(&self.platform.context(), |ui| {
                ui.label("Hello world");
            });
    }
}

pub struct Node {
    content: Option<Content>,
}

pub enum Content {
    Text { text: String },
    Image { width: f32, height: f32 },
    List { children: Vec<Node> },
}
