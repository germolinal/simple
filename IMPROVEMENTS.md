# Performance and correctness review

## Improvements made

Release binary, Apple M1 Pro. Annual runs: medians of 5 back-to-back runs.
Every step (except where noted) gave byte-identical annual outputs; the
random DC pre-processing was instead checked with 11 rounds of the
validations before and after (no significant change in 73 indicators).

| Step | Box, 1 year | Cold apartment, 1 year |
|--|--|--|
| Original code | 8.6 s | 19.7 s |
| Weather fixes #129 (changed results) | 8.69 s | 19.47 s |
| A1: lock-free state indices (`OnceLock`) | 7.37 s | 16.92 s |
| A4: O(n) tridiagonal solve | 6.66 s | 16.69 s |
| A5a: border conditions once per timestep | 3.92 s | 15.60 s |
| A6: buffered output | 3.37 s | 14.93 s |
| A2: sparse/dense DC × sky product | 3.39 s | 8.15 s |
| E1 + E2: `matrix` fixes, band storage, products, direct indexing | 2.69 s | 7.50 s |
| A3: sky bin directions calculated once | **2.49 s (−71%)** | **5.04 s (−74%)** |

Pre-processing (daylight coefficients and IR view factors), cold apartment
from scratch (DCs + 1 year), `--features parallel`:

| Step | Wall | CPU |
|--|--|--|
| Before | ~70 s | ~280 s |
| B3 + C1: one row per DC ray; BVH pruned by closest hit | 12.8 s | 45 s |
| C6: IR view factors cast their ray once (not 10,000 times) | 9.3 s | 24.5 s |
| C5: BVH leaf size, `t`-based hits, branch-free box test | **8.6 s** | **18.7 s** |

Validations: the `validate` crate is now part of the workspace
(`validate/`, its proc-macro renamed `validate_derive`; history in its own
repository), and `make validate` has two new reports comparing with analytic
solutions: `view_factors` (tilted surfaces, street canyons, overhangs) and
`light_sources` (square and sphere lights over a floor, glossy reflections
of lights).

CI (`.github/workflows/test.yaml`, on pull requests and pushes to `main`):
formatting, clippy on all targets with warnings denied (default and
`parallel`), tests, `check --all-targets` for every feature set (including
`float`), docs with warnings denied, `--locked`, and a warning when the
validation reports are stale; `make ci` runs the same checks locally.

Also: edition 2024 and up-to-date dependencies; no threading inside
`matrix` (B1); `make validate` regenerates all reports, and a stamp file
tells when they are stale.

Tried and reverted (no gain, or worse): `cbrt` in TARP (A5b; changed
results, see below), marching surfaces in parallel (B2; 3-5× slower, too
fine-grained), precomputed triangle edges (C5.2), a single parallel loop for
all DC rays.

Benchmarks: `cargo bench --bench simulation` (one week of the box and the
cold apartment), `-p matrix --bench hot_kernels`, `-p light --bench
dc_matrix`, `-p rendering --bench benchmark`, `-p heat --bench my_benchmark`.

## Bugs

### Fixed

- **Results changed when the optical data came from its cache**: `serde_json`
  parses floats without guaranteeing a correct rounding, so daylight
  coefficients read from `optical_data_path` could differ by one ulp from
  the calculated ones, which moved the box's temperatures by up to 1 °C
  (0.04 °C in the cold apartment). The workspace now enables
  `float_roundtrip`: a run that calculates the data and one that reads it
  are byte-identical.
- **Clippy** is clean on all targets (tests included), with and without
  `parallel`; `make pre_commit` now checks all targets. On the way:
  `test_iterate_full_year` (`calendar`) broke out of its loop after the
  first time step (now tests all six); constant assertions in tests are
  `panic!`s or `assert_eq!`s.
- **`validate` reported the mean squared error as the RMSE**
  (`root_mean_squared_error` lacked the square root; its examples only had
  errors of ±1). Every "Root Mean Squared Error" in the reports so far, and
  in this file, is a mean squared error (e.g., the ray tracer's specular
  metal "RMSE 59.5" was an RMSE of 7.7). RMSE thresholds were converted to
  keep the same criteria (convection 0.01 → 0.1, walls 0.2 → √0.2).
  Scatter reports now also show the mean bias error, the RMSE and the
  maximum absolute error, with units.
