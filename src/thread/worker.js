console.log("JScript: initializing standalone worker")

// Wait for the main thread to send us the shared module/memory. Once we've got
// it, initialize it all with the 'wasm_bindgen' global we imported via
// 'importScripts'.
self.onmessage = async event => {
    let {type, url, module, memory, task} = event.data;
    
    let {default: init} = await import(url + '/pkg/playground.js');
    let wasm = await init(module, memory);
    wasm.handle_msg({type, task})

    console.log("JScript: Worker ", thread_id, " finished its job!");
}