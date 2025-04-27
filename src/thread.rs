use std::{any::Any, cell::{LazyCell, RefCell, UnsafeCell}, panic, sync::{atomic::{AtomicU32, Ordering}, Arc}};

use worker_handle::WorkerHandle;

use crate::error::Error;

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
    native: WorkerHandle,
    result: Arc<UnsafeCell<Option<Result<T, Box<dyn Any + Send>>>>>
}

impl<T> JoinHandle<T> {
    pub fn join(self) -> Result<T, Box<dyn Any + Send + 'static>> {
        unimplemented!("join() not yet implemented!")
    }
}

fn thread_spawn_inner<F: Task<T>, T: Send + 'static>(f: F) -> Result<JoinHandle<T>, Error> {
    let read_loc = Arc::new(UnsafeCell::new(None));
    let write_loc = read_loc.clone();
    let run = move || {
        // TODO: How to pass the thread ID?


        // TODO: Maybe this can be omitted by using the trait boundary for UnwindSafe
        let try_result = panic::catch_unwind(panic::AssertUnwindSafe(f));

        // SAFETY: `write_loc` has been defined just above and moved by the closure (being an Arc<...>).
        // `read_loc` is only given to the returned JoinHandle so the modification will not affect 
        // some values far away.
        unsafe { *write_loc.get() = Some(try_result); }

        // TODO: How to signal that the thread is ready?
    };
    
    let thread = WorkerHandle::spawn()?;
    thread.execute(run)?;
    Ok(JoinHandle { native: thread, result: read_loc })
}

pub fn thread_spawn<F: Task<T>, T: Send + 'static>(f: F) -> JoinHandle<T> {
    unimplemented!("thread_spawn() is not yet implemented!");
}

pub fn thread_id() -> u32 {
    THREAD_ID.try_with(|id| **id).expect("Thread ID has been deallocated early!")
}
