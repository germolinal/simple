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

## Daylight on workplanes

Workplanes are declared in the model (see the `Workplane` object). They are
only for reporting: daylight only, not part of the state, and not available to
control scripts.

```
Workplane { name: "Desks", space: "Office" }                  // 0.7m high, sensors 0.5m apart
Workplane { name: "Counters", space: "Office", height: 0.85, spacing: 1 }
```

Their daylight coefficients are calculated when the `Simulation` is built.
Then, the illuminance at their sensors is recorded in every timestep:

```js
const names = sim.workplane_names();     // workplanes are referred to by index
const xyz = sim.workplane_sensors(0);    // [x0, y0, z0, x1, y1, z1, ...]
const areas = sim.workplane_areas(0);    // m2 of floor represented by each sensor

sim.run();

// Per sensor, counting only the hours between 8:00 and 18:00 (omit both
// for every hour of the day)
const over300 = sim.hours_above(0, 300, 8, 18);        // hours with >= 300 lux
const useful = sim.hours_within(0, 100, 2000, 8, 18);  // hours within 100-2000 lux
const occupied = sim.occupied_hours(0, 8, 18);

// The raw data: the latest timestep, or every timestep so far
sim.illuminance(0);
sim.illuminance_history(0);
```
