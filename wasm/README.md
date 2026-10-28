# simple-wasm

WebAssembly bindings for running SIMPLE simulations in the browser (or Node).

## Building

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version <the wasm-bindgen version in Cargo.lock>
make wasm                      # browsers (ES module), into wasm/pkg
make wasm WASM_TARGET=nodejs   # Node (CommonJS)
```

## Usage

There is no file system, so the model (`.spl` or JSON), the weather (EPW or
`.sw`) and the optional Rhai control script are all passed as strings.
`optical_data_path` is ignored: optical data is always calculated.

```js
import init, { Simulation, wasm_memory } from "./pkg/simple_wasm.js";
await init();

const model = await (await fetch("box.spl")).text();
const epw = await (await fetch("wellington.epw")).text();

// model, weather, timesteps per hour, Rhai script (or undefined), research mode
const sim = new Simulation(model, epw, 1, undefined, false);

const header = sim.header();         // name of every state element
const report = sim.report_indexes(); // the elements in the model's Outputs

// Step by step...
while (sim.step()) {
  // ... reading/writing the state directly, without copying. The view is
  // detached when the WebAssembly memory grows, so re-create it if needed
  const state = new Float64Array(wasm_memory().buffer, sim.state_ptr(), sim.state_len());
  console.log(sim.date(), state[report[0]]);
}

// ... or all at once: one row of reported values per timestep
const results = sim.run(); // Float64Array of (timesteps * report.length)
```

The state can also be accessed through `state()`/`set_state()` (copies) and
`get(i)`/`set(i, v)`. With the `float` feature, every `Float64Array` becomes a
`Float32Array`.

When a control script is given, it runs before every timestep, exactly as with
`simple -c script.rhai`.
