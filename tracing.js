let tracing_instance = undefined;

async function setup_tracing_internal(wasm_url, bindgen_url, tid, memory) {
    let {default: init} = await import(bindgen_url);
    tracing_instance = await init(wasm_url, memory)
    tracing_instance.set_thread_id(tid)
}

async function initialize_tracing_internal(wasm_url, bindgen_url, tid) {
    let {default: init} = await import(bindgen_url);
    tracing_instance = await init(wasm_url)
    tracing_instance.set_thread_id(tid)
}

function get_tracing_memory_internal() {
    return tracing_instance.memory
}

function generate_binary_trace() {
    return tracing_instance.generate_binary_trace()
}

function read_event(addr, n, fidx, iidx) {
    if (tracing_instance) {
        tracing_instance.read_event(addr, n, fidx, iidx);
    } else {
        console.warn("Tracing not initialized yet. Skipping read event ...")
    }
}

function write_event(addr, n, fidx, iidx) {
    if (tracing_instance) {
        tracing_instance.write_event(addr, n, fidx, iidx);
    } else {
        console.warn("Tracing not initialized yet. Skipping write event ...")
    }
}

function aquire_event(lock_id, fidx, iidx) {
    if (tracing_instance) {
        tracing_instance.aquire_event(lock_id, fidx, iidx);
    } else {
        console.warn("Tracing not initialized yet. Skipping aquire event ...")
    }
}

function request_event(lock_id, fidx, iidx) {
    if (tracing_instance) {
        tracing_instance.request_event(lock_id, fidx, iidx);
    } else {
        console.warn("Tracing not initialized yet. Skipping request event ...")
    }
}

function release_event(lock_id, fidx, iidx) {
    if (tracing_instance) {
        tracing_instance.release_event(lock_id, fidx, iidx);
    } else {
        console.warn("Tracing not initialized yet. Skipping release event ...")
    }
}

function fork_event(thread_id, fidx, iidx) {
    if (tracing_instance) {
        tracing_instance.fork_event(thread_id, fidx, iidx);
    } else {
        console.warn("Tracing not initialized yet. Skipping fork event ...")
    }
}

function join_event(thread_id, fidx, iidx) {
    if (tracing_instance) {
        tracing_instance.join_event(thread_id, fidx, iidx);
    } else {
        console.warn("Tracing not initialized yet. Skipping join event ...")
    }
}

export { 
    setup_tracing_internal, 
    initialize_tracing_internal, 
    get_tracing_memory_internal, 
    generate_binary_trace,
    read_event,
    write_event,
    aquire_event,
    request_event,
    release_event,
    fork_event,
    join_event,
}