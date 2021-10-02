mod renderer;
mod state;

use glam::{Quat, Vec3};
use renderer::*;
use state::*;

use std::fs;
use std::f32::consts::PI;
use std::path::PathBuf;

use winit::{event::{DeviceEvent, ElementState, Event, KeyboardInput, MouseScrollDelta, WindowEvent}, event_loop::{ControlFlow, EventLoop}, window::WindowBuilder};

use glam::Mat3;

use native_dialog::FileDialog;

use serde::{Serialize, Deserialize};

use thiserror::Error;

#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    version: String,
    fable_dir: Option<PathBuf>,
}

#[derive(Error, Debug)]
enum InitError {
    #[error("Failed to read settings.toml.")]
    FailedToReadSettings,
    #[error("Settings file not found.")]
    SettingsNotFound,
    #[error("Config directory not found.")]
    ConfigDirNotFound,
    #[error("Failed to parse settings.toml")]
    FailedToParseSettings,
    #[error("Failed to parse settings.toml at {0}:{1}")]
    FailedToParseSettingsAt(usize, usize),
}

impl Settings {
    fn new() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_owned(),
            fable_dir: None,
        }
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    // TODO: File logger?
    env_logger::init();

    let settings_file = dirs::config_dir()
        .map(|x| x.join(env!("CARGO_PKG_NAME")).join("settings.toml"));

    let settings = match settings_file {
        Some(settings_file) => match settings_file.exists() {
            true => match fs::read(settings_file) {
                Ok(data) => match toml::from_slice::<Settings>(&data) {
                    Ok(mut settings) => {
                        if settings.fable_dir.is_none() {
                            log::debug!("Fable's directory not found. Opening a file dialog.");

                            match FileDialog::new().show_open_single_dir().unwrap() {
                                Some(fable_dir) => settings.fable_dir = Some(fable_dir),
                                None => return,
                            };

                            match toml::to_string_pretty(&settings) {
                                Ok(settings_data) => {
                                    if let Err(_) = fs::write(settings_file, settings_data) {
                                        log::warn!("Failed to write the settings file.");
                                    }
                                },
                                Err(_) => {
                                    log::warn!("Failed to serialize the settings.");
                                }
                            }
                        }

                        Ok(settings)
                    },
                    Err(error) => match error.line_col() {
                        Some((line, col)) => Err(InitError::FailedToParseSettingsAt(line, col)),
                        None => Err(InitError::FailedToParseSettings),
                    }
                },
                Err(_) => Err(InitError::FailedToReadSettings),
            },
            false => Err(InitError::SettingsNotFound),
        },
        None => Err(InitError::ConfigDirNotFound)
    };

    let settings = settings.unwrap_or_else(|err| {
        log::error!("{:?}", err);
        log::debug!("Falling back to default settings.");
        Settings::new()
    });

    let mut state = State::new(&settings).unwrap_or_else(|err| {
        log::error!("Failed to make state from settings: {:?}", err);
        panic!("{:?}", err);
    });

    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("Open Albion")
        .with_inner_size(winit::dpi::LogicalSize::new(1024, 768))
        // .with_fullscreen(Some(Fullscreen::Borderless(event_loop.primary_monitor())))
        .with_resizable(true)
        .with_visible(false)
        .build(&event_loop)
        .unwrap();

    let mut renderer = Renderer::create(&window, &state).await;

    state.update();

    renderer.render(&state);

    window.set_visible(true);

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput { input: KeyboardInput { virtual_keycode, scancode, state: element_state, .. }, .. } => match element_state {
                    ElementState::Pressed => state.input.key_down(virtual_keycode, scancode),
                    ElementState::Released => state.input.key_up(virtual_keycode, scancode),
                },
                WindowEvent::MouseInput { button, state: element_state, .. } => match element_state {
                    ElementState::Pressed => state.input.mouse_down(button),
                    ElementState::Released => state.input.mouse_up(button),
                },
                WindowEvent::ModifiersChanged(modifiers) => state.input.modifiers = modifiers,
                WindowEvent::CursorLeft { .. } => state.input.cursor_position = None,
                // TODO
                // WindowEvent::ScaleFactorChanged { scale_factor, new_inner_size } => { },
                WindowEvent::Resized(size) => {
                    renderer.resize(size.width, size.height);
                },
                WindowEvent::CloseRequested => {
                    // self.exit();
                    *control_flow = ControlFlow::Exit;
                },
                WindowEvent::CursorMoved { position, .. } => {
                    state.input.cursor_position = Some(position)
                },
                _ => {}
            },
            Event::DeviceEvent { event, device_id: _device_id } => match event {
                DeviceEvent::MouseWheel { delta } => match delta {
                    MouseScrollDelta::LineDelta(x, y) => {
                        if state.input.cursor_position.is_some() {
                            state.camera.mouse_wheel((x, y));
                        }
                        // state.camera.pos = (state.camera.pos.normalize() * (x * 5.0));
                    }
                    MouseScrollDelta::PixelDelta(pos) => {
                        if state.input.cursor_position.is_some() {
                            state.camera.mouse_wheel((pos.x as f32, pos.y as f32));
                        }
                        // state.camera.pos = (state.camera.pos.normalize() * (pos.x as f32 * 5.0));
                    }
                },
                DeviceEvent::MouseMotion { delta } => {
                    if state.input.mouse_left.is_some() {
                        state.camera.mouse_motion(delta);
                    }
                },
                _ => {},
            },
            Event::MainEventsCleared => {
                state.update();
                renderer.render(&state);
            }
            _ => {}
        }
    })
}
