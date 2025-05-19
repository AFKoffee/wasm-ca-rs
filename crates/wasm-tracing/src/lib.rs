use js_sys::Uint8Array;
use parking_lot::Mutex;
use rapidbin::BinaryTraceBuilder;
use wasm_bindgen::prelude::wasm_bindgen;

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
pub fn add_event(t: u32, op: Op, loc: (usize, usize)) {
    let event = Event {
        t,
        op,
        loc,
    };
    TRACE.lock().push(event);
}

#[wasm_bindgen]
pub fn generate_binary_trace() -> Uint8Array {
    let mut output = BinaryTraceBuilder::new();

    for e in TRACE.lock().iter() {
        output.push_event(e);
    };

    Uint8Array::from(output.build().as_slice())
}
