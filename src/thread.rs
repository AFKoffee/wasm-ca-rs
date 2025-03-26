use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};

mod thread_wasm_abi {
    #[link(wasm_import_module = "wasm_ca")]
    unsafe extern "C" {
        pub fn thread_spawn(work_id: u32);
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

    fn execute(&mut self, key: u32) -> Option<()> {
        if let Some(closure) = self.map.remove(&key) {
            self.free_list.push(key);
            closure();
            Some(())
        } else {
            None
        }
    }
}

static WORK_MONITOR: LazyLock<Mutex<WorkMonitor>> =
    LazyLock::new(|| Mutex::new(WorkMonitor::new()));

#[export_name = "wasm_ca_thread_entrypoint"]
pub extern "C" fn thread_entrypoint(work_id: u32) {
    if WORK_MONITOR.lock().unwrap().execute(work_id).is_none() {
        // TODO: Properly handle this! E.g.: Return an error code or sth.
        panic!("Work ID {work_id} did not have a function associated to it!")
    };
}

pub fn thread_spawn<F: FnOnce() + Send + 'static>(start_routine: F) {
    let id = WORK_MONITOR.lock().unwrap().insert(start_routine);
    unsafe { thread_wasm_abi::thread_spawn(id) };
}
