(module
    ;; The thread ID of this module: Has to be provided by the embedder upon creation
    (global $tid (import "wasm_ca" "tid") i32)

    ;; API-Endpoints: hooks for lock events
    (import "wasm_ca" "begin_acquisition" (func $begin_acquisition (param (; tid ;) i32) (param (; mutex id ;) i32)))
    (import "wasm_ca" "finish_acquisition" (func $finish_acquisition (param (; tid ;) i32) (param (; mutex id ;) i32)))
    
    (import "wasm_ca" "begin_release" (func $begin_release (param (; tid ;) i32) (param (; mutex id ;) i32)))
    (import "wasm_ca" "finish_release" (func $finish_release (param (; tid ;) i32) (param (; mutex id ;) i32)))

    ;; INTERNAL: functions to be called from Rust source code
    (func (export "wasm_ca_begin_lock") (param $mutex_id i32)
        global.get $tid
        local.get $mutex_id
        call $begin_acquisition
    )

    (func (export "wasm_ca_finish_lock") (param $mutex_id i32)
        global.get $tid
        local.get $mutex_id
        call $finish_acquisition
    )

    (func (export "wasm_ca_begin_unlock") (param $mutex_id i32)
        global.get $tid
        local.get $mutex_id
        call $begin_release
    )

    (func (export "wasm_ca_finish_unlock") (param $mutex_id i32)
        global.get $tid
        local.get $mutex_id
        call $finish_release
    )
)