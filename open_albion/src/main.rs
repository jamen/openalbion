// TODO: There's a bug when this and stdout loggers are used together.
// #![windows_subsystem = "windows"]

// TODO: Check for presence of tlse.dll if functionality is restored (like userst.ini options)

mod renderer;
mod state;

use renderer::Renderer;
use state::State;

use winit::{event::{Event, WindowEvent}, event_loop::{ControlFlow, EventLoop}, window::WindowBuilder};

use native_dialog::FileDialog;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    env_logger::init();

    let fable_dir = match FileDialog::new().show_open_single_dir().unwrap() {
        Some(fable_dir) => fable_dir,
        None => return
    };

    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("Open Albion")
        .with_inner_size(winit::dpi::LogicalSize::new(1024, 768))
        // .with_fullscreen(Some(Fullscreen::Borderless(event_loop.primary_monitor())))
        .with_resizable(true)
        .with_visible(false)
        .build(&event_loop)
        .unwrap();

    let mut state = State::new(&window, fable_dir);

    let mut renderer = Renderer::new(&window, wgpu::Backends::PRIMARY).await;

    renderer.render(&mut state);

    window.set_visible(true);

    event_loop.run(move |event, _, control_flow| {
        state.gui.platform.handle_event(&event);

        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event: window_event, .. } => {
                match window_event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    },
                    WindowEvent::Resized(size)
                        if *renderer.size() != [size.width, size.height].into()
                    => {
                        renderer.reconfigure_surface([size.width, size.height].into());
                    },
                    WindowEvent::ScaleFactorChanged { scale_factor, new_inner_size: size } => {
                        *renderer.scale_factor_mut() = scale_factor as f32;
                        renderer.reconfigure_surface([size.width, size.height].into());
                    },
                    _ => {}
                }
            }
            Event::MainEventsCleared => {
                renderer.render(&mut state);
            }
            _ => {}
        }
    })
}
