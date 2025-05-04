use js_sys::{Array, Uint8Array};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{console_log, mutex::TracingMutex, thread};

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
        todo!()
    }
}

static TRACE: TracingMutex<Vec<Event>> = TracingMutex::new(Vec::new());

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
pub fn generate_trace_download_url() -> String {
    let thread = thread::thread_spawn(|| {
        let mut output = Vec::new();

        for e in TRACE.lock().iter() {
            output.extend(e.to_binary());
        }

        output
    });

    // TODO: Somehow solve this with futures such that the main thread will not block busy waiting
    let output = match thread.join() {
        Ok(out) => out,
        Err(_) => {
            console_log!("Failed to generate binary trace. Returning empty vector ...");
            Vec::new()
        }
    };

    let options = web_sys::BlobPropertyBag::new();
    options.set_type("application/octet-stream");
    let blob = web_sys::Blob::new_with_u8_slice_sequence_and_options(
        Array::from_iter([Uint8Array::from(output.as_slice())]).as_ref(),
        &options,
    )
    .unwrap();

    web_sys::Url::create_object_url_with_blob(&blob).unwrap()
}
