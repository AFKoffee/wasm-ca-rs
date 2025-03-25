mod mutex_wasm_abi {
    #[link(name = "harness")]
    unsafe extern "C" {
        #[link_name = "wasm_ca_begin_lock"]
        pub fn begin_lock(mutex_id: i32);

        #[link_name = "wasm_ca_finish_lock"]
        pub fn finsh_lock(mutex_id: i32);

        #[link_name = "wasm_ca_begin_unlock"]
        pub fn begin_unlock(mutex_id: i32);

        #[link_name = "wasm_ca_finish_unlock"]
        pub fn finsh_unlock(mutex_id: i32);
    }
}

pub fn begin_lock(mutex_id: i32) {
    unsafe { mutex_wasm_abi::begin_lock(mutex_id); }
}

pub fn finsh_lock(mutex_id: i32) {
    unsafe { mutex_wasm_abi::finsh_lock(mutex_id); }
}

pub fn begin_unlock(mutex_id: i32) {
    unsafe { mutex_wasm_abi::begin_unlock(mutex_id); }
}

pub fn finsh_unlock(mutex_id: i32) {
    unsafe { mutex_wasm_abi::finsh_unlock(mutex_id); }
}

/*
#[export_name = "wasm_ca_begin_acquisition"]
fn begin_acquisition_stub() {
    panic!("This function is a stub and should have been replaced by instrumentation!")
}

#[export_name = "wasm_ca_cancel_acquisition"]
fn cancel_acquisition_stub() {
    panic!("This function is a stub and should have been replaced by instrumentation!")
}

#[export_name = "wasm_ca_finish_acquisition"]
fn finish_acquisition_stub() {
    panic!("This function is a stub and should have been replaced by instrumentation!")
}


#[export_name = "wasm_ca_begin_release"]
fn begin_release_stub() {
    panic!("This function is a stub and should have been replaced by instrumentation!")
}

#[export_name = "wasm_ca_cancel_release"]
fn cancel_release_stub() {
    panic!("This function is a stub and should have been replaced by instrumentation!")
}

#[export_name = "wasm_ca_finish_release"]
fn finish_release_stub() {
    panic!("This function is a stub and should have been replaced by instrumentation!")
}*/