- **Cached optical data ignored the options**: a file in
  `SolarOptions.optical_data_path` was used even if it had been calculated
  with other options (sky discretization, points, rays). It now records
  them, and is recalculated (and overwritten) when they differ, or when it
  does not record them (files written before this).
- **IR view factors** (`SolarSurface::calc_view_factors`): whether a point saw
  the outside was decided by its normal ray alone (a wall facing another
  wall saw no sky and no ground; a window under an overhang saw as much sky
  as an unobstructed one), and directions were uniformly distributed over
  the hemisphere instead of cosine-weighted (a 45° roof saw 0.75 of sky
  instead of 0.854). Now `SolarOptions.n_ir_view_factor_rays` (default
  16,384) rays per side of a surface, each from a random point on it, with stratified cosine-weighted
  directions; rays that hit the scene count as neither sky nor ground.
  Standard deviation ≤ 0.002, < 1 ms per side (`view_factor_convergence`;
  test: `view_factors_tilted_and_obstructed`). Validations (IR, box, cold
  apartment; 11 rounds): no significant change (0 of 51 indicators); their
  surfaces are vertical or horizontal, and unobstructed.
- **`geometry`'s collinearity test was absolute** (`|ab × bc| < 1e-5`, in
  m²): it rejected all triangles smaller than ~3 mm, and accepted very
  different angles for short and long edges. Now on the sine of the angle
  (`< 1e-5`). Annual results: byte-identical. (Points closer than 1 mm are
  still considered equal: a modelling tolerance, used in ~190 places.)
- **`--features float` (`f32`) was broken**: 18 tests failed, and `f32`
  debug builds could panic.
  - `Segment3D::get_intersection_pt` chose the projection plane with an
    absolute threshold on an unnormalized normal, so a vertical wall could
    be projected onto XY (wrong intersections; e.g., points of a door
    "outside" its wall), and its coplanarity check was below the rounding
    error of long segments. Thresholds are now relative to the segments'
    lengths. Annual results (`f64`): byte-identical.
  - Thresholds below `f32`'s precision (1e-7 to 1e-15) in `geometry`, in
    `debug_assert!`s of the sun's position and of DC ray directions, and in
    tests, are now `max(threshold, k ε)`: the same for `f64`.
  - `float` features did not reach every dependency (e.g., `cargo test -p
    model --features float` did not compile): `model`, `communication`,
    `heat`, `light`, `rendering` and the root crate now enable `float` in
    all their dependencies.
  - Tests at exact boundaries (orientations at 22.5°, a period ending at
    23.99999999 h, which is 24 h in `f32`) now keep a margin.
- **Glossy surfaces reflected lights wrongly; large lights were
  underestimated** (`rendering`). Three problems, now checked against direct
  integrations of the BSDF (`rendering/tests/glossy_light.rs`) and analytic
  solutions (`rendering/tests/triangle_light.rs`, including sphere lights):
  - Ward sampling did not reflect the ray around the sampled half vector
    (`v.z += d` instead of Radiance's `rdir + d*h`), retried directions below
    the surface, and returned a pdf (and spectrum) with an extra `1/cos`;
    `ward_pdf` did not describe the sampled directions (it integrated to
    1.1-1.9). Now the true density, `s · p_h(h) / (4 |h·wo|)`, and the
    BSDF's value. BSDF sampling alone was 3-200% too bright on a glossy
    metal reflecting a light; now within 0.1%.
  - Multiple importance sampling was one-sided: BSDF samples hitting lights
    in the scene were not added, so large, close lights were underestimated
    (−23% for a 4 × 4 m light 0.5 m above a floor; −17% for a large sphere
    with 2 shadow samples). Now they are, weighted with the light's pdf
    (`Scene::light_indexes` tells which light each triangle belongs to;
    lights are sampled on their primitive, not on their triangles). Both
    strategies use `Material::pdf()`, and distant lights are weighted
    considering the number of shadow samples and the probability of
    choosing the light.
  - With one shadow sample, shadow rays went to the light's centre (as
    Radiance with `-dj 0`): +10-19% on glossy surfaces. Now always random.

  Ray tracer validation (11 rounds before/after): specular plastic RMSE 5.63
  → 0.12 (MBE −0.63 → −0.08), full metal 0.28 → 0.17, specular metal 0.051 →
  0.049; full plastic 0.0001 → 0.0015 (MBE −0.006 → +0.006); no other change.
  DC, solar and IR validations: no change (0 of 49 indicators). The Veach
  MIS scene (`veach_mis` in `rendering/tests/test_scenes.rs`) now shows all
  16 highlights.
