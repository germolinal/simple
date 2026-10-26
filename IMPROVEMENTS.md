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
| **A6** buffered output | (not measured: `march_one_week` writes no output) | | | | | | Identical |
| **A2** sparse/dense DC × sky | 66.9 ms | +0.7% (noise) | −56.5% | 175.3 ms | −45.5% | −55.3% | Identical |
| **E1 + E2** `matrix` fixes, speed-ups and direct indexing in `heat` | 54.0 ms | −19.2% | −64.8% | 163.1 ms | −6.9% | −58.4% | Identical |
| **A3** sky bin directions calculated once | 49.5 ms | −8.4% | −67.8% | 102.8 ms | −37.0% | −73.8% | Identical |

### One week (`march_one_week`), `--features parallel`

| Step | Box | vs prev. | vs `merged` | Cold apt. | vs prev. | vs `merged` |
|--|--|--|--|--|--|--|
| Original code (`baseline_par`) | 199 ms | | | 349 ms | | |
| Weather fixes #129 (`merged_par`) | 195.5 ms | | | 340.4 ms | | |
| **A1** | 172.0 ms | −12.0% | −12.0% | 318.9 ms | −6.3% | −6.3% |
| **A4** | 165.4 ms | −3.8% | −15.4% | 304.5 ms | −4.5% | −10.5% |
| **A5a** | 107.1 ms | −35.3% | −45.2% | 272.3 ms | −10.6% | −20.0% |
| **A2** | 67.4 ms | −37.0% | −65.5% | 177.7 ms | −34.7% | −47.8% |
| **E1 + E2** | 54.0 ms | −20.0% | −72.4% | 164.1 ms | −7.6% | −51.8% |
| **A3** | 49.1 ms | −9.0% | −74.9% | 103.0 ms | −37.2% | −69.7% |

### One year (release binary, serial)

| Step | Box | vs prev. | vs `merged` | Cold apt. | vs prev. | vs `merged` |
|--|--|--|--|--|--|--|
| Original code | 8.6 s | | | 19.7 s | | |
| Weather fixes #129 (`merged`) | 8.69 s | | | 19.47 s | | |
| **A1** | 7.37 s | −15.2% | −15.2% | 16.92 s | −13.1% | −13.1% |
| **A4** | 6.66 s | −8.6% | −23.4% | 16.69 s | −0.7% | −14.3% |
| **A5a** | 3.92 s | −40.9% | −54.9% | 15.60 s | −5.2% | −19.9% |
| **A6** buffered output | 3.37 s | −6.4% | −61.2% | 14.93 s | −5.4% | −23.3% |
| **A2** sparse/dense DC × sky | 3.39 s | −2.0% | −61.0% | 8.15 s | −45.0% | −58.1% |
| **E1** `matrix` bug fixes | 3.46 s | +2.4% (noise) | −60.2% | 8.10 s | −0.4% | −58.4% |
| **E2a** `matrix` speed-ups (band storage, products) | 2.90 s | −13.9% | −66.6% | 7.84 s | −2.6% | −59.7% |
| **E2b** direct indexing in `heat` | 2.69 s | −7.2% | −69.0% | 7.50 s | −3.2% | −61.5% |
| **A3** sky bin directions calculated once | 2.49 s | −7.4% | −71.3% | 5.04 s | −32.7% | −74.1% |

Single annual runs vary by about ±5%. Up to A5a, "vs prev." comes from one
back-to-back run of each binary; from A6 on, from the medians of 5 alternating
runs of each (A6: box 3.60 → 3.37 s, cold apartment 15.78 → 14.93 s; A2:
3.46 → 3.39 s and 14.81 → 8.15 s; E1: 3.38 → 3.46 s and 8.13 → 8.10 s; E2a:
3.37 → 2.90 s and 8.05 → 7.84 s; E2b: 2.90 → 2.69 s and 7.75 → 7.50 s; A3:
2.69 → 2.49 s and 7.49 → 5.04 s). Medians from different sessions differ by
up to ~3%, so "vs prev." comes from runs in the same session.
`march_one_week` does not write output, so it does not measure A6.

