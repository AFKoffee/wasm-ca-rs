use parking_lot::{lock_api::{self, Mutex}, RawMutex};

mod lock_wasm_abi {
    #[link(wasm_import_module = "wasm_ca")]
    unsafe extern "C" {
        pub fn start_lock(lock_id: usize);
        pub fn finish_lock(lock_id: usize);
        pub fn start_unlock(lock_id: usize);
        pub fn finish_unlock(lock_id: usize);
    }
}

pub struct TracingRawMutex {
    inner: RawMutex
}

unsafe impl lock_api::RawMutex for TracingRawMutex {
    #[allow(clippy::declare_interior_mutable_const)]
    const INIT: Self = Self { inner: RawMutex::INIT };

    type GuardMarker = <parking_lot::RawMutex as parking_lot::lock_api::RawMutex>::GuardMarker;

    fn lock(&self) {
        unsafe { lock_wasm_abi::start_lock(self as *const _ as usize); }
        self.inner.lock();
        unsafe { lock_wasm_abi::finish_lock(self as *const _ as usize); }
    }

    fn try_lock(&self) -> bool {
        self.inner.try_lock()
    }

    unsafe fn unlock(&self) {
        unsafe { lock_wasm_abi::start_unlock(self as *const _ as usize); }
        self.inner.unlock();
        unsafe { lock_wasm_abi::finish_unlock(self as *const _ as usize); }
    }
}

pub type TracingMutex<T> = Mutex<TracingRawMutex, T>;