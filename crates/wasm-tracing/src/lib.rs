use std::cell::Cell;

use js_sys::Uint8Array;
use parking_lot::Mutex;
use rapidbin::BinaryTraceBuilder;
use wasm_bindgen::prelude::wasm_bindgen;

mod rapidbin;

thread_local! {
    static THREAD_ID: Cell<Option<u32>> = const { Cell::new(None) };
}

pub fn thread_id() -> u32 {
    THREAD_ID
        .try_with(|id_cell| {
            if let Some(id) = id_cell.get() {
                id
            } else {
                u32::MAX
            }
        })
        .expect("Thread ID has been deallocated early!")
}

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
        t: thread_id(),
        op,
        loc,
    };
    TRACE.lock().push(event);
}

#[wasm_bindgen]
pub fn set_thread_id(id: u32) {
    THREAD_ID
        .try_with(|id_cell| id_cell.replace(Some(id)))
        .expect("Thread ID has been deallocated early!");
}

#[wasm_bindgen]
pub fn generate_binary_trace() -> Uint8Array {
    let mut output = BinaryTraceBuilder::new();

    for e in TRACE.lock().iter() {
        output.push_event(e);
    };

    Uint8Array::from(output.build().as_slice())
}

#[wasm_bindgen]
pub fn read_event(addr: usize, n: usize, fidx: usize, iidx: usize) {
    add_event(Op::Read { addr, n }, (fidx, iidx));
}

#[wasm_bindgen]
pub fn write_event(addr: usize, n: usize, fidx: usize, iidx: usize) {
    add_event(Op::Write { addr, n }, (fidx, iidx));
}

#[wasm_bindgen]
pub fn aquire_event(lock_id: usize, fidx: usize, iidx: usize) {
    add_event(Op::Aquire { lock: lock_id }, (fidx, iidx));
}

#[wasm_bindgen]
pub fn request_event(lock_id: usize, fidx: usize, iidx: usize) {
    add_event(Op::Request { lock: lock_id }, (fidx, iidx));
}

#[wasm_bindgen]
pub fn release_event(lock_id: usize, fidx: usize, iidx: usize) {
    add_event(Op::Release { lock: lock_id }, (fidx, iidx));
}

#[wasm_bindgen]
pub fn fork_event(thread_id: u32, fidx: usize, iidx: usize) {
    add_event(Op::Fork { tid: thread_id }, (fidx, iidx));
}

#[wasm_bindgen]
pub fn join_event(thread_id: u32, fidx: usize, iidx: usize) {
    add_event(Op::Join { tid: thread_id }, (fidx, iidx));
}