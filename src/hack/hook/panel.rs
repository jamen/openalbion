pub use super::{Hook,HackError};

pub struct HookPanel;

impl Hook for HookPanel {
    unsafe fn enable() -> Result<(), HackError> {
        Ok(())
    }
}