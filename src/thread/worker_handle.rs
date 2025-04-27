use std::sync::LazyLock;

use js_sys::{Array, Uint8Array};

use crate::error::Error;

use super::message::MsgToWorker;

static WORKER_URL: LazyLock<String> = LazyLock::new(|| {
    let js = include_str!("worker.js");
    let options = web_sys::BlobPropertyBag::new();
    options.set_type("application/javascript");
    let blob = web_sys::Blob::new_with_u8_slice_sequence_and_options(
        Array::from_iter([Uint8Array::from(js.as_bytes())]).as_ref(), 
        &options
    ).unwrap();

    web_sys::Url::create_object_url_with_blob(&blob).unwrap()
});

pub struct WorkerHandle {
    worker: web_sys::Worker,
}

impl WorkerHandle {
    pub fn spawn() ->  Result<Self, Error> {
        let options = web_sys::WorkerOptions::new();
        options.set_type(web_sys::WorkerType::Module);
        
        let worker = web_sys::Worker::new_with_options(
            &WORKER_URL, &options
        ).map_err(Error::from)?;
        
        let handle = WorkerHandle { worker };

        Ok(handle)
    }

    pub fn execute<F: FnOnce() /* TODO: Evaluate if we should put this in again ==> + Send + 'static */>(&self, f: F) -> Result<(), Error> {
        // Todo: Properly deallocate the f_ptr in case of an error!
        self.worker.post_message(
            &MsgToWorker::Init {
                f_ptr: Box::into_raw(Box::new(f)) as usize 
            }.try_to_js().map_err(Error::from)?
        ).map_err(Error::from)
    }
}

impl Drop for WorkerHandle {
    fn drop(&mut self) {
        self.worker.terminate();
    }
}
