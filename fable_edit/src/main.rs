use wasm_bindgen::prelude::*;

use js_sys::Object;

#[wasm_bindgen]
extern "C" {

}

#[wasm_bindgen(module = "fs")]
extern "C" {
    #[wasm_bindgen(js_name = readFileSync)]
    fn read_file_sync(path: &str, options: &Object) -> JsValue;
}

#[wasm_bindgen(start)]
fn main_js() {

}