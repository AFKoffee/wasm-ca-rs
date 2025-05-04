use std::sync::LazyLock;

use js_sys::{Array, Uint8Array};
use parking_lot::Mutex;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(module = "/wasm_ca.js")]
extern "C" {
    #[wasm_bindgen]
    fn get_origin() -> String;
}

static BINDGEN_URL_SUFFIX: Mutex<Option<String>> = Mutex::new(None);

#[wasm_bindgen(js_name = "set_bindgen_url_suffix")]
pub fn set_bindgen_url_suffix_js(suffix: String) {
    *BINDGEN_URL_SUFFIX.lock() = Some(suffix);
}

pub fn get_bindgen_url() -> String {
    let mut url = get_origin();

    url.push_str(BINDGEN_URL_SUFFIX.lock().as_deref().unwrap_or("/index.js"));

    url
}

static WORKER_URL: LazyLock<String> = LazyLock::new(|| {
    let js = include_str!("worker.js");
    let options = web_sys::BlobPropertyBag::new();
    options.set_type("application/javascript");
    let blob = web_sys::Blob::new_with_u8_slice_sequence_and_options(
        Array::from_iter([Uint8Array::from(js.as_bytes())]).as_ref(),
        &options,
    )
    .unwrap();

    web_sys::Url::create_object_url_with_blob(&blob).unwrap()
});

pub fn get_worker_url() -> &'static str {
    &WORKER_URL
}
