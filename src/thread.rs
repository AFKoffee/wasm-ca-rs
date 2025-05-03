use std::{
    any::Any,
    cell::{Cell, UnsafeCell},
    panic,
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        Arc,
    },
};

use worker_handle::WorkerHandle;

use crate::{console_log, error::Error};

mod message;
mod url;
mod worker_handle;

// TODO: Reevaluate if this export should maybe be removed such that
// it is only aviable via javascript.
pub use url::set_bindgen_url_suffix_js as set_bindgen_url_suffix;

static THREAD_ID_COUNTER: AtomicU32 = AtomicU32::new(0);

fn next_available_thread_id() -> u32 {
    THREAD_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
}

thread_local! {
    static THREAD_ID: Cell<Option<u32>> = const { Cell::new(None) };
}

pub struct JoinHandle<T> {
    native: WorkerHandle,
    internals: Arc<ThreadInternals<T>>,
    finished: Arc<AtomicBool>,
}

impl<T> JoinHandle<T> {
    pub fn join(mut self) -> Result<T, Box<dyn Any + Send + 'static>> {
        while !self.finished.load(Ordering::Relaxed) {}
        if let Some(internals_mut) = Arc::get_mut(&mut self.internals) {
            if let Some(result) = internals_mut.take_result() {
                // Terminate the WebWorker (has to be done manually)
                self.native.terminate().expect("Could not terminate worker!");

                result
            } else {
                console_log!("Thread was marked as finished without having a result set!");
                panic!();
            }
        } else {
            console_log!("Thread was marked as finished but had more than one reference to the result struct!");
            panic!();
        }
    }
}

pub type ThreadResult<T> = Result<T, Box<dyn Any + Send>>;

struct ThreadInternals<T> {
    tid: u32,
    result: UnsafeCell<Option<ThreadResult<T>>>,
}

// SAFTETY: Same reasoning as in the Rust std library:
//
// Due to the usage of `UnsafeCell` we need to manually implement Sync.
// The type `T` should already always be Send (otherwise the thread could not
// have been created) and the ThreadInternals struct is Sync because all access to the
// `UnsafeCell` synchronized (by the `join()` boundary).
unsafe impl<T: Send> Sync for ThreadInternals<T> {}

impl<T> ThreadInternals<T> {
    fn new() -> Self {
        Self {
            tid: next_available_thread_id(),
            result: UnsafeCell::new(None),
        }
    }

    #[inline]
    fn tid(&self) -> u32 {
        self.tid
    }

    // SAFETY: Callers of this function have to ensure, that no other
    // function accesses the result attribute at the same time
    // (i.e., when using the structs' API this means calling this function from another thread)
    unsafe fn set_result(&self, result: ThreadResult<T>) {
        *self.result.get() = Some(result)
    }

    fn take_result(&mut self) -> Option<ThreadResult<T>> {
        self.result.get_mut().take()
    }
}

fn thread_spawn_inner<F: FnOnce() -> T + Send + 'static, T: Send + 'static>(f: F) -> Result<JoinHandle<T>, Error> {
    let read_internals = Arc::new(ThreadInternals::new());
    let read_finished = Arc::new(AtomicBool::new(false));
    
    let write_internals = read_internals.clone();
    let write_finished = read_finished.clone();
    
    let main = move || {
        // TODO: Remove the panics and find a better solution!
        let old_id_state = THREAD_ID
            .try_with(|id_cell| id_cell.replace(Some(write_internals.tid())))
            .expect("Thread ID has been deallocated early!");
        assert!(
            old_id_state.is_none(),
            "Thread ID has already been initialized!"
        );

        // TODO: Maybe this can be omitted by using the trait boundary for UnwindSafe
        let try_result = panic::catch_unwind(panic::AssertUnwindSafe(f));

        // SAFETY: `write_internals` has been defined just above and moved by the closure (being an Arc<...>).
        // `read_internals` is only given to the returned JoinHandle so the modification will not affect
        // some values far away.
        unsafe {
            write_internals.set_result(try_result);
        }

        // Finish thread operation
        drop(write_internals); // We drop explicitly here to decrement the arc count on the result
        write_finished.store(true, Ordering::Relaxed);
    };

    let main = Box::new(main);
    // SAFETY: dynamic size and alignment of the Box remain the same. The lifetime change is
    // justified, because the closure is passed over a ffi-boundary (in this case to JScript)
    // into a WebWorker, where there is no way to enforce lifetimes of the closure.
    // 
    // The caller of this function has to ensure, that the thread will not outlive any variables
    // bound by the closure (or the reference to the closure itself). This is enforced statically
    // by the 'static trait bound of the public `thread_spawn()` function.
    //
    // The thread execution mechanism inside the WebWorker has to ensure, that there are no
    // references to the closure after the thread has terminated (when `join()` returns).
    let main =
            unsafe { Box::from_raw(Box::into_raw(main) as *mut (dyn FnOnce() + Send + 'static)) };

    
    let thread = WorkerHandle::spawn()?;
    thread.run(main)?;
    Ok(JoinHandle {
        native: thread,
        internals: read_internals,
        finished: read_finished
    })
}

pub fn thread_spawn<F: FnOnce() -> T + Send + 'static, T: Send + 'static>(f: F) -> JoinHandle<T> {
    thread_spawn_inner(f).expect("Thread creation failed!")
}

pub fn thread_id() -> u32 {
    THREAD_ID
        .try_with(|id_cell| {
            if let Some(id) = id_cell.get() {
                id
            } else {
                // TODO: This case should only happen in the main thread.
                // Thread IDs of spawned threads should have their ThreadIDs initialized in the wrapped closure.
                // Can we check or alleviate this somehow???
                let id = next_available_thread_id();
                id_cell.set(Some(id));
                id
            }
        })
        .expect("Thread ID has been deallocated early!")
}
