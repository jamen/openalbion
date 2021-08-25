mod renderer;
mod state;

use glam::{Quat, Vec3};
use renderer::*;
use state::*;

use std::f32::consts::PI;

use winit::{event::{DeviceEvent, ElementState, Event, KeyboardInput, MouseScrollDelta, WindowEvent}, event_loop::{ControlFlow, EventLoop}, window::WindowBuilder};

use glam::Mat3;

use native_dialog::FileDialog;

// use std::error::Error;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    env_logger::init();

    // TODO: Load settings file.

    // Select Fable directory

    // TODO: Store and load directory from settings file, verifying that the directory exists and Fable.exe is inside it, and falling back to the prompt if not.

    let fable_dir = FileDialog::new().show_open_single_dir().unwrap();
    let fable_dir = if let Some(d) = fable_dir { d } else { return };

    // TODO: Verify Fable.exe is in the selected directory.

    // Initialize window

    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("Open Albion")
        .with_inner_size(winit::dpi::LogicalSize::new(1024.0, 768.0))
        // TODO: .with_fullscreen(Some(Fullscreen::Borderless(event_loop.primary_monitor())))
        .with_resizable(true)
        .with_visible(false)
        .build(&event_loop)
        .unwrap();

    let mut state = State::new(fable_dir);

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