- **The Perez sky could be negative** (`weather/src/solar/perez.rs`): with the
  sun close to the horizon and little direct radiation, patches went down to
  −9e6 in Wellington's year (which, with the back irradiance not clamped,
  took the cold apartment's bathroom to −3.1 °C). Now as Radiance's
  `gendaymtx`: negative patches are set to zero, sky brightness is clamped
  to [0.01, 0.6] (no upper limit before) and clearness to [1, 11.9]. Annual
  temperatures: box up to 1.6 °C (99.9th percentile 0.8 °C), cold
  apartment up to 0.04 °C. Validations (11 rounds): Barcelona North solar
  RMSE 18.54 → 18.24; nothing else changed. Test:
  `test_sky_vec_is_not_negative`.
- **`make weather` never ran** (the `weather` directory made Make consider it
  up to date, so `make validate` never regenerated `weather.html`), and it
  would have failed (`weather` has no `parallel` feature). All targets are
  now `.PHONY`.
- **Back solar irradiance was never clamped**: `ThermalSurfaceData::march`
  (`heat/src/surface.rs`) tested `solar_front < 0.0` instead of `solar_back`.
  It matters because the sky could be negative (fixed above): in the cold
  apartment, the bathroom went from 25.8 °C to −3.1 °C on 03/10 at 18:45
  (back irradiance of −1.3e4 W/m² on a surface); now 25.9 °C. Elsewhere,
  differences in zone and surface temperatures are small (99th percentile
  0.3-0.5 °C).
- **Rays missed small triangles and slivers**: Möller–Trumbore treated rays as
  parallel when `|a| < 1e-5`, but `a` scales with the edges' lengths
  (rays at 60° missed a 4 × 4 mm triangle, and 1 m × 15 µm slivers). Now
  relative to `|edge1| |edge2| |direction|`; hits are rejected by distance
  (as before) instead of also by `t > 1e-5`. With the next fix, checked with
  10 rounds of the validations before/after: 1 of 94 indicators changed
  significantly (diffuse metal MBE 0.0037 → −0.0007).
- **Shadow rays could leak**: `unobstructed_distance` only checked the closest
  hit in each BVH leaf; when that one was rejected as a self-intersection,
  other triangles in the leaf that block were ignored. Now any blocking
  triangle in the leaf stops the ray. Tests for both:
  `rendering/tests/intersection.rs`.
- Progress bars printed to stdout, mixing DC progress into results written to
  stdout; now stderr.
- With `--features float`, several test targets did not compile (`weather`
  solar tests, `air/src/eplus.rs`, `rendering/tests/validate_dc.rs`); CI
  now checks all targets (`--all-targets`) with `float`.
- Ground boundaries panicked (`unimplemented!()`, `unwrap()`); now setting one
  returns an error ("not supported yet").
- `PerezSky::update_sky_vec` added to the vector during the day but set it to
  zero at night; now it always overwrites it (all callers zeroed it first, so
  results do not change).
- The root `Cargo.toml` had `[[examples]]` (ignored by Cargo, and naming
  `cold_apartment.rs` "box"); removed, as Cargo finds the examples by itself.
  `rendering` benchmarks: random triangles were ~30 m wide and all
  overlapped (now ≤ ~1 m); "intersect_dining" loaded Sponza (now
  `intersect_sponza_oblique`); all rays had a zero direction component (only
  `intersect_sponza` keeps one).
