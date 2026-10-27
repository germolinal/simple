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

- **Back solar irradiance was never clamped**: `ThermalSurfaceData::march`
  (`heat/src/surface.rs`) tested `solar_front < 0.0` instead of `solar_back`.
  It matters because the sky can be negative (open item 2): in the cold
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
  plastic 0.10 → 0.0001, specular plastic 21.8 → 5.55 (see open item 1).
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

### Open

Change results (to fix and validate one by one):

1. **Specular plastic (roughness 0) still differs from Radiance** (ray tracer
   validation RMSE 5.55). Radiance treats roughness 0 as a pure mirror with a
   Fresnel term (`SP_PURE`, `FRESNE()` in `normal.c`); we use a Gaussian lobe
   of roughness 0.001 and no Fresnel.
1b. **Lights in the scene (triangles, spheres) lack half of multiple
   importance sampling** (`rendering/src/ray_tracer.rs`). Light samples are
   weighted by MIS, but BSDF samples that hit such a light after a
   non-specular bounce are not added, so large, close lights are
   underestimated: −23% for a 4 × 4 m light 0.5 m above a floor (ignored test
   `large_triangle_light_matches_analytic`). Giving light samples a weight
   of 1 instead was tried and reverted: it fixes that test, but light
   sampling is a terrible strategy for near-mirror surfaces, and the ray
   tracer validation got much worse (specular plastic RMSE 21.8 → 13.6
   million). Needs proper two-sided MIS: add BSDF hits on lights, weighted
   with the light's pdf, which requires knowing which light a triangle
   belongs to. Related: the pdf `sample_bsdf` returns for diffuse/specular
   mixtures differs from `pdf()` (MIS weights then do not add to 1); the
   escape branch weights distant lights with `power_heuristic(1, ray_prob,
   1, light_pdf)`, ignoring the number of shadow samples and the
   probability of choosing the light; and the Ward pdf (as in Radiance's
   sampling weight) integrates to ~1.1-1.9 at oblique angles.

   With one shadow sample, the shadow ray goes to the light's centre (as
   Radiance with `-dj 0`); this is intentional.
2. **The Perez sky can be negative** (`weather/src/solar/perez.rs`): in 12
   timesteps of Wellington's year (sun close to the horizon, almost no direct
   radiation), up to 2,112 of the 2,305 patches are negative, down to −9e6.
   The normalization ignores negative patches, but the returned sky does
   not. Probably clamp to zero (as the normalization does); compare with
   Radiance's `gendaylit`.
3. **`geometry` uses absolute thresholds**: `Triangle3D::new` rejects points
   closer than 1 mm and triangles with `|ab × bc| < 1e-5` (right triangles
   with sides below ~3.2 mm, and thin slivers).
4. **With `--features float`, 18 tests fail** (they compile now): 7 in
   `geometry` (loops, polygons with doors, triangulation, `Segment3D::contains`),
   `heat` `test_9050` (glazing), `model` `test_get_orientation`, 5 in
   `weather` (Reinhart rows, Perez colour matrix, cloud cover, weather lines,
   `go_through`), and the DC, solar, IR and box validations. Probably
   tolerances written for `f64`.
5. **IR view factors only look along the normal**
   (`SolarSurface::calc_view_factors`): it casts the sensor's normal ray, not
   the sampled direction, so a point counts as seeing either no sky and no
   ground, or the full unobstructed hemisphere. Fix:
   `cast_ray(Ray3D { origin: ray.origin, direction: dir }, ..)`. Pending
   Germán's decision (changes IR exchange of partly shaded surfaces).

## Methods (to discuss)

- **`march_nomass`** (windows, cavities): a fixed-point iteration with 0.5
  damping that stops on thresholds (`err < 0.01`, `err` going up, or 19,000
  iterations). Results depend on rounding noise: changing convection
  coefficients by one unit in the last place (`cbrt` instead of
  `powf(1. / 3.)`) moved the box's zone temperature by up to 1.2 °C.
  **Most of the box's solves do not converge**: over a year, 62% of its 2.94
  M calls to `march_nomass` stop because the error went up (after 1-3
  iterations), with node temperatures still changing by 0.73 °C on average
  (median; 13.5 °C at the 99th percentile, up to 187 °C); 38% converge; 18
  reach 19,000 iterations. In the cold apartment, 0.2% stop that way. This
  is probably why many sub-steps are needed.
  Also: border conditions (convection, radiation) use the previous timestep's
  surface temperatures; `dt_subdivisions *= 2` is applied to all models for
  stability.
- **Direct + diffuse split** in the solar model: the sun is binned into a
  Reinhart patch, which needs MF4 (2305 bins); tracing direct sun exactly
  would allow MF1 (145 bins) for the diffuse sky.
- **Watertight triangle intersection** (e.g., Woop et al.), and **`f32`
  geometry** (half the memory, twice the SIMD lanes).
