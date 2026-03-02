use wasm_bindgen::prelude::*;
use js_sys::{Array};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_rayon::init_thread_pool;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn init_threads(n: usize) -> js_sys::Promise {
    init_thread_pool(n)
}

#[wasm_bindgen(typescript_custom_section)]
const TS_CONSTANTS_BOUNDS: &'static str = r#"
export const BOX_ID_LEFT = -1;
export const BOX_ID_RIGHT = -2;
export const BOX_ID_FRONT = -3;
export const BOX_ID_BACK = -4;
export const BOX_ID_BOTTOM = -5;
export const BOX_ID_TOP = -6;
"#;

#[wasm_bindgen(typescript_custom_section)]
const TS_CONSTANTS_WALL: &'static str = r#"
export const WALL_ID_START = -1000;
"#;

pub fn parse_js_point<const D: usize>(val: &JsValue) -> Option<[f64; D]> {
    let arr = val.dyn_ref::<Array>()?;
    if arr.length() < D as u32 {
        return None;
    }
    let mut point = [0.0; D];
    for i in 0..D {
        point[i] = arr.get(i as u32).as_f64()?;
    }
    Some(point)
}