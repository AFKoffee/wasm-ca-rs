use js_sys::{Array, Uint8Array};
use parking_lot::Mutex;
use rapidbin::BinaryTraceBuilder;
use wasm_bindgen::{prelude::wasm_bindgen, JsCast, JsValue};
use web_sys::DedicatedWorkerGlobalScope;

use crate::thread::{self, message::WorkerMessage, worker_handle::{WorkerHandle}};

mod rapidbin;

pub enum Op {
    Read { addr: usize, n: usize },
    Write { addr: usize, n: usize },
    Aquire { lock: usize },
    Request { lock: usize },
    Release { lock: usize },
    Fork { tid: u32 },
    Join { tid: u32 },
}

impl Op {
    fn id(&self) -> u8 {
        match self {
            Op::Read { addr: _, n: _ } => 2,
            Op::Write { addr: _, n: _ } => 3,
            Op::Aquire { lock: _ } => 0,
            Op::Request { lock: _ } => 8,
            Op::Release { lock: _ } => 1,
            Op::Fork { tid: _ } => 4,
            Op::Join { tid: _ } => 5,
        }
    }
}

struct Event {
    t: u32,              // ID of the executing thread
    op: Op,              // executed operation
    loc: (usize, usize), // location in the program: (function_idx, instr_idx)
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
        let mut output = BinaryTraceBuilder::new();

        for e in TRACE.lock().iter() {
            output.push_event(e);
        }

        let options = web_sys::BlobPropertyBag::new();
        options.set_type("application/octet-stream");
        let blob = web_sys::Blob::new_with_u8_slice_sequence_and_options(
            Array::from_iter([Uint8Array::from(output.build().as_slice())]).as_ref(),
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
