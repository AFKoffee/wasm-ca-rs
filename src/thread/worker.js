console.log("JScript: initializing standalone worker")

// Wait for the main thread to send us the shared module/memory. Once we've got
// it, initialize it all with the 'wasm_bindgen' module
let wasm = undefined;
self.onmessage = async event => {
    if (event.data.type == "init") {
        let {default: init} = await import('http://localhost:8000/pkg/playground.js');
        let {type, /*url,*/ module, memory, task} = event.data;
        wasm = await init(module, memory);
        wasm.handle_msg({type, task})
    } else if (!wasm) {
        console.warn("Wasm module has not been initialized. Ignoring message ...")
    } else if (event.data.type == "close") {
        wasm.__wbindgen_thread_destroy(); // Deallocate TLS and thread stack
        self.close();
    } else {
        wasm.handle_msg(event.data)
    }
}