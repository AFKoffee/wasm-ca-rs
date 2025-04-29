/*let THREAD_REGISTRY = new Map();
let THREAD_COUNTER = 1; // THREAD_ID 0 is reserved to refer to the initial thread

function register_thread(worker_handle) {
    let thread_id = THREAD_COUNTER;

    THREAD_REGISTRY.set(thread_id, worker_handle)
    THREAD_COUNTER = THREAD_COUNTER + 1;

    return thread_id;
}

const worker_script = `
    console.log("JScript: initializing standalone worker")

    // Wait for the main thread to send us the shared module/memory. Once we've got
    // it, initialize it all with the 'wasm_bindgen' global we imported via
    // 'importScripts'.
    self.onmessage = async event => {
        let {url, payload: {module, memory, thread_id, job_id}} = event.data;
        let {default: init} = await import(url + '/pkg/playground.js');
        self.WASM_CA_TID = thread_id;
        
        let wasm = await init(module, memory);
        console.log("JScript: Ready to execute workload for thread ", thread_id);
        wasm.wasm_ca_thread_entrypoint(job_id);
        console.log("JScript: Worker ", thread_id, " finished its job!");
    }
`;
const worker_blob = new Blob([worker_script], {type: 'text/javascript'});
const worker_url = URL.createObjectURL(worker_blob);

self.WASM_CA_TID = 0;

function spawn_worker(job_id, module, memory) {
    let thread = new Worker(worker_url, { type: "module" });
    thread.addEventListener("message", (event) => {
        let {code, payload} = event.data;
        if (code == 20) {
            let {thread_id, description, lock_id} = payload;
            console.log("Got event from Thread ", thread_id, ": ", description, lock_id)
        } else if (code == 10) {
            let {thread_id, job_id, module, memory} = payload;
            let tid = spawn_worker(job_id, module, memory);
            console.log("Thread ", thread_id, " spawned a new thread ", tid);
        } else {
            console.warn("Unrecognized message code: ", code, ". This callback is now a noop!")
        }
    });
    let tid = register_thread(thread);
    thread.postMessage({url: document.location.protocol + '//' + document.location.host, payload: {module, memory, thread_id: tid, job_id}});

    return tid;
}

function thread_spawn(job_id, module, memory) {
    if (self.WASM_CA_TID === 0) {
        let tid = spawn_worker(job_id, module, memory);
        console.log("Thread ", self.WASM_CA_TID, " spawned a new thread ", tid);
    } else {
        self.postMessage({code: 10, payload: {thread_id: self.WASM_CA_TID, job_id, module, memory}})
    }
}*/

function start_lock(thread_id, lock_id) {
    console.log("Thread ", thread_id, " requested lock ", lock_id);
}

function finish_lock(thread_id, lock_id) {
    console.log("Thread ", thread_id, " aquired lock ", lock_id);
}

function start_unlock(thread_id, lock_id) {
    console.log("Thread ", thread_id, " started to release lock ", lock_id);
}

function finish_unlock(thread_id, lock_id) {
    console.log("Thread ", thread_id, " finished to release lock ", lock_id);
}

export {/*thread_spawn, */start_lock, finish_lock, start_unlock, finish_unlock};