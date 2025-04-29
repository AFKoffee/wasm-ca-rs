use std::sync::LazyLock;

use js_sys::{Array, Uint8Array};
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

use crate::error::Error;

use super::message::MsgToWorker;

static WORKER_URL: LazyLock<String> = LazyLock::new(|| {
    let js = include_str!("worker.js");
    let options = web_sys::BlobPropertyBag::new();
    options.set_type("application/javascript");
    let blob = web_sys::Blob::new_with_u8_slice_sequence_and_options(
        Array::from_iter([Uint8Array::from(js.as_bytes())]).as_ref(),
        &options,
    )
    .unwrap();

    web_sys::Url::create_object_url_with_blob(&blob).unwrap()
});

struct Work {
    func: Box<dyn FnOnce() + Send + 'static>,
}

impl Work {
    fn new<F: FnOnce() + Send + 'static>(f: F) -> Self {
        Self {
            func: Box::new(f)
        }
    }

    fn execute(self) {
        (self.func)()
    }
}

pub struct WorkerHandle {
    worker: web_sys::Worker,
}

impl WorkerHandle {
    pub fn spawn() -> Result<Self, Error> {
        let options = web_sys::WorkerOptions::new();
        options.set_type(web_sys::WorkerType::Module);

        let worker =
            web_sys::Worker::new_with_options(&WORKER_URL, &options).map_err(Error::from)?;

        let handle = WorkerHandle { worker };

        Ok(handle)
    }

    pub fn run<
        F: FnOnce() + Send + 'static, /* TODO: Evaluate if we should put this in again ==> + Send + 'static */
    >(
        &self,
        f: F,
    ) -> Result<(), Error> {
        // Todo: Properly deallocate the f_ptr in case of an error!
        self.worker
            .post_message(
                &MsgToWorker::Init {
                    f_ptr: Box::into_raw(Box::new(Work::new(f))) as usize,
                }
                .try_to_js()
                .map_err(Error::from)?,
            )
            .map_err(Error::from)
    }

    pub fn terminate(self) {
        // TODO: Thread Local Storage should be deinitialized manually ...
        self.worker.terminate();
    }
}

#[wasm_bindgen(js_name = "handle_msg")]
pub fn handle_js_message(msg: JsValue) -> Result<(), JsValue> {
    match MsgToWorker::try_from_js(msg)? {
        MsgToWorker::Init {f_ptr} => execute_work(f_ptr),
    }
    Ok(())
}

fn execute_work(f_ptr: usize) {
    let f = unsafe { Box::from_raw(f_ptr as *mut Work) };
    f.execute();
}

/*impl Drop for WorkerHandle {
    fn drop(&mut self) {
        // TODO: Thread Local Storage should be deinitialized manually ...
        // TODO: This disables threads from running in detached state
        self.worker.terminate();
    }
}*/
