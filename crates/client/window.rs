use crate::{game::GameHandle, render::RenderHandle};
use std::sync::{
    mpsc::{Receiver, Sender},
    Arc, LockResult, Mutex, MutexGuard,
};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub type WindowRef = Arc<Window>;
pub type EventSender<T> = Sender<Event<'static, T>>;
pub type EventReceiver<T> = Receiver<Event<'static, T>>;

pub fn new() -> Result<(EventLoop<()>, WindowRef), winit::error::OsError> {
    let event_loop = EventLoop::new();

    let window_ref = WindowRef::new(
        WindowBuilder::new()
            .with_inner_size(LogicalSize::<u32>::from([1024, 768]))
            .with_visible(true)
            .with_title("OpenAlbion")
            .with_resizable(false)
            .build(&event_loop)
            .unwrap(),
    );

    Ok((event_loop, window_ref))
}

pub struct WindowSystemParams {
    pub event_loop: EventLoop<()>,
    pub control_flow: ControlFlowRef,
    pub event_sender: EventSender<()>,
    pub render_handle: RenderHandle,
    pub game_handle: GameHandle,
}

pub struct WindowSystem {
    event_loop: EventLoop<()>,
    control_flow: ControlFlowRef,
    event_sender: EventSender<()>,
    render_handle: RenderHandle,
    game_handle: GameHandle,
}

impl WindowSystem {
    pub fn start(params: WindowSystemParams) -> ! {
        Self::new(params).run()
    }

    fn new(params: WindowSystemParams) -> Self {
        Self {
            event_loop: params.event_loop,
            control_flow: params.control_flow,
            event_sender: params.event_sender,
            render_handle: params.render_handle,
            game_handle: params.game_handle,
        }
    }

    fn run(self) -> ! {
        // Start event loop
        self.event_loop
            .run(move |event, _target, control_flow_target| {
                {
                    let mut control_flow = self.control_flow.lock().unwrap();

                    if self.render_handle.as_ref().is_finished()
                        || self.game_handle.as_ref().is_finished()
                    {
                        *control_flow = ControlFlow::Exit;
                    }

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
                            if let Err(err) = self.event_sender.send(static_event) {
                                log::error!("event_sender error: {:?}", err);
                            }
                        }
                    }
                }
            })
    }
}

#[derive(Clone)]
pub struct ControlFlowRef {
    inner: Arc<Mutex<ControlFlow>>,
}

impl Default for ControlFlowRef {
    fn default() -> Self {
        Self::new(ControlFlow::Poll)
    }
}

impl ControlFlowRef {
    pub fn new(c: ControlFlow) -> Self {
        Self {
            inner: Arc::new(Mutex::new(c)),
        }
    }
    pub fn lock(&self) -> LockResult<MutexGuard<ControlFlow>> {
        self.inner.lock()
    }
}
