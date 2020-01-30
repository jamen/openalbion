use winapi::shared::windef::*;
use winapi::shared::minwindef::*;
use winapi::um::*;

use winuser::*;

pub struct HackContext {
    pub dll_handle: Option<HINSTANCE>,
    pub pid: u32,
    pub hwnd: Option<HWND>,
    pub wnd_proc: Option<WNDPROC>,
}

pub static mut HACK_CONTEXT: HackContext = HackContext {
    dll_handle: None,
    pid: 0,
    hwnd: None,
    wnd_proc: None,
};