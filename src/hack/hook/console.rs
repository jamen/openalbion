use winapi::shared::d3d9::*;
use winapi::shared::d3d9types::*;
use winapi::shared::minwindef::*;
use winapi::um::*;

use winnt::*;
use memoryapi::*;

use std::ptr::null_mut;
use std::mem;
use std::os::raw::c_void;

pub use super::{Hook,HACK,HackError};

pub struct HookConsole;

static mut ORIGINAL_END_SCENE: *mut unsafe fn(&IDirect3DDevice9) -> HRESULT = null_mut();

impl Hook for HookConsole {
    unsafe fn enable() -> Result<(), HackError> {
        let d3d: *mut IDirect3D9 = Direct3DCreate9(D3D_SDK_VERSION);

        if d3d.is_null() {
            return Err(HackError::D3D9ContextUnavailable)
        }

        let mut dummy_device: *mut IDirect3DDevice9 = null_mut();
        let mut d3dpp: D3DPRESENT_PARAMETERS = Default::default();
        d3dpp.Windowed = 1; // NOTE: This can produce error.
        d3dpp.SwapEffect = D3DSWAPEFFECT_DISCARD;
        d3dpp.hDeviceWindow = HACK.hwnd.unwrap();

        let r = (*d3d).CreateDevice(D3DADAPTER_DEFAULT, D3DDEVTYPE_HAL, d3dpp.hDeviceWindow, D3DCREATE_HARDWARE_VERTEXPROCESSING, &mut d3dpp as *mut D3DPRESENT_PARAMETERS, &mut dummy_device as *mut *mut IDirect3DDevice9);
        println!("createdevice {:x}", r);

        println!("dummy_device {:p}", dummy_device);



        // let table: [*mut c_void; 119] = [null_mut(); 119];

        let table_ptr = mem::transmute::<*mut IDirect3DDevice9, *mut *mut *mut c_void>(dummy_device);

        println!("table_ptr {:p}", table_ptr);

        let entry_ptr = *(*table_ptr).offset(42);

        ORIGINAL_END_SCENE = entry_ptr as *mut unsafe fn(&IDirect3DDevice9) -> HRESULT;

        println!("entry_ptr {:p}", entry_ptr);

        let mut mbi: MEMORY_BASIC_INFORMATION = Default::default();

        VirtualQuery(entry_ptr as LPVOID, &mut mbi, mem::size_of::<MEMORY_BASIC_INFORMATION>());
        VirtualProtect(mbi.BaseAddress, mbi.RegionSize, PAGE_READWRITE, &mut mbi.Protect);
        let protection = mbi.Protect;

        *(*table_ptr).offset(42) = &mut EndSceneHook as *mut _ as *mut c_void;

        VirtualProtect(mbi.BaseAddress, mbi.RegionSize, protection, &mut mbi.Protect);

        println!("new vtable entry_ptr {:p}", *(*table_ptr).offset(42));

        // let table = dummy_device as *mut *mut c_void;

        // let entry = table.offset(42); // IDirect3DDevice9::EndScene

        // ORIGINAL_END_SCENE = entry as *mut unsafe fn(&IDirect3DDevice9) -> HRESULT;

        // println!("ORIGINAL_END_SCENE {:p}", ORIGINAL_END_SCENE);

        // let mut mbi: MEMORY_BASIC_INFORMATION = Default::default();

        // VirtualQuery(entry as LPVOID, &mut mbi, mem::size_of::<MEMORY_BASIC_INFORMATION>());
        // VirtualProtect(mbi.BaseAddress, mbi.RegionSize, PAGE_READWRITE, &mut mbi.Protect);
        // let protection = mbi.Protect;

        // *entry = &mut EndSceneHook as *mut _ as *mut c_void;

        // VirtualProtect(mbi.BaseAddress, mbi.RegionSize, protection, &mut mbi.Protect);





        // let table = *mem::transmute::<*mut IDirect3DDevice9, *mut *mut *mut c_void>(dummy_device);
        // let mut table = mem::transmute_copy::<_, [*mut c_void; 119]>(&table);

        // ORIGINAL_END_SCENE = table[42] as *mut _ as *mut unsafe fn(&IDirect3DDevice9) -> HRESULT;

        // table[42] = &mut EndSceneHook as *mut _ as *mut c_void;

        Ok(())
    }
}

pub unsafe fn EndSceneHook(device: &IDirect3DDevice9) -> HRESULT {
    let rect = D3DRECT { x1: 0, y1: 0, x2: 100, y2: 100 };

    device.Clear(
        1,
        &rect,
        D3DCLEAR_TARGET,
        D3DCOLOR_XRGB(0, 0, 0),
        0.0,
        0,
    );

    (*ORIGINAL_END_SCENE)(device)
}