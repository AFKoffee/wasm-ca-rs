use crate::{console_log, tracing::{self, Op}};

#[no_mangle]
pub extern "C" fn start_lock(_lock_id: usize) {
    // Shold resolve to a call to `request_event`
    //request_event(_lock_id, 0, 0);
}

#[no_mangle]
pub extern "C" fn finish_lock(_lock_id: usize) {
    // Should resolve to a call to `aquire_event`
    //aquire_event(_lock_id, 0, 0);
}

#[no_mangle]
pub extern "C" fn start_unlock(_lock_id: usize) {
    // TODO: Does this even have a tracing pendant???
}

#[no_mangle]
pub extern "C" fn finish_unlock(_lock_id: usize) {
    // Should resolve to a call to `release_event`
    //release_event(_lock_id, 0, 0);
}

#[no_mangle]
pub extern "C" fn spawn_thread(_thread_id: u32) {
    // Should resolve to a call to `fork_event`
    //fork_event(_thread_id, 0, 0);
}

#[no_mangle]
pub extern "C" fn join_thread(_thread_id: u32) {
    // Should resolve to a call to `join_thread`
    //join_event(_thread_id, 0, 0);
}

#[no_mangle]
pub extern "C" fn read_event(addr: usize, n: usize, fidx: usize, iidx: usize) {
    console_log!("Read Event: addr: {}, n: {}, fidx: {}, iidx: {}", addr, n, fidx, iidx);
    tracing::add_event(Op::Read { addr, n }, (fidx, iidx));
}

#[no_mangle]
pub extern "C" fn write_event(addr: usize, n: usize, fidx: usize, iidx: usize) {
    console_log!("Write Event: addr: {}, n: {}, fidx: {}, iidx: {}", addr, n, fidx, iidx);
    tracing::add_event(Op::Write { addr, n }, (fidx, iidx));
}

#[no_mangle]
pub extern "C" fn aquire_event(lock_id: usize, fidx: usize, iidx: usize) {
    console_log!("Aquire Event: lock: {}, fidx: {}, iidx: {}", lock_id, fidx, iidx);
    tracing::add_event(Op::Aquire { lock: lock_id }, (fidx, iidx));
}

#[no_mangle]
pub extern "C" fn request_event(lock_id: usize, fidx: usize, iidx: usize) {
    console_log!("Request Event: lock: {}, fidx: {}, iidx: {}", lock_id, fidx, iidx);
    tracing::add_event(Op::Request { lock: lock_id }, (fidx, iidx));
}

#[no_mangle]
pub extern "C" fn release_event(lock_id: usize, fidx: usize, iidx: usize) {
    console_log!("Release Event: lock: {}, fidx: {}, iidx: {}", lock_id, fidx, iidx);
    tracing::add_event(Op::Release { lock: lock_id }, (fidx, iidx));
}

#[no_mangle]
pub extern "C" fn fork_event(thread_id: u32, fidx: usize, iidx: usize) {
    console_log!("Fork Event: lock: {}, fidx: {}, iidx: {}", thread_id, fidx, iidx);
    tracing::add_event(Op::Fork { tid: thread_id }, (fidx, iidx));
}

#[no_mangle]
pub extern "C" fn join_event(thread_id: u32, fidx: usize, iidx: usize) {
    console_log!("Join Event: lock: {}, fidx: {}, iidx: {}", thread_id, fidx, iidx);
    tracing::add_event(Op::Join { tid: thread_id }, (fidx, iidx));
}
