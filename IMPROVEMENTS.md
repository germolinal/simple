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

### Open

Change results (to fix and validate one by one):

1. **Triangle light pdf uses the ray's origin instead of its direction.**
   `rendering/src/primitive_samplers.rs` (`triangle_solid_angle_pdf`):
   `cos_theta = ray.origin * info.normal` should be
   `ray.direction * info.normal`. Affects ray-traced images with triangle
   lights (`spict`, the ray tracer validation); not the DCs. The copy in
   `rendering/src/triangle/mod.rs` has the same bug and is unused.
2. **Back solar irradiance is never clamped**: in
   `ThermalSurfaceData::march` (`heat/src/surface.rs`),
   `if solar_back.is_nan() || solar_front < 0.0` should test `solar_back`.
3. **Small triangles are missed**: the triangle test treats rays as parallel
   when `|a| < 1e-5`, an absolute threshold, but `a` scales with the
   triangle's area (all rays miss a ~3 × 3 mm triangle). Likewise, hits closer
   than `t = 1e-5` are ignored, in units of the ray direction.
4. **Shadow rays can leak**: `unobstructed_distance` only checks the closest
   hit in each leaf; if that one is rejected (e.g., as a self-intersection),
   other triangles in the same leaf that do block are ignored.
5. **IR view factors only look along the normal**
   (`SolarSurface::calc_view_factors`): it casts the sensor's normal ray, not
   the sampled direction, so a point counts as seeing either no sky and no
   ground, or the full unobstructed hemisphere. Fix:
   `cast_ray(Ray3D { origin: ray.origin, direction: dir }, ..)`. Pending
   Germán's decision (changes IR exchange of partly shaded surfaces).

Do not change results:

6. `ProgressBar` prints to stdout, so DC progress gets mixed into the results
   when they go to stdout (no `-o`).
7. With `--features float`, several test targets do not compile (`weather`
   solar tests, `air/src/eplus.rs`, `rendering/tests/validate_dc.rs`,
   `test_models`); CI only checks the libraries.
8. Ground boundaries panic (`unimplemented!()` in
   `get_boundary_temperature`, `unreachable!()` in
   `calc_border_conditions`) instead of returning an error;
   `get_boundary_temperature(..).unwrap()` in `iterate_surfaces`.
9. `PerezSky::update_sky_vec` adds to the vector during the day but sets it
   to zero at night.
10. The root `Cargo.toml` has `[[examples]]` (ignored by Cargo; should be
    `[[example]]`); the `intersect_dining` benchmark loads Sponza; the
    random-triangle benchmarks are degenerate (overlapping triangles), and
    all intersection benchmarks use rays with a zero direction component.

## Methods (to discuss)

- **`march_nomass`** (windows, cavities): a fixed-point iteration with 0.5
  damping that stops on thresholds (`err < 0.01`, `err` going up, or 19,000
  iterations). Results depend on rounding noise: changing convection
  coefficients by one unit in the last place (`cbrt` instead of
  `powf(1. / 3.)`) moved the box's zone temperature by up to 1.2 °C. Also:
  border conditions (convection, radiation) use the previous timestep's
  surface temperatures; `dt_subdivisions *= 2` is applied to all models for
  stability.
- **Direct + diffuse split** in the solar model: the sun is binned into a
  Reinhart patch, which needs MF4 (2305 bins); tracing direct sun exactly
  would allow MF1 (145 bins) for the diffuse sky.
- **Watertight triangle intersection** (e.g., Woop et al.), and **`f32`
  geometry** (half the memory, twice the SIMD lanes).
