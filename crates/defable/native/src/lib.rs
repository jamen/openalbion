#[macro_use]
extern crate neon;
extern crate winapi;

use neon::prelude::*;

fn find_process(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    unimplemented!()
}

fn create_process_and_inject_dll(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    unimplemented!();
}

fn inject_dll(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    unimplemented!()
}

fn close_process(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    unimplemented!()
}

register_module!(mut cx, {
    cx.export_function("find_process", find_process);
    cx.export_function("inject_dll", inject_dll);
    cx.export_function("create_process", create_process);
    cx.export_function("close_process", close_process);
    Ok(())
});