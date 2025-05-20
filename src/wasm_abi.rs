use crate::{console_log, thread};


#[no_mangle]
pub extern "C" fn start_lock(_lock_id: usize) -> u32 {
    // Shold resolve to a call to `request_event`
    //request_event(_lock_id, 0, 0);
    console_log!("{}: Start Lock {_lock_id}", thread::thread_id());
    1
}

#[no_mangle]
pub extern "C" fn finish_lock(_lock_id: usize) -> u32 {
    // Should resolve to a call to `aquire_event`
    //aquire_event(_lock_id, 0, 0);
    console_log!("{}: Finish Lock {_lock_id}", thread::thread_id());
    1
}

#[no_mangle]
pub extern "C" fn start_unlock(_lock_id: usize) -> u32 {
    // TODO: Does this even have a tracing pendant???
    console_log!("{}: Start Unlock {_lock_id}", thread::thread_id());
    1
}

#[no_mangle]
pub extern "C" fn finish_unlock(_lock_id: usize) -> u32 {
    // Should resolve to a call to `release_event`
    //release_event(_lock_id, 0, 0);
    console_log!("{}: Finish Unlock {_lock_id}", thread::thread_id());
    1
}

#[no_mangle]
pub extern "C" fn spawn_thread(_thread_id: u32) -> u32 {
    // Should resolve to a call to `fork_event`
    //fork_event(_thread_id, 0, 0);
    console_log!("{}: Spawn Thread {_thread_id}", thread::thread_id());
    1
}

#[no_mangle]
pub extern "C" fn join_thread(_thread_id: u32) -> u32 {
    // Should resolve to a call to `join_thread`
    //join_event(_thread_id, 0, 0);
    console_log!("{}: Join Thread {_thread_id}", thread::thread_id());
    1
}