- **Glossy highlights of light sources were up to 4× too bright**:
  `evaluate_ward_anisotropic` (`rendering/src/material/ward.rs`, used when
  sampling lights) normalized the half vector and divided by cos θo, unlike
  Radiance's `dirnorm()` (`src/rt/normal.c`), which uses the unnormalized
  `h = l + v` (`h·h = 4 cos²θd`). Its reflectance was 4× too large at normal
  incidence (2× at 60°), while BSDF sampling (as Radiance's `gaussamp()`)
  was right. Ray tracer validation (11 rounds before/after): specular metal
  RMSE 59.5 → 0.05 (MBE 2.16 → −0.07), full metal 11.45 → 0.28, full
  plastic 0.10 → 0.0001, specular plastic 21.8 → 5.55 (fixed later by the
  Ward sampling fix above).
  New test: `rendering/tests/ward_energy.rs`.
- **Triangle lights gave no direct light**: `triangle_solid_angle_pdf`
  (`rendering/src/primitive_samplers.rs`, and an unused copy in
  `rendering/src/triangle/mod.rs`) used `ray.origin · normal` instead of the
  cosine between the light's normal and the ray direction. Light samples
  with a pdf of 0 are skipped, so triangle lights contributed no direct
  light wherever that dot product was ≤ 0 (e.g., at the coordinates' origin),
  and wrong amounts elsewhere. A new test
  (`rendering/tests/triangle_light.rs`) compares a diffuse floor under a
  1 × 1 m light 2 m away with the analytic solution, with the scene at the
  origin and moved to (1000, 1000, 0): the old code gave 0 (−100%) in both
  places; now +0.1% (16 shadow samples). The validations do not use triangle
  lights (which is also why this went unnoticed).
- **BVH missed hits** of rays exactly parallel to an axis that start on the
  plane of a box face (`0 * inf = NaN` made the box test reject the box in
  x). Wrong for 1,698 of 18,000 such test rays; right for all after C5.4.
- **Output could be silently truncated**: results were written with
  `write()`, ignoring how many bytes were written (A6).
- `matrix`: `NDiagGenericMatrix::new(v, ..)` ignored `v`; products "cleared"
  their result by multiplying by zero (keeping NaN/inf); deserialized
  matrices were not checked for consistent sizes; `is_empty()` was only true
  for 0×0; a `debug_assert!` failed in `f32` builds (E1).
- **The Versailles validation compared the cold apartment's results**; it
  was meant to compare with energy bills, and was removed.
- `make validate` missed 5 of the 12 reports; reports that came out
  byte-identical could never be marked as up to date (now a stamp file).
- `ReinhartSky::bin_dir` accepted `nbin == n_bins` (out of bounds).
- **The box validation compared SIMPLE's zone air temperature with
  EnergyPlus' inside surface temperature of `ZN001:WALL001`**
  (`tests/box/cold_box_eplus.csv` is that column of `eplusout.csv`). It now
  reads `ZONE ONE:Zone Mean Air Temperature` from `eplusout.csv`.
- **The box had a 15.8 m² hole above its window**: `Zn001:Wall001`'s loop
  went around the window and closed straight to the outer loop's first
  vertex instead of returning to the one it came from, cutting out the
  triangle between the top corners and the window (area 43.9 instead of
  59.7 m²). 2.2% of the rays from inside the box escaped to the sky, so the
  inside faces absorbed 2.3 times the solar radiation the window transmits
  (same in `model/tests/box_with_window.spl`). `light/tests/closed_box.rs`
  now checks that rays do not leave the box. With both fixes (and the
  converged `march_nomass`), the box against EnergyPlus' zone air went from
  MBE +0.55 / RMSE 0.72 °C to MBE −0.25 / RMSE 0.47 °C; every face is within
  −0.3 to +0.5 °C on average.

### Open

1. **Cached optical data is not invalidated when the model changes**: the
   file in `SolarOptions.optical_data_path` is recalculated when the options
   change, but not when the geometry or the materials do, so editing a model
   silently keeps its old daylight coefficients and view factors. Could
   store a hash of the model's surfaces, fenestrations and materials
   ([#132](https://github.com/germolinal/simple/issues/132)).
2. **RMSE thresholds** (convection 0.1, walls √0.2) only keep the old criteria;
   they may deserve tightening now that the RMSE is right.

## Methods (to discuss)

- **No long-wave (IR) exchange between surfaces.** Inside, a surface's
  radiant temperature is the zone's air temperature (no exchange with the
  other surfaces of the zone, no mean radiant temperature), and the IR it
  exchanges is not added to the zone's air either, so it is not conserved
  (in the box, ±1.5 kW at the 5th/95th percentiles, ~0 over a year).
  Tried: one radiant temperature per zone (the area×emissivity-weighted mean
  of its inside faces, from the previous step), which conserves it: the box
  went from RMSE 0.38 to 0.64 °C and the cold apartment's zones warmed by
  0.7-1 °C (mean RMSE 0.80 → 1.45 °C), so it was not kept. EnergyPlus uses
  view factors (`ScriptF`) solved together with the inside heat balance.
  Outside, the
  "air" part of the view (other buildings, the building itself) is assumed
  to be at the outdoor air temperature. Surface-to-surface view factors
  could be calculated as the IR view factors now are (random points,
  stratified rays).
- **`march_nomass`** (windows, cavities). *Done:* the outer faces' IR
  exchange used to be evaluated at the previous iteration's temperature, so
  the 0.5-damped fixed point diverged whenever `rad_hs` was larger than
  about three times the convection coefficient plus the U-value (low inside
  convection), and the loop silently stopped "because the error went up":
  62% of the box's 2.94 M solves per year ended that way, with node
  temperatures still changing by 0.73 °C (median; up to 187 °C), and results
  depended on rounding noise (`cbrt` instead of `powf(1. / 3.)` moved the
  box's zone by up to 1.2 °C). Now `march_nomass` puts `rad_hs` in `K`
  (as EnergyPlus' `SolveForWindowTemperatures()` does), iterates to
  `err < 0.01` with at most 100 iterations, and warns (once) if that is ever
  reached. All solves now converge (1.6 linear solves per call in the box,
  1.9 in the cold apartment), and the box simulates twice as fast (2.6 →
  1.3 s; cold apartment unchanged, 5.5 s). Massive chunks (`march_mass`) are unchanged.
  Effect: the box's zone is 0.96 °C warmer (cold apartment: within
  0.06 °C), so the box against EnergyPlus goes from MBE −0.33 / RMSE 0.46 °C
  to MBE +0.63 / RMSE 0.72 °C: the old, closer agreement came from windows
  whose solves stopped early. That +0.63 °C came from the box's wall hole
  and a wrong reference column (see Fixed). *Left:* EnergyPlus re-evaluates
  the inside and outside `h_c` and `h_r` within its iteration (SIMPLE uses
  the previous timestep's surface temperatures); check whether
  `dt_subdivisions *= 2`
  (applied to all models, "mostly for the no-mass walls and windows") is
  still needed.
- **Interior reflections of solar radiation are ignored** (on purpose, to
  stay comparable with EnergyPlus; left for research): the solar
  daylight coefficients use `max_depth: 1` (`light/src/optical_info.rs`), so
  light entering through a window is absorbed (α) by the first surface it
  hits and the rest (ρ) is lost: in the box, 2,385 of 3,386 kWh/year
  transmitted are absorbed, and a wall coplanar with its window gets none.
  (To check how far EnergyPlus' `FullInteriorAndExterior` goes.) Letting Russian
  roulette end the paths (`max_depth: 190`): box MBE −0.16 / RMSE 0.38 °C
  (2,724 kWh absorbed), but the cold apartment's zones warm by 0.5-0.65 °C
  (mean RMSE 0.64 → 0.80 °C); exterior incident solar does not change.
  (Changing it would need the optical caches recalculated, see #132.)
- **Direct + diffuse split** in the solar model: the sun is binned into a
  Reinhart patch, which needs MF4 (2305 bins); tracing direct sun exactly
  would allow MF1 (145 bins) for the diffuse sky.
- **Roughness 0** (`rendering/src/material/ward.rs`): Radiance treats it as a
  pure mirror with a Fresnel term (`SP_PURE`, `FRESNE()` in `normal.c`); we
  use a Ward lobe of roughness 0.001 and no Fresnel term. The validation
  agrees now (specular plastic RMSE 0.12), but this is a modelling choice.
- **Watertight triangle intersection** (e.g., Woop et al.), and **`f32`
  geometry** (half the memory, twice the SIMD lanes).
