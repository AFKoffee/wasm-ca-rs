use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};

#[cfg(not(feature = "wbg"))]
mod thread_wasm_abi {
    #[link(wasm_import_module = "wasm_ca")]
    unsafe extern "C" {
        pub fn thread_spawn(work_id: u32);
    }

    #[export_name = "wasm_ca_thread_entrypoint"]
    extern "C" fn thread_entrypoint(work_id: u32) {
        if let Some(work) = super::retrieve_work(work_id) {
            work()
        } else {
            panic!("Work ID {work_id} did not have a function associated to it!")
        };
    }
}

#[cfg(feature = "wbg")]
mod thread_wasm_abi {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(module = "/wasm_ca.js")]
    extern "C" {
        #[wasm_bindgen(js_name = thread_spawn)]
        pub fn thread_spawn_inner(work_id: u32, module: JsValue, memory: JsValue);
    }

    pub fn thread_spawn(work_id: u32) {
        thread_spawn_inner(work_id, wasm_bindgen::module(), wasm_bindgen::memory());
    }

    #[wasm_bindgen(js_name = "wasm_ca_thread_entrypoint")]
    pub fn thread_entrypoint(work_id: u32) {
        if let Some(work) = super::retrieve_work(work_id) {
            work()
        } else {
            panic!("Work ID {work_id} did not have a function associated to it!")
        };
    }
}

struct WorkMonitor {
    map: HashMap<u32, Box<dyn FnOnce() + Send + 'static>>,
    key_counter: u32,
    free_list: Vec<u32>,
}

impl WorkMonitor {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
            key_counter: u32::MIN,
            free_list: Vec::new(),
        }
    }

    fn insert<F>(&mut self, closure: F) -> u32
    where
        F: FnOnce() + Send + 'static,
    {
        let key = self.free_list.pop().unwrap_or_else(|| {
            let key = self.key_counter;
            self.key_counter += 1;
            key
        });

        self.map.insert(key, Box::new(closure));

        key
    }

    fn retrieve(&mut self, key: u32) -> Option<Box<dyn FnOnce() + Send + 'static>> {
        if let Some(closure) = self.map.remove(&key) {
            self.free_list.push(key);
            Some(closure)
        } else {
            None
        }
    }
}

static WORK_MONITOR: LazyLock<Mutex<WorkMonitor>> =
    LazyLock::new(|| Mutex::new(WorkMonitor::new()));

fn retrieve_work(key: u32) -> Option<Box<dyn FnOnce() + Send + 'static>> {
    WORK_MONITOR.lock().unwrap().retrieve(key)
}

pub fn thread_spawn<F: FnOnce() + Send + 'static>(start_routine: F) {
    let id = WORK_MONITOR.lock().unwrap().insert(start_routine);
    
    #[cfg(not(feature = "wbg"))]
    unsafe { thread_wasm_abi::thread_spawn(id) };

    #[cfg(feature = "wbg")]
    thread_wasm_abi::thread_spawn(id);
}