With A2, `--features parallel` stopped being slower than serial: the
remaining difference came from the generic product splitting small DC
products across threads.

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

### A2. Fast DC × sky product ✅

Done, without changing results: `light::DCMatrix` (`light/src/dc_matrix.rs`)
stores each row of a daylight coefficient matrix as all-zero, sparse (non-zero
values and their columns) or dense, depending on how many values are non-zero
(sparse below 90%). `SolarModel` builds one per DC matrix when it is created,
and multiplies them by the sky vector into buffers kept in
`SolarModelMemory` (which used to be `()`). Terms are added in the same order
as before, and the skipped terms are exactly zero, so results are
byte-identical (a test checks this bit by bit against the `Matrix` product).

It does not rely on matrices being sparse: walls see part of the sky and get
sparse rows, but roofs (or DCs that account for solar bounces) get dense
rows, which are also faster than the generic product (no allocation, no
strided access). With `--features parallel`, products with at least 32,768
multiplications are split by rows across threads (each row still added in
order). `light/benches/dc_matrix.rs` (serial, per product):

| Matrix | `Matrix` | `DCMatrix` |
|--|--|--|
| Cold apartment, front surfaces (43×2306, 16% non-zero) | 131.7 µs | 23.4 µs |
| Cold apartment, back surfaces (3% non-zero) | 134.6 µs | 2.9 µs |
| Cold apartment, front fenestrations (31% non-zero) | 53.6 µs | 21.5 µs |
| Box, front surfaces (6×146) | 1.16 µs | 0.40 µs |
| Fully dense 60×2306 (e.g., with bounces) | 184.7 µs | 124.2 µs |

The cold apartment is 45% faster. Using several accumulators (which changes
the order of additions, and so results in the last bits) is left for after
the no-mass discussion.

Originally: `light/src/solar_model.rs` (`update_solar_radiation`) multiplied four dense
daylight-coefficient matrices by the sky vector every timestep, through the
generic `GenericMatrix::prod_into`. Its single-accumulator `fold` cannot
auto-vectorize, because floating-point addition order must be preserved.

- Write a dedicated matrix × vector with 4-8 independent accumulators, so the
  compiler vectorizes it (no explicit SIMD needed).
- Skip sky bins whose DC column is zero; any given surface cannot see much of
  the sky.
- Optionally store DCs as `f32`, halving memory traffic.

Expected 4-10× on this kernel, which is ~35% of the cold apartment.

### A3. Cache sky-bin directions ✅

Done: `ReinhartSky` calculates the direction of every bin once, when it is
created, and `bin_dir()` returns the stored value (a test checks they are
bit-identical to calculating them). `PerezSky::update_sky_vec_for()` updates
a slice for an existing `ReinhartSky`; `gen_sky_vec()` and `update_sky_vec()`
are now wrappers around it, with the same behaviour as before. `SolarModel`
keeps one `ReinhartSky` (replacing its `solar_sky_discretization` field) and
reuses a sky vector in `SolarModelMemory`. Before, every daytime timestep
created the sky twice (allocating its tables) and calculated every bin's
direction twice (sky, then sun). Outputs are byte-identical; the cold
apartment is 33% faster. A re-profile after E2 showed `bin_dir` at 17% of the
cold apartment's samples (plus `acos` and `sincos`).

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

### A6. Buffered output ✅

