#[macro_use]
extern crate neon;
extern crate winapi;
extern crate fable_dll_inject;

use neon::prelude::*;
use fable_dll_inject::Injector;

fn create_and_inject(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let executable_path = cx.argument::<JsString>(0)?.value();
    let dll_path = cx.argument::<JsString>(1)?.value();

    let mut injector = Injector::create_process(&executable_path).expect("Failed to create process.");

    injector.inject_dll(&dll_path).expect("Failed to inject dll.");

    Ok(JsUndefined::new())
}

register_module!(mut cx, {
    cx.export_function("create_and_inject", create_and_inject);
    Ok(())
});