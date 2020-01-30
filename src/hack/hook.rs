pub mod console;
pub mod panel;

pub use console::HookConsole;
pub use panel::HookPanel;

pub use super::*;

pub trait Hook {
    unsafe fn enable() -> Result<(), HackError>;
    unsafe fn disable() -> Result<(), HackError> {
        Ok(())
    }
}