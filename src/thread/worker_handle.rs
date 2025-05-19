use js_sys::Function;
use wasm_bindgen::{prelude::{wasm_bindgen, Closure}, JsCast, JsValue};
use web_sys::MessageEvent;

use crate::error::Error;

use super::{message::WorkerMessage, url::get_worker_url};

struct Work {
    func: Box<dyn FnOnce() + Send + 'static>,
}

impl Work {
    fn new<F: FnOnce() + Send + 'static>(f: F) -> Self {
        Self { func: Box::new(f) }
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
            web_sys::Worker::new_with_options(get_worker_url(), &options).map_err(Error::from)?;

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
                &WorkerMessage::Init {
                    f_ptr: Box::into_raw(Box::new(Work::new(f))) as usize,
                }
                .try_to_js()
                .map_err(Error::from)?,
            )
            .map_err(Error::from)
    }

    pub fn set_onmessage(&mut self, callback: Function) {
        let event_handler = Closure::<dyn FnMut(_)>::new(move |event: MessageEvent|  {
            match WorkerMessage::try_from_js(event.data()) {
                Ok(msg) => if let WorkerMessage::Url { url } = msg {
                    let _ = callback.call1(&JsValue::null(), &JsValue::from_str(&url));
                },
                Err(_) => todo!(),
            }
        });
        let event_handler = Box::new(event_handler);
        let js_callback = event_handler.as_ref().as_ref().unchecked_ref();
        self.worker.set_onmessage(Some(js_callback));

        // FIXME: This leaks memory to stop the closure from being deallocated, which raises an error in JavaScript
        let _ = Box::into_raw(event_handler);
    }

    pub fn terminate(self) -> Result<(), Error> {
        self.worker
            .post_message(&WorkerMessage::Close.try_to_js().map_err(Error::from)?)
            .map_err(Error::from)
    }
}

#[wasm_bindgen(js_name = "handle_msg")]
pub fn handle_js_message(msg: JsValue) -> Result<(), JsValue> {
    match WorkerMessage::try_from_js(msg)? {
        WorkerMessage::Init { f_ptr } => execute_work(f_ptr),
        WorkerMessage::Close => (), // Noop, because this msg is handled in JS,
        WorkerMessage::Url { url: _ } => (), // This serves only for internal onmessage callbacks
    }
    Ok(())
}

fn execute_work(f_ptr: usize) {
    let f = unsafe { Box::from_raw(f_ptr as *mut Work) };
    f.execute();
}