console.log("JScript: initializing standalone worker")

// Wait for the main thread to send us the shared module/memory. Once we've got
// it, initialize it all with the 'wasm_bindgen' global we imported via
// 'importScripts'.
self.onmessage = async event => {
    let {type, module, memory, task} = event.data;
    
    let {default: init} = await import(url + '/pkg/playground.js');
    let wasm = await init(module, memory);
    wasm.wasm_ca_thread_entrypoint(task);

    console.log("JScript: Worker ", thread_id, " finished its job!");
}