use super::{Hook,HackError,HACK_CONTEXT};

pub struct HookDirectX;

impl Hook for HookDirectX {
    unsafe fn start() -> Result<(), HackError> {
        Ok(())
    }

    unsafe fn stop() -> Result<(), HackError> {
        Ok(())
    }
}