Done: `run` and `run_rhai` wrap the output in a `BufWriter` (flushed at the
end), and share two helpers, `write_header` and `write_row`, that use `write!`.
Output is byte-identical (to a file, to stdout, and with a Rhai control
script). This also fixed a latent bug: the old code called `out.write(...)` and
ignored how many bytes were written, so a short write would have silently
truncated the results; `write!` writes everything or returns an error. Write
errors are now returned as `Err` instead of panicking. Annual runs are 5-6%
faster. See [Progress](#progress).

`run_simulation::run` used to write every value with its own `write` call and a new
`String`. Files are unbuffered (one syscall per value) and stdout flushes at
every line. Wrap the writer in a `BufWriter` and use `write!` directly. 3-4.5%
of runtime.

## B. Parallelism at the right level

### B1. Remove threading from `matrix` ✅

Done: the `matrix` crate no longer has a `parallel` feature (nor depends on
`rayon`), and `light` and `rendering` no longer enable it. After A2 and E2a,
nothing in a simulation used it: the DC × sky products go through `DCMatrix`,
and `prod_into` only used threads for large products. Results cannot change
(each element is calculated the same way with or without threads).
`DCMatrix` keeps its own row-parallel path, which only applies to large dense
DC matrices (none of the test models).

Originally:

The `parallel` feature in `matrix` (pulled in by `light` and `rendering`)
makes the box 27% slower overall and small products 16× slower. It only helps
the 60×2305 DC product, which A2 makes cheap anyway.

### B2. March surfaces in parallel in `heat` ❌

Tried and reverted. Marching surfaces (and fenestrations) with `rayon` in
`iterate_surfaces` gave byte-identical results, but made annual runs much
slower (`--features parallel`, medians of 5):

| | Surfaces serial | Surfaces parallel |
|--|--|--|
| Box | 2.49 s | 13.21 s (+430%) |
| Cold apartment | 5.07 s | 17.42 s (+244%; 26 s of CPU) |

`iterate_surfaces` runs a couple of times per sub-step (hundreds of thousands
of times per year), each surface takes microseconds, and each fork-join of
`rayon` costs tens of microseconds (mostly waking threads up). With at most 43
surfaces per group, there is not enough work per call. Surfaces at each
sub-step are too fine-grained; parallelism pays off for coarser work (e.g.,
DC pre-processing, B3, or running several simulations at once). It might pay
off for very large models, but that would need a large test model to check.

Originally:

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
- With `--features float`, the libraries build, but several test targets do
  not (`f32`/`f64` mismatches in `weather/src/solar/mod.rs` tests,
  `air/src/eplus.rs` and `rendering/tests/validate_dc.rs`). CI only runs
  `cargo check --features float`, which skips tests.

- `heat_model::get_boundary_temperature` calls `Model::get_space(name)` per
  surface and per sub-step, which allocates a `String`, compares names
  linearly and clones an `Arc`. Use the space index the surface already stores
  (`front_space_index` / `back_space_index`).
- `iterate_surfaces` and `calculate_zones_abc` allocate several `Vec`s per
  sub-step; keep them in the model's memory.
- Hot loops (`get_k_q`, `rearrange_k`) use bounds-checked
  `get`/`set`/`add_to_element` returning `Result<_, String>`; index the
  storage directly.

## E. The `matrix` crate

**E1 and E2 ✅ Done** (E3 and E4 not done: unused code was kept, and the design
is unchanged). Every change keeps the order of floating point operations of
each result, so outputs are byte-identical; new tests check the products bit
by bit against the previous algorithms. Annual runs are 22% (box) and 7% (cold
apartment) faster than after A2. What changed:

- E1: `NDiagGenericMatrix::new()` fills with `v`; `fill()` (in both matrix
  types, and in `heat`'s `get_k_q`) replaces clearing by multiplying by zero;
  deserialization checks sizes (`#[serde(try_from)]`; same JSON format);
  `is_empty()` means "no elements"; `add_to_element`/`scale_element` of the
  n-diagonal matrix return `Result<(), String>`, with correct docs. Also, a
  `debug_assert!` in `mut_n_diag_gaussian` required pivots to be within
  `1e-15` of 1, which fails in `f32` (`--features float`, debug builds); it
  now uses `4 * Float::EPSILON`.
- E2a: `NDiagGenericMatrix` uses band storage (`N` elements per row, padded at
  the corners), so element `(r, c)` is at `r * N + (c + s - r)`.
  `Index`/`IndexMut<(usize, usize)>` for both types, and `as_mut_slice()`.
  `prod_into` no longer allocates, goes through both matrices in storage order
  (i-k-j; each element still adds its terms in order), uses a dot product when
  `other` has one column, and uses threads only above 32,768 multiplications;
  `prod_n_diag_into` likewise. Kernels, compared with the code before E1/E2:

  | Benchmark | Change |
  |--|--|
  | `prod_into` (30×30 by 30×30) | −73% |
  | `dc_times_sky/6x145` (matrix × vector) | −58% (1.10 µs → 460 ns) |
  | `dc_times_sky/60x2305` | −34% (184 → 123 µs) |
  | `prod_n_diag_into` / `prod_tri_diag_into` | −69% / −70% |
  | `from_prod_n_diag` | −65% |
  | `add_into` / `sub_into` / `scale_into` (unchanged code) | −3.5% |
  | `tri_diag_mut_gaussian`, `rk4` | within ±8% (unchanged) |

  A first version of the i-k-j product made matrix × vector 2.7× *slower*
  (the running sum went through memory instead of a register); the
  one-column case fixes that.
- E2b: `heat`'s hot loops (`get_k_q`, `rearrange_k`, and the copies in
  `march_mass` and `march_nomass`) index matrices directly instead of
  `get`/`set`/`add_to_element`/`scale_element` with `?`. In `get_k_q`, the
  `if let Ok(..) = k.get(..)` checks were always true, and are gone.

Not done yet: n×1 matrices used as vectors (`heat` temperatures, absorbed
solar radiation, the sky vector) would be simpler and faster as slices, but
that changes `model`'s API (e.g., `set_node_temperatures(&Matrix)`).

Review notes, from before the changes:

The crate is a hand-written, generic dense matrix (`GenericMatrix<T>`, with
`Matrix = GenericMatrix<Float>` and `ColourMatrix = GenericMatrix<Spectrum>`)
plus a compact n-diagonal matrix (`NDiagGenericMatrix<N, T>`). What the rest of
the workspace actually uses:

- `Matrix` as storage with `get`/`set`, very often as n×1 "vectors" (heat node
  temperatures, absorbed solar radiation, the sky vector);
- `NDiagMatrix<3>` in `heat`: `column_prod_into` (`rk4`) and `mut_gaussian`
  (`march_nomass`);
- `&Matrix * &Matrix`, only for DC × sky (replaced by `DCMatrix`, see A2);
- `concat_rows`, when building DC matrices; `ColourMatrix` in `rendering`.

Not used outside the crate's tests and benchmarks (other than in the
commented-out `/* ... */` code in `heat/src/surface.rs`): `gauss_seidel`,
`prod_n_diag_into`, `prod_tri_diag_into`, `from_prod_n_diag`,
`n_diag_gaussian` / `mut_n_diag_gaussian` (now only the reference in A4's
test), `add_into`, `sub_into`.

### E1. Bugs (latent)

- **`NDiagGenericMatrix::new(v, ...)` ignores `v`.** It fills the data with
  `0, 1, 2, 3, ...` (`T::one() * i`). `heat` clears its matrices before using
  them (`memory.k *= 0.`), so results are not affected, but
  `NDiagMatrix::new(0.0, n, n)` does not return zeros.
- **Clearing by multiplying by zero.** `prod_into` and `prod_n_diag_into`
  "clear" the result with `*into *= T::zero()`, which leaves NaN and infinite
  values in place (`0 × NaN = NaN`). `prod_into` overwrites them anyway, but
  `prod_n_diag_into` then adds into that buffer. The same pattern is used in
  `heat` (`memory.k *= 0.`). Use `fill(zero)`.
- **Deserialization is not validated.** `GenericMatrix` derives `Deserialize`,
  so a file with `data.len() != nrows * ncols` (e.g., a corrupt
  `optical_data.json`) is accepted and fails later, or gives wrong results.
  `from_data` checks this, but deserialization bypasses it.
- `is_empty()` is only true for 0×0, so e.g. a 0×145 DC matrix is "not empty".
- `NDiagGenericMatrix::add_to_element` and `scale_element` are documented as
  "Sets an element", and return `Ok(v)`, which means nothing.

