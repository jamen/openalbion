use super::{Hook,HackError,HACK_CONTEXT};

pub struct HookWindow;

impl Hook for HookWindow {
    unsafe fn start() -> Result<(), HackError> {
        Ok(())
    }

    unsafe fn stop() -> Result<(), HackError> {
        Ok(())
    }
}