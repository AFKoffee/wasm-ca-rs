use std::sync::LazyLock;

use js_sys::{Array, Uint8Array, WebAssembly::{self}};
use wasm_bindgen::{prelude::wasm_bindgen, JsCast, JsValue};
use web_sys::DedicatedWorkerGlobalScope;

use crate::thread::{self, message::WorkerMessage, worker_handle::WorkerHandle};

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
    async fn setup_tracing_internal(wasm_url: String, bindgen_url: String, tid: u32, memory: JsValue);
    async fn initialize_tracing_internal(wasm_url: String, bindgen_url: String, tid: u32);
    fn get_tracing_memory_internal() -> WebAssembly::Memory;
    fn generate_binary_trace() -> Uint8Array;

    fn read_event(addr: usize, n: usize, fidx: usize, iidx: usize);
    fn write_event(addr: usize, n: usize, fidx: usize, iidx: usize);
    fn aquire_event(lock_id: usize, fidx: usize, iidx: usize);
    fn request_event(lock_id: usize, fidx: usize, iidx: usize);
    fn release_event(lock_id: usize, fidx: usize, iidx: usize);
    fn fork_event(thread_id: u32, fidx: usize, iidx: usize);
    fn join_event(thread_id: u32, fidx: usize, iidx: usize);
}

#[no_mangle]
pub fn read_hook(addr: usize, n: usize, fidx: usize, iidx: usize) {
    read_event(addr, n, fidx, iidx);
}

#[no_mangle]
pub fn write_hook(addr: usize, n: usize, fidx: usize, iidx: usize) {
    write_event(addr, n, fidx, iidx);
}

#[no_mangle]
pub fn aquire_hook(lock_id: usize, fidx: usize, iidx: usize) {
    aquire_event(lock_id, fidx, iidx);
}

#[no_mangle]
pub fn request_hook(lock_id: usize, fidx: usize, iidx: usize) {
    request_event(lock_id, fidx, iidx);
}

#[no_mangle]
pub fn release_hook(lock_id: usize, fidx: usize, iidx: usize) {
    release_event(lock_id, fidx, iidx);
}

#[no_mangle]
pub fn fork_hook(thread_id: u32, fidx: usize, iidx: usize) {
    fork_event(thread_id, fidx, iidx);
}

#[no_mangle]
pub fn join_hook(thread_id: u32, fidx: usize, iidx: usize) {
    join_event(thread_id, fidx, iidx);
}

pub fn get_tracing_memory() -> JsValue {
    JsValue::from(get_tracing_memory_internal())
}

#[wasm_bindgen]
pub async fn setup_tracing(memory: JsValue) {
    setup_tracing_internal(TRACING_WASM_URL.clone(), TRACING_WBG_URL.clone(), thread::thread_id(), memory).await
}

#[wasm_bindgen]
pub async fn initialize_tracing() {
    initialize_tracing_internal(TRACING_WASM_URL.clone(), TRACING_WBG_URL.clone(), thread::thread_id()).await
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