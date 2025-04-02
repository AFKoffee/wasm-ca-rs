use parking_lot::{
    lock_api::{self, Mutex},
    RawMutex,
};

use crate::tracing;

mod lock_wasm_abi {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(module = "/wasm_ca.js")]
    extern "C" {
        #[wasm_bindgen]
        pub fn start_lock(lock_id: usize);

        #[wasm_bindgen]
        pub fn finish_lock(lock_id: usize);

        #[wasm_bindgen]
        pub fn start_unlock(lock_id: usize);

        #[wasm_bindgen]
        pub fn finish_unlock(lock_id: usize);
    }
}

pub struct TracingRawMutex {
    inner: RawMutex,
}

unsafe impl lock_api::RawMutex for TracingRawMutex {
    #[allow(clippy::declare_interior_mutable_const)]
    const INIT: Self = Self {
        inner: RawMutex::INIT,
    };

    type GuardMarker = <parking_lot::RawMutex as parking_lot::lock_api::RawMutex>::GuardMarker;

    fn lock(&self) {
        lock_wasm_abi::start_lock(self as *const _ as usize);

        self.inner.lock();

        lock_wasm_abi::finish_lock(self as *const _ as usize);
    }

    fn try_lock(&self) -> bool {
        self.inner.try_lock()
    }

    unsafe fn unlock(&self) {
        lock_wasm_abi::start_unlock(self as *const _ as usize);

        self.inner.unlock();

        lock_wasm_abi::finish_unlock(self as *const _ as usize);
    }
}

pub type TracingMutex<T> = Mutex<TracingRawMutex, T>;
