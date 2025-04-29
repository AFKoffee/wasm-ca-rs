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

use crate::error::Error;

mod message;
mod worker_handle;

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
}

impl<T> JoinHandle<T> {
    pub fn join(mut self) -> Result<T, Box<dyn Any + Send + 'static>> {
        while !self.internals.is_finished() {}
        if let Some(internals_mut) = Arc::get_mut(&mut self.internals) {
            if let Some(result) = internals_mut.take_result() {
                // Terminate the WebWorker (has to be done manually)
                self.native.terminate();

                result
            } else {
                panic!("Thread was marked as finished without having a result set!")
            }
        } else {
            panic!("Thread was marked as finished but had more than one reference to the result struct!")
        }
    }
}

pub type ThreadResult<T> = Result<T, Box<dyn Any + Send>>;

struct ThreadInternals<T> {
    tid: u32,
    result: UnsafeCell<Option<ThreadResult<T>>>,
    finished: AtomicBool,
}

impl<T> ThreadInternals<T> {
    fn new() -> Self {
        Self {
            tid: next_available_thread_id(),
            result: UnsafeCell::new(None),
            finished: AtomicBool::new(false),
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

    // Marks the thread as being finished.
    //
    // This function should only be called when the result has been writen
    // because this acts as a signal for the JoinHandle to access the UnsafeCell !!!
    fn set_finished(&self) {
        self.finished.store(true, Ordering::Relaxed);
    }

    fn is_finished(&self) -> bool {
        self.finished.load(Ordering::Relaxed)
    }
}

fn thread_spawn_inner<F: FnOnce() -> T + Send + 'static, T: Send + 'static>(f: F) -> Result<JoinHandle<T>, Error> {
    let read_internals = Arc::new(ThreadInternals::new());
    let write_internals = read_internals.clone();
    let run = move || {
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
        write_internals.set_finished();
        drop(write_internals); // We drop explicitly here just to make clearer whats going on ...
    };

    let thread = WorkerHandle::spawn()?;
    thread.execute(run)?;
    Ok(JoinHandle {
        native: thread,
        internals: read_internals,
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
