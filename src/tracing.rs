use js_sys::{Array, Uint8Array};
use parking_lot::Mutex;
use wasm_bindgen::{prelude::wasm_bindgen, JsCast, JsValue};
use web_sys::DedicatedWorkerGlobalScope;

use crate::{console_log, thread::{self, message::WorkerMessage, worker_handle::{self, WorkerHandle}}};

pub enum Op {
    Read { addr: usize, n: usize },
    Write { addr: usize, n: usize },
    Aquire { lock: usize },
    Request { lock: usize },
    Release { lock: usize },
    Fork { tid: u32 },
    Join { tid: u32 },
}

struct Event {
    t: u32,              // ID of the executing thread
    op: Op,              // executed operation
    loc: (usize, usize), // location in the program: (function_idx, instr_idx)
}

impl Event {
    fn to_binary(&self) -> [u8; 8] {
        [0; 8]
    }
}

static TRACE: Mutex<Vec<Event>> = Mutex::new(Vec::new());

#[inline]
pub fn add_event(op: Op, loc: (usize, usize)) {
    let event = Event {
        t: thread::thread_id(),
        op,
        loc,
    };
    TRACE.lock().push(event);
}

#[wasm_bindgen]
pub fn generate_trace_download_url(callback: js_sys::Function) -> Result<(), JsValue> {
    let mut worker = WorkerHandle::spawn().map_err(|e| JsValue::from_str(&format!("{e}")))?;
    worker.set_onmessage(callback);
    worker.run(move || {
        let mut output = Vec::new();

        for e in TRACE.lock().iter() {
            output.extend(e.to_binary());
        }

        let options = web_sys::BlobPropertyBag::new();
        options.set_type("application/octet-stream");
        let blob = web_sys::Blob::new_with_u8_slice_sequence_and_options(
            Array::from_iter([Uint8Array::from(output.as_slice())]).as_ref(),
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
