use std::{any::Any, cell::LazyCell, sync::atomic::{AtomicU32, Ordering}};

mod message;
mod worker_handle;

static THREAD_ID_COUNTER: AtomicU32 = AtomicU32::new(0);

fn next_available_thread_id() -> u32 {
    THREAD_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
}

thread_local! {
    static THREAD_ID: LazyCell<u32> = LazyCell::new(next_available_thread_id)
}


pub trait Task<T: Send + 'static>: FnOnce() -> T + Send + 'static {}

pub struct JoinHandle<T> {
    __: T,
}

impl<T> JoinHandle<T> {
    pub fn join(self) -> Result<T, Box<dyn Any + Send + 'static>> {
        unimplemented!("join() not yet implemented!")
    }
}

pub fn thread_spawn<F: Task<T>, T: Send + 'static>(f: F) -> JoinHandle<T> {
    unimplemented!("thread_spawn() is not yet implemented!");
}

pub fn thread_id() -> u32 {
    THREAD_ID.try_with(|id| **id).expect("Thread ID has been deallocated early!")
}