### E2. Performance

- **Element access returns `Result<T, String>`.** Every `get`/`set` checks
  bounds by hand and then again when indexing the `Vec`, and callers in hot
  loops need `?` or `unwrap()` (e.g., `get_k_q`, `march_nomass`,
  `iterate_surfaces`). Implementing `Index<(usize, usize)>`/`IndexMut` (which
  panic on out-of-bounds, like slices) and exposing `as_mut_slice()` would let
  hot code index directly. Errors only allocate their `String` when they
  happen, so the cost is the double checks and branches, not allocation.
- **n×1 matrices used as vectors.** Temperatures, absorbed solar radiation
  and the sky vector would be simpler and faster as `Vec<Float>`/slices.
- **`prod_into`** allocates a `Vec` of row slices on every call, walks `other`
  column by column (a strided, cache-unfriendly access for large matrices),
  and, with `parallel`, splits rows across threads regardless of size (see
  B1). The standard fixes are the i-k-j loop order (or packing `other`
  transposed) and a size threshold for threads. Note that changing the loop
  order changes the order of additions, so results would change in the last
  bits.
- **n-diagonal storage.** Rows are stored with different lengths (shorter at
  the top and bottom), so finding an element needs arithmetic (it used to be a
  loop over all previous rows; A4 made it O(1)). The usual band storage
  (`nrows × N`, padded with zeros at the corners, as in LAPACK) makes
  `index = row * N + (col + s - row)`, keeps every row the same length, and
  simplifies `column_prod_into`, `mut_gaussian` and `rearrange_k`. For `heat`,
  a dedicated tridiagonal type (three vectors) would be simplest of all.

### E3. Dead code

Remove (or move to tests) the functions listed above as unused. The dense
`mut_n_diag_gaussian` can stay as a test-only reference for `mut_gaussian`.

### E4. Design

- `Numberish` requires `Serialize`, `Display`, `Send`, `Sync`, `Mul<Float>` and
  more from every element type, mostly so that one generic type can serve both
  `Float` and `Spectrum`. Only a handful of operations are needed for
  `Spectrum` (`ColourMatrix` in `rendering`).
- Errors are `String`s, so callers cannot tell a size mismatch from an
  out-of-bounds access.
- An external crate (`nalgebra`, `faer`, `ndarray`) is an option, but the
  sizes here are small and the structures special (banded, sparse DC rows), so
  a smaller, simpler hand-written crate is probably the better fit: slices for
  vectors, a proper band type, `Index` access, and only the operations that are
  used.

The performance items in E2 that keep the order of operations (indexing,
vectors, band storage) keep results byte-identical; the `prod_into` loop
order does not.

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

  *Question (Germán): shouldn't the no-mass calculation recalculate the border
  conditions in every iteration? Could hoisting them (A5a), rather than `cbrt`,
  explain the A5b deviation?*

  Hoisting cannot explain it. The original loop did call
  `calc_border_conditions` in every iteration, but it always returned the same
  values: it reads surface temperatures from `state`, which `march_nomass`
  cannot modify (it iterates on `memory.temperatures`, and `state` is only
  updated after all surfaces have marched). That is why A5a's outputs were
  byte-identical. The A5b test compared code with A5a, with and without
  `cbrt`, so the deviation comes from `cbrt` (amplified by the loop's stopping
  rules; see "Sensitivity of `march_nomass`").

  But the physics point stands: the intent was apparently to update convection
  (and radiation) as surface temperatures converge, and the code never did.
  Doing so means passing the iterating node temperatures to
  `calc_border_conditions`; it changes results, and makes the loop more
  nonlinear. To discuss together with the solver. Whether the stale inputs
  contribute to the convergence problems is not known yet; logging how each
  `march_nomass` call ends (reason, iterations, `err`) would tell.
- **`dt_subdivisions *= 2`** (`heat/src/heat_model.rs:375`) is applied to every
  model to keep the no-mass solve stable; a better solver may make it
  unnecessary.
