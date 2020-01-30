mod directx;
mod window;

pub use directx::DirectXHook;
pub use window::WindowHook;

use super::shared::HACK_CONTEXT;

pub use super::shared::HackContext;

pub trait Hook {
    unsafe fn start(ctx: &mut HackContext) -> Result<(), HookError>;
    unsafe fn stop(ctx: &mut HackContext) -> Result<(), HookError>;
}

#[derive(Debug)]
pub enum HookError {

}

pub unsafe fn start() -> Result<(), HookError> {
    WindowHook::start(&mut HACK_CONTEXT)?;
    DirectXHook::start(&mut HACK_CONTEXT)?;
    Ok(())
}