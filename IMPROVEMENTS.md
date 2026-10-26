# Performance improvements

A review of where `SIMPLE` spends its time, with baseline measurements and a
list of candidate improvements. See [Progress](#progress) for what has been
done.

## How to measure

Criterion baselines (serial / `--features parallel`):

| Name | Code |
|--|--|
| `baseline` / `baseline_par` | Original code, before any change |
| `merged` / `merged_par` | After the edition 2024 upgrade and the weather fixes (#129), before A1. Use these for further comparisons |

```sh
cargo bench --bench simulation -- --baseline merged           # full simulations
cargo bench -p matrix --bench hot_kernels -- --baseline merged # matrix kernels
cargo bench -p heat --bench my_benchmark -- --baseline merged  # rk4
cargo bench -p rendering --bench benchmark -- --baseline merged # ray intersection
```

Every change is also checked for byte-identical annual outputs (box and cold
apartment) against the code just before it.

`benches/simulation.rs` marches the box and cold-apartment validation models
(`tests/box`, `tests/cold_apartment`) for one week, and times the
daylight-coefficient pre-processing. Annual runs were timed with the release
binary:

```sh
simple -i tests/box/box.spl -w tests/wellington.epw -n 4 -o out.csv
```

## Progress

Each step is checked for byte-identical annual outputs against the step before
it (unless noted), and benchmarked against it. "vs prev." is the change from the
previous step; "vs `merged`" is the cumulative change since the `merged`
baseline. The weather fixes (#129) changed results, so earlier steps are not
directly comparable; the original code is listed for reference only.

Percentages come from Criterion's point estimates (`march_one_week`) or from
back-to-back runs of the release binary (1 year). Apple M1 Pro.

### One week (`march_one_week`), serial

| Step | Box | vs prev. | vs `merged` | Cold apt. | vs prev. | vs `merged` | Outputs |
|--|--|--|--|--|--|--|--|
| Original code (`baseline`) | 181 ms | | | 405 ms | | | |
| Edition 2024 + dependencies | ≈ same | | | ≈ same | | | Identical |
| Weather fixes #129 (`merged`) | 153.7 ms | | | 391.8 ms | | | Changed (expected) |
| **A1** lock-free state indices | 133.7 ms | −13.0% | −13.0% | 358.8 ms | −8.4% | −8.4% | Identical |
| **A4** O(n) tridiagonal solve | 121.6 ms | −9.0% | −20.9% | 353.5 ms | −1.5% | −9.8% | Identical |
| **A5a** border conditions once per step | 66.4 ms | −45.4% | −56.8% | 321.4 ms | −9.1% | −18.0% | Identical |

### One week (`march_one_week`), `--features parallel`

| Step | Box | vs prev. | vs `merged` | Cold apt. | vs prev. | vs `merged` |
|--|--|--|--|--|--|--|
| Original code (`baseline_par`) | 199 ms | | | 349 ms | | |
| Weather fixes #129 (`merged_par`) | 195.5 ms | | | 340.4 ms | | |
| **A1** | 172.0 ms | −12.0% | −12.0% | 318.9 ms | −6.3% | −6.3% |
| **A4** | 165.4 ms | −3.8% | −15.4% | 304.5 ms | −4.5% | −10.5% |
| **A5a** | 107.1 ms | −35.3% | −45.2% | 272.3 ms | −10.6% | −20.0% |

### One year (release binary, serial)

| Step | Box | vs prev. | vs `merged` | Cold apt. | vs prev. | vs `merged` |
|--|--|--|--|--|--|--|
| Original code | 8.6 s | | | 19.7 s | | |
| Weather fixes #129 (`merged`) | 8.69 s | | | 19.47 s | | |
| **A1** | 7.37 s | −15.2% | −15.2% | 16.92 s | −13.1% | −13.1% |
| **A4** | 6.66 s | −8.6% | −23.4% | 16.69 s | −0.7% | −14.3% |
| **A5a** | 3.92 s | −40.9% | −54.9% | 15.60 s | −5.2% | −19.9% |

### Other effects

- The `rand` 0.10 upgrade made the DC pre-processing 24% (serial) to 41%
  (`parallel`) faster (see B3). No step since has changed it.
- A4 made `tri_diag_mut_gaussian` 33% (n = 3), 49% (n = 8) and 60% (n = 20)
  faster: 45 / 124 / 356 ns.
- A5b (`cbrt` instead of `powf(1. / 3.)`) was tried and reverted: 2-4% faster,
  but it changed the box's zone temperature by up to 1.2 °C. See A5.

## Baselines

Apple M1 Pro (8 cores), rustc 1.99.1, release profile. Measured on the original
code.

### Full simulations

| Run | Serial | `parallel` |
|--|--|--|
| Box, 1 year (DC cached) | 8.6 s | 10.9 s (27% slower) |
| Cold apartment, 1 year (DC cached) | 19.7 s | 18.1 s (8% faster, 37% more CPU) |
| Box, DC pre-processing (validation settings) | | ≈ 4 s |
| Cold apartment, DC pre-processing (MF4, 3000 samples) | | ≈ 52 s, ~4.4 of 8 cores used |
| `march_one_week/box` | 181 ms | 199 ms |
| `march_one_week/cold_apartment` | 405 ms | 349 ms |
| `optical_info/box_10pts_300samples` | 127 ms | 68 ms (1.9× on 8 cores) |

### Kernels

| Benchmark | Time | Note |
|--|--|--|
| `dc_times_sky/6x145` (box) | 1.1 µs | 17.4 µs with `parallel` |
| `dc_times_sky/60x2305` (cold apartment) | 184 µs | ~0.75 GFLOP/s; 71 µs with `parallel` |
| `tri_diag_mut_gaussian/3`, `/8`, `/20` | 68 / 247 / 880 ns | An O(n) solve should take tens of ns |
| `rk4` (23 nodes) | 342 ns | |
| `intersect_room` / `intersect_sponza` | 72 / 623 ns | |
| `intersect_1200/12k/120k_triangles` | 6.5 µs / 136 µs / 1.86 ms | Scene is degenerate, see C4 |

### Where the time goes

Sampling profile (`sample`) of the serial annual runs:

| Share of runtime | Box | Cold apartment |
|--|--|--|
| DC × sky products (`light`) | small | ~35% |
| Sky-vector generation (`ReinhartSky::bin_dir`, trigonometry) | small | ~13% |
| Heat march (total) | dominant | ~44% |
| Mutex lock/unlock in state accessors | 12.5% | 11% |
| `pow` and other `libm` | 29% | 15% |
| `malloc` / `free` | 18% | 3% |
| Output `write` syscalls | 3% | 4.5% |

## A. High impact, results unchanged

### A1. Lock-free state indices ✅

Done: `StateElementField` is now `Arc<OnceLock<usize>>`, and the accessors
generated by `derive` use `OnceLock::get`/`set`. See [Progress](#progress).

`derive/src/object_struct.rs:127` stored every state index as
`Arc<Mutex<Option<usize>>>`, and every state read or write
(`front_ir_irradiance`, `first_node_temperature_index`, ...) locks it. That is
11-12% of runtime, and it would become contention as soon as surfaces are
marched in parallel. The indices are written once during setup, so
`OnceLock<usize>` (or a plain `usize` set while building) needs no lock.

### A2. Fast DC × sky product

`light/src/solar_model.rs` (`update_solar_radiation`) multiplies four dense
daylight-coefficient matrices by the sky vector every timestep, through the
generic `GenericMatrix::prod_into`. Its single-accumulator `fold` cannot
auto-vectorize, because floating-point addition order must be preserved.

- Write a dedicated matrix × vector with 4-8 independent accumulators, so the
  compiler vectorizes it (no explicit SIMD needed).
- Skip sky bins whose DC column is zero; any given surface cannot see much of
  the sky.
- Optionally store DCs as `f32`, halving memory traffic.

Expected 4-10× on this kernel, which is ~35% of the cold apartment.

### A3. Cache sky-bin directions

`PerezSky::gen_sky_vec` (`weather/src/solar/perez.rs`) recomputes each bin's
direction with `acos`/`sincos` and allocates a new vector every timestep. The
directions never change: precompute them once in `SolarModel` and reuse a
preallocated vector. ~13% of the cold apartment.

### A4. O(n) tridiagonal solve ✅

Done: `NDiagMatrix::mut_gaussian` now eliminates on the compact storage, with
no allocation, performing exactly the same floating point operations as the
dense `Matrix::mut_n_diag_gaussian` (a test checks they agree bit by bit). It
also made `NDiagGenericMatrix::index()` (and so `get`/`set`) O(1), by
computing `acc_n_in_row` in closed form; it used to loop over all previous
rows. See [Progress](#progress).

A textbook Thomas algorithm would be faster still (no explicit row scaling),
but would change results in the last bits; see the `march_nomass` item under
Methods.

`NDiagMatrix::mut_gaussian` (`matrix/src/n_diag_matrix.rs:19`) used to copy the
compact tridiagonal matrix into a newly allocated dense n×n `Matrix` and runs
dense Gaussian elimination. `heat` calls it in every iteration of
`march_nomass` (`heat/src/surface.rs:1085`). This is most of the box's 18% in
`malloc`/`free`. A Thomas-algorithm solve on the compact storage is O(n) and
allocation-free.

### A5. Hoist loop-invariant work out of `march_nomass`

**A5a ✅ Done.** `ThermalSurfaceData::march` now calculates the border
conditions (and radiative coefficients) once per timestep, in a
`BorderConditions` struct passed to `march_nomass` and `march_mass`. Before,
they were recalculated in every iteration of the `march_nomass` loop, twice more
in `march`, and once per massive chunk, always with the same inputs (the state
does not change while a surface marches). `march_nomass` also no longer clones
`k` and `q` on every call: it uses scratch buffers in `ChunkMemory`. Outputs
are byte-identical; the box is 41-45% faster. See [Progress](#progress).

**A5b ❌ Tried and reverted.** Replacing `powf(1. / 3.)` with `cbrt` in
`get_tarp_natural_convection_coefficient` (`heat/src/convection.rs`) changes
each convection coefficient by at most one unit in the last place, and made
annual runs 2-4% faster. But the box's zone temperature then changed in 15% of
the timesteps by more than 0.1 °C, and in 187 timesteps by more than 0.5 °C
(maximum 1.22 °C); the cold apartment barely changed (303 of 280,280 values,
maximum 0.019 °C). Not worth it until the no-mass solver is made robust; see
"Sensitivity of `march_nomass`" under Methods.

Originally:

- `calc_border_conditions` runs on every iteration of the `march_nomass` loop,
  but its inputs (read from `state`) do not change inside the loop. Calculate
  it once per call. Most of the box's 29% in `pow` comes from here.
- `get_tarp_natural_convection_coefficient` (`heat/src/convection.rs`) uses
  `powf(1. / 3.)` three times: use `cbrt`.
- `temp_k` and `temps` are cloned on every call; keep them in `ChunkMemory`.

### A6. Buffered output

`run_simulation::run` writes every value with its own `write` call and a new
`String`. Files are unbuffered (one syscall per value) and stdout flushes at
every line. Wrap the writer in a `BufWriter` and use `write!` directly. 3-4.5%
of runtime.

## B. Parallelism at the right level

### B1. Remove threading from `matrix`

The `parallel` feature in `matrix` (pulled in by `light` and `rendering`)
makes the box 27% slower overall and small products 16× slower. It only helps
the 60×2305 DC product, which A2 makes cheap anyway.

### B2. March surfaces in parallel in `heat`

`heat_model::iterate_surfaces` already works in two phases: each surface is
marched with read-only access to `state` and its own `SurfaceMemory`, and the
results are written to `state` in a second pass. So no unsafe slicing is
needed: the commented-out `par_iter` (`heat/src/heat_model.rs:117`) is enough.
Caveats:

- A1 (done) removed the state-index mutexes that threads would have contended
  on.
- One surface step takes microseconds, so this only pays off above some model
  size. Use a threshold, or parallelize larger units of work.

### B3. Coarser parallelism in DC pre-processing

`SolarSurface::calc_solar_dc_matrix` (`light/src/solar_surface.rs`) processes
surfaces one at a time, and `DCFactory::calc_dc`
(`rendering/src/daylight_coefficients.rs`) nests `par_iter` over a few rays and
then over individual ambient samples. Each sample allocates a full
`1 × n_bins` `ColourMatrix`, although `trace_ray` writes at most one bin, and
all of them are kept until they are summed at the end.

- Flatten into a single parallel loop over all (surface, ray) pairs.
- Accumulate into one buffer per thread (`fold` + `reduce`), recording the
  single `(bin, value)` each sample contributes.
- `calc_dc` creates a new RNG for every ray and every ambient sample
  (`get_rng()`). Moving from `SmallRng::from_entropy()` (an OS syscall each
  time) to `rand::make_rng()` in the `rand` 0.10 upgrade already made this
  benchmark 24% (serial) to 41% (`parallel`) faster; create one RNG per thread
  instead.
- `ProgressBar::tic` bumps a shared atomic once per sample; tick per chunk
  instead (minor).

The cold apartment currently takes ~52 s using ~4.4 of 8 cores.

## C. Ray tracing

### C1. Prune the BVH with the closest hit

`BoundingVolumeTree::intersect` (`rendering/src/bvh.rs:547`) tests boxes with
`BBox3D::intersect` (`geometry/src/bbox3d.rs:243`), which takes no maximum
distance. Once a hit is found, traversal still visits every farther box along
the ray. Passing the current closest distance to the box test is the standard
large win for closest-hit queries.

### C2. Do not intersect the winning triangle twice

`Scene::cast_ray` (`rendering/src/scene.rs:243`) recomputes the barycentric
intersection of the triangle the BVH just found. Return `u`, `v` (and `t`)
from the BVH instead.

### C3. Triangle layout

Store precomputed edges instead of vertices, and possibly lay out each leaf as
structure-of-arrays so several triangles are tested at once. This matches the
observation that limiting rendering to triangles helped most: cache layout
matters more than explicit SIMD here.

### C4. Fix the benchmarks

- The random-triangle scenes in `rendering/benches/benchmark.rs` use 30-unit
  triangles in a 30-unit box. They overlap so much that no BVH can separate
  them, so these numbers say little about real scenes.
- `intersect_dining` loads `sponza.obj` again instead of a dining scene.

## D. Small items

- **Bug:** in `ThermalSurfaceData::march` (`heat/src/surface.rs`), the back
  solar irradiance is clamped with `if solar_back.is_nan() || solar_front < 0.0`;
  it should test `solar_back < 0.0`. Fixing it may change results, so it was
  left as is.

- `heat_model::get_boundary_temperature` calls `Model::get_space(name)` per
  surface and per sub-step, which allocates a `String`, compares names
  linearly and clones an `Arc`. Use the space index the surface already stores
  (`front_space_index` / `back_space_index`).
- `iterate_surfaces` and `calculate_zones_abc` allocate several `Vec`s per
  sub-step; keep them in the model's memory.
- Hot loops (`get_k_q`, `rearrange_k`) use bounds-checked
  `get`/`set`/`add_to_element` returning `Result<_, String>`; index the
  storage directly.

## Suggested order

A1 → A4 → A5 → A6 (simple and safe), then A2 → A3, then B1 → B2 → B3, then C.
Tier A alone should make annual runs roughly 2× faster or more (an estimate
from the profile shares; each change should be checked against the baselines).

## Methods (to discuss separately; these change results)

- **Direct + diffuse split in `light`.** Today one DC matrix is used and the
  sun is binned into a Reinhart patch, which is why the cold apartment needs
  MF4 (2305 bins). Tracing direct sun exactly from each sensor point and
  keeping a coarse MF1 (145 bins) DC for the diffuse sky would cut both the
  pre-processing and the per-step product by about 16×.
- **`march_nomass` solver.** A fixed-point iteration with 0.5 damping, capped
  at 19,000 iterations. A Newton step, or a direct solve if the nonlinearity is
  mild, should converge much faster.
- **Sensitivity of `march_nomass`.** A5b showed that changing convection
  coefficients by one unit in the last place moves the box's zone temperature
  by up to 1.2 °C. The loop stops on thresholds (`err < 0.01`, or `err` going
  up), so tiny input changes can change the number of iterations and therefore
  the result. A solver that converges to a tolerance independent of the path
  would make results reproducible, and allow optimizations such as A5b.
- **Stale convection inputs.** Inside `march_nomass`, border conditions use the
  previous timestep's surface temperatures from `state` (related to A5).
- **`dt_subdivisions *= 2`** (`heat/src/heat_model.rs:375`) is applied to every
  model to keep the no-mass solve stable; a better solver may make it
  unnecessary.
