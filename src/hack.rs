pub mod cli;
pub mod error;
pub mod hook;

pub use error::HackError;
pub use hook::{Hook,HookConsole};

use winapi::shared::windef::*;
use winapi::shared::minwindef::*;
use winapi::um::*;

use winuser::*;

pub struct Hack {
    pub dll_handle: Option<HINSTANCE>,
    pub pid: u32,
    pub hwnd: Option<HWND>,
    pub wnd_proc: Option<WNDPROC>,
}

pub static mut HACK: Hack = Hack {
    dll_handle: None,
    pid: 0,
    hwnd: None,
    wnd_proc: None,
};

pub unsafe fn start() -> Result<(), HackError> {
    // Hooks
    hook::HookConsole::enable()?;
    hook::HookPanel::enable()?;

    // CLI
    cli::start()?;

    Ok(())
}