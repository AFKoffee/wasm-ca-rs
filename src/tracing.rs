use std::sync::LazyLock;

use js_sys::{Array, Uint8Array, WebAssembly::{self}};
use wasm_bindgen::{prelude::wasm_bindgen, JsCast, JsValue};
use web_sys::DedicatedWorkerGlobalScope;

use crate::thread::{message::WorkerMessage, worker_handle::WorkerHandle};

static TRACING_WASM_URL: LazyLock<String> = LazyLock::new(|| {
    let js = include_bytes!("wasm_tracing_bg.wasm");
    let options = web_sys::BlobPropertyBag::new();
    options.set_type("application/wasm");
    let blob = web_sys::Blob::new_with_u8_slice_sequence_and_options(
        Array::from_iter([Uint8Array::from(js.as_slice())]).as_ref(),
        &options,
    )
    .unwrap();

    web_sys::Url::create_object_url_with_blob(&blob).unwrap()
});

static TRACING_WBG_URL: LazyLock<String> = LazyLock::new(|| {
    let js = include_str!("wasm_tracing.js");
    let options = web_sys::BlobPropertyBag::new();
    options.set_type("application/javascript");
    let blob = web_sys::Blob::new_with_u8_slice_sequence_and_options(
        Array::from_iter([Uint8Array::from(js.as_bytes())]).as_ref(),
        &options,
    )
    .unwrap();

    web_sys::Url::create_object_url_with_blob(&blob).unwrap()
});

#[wasm_bindgen(module = "/tracing.js")]
extern "C" {
    async fn setup_tracing_internal(wasm_url: String, bindgen_url: String, memory: JsValue);
    async fn initialize_tracing_internal(wasm_url: String, bindgen_url: String);
    fn get_tracing_memory_internal() -> WebAssembly::Memory;
    fn generate_binary_trace() -> Uint8Array;
}

pub fn get_tracing_memory() -> JsValue {
    JsValue::from(get_tracing_memory_internal())
}

#[wasm_bindgen]
pub async fn setup_tracing(memory: JsValue) {
    setup_tracing_internal(TRACING_WASM_URL.clone(), TRACING_WBG_URL.clone(), memory).await
}

#[wasm_bindgen]
pub async fn initialize_tracing() {
    initialize_tracing_internal(TRACING_WASM_URL.clone(), TRACING_WBG_URL.clone()).await
}

#[wasm_bindgen]
pub fn generate_trace_download_url(callback: js_sys::Function) -> Result<(), JsValue> {
    let mut worker = WorkerHandle::spawn().map_err(|e| JsValue::from_str(&format!("{e}")))?;
    worker.set_onmessage(callback);
    worker.run(move || {
        let options = web_sys::BlobPropertyBag::new();
        options.set_type("application/octet-stream");
        let blob = web_sys::Blob::new_with_u8_slice_sequence_and_options(
            Array::from_iter([generate_binary_trace()]).as_ref(),
            &options,
        )
        .unwrap();

        let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();
        
        // This is fine as we are guaranteed to be in a worker by implementation
        let global = js_sys::global().unchecked_into::<DedicatedWorkerGlobalScope>();
        let _ = global.post_message(&WorkerMessage::Url { url }.try_to_js().unwrap());
    }).map_err(|e| JsValue::from_str(&format!("{e}")))?;

    Ok(())
}