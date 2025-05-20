use parking_lot::{
    lock_api::{self, Mutex},
    RawMutex,
};

use crate::wasm_abi;

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
        assert_eq!(1, wasm_abi::start_lock(self as *const _ as usize));

        self.inner.lock();

        assert_eq!(1, wasm_abi::finish_lock(self as *const _ as usize));
    }

    fn try_lock(&self) -> bool {
        self.inner.try_lock()
    }

    unsafe fn unlock(&self) {
        assert_eq!(1, wasm_abi::start_unlock(self as *const _ as usize));

        self.inner.unlock();

        assert_eq!(1, wasm_abi::finish_unlock(self as *const _ as usize));
    }
}

pub type TracingMutex<T> = Mutex<TracingRawMutex, T>;
