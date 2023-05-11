use std::sync::{mpsc::Sender, Arc, LockResult, Mutex, MutexGuard};
use thiserror::Error;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

#[derive(Error, Debug)]
pub enum NewWindowError {
    #[error("winit error. {0}")]
    WinitError(#[from] winit::error::OsError),
}

pub fn new() -> Result<(EventLoop<()>, Arc<Window>), NewWindowError> {
    let event_loop = EventLoop::new();

    let window_ref = Arc::new(
        WindowBuilder::new()
            .with_inner_size(LogicalSize::<u32>::from([1024, 768]))
            .with_visible(true)
            .with_title("OpenAlbion")
            .with_resizable(false)
            .build(&event_loop)?,
    );

    Ok((event_loop, window_ref))
}

pub struct WindowSystemParams {
    pub event_loop: EventLoop<()>,
    pub control_flow: SharedControlFlow,
    pub event_sender: Sender<Event<'static, ()>>,
}

pub fn spawn(params: WindowSystemParams) {
    let WindowSystemParams {
        event_loop,
        control_flow,
        event_sender,
    } = params;

    // Start event loop
    event_loop.run(move |event, _target, control_flow_target| {
        {
            let control_flow = control_flow.lock().unwrap();
            *control_flow_target = *control_flow;
        }

        match event {
            // Skipped events
            Event::RedrawEventsCleared
            | Event::MainEventsCleared
            | Event::NewEvents(winit::event::StartCause::Poll) => {}

            // Thread unsafe event
            // TODO: Properly handle scale factor changes from here
            Event::WindowEvent {
                event: WindowEvent::ScaleFactorChanged { .. },
                ..
            } => {}

            // Send all other events over event channel
            event => {
                if let Some(static_event) = event.to_static() {
                    if let Err(err) = event_sender.send(static_event) {
                        log::error!("event_sender error: {:?}", err);
                    }
                }
            }
        }
    })
}

#[derive(Clone)]
pub struct SharedControlFlow {
    inner: Arc<Mutex<ControlFlow>>,
}

impl Default for SharedControlFlow {
    fn default() -> Self {
        Self::new(ControlFlow::Poll)
    }
}

impl SharedControlFlow {
    pub fn new(c: ControlFlow) -> Self {
        Self {
            inner: Arc::new(Mutex::new(c)),
        }
    }
    pub fn lock(&self) -> LockResult<MutexGuard<ControlFlow>> {
        self.inner.lock()
    }
}
