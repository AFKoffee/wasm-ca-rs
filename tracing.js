let tracing_instance = undefined;

async function setup_tracing_internal(wasm_url, bindgen_url, memory) {
    let {default: init} = await import(bindgen_url);
    tracing_instance = await init(wasm_url, memory)
}

async function initialize_tracing_internal(wasm_url, bindgen_url) {
    let {default: init} = await import(bindgen_url);
    tracing_instance = await init(wasm_url)
}

function get_tracing_memory_internal() {
    return tracing_instance.memory
}

function generate_binary_trace() {
    return tracing_instance.generate_binary_trace()
}

export { 
    setup_tracing_internal, 
    initialize_tracing_internal, 
    get_tracing_memory_internal, 
    generate_binary_trace 
}