pub mod cli;
pub mod context;
pub mod hook_directx;
pub mod hook_window;
pub mod error;

pub use hook_directx::HookDirectX;
pub use hook_window::HookWindow;
pub use context::HackContext;
pub use error::HackError;

use context::HACK_CONTEXT;

pub trait Hook {
    unsafe fn start() -> Result<(), HackError>;
    unsafe fn stop() -> Result<(), HackError>;
}

pub unsafe fn start() -> Result<(), HackError> {
    // Hooks
    HookDirectX::start()?;
    HookWindow::start()?;

    // CLI
    cli::start()?;

    Ok(())
}