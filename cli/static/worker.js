import init, { run_worker } from '/wasm/lumina_node_wasm.js';

Error.stackTraceLimit = 99;

// Events are delivered on each await point. If callbacks are not registered
// then they are discarded. Our first await point is wasm-bindgen's `init()`,
// so we need to collect the events until `run_worker` register it's own.
let queued = [];
if (typeof SharedWorkerGlobalScope !== 'undefined' && self instanceof SharedWorkerGlobalScope) {
  // for SharedWorker we queue incoming connections
  onconnect = (event) => {
    queued.push(event)
  }
} else {
  // for dedicated Worker we queue incoming messages (coming from the single client)
  onmessage = (event) => {
    queued.push(event);
  }
}

init().then(() => {
  console.log("starting worker, queued messages: ", queued.length);
  run_worker(queued);
})
