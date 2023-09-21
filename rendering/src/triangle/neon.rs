use super::fallback;
use crate::{Float, Scene};
use geometry::{intersection::IntersectionInfo, Point3D};

use std::arch::aarch64::*;

type VLoad = unsafe fn(ptr: *const Float) -> BaseSimd;
type Vop = unsafe fn(a: BaseSimd, b: BaseSimd) -> BaseSimd;
type Vtofloat = unsafe fn(a: BaseSimd) -> Float;

#[cfg(feature = "float")]
const PACK_SIZE: usize = 4;
#[cfg(feature = "float")]
pub(crate) const LEAF_SIZE: usize = 24;
#[cfg(feature = "float")]
type BaseSimd = float32x4_t;

#[cfg(feature = "float")]
const VLOAD: VLoad = vld1q_f32;
#[cfg(feature = "float")]
const VSUB: Vop = vsubq_f32;
#[cfg(feature = "float")]
const VADD: Vop = vaddq_f32;
#[cfg(feature = "float")]
const VDIV: Vop = vdivq_f32;
#[cfg(feature = "float")]
const VMUL: Vop = vmulq_f32;
#[cfg(feature = "float")]
const VMINV: Vtofloat = vminvq_f32;
#[cfg(feature = "float")]
const VMAXV: Vtofloat = vmaxvq_f32;

#[cfg(not(feature = "float"))]
pub(crate) const PACK_SIZE: usize = 2;
#[cfg(not(feature = "float"))]
type BaseSimd = float64x2_t;
#[cfg(not(feature = "float"))]
pub(crate) const LEAF_SIZE: usize = 24;
#[cfg(not(feature = "float"))]
const VLOAD: VLoad = vld1q_f64;
#[cfg(not(feature = "float"))]
const VSUB: Vop = vsubq_f64;
#[cfg(not(feature = "float"))]
const VADD: Vop = vaddq_f64;
#[cfg(not(feature = "float"))]
const VDIV: Vop = vdivq_f64;
#[cfg(not(feature = "float"))]
const VMUL: Vop = vmulq_f64;
#[cfg(not(feature = "float"))]
const VMINV: Vtofloat = vminvq_f64;
#[cfg(not(feature = "float"))]
const VMAXV: Vtofloat = vmaxvq_f64;

#[target_feature(enable = "neon")]
pub(crate) unsafe fn simple_intersect_triangle_slice(
    scene: &Scene,
    ray: &geometry::Ray3D,
    ini: usize,
    fin: usize,
) -> Option<(usize, Point3D)> {
    const MIN_T: Float = 0.0000001;
    let mut t_squared = Float::MAX;
    let mut ret = None;

    let ax = scene.ax[ini..fin].chunks(PACK_SIZE);
    let ay = scene.ay[ini..fin].chunks(PACK_SIZE);
    let az = scene.az[ini..fin].chunks(PACK_SIZE);

    let bx = scene.bx[ini..fin].chunks(PACK_SIZE);
    let by = scene.by[ini..fin].chunks(PACK_SIZE);
    let bz = scene.bz[ini..fin].chunks(PACK_SIZE);

    let cx = scene.cx[ini..fin].chunks(PACK_SIZE);
    let cy = scene.cy[ini..fin].chunks(PACK_SIZE);
    let cz = scene.cz[ini..fin].chunks(PACK_SIZE);

    let iter = ax
        .zip(ay)
        .zip(az)
        .zip(bx)
        .zip(by)
        .zip(bz)
        .zip(cx)
        .zip(cy)
        .zip(cz);

    for (n_pack, aux) in iter.enumerate() {
        let ((((((((ax, ay), az), bx), by), bz), cx), cy), cz) = aux;
        if ax.len() == PACK_SIZE {
            if let Some((i, point, ..)) = baricentric_coordinates_neon(
                ray,
                ax.as_ptr(),
                ay.as_ptr(),
                az.as_ptr(),
                bx.as_ptr(),
                by.as_ptr(),
                bz.as_ptr(),
                cx.as_ptr(),
                cy.as_ptr(),
                cz.as_ptr(),
            ) {
                // If hit, check the distance.
                let this_t_squared = (point - ray.origin).length_squared();
                // if the distance is less than the prevous one, update the info
                if this_t_squared > MIN_T && this_t_squared < t_squared {
                    // If the distance is less than what we had, update return data
                    t_squared = this_t_squared;

                    ret = Some((ini + i + n_pack * PACK_SIZE, point));
                }
            }
        } else {
            for i in 0..ax.len() {
                if let Some((point, ..)) = fallback::baricentric_coordinates(
                    ray, ax[i], ay[i], az[i], bx[i], by[i], bz[i], cx[i], cy[i], cz[i],
                ) {
                    // If hit, check the distance.
                    let this_t_squared = (point - ray.origin).length_squared();
                    // if the distance is less than the prevous one, update the info
                    if this_t_squared > MIN_T && this_t_squared < t_squared {
                        // If the distance is less than what we had, update return data
                        t_squared = this_t_squared;

                        ret = Some((ini + i + n_pack * PACK_SIZE, point));
                    }
                }
            }
        }
    }

    ret
}

#[target_feature(enable = "neon")]
pub(crate) unsafe fn intersect_triangle_slice(
    scene: &Scene,
    ray: &geometry::Ray3D,
    ini: usize,
    fin: usize,
) -> Option<(usize, IntersectionInfo)> {
    const MIN_T: Float = 0.0000001;
    let mut t_squared = Float::MAX;
    let mut ret = None;

    let ax = scene.ax[ini..fin].chunks(PACK_SIZE);
    let ay = scene.ay[ini..fin].chunks(PACK_SIZE);
    let az = scene.az[ini..fin].chunks(PACK_SIZE);

    let bx = scene.bx[ini..fin].chunks(PACK_SIZE);
    let by = scene.by[ini..fin].chunks(PACK_SIZE);
    let bz = scene.bz[ini..fin].chunks(PACK_SIZE);

    let cx = scene.cx[ini..fin].chunks(PACK_SIZE);
    let cy = scene.cy[ini..fin].chunks(PACK_SIZE);
    let cz = scene.cz[ini..fin].chunks(PACK_SIZE);

    let iter = ax
        .zip(ay)
        .zip(az)
        .zip(bx)
        .zip(by)
        .zip(bz)
        .zip(cx)
        .zip(cy)
        .zip(cz);

    for (n_pack, aux) in iter.enumerate() {
        let ((((((((ax, ay), az), bx), by), bz), cx), cy), cz) = aux;
        if ax.len() == PACK_SIZE {
            if let Some((i, point, u, v)) = baricentric_coordinates_neon(
                ray,
                ax.as_ptr(),
                ay.as_ptr(),
                az.as_ptr(),
                bx.as_ptr(),
                by.as_ptr(),
                bz.as_ptr(),
                cx.as_ptr(),
                cy.as_ptr(),
                cz.as_ptr(),
            ) {
                // If hit, check the distance.
                let this_t_squared = (point - ray.origin).length_squared();
                // if the distance is less than the prevous one, update the info
                if this_t_squared > MIN_T && this_t_squared < t_squared {
                    // If the distance is less than what we had, update return data
                    t_squared = this_t_squared;
                    let info = super::new_info(
                        ax[i],
                        ay[i],
                        az[i],
                        bx[i],
                        by[i],
                        bz[i],
                        cx[i],
                        cy[i],
                        cz[i],
                        point,
                        u,
                        v,
                        ray.direction,
                    );
                    ret = Some((ini + i + n_pack * PACK_SIZE, info));
                }
            }
        } else {
            for i in 0..ax.len() {
                if let Some((point, u, v)) = fallback::baricentric_coordinates(
                    ray, ax[i], ay[i], az[i], bx[i], by[i], bz[i], cx[i], cy[i], cz[i],
                ) {
                    // If hit, check the distance.
                    let this_t_squared = (point - ray.origin).length_squared();
                    // if the distance is less than the prevous one, update the info
                    if this_t_squared > MIN_T && this_t_squared < t_squared {
                        // If the distance is less than what we had, update return data
                        t_squared = this_t_squared;
                        let info = super::new_info(
                            ax[i],
                            ay[i],
                            az[i],
                            bx[i],
                            by[i],
                            bz[i],
                            cx[i],
                            cy[i],
                            cz[i],
                            point,
                            u,
                            v,
                            ray.direction,
                        );
                        ret = Some((ini + i + n_pack * PACK_SIZE, info));
                    }
                }
            }
        }
    }

    ret
}

unsafe fn destr(v: BaseSimd) -> [Float; PACK_SIZE] {
    union Float64x2ToF64 {
        float64x2: BaseSimd,
        f64_array: [Float; PACK_SIZE],
    }

    let float64x2_to_f64 = Float64x2ToF64 { float64x2: v };
    let us = float64x2_to_f64.f64_array;
    us
}

/// Tests the intersection between a `Ray3D` and a pack (i.e., `&[]`)
/// of [`Triangle`]. Returns the index of the intersected triangle within the
/// pack, the point of intersection, and the `u` and `v` baricentric coordinates
/// of the intersection point.

#[target_feature(enable = "neon")]
unsafe fn baricentric_coordinates_neon(
    ray: &geometry::Ray3D,
    ax: *const Float,
    ay: *const Float,
    az: *const Float,
    bx: *const Float,
    by: *const Float,
    bz: *const Float,
    cx: *const Float,
    cy: *const Float,
    cz: *const Float,
) -> Option<(usize, geometry::Point3D, Float, Float)> {
    let ax = VLOAD(ax);
    let ay = VLOAD(ay);
    let az = VLOAD(az);

    let bx = VLOAD(bx);
    let by = VLOAD(by);
    let bz = VLOAD(bz);

    let cx = VLOAD(cx);
    let cy = VLOAD(cy);
    let cz = VLOAD(cz);

    // Calculate baricentric coordinates
    let ox = [ray.origin.x; PACK_SIZE];
    let ox = VLOAD(&ox[0]);
    let oy = [ray.origin.y; PACK_SIZE];
    let oy = VLOAD(&oy[0]);
    let oz = [ray.origin.z; PACK_SIZE];
    let oz = VLOAD(&oz[0]);

    let dx = [ray.direction.x; PACK_SIZE];
    let dx = VLOAD(&dx[0]);
    let dy = [ray.direction.y; PACK_SIZE];
    let dy = VLOAD(&dy[0]);
    let dz = [ray.direction.z; PACK_SIZE];
    let dz = VLOAD(&dz[0]);

    let edge1 = [VSUB(bx, ax), VSUB(by, ay), VSUB(bz, az)];
    let edge2 = [VSUB(cx, ax), VSUB(cy, ay), VSUB(cz, az)];
    let ray_direction = [dx, dy, dz];
    const TINY: Float = 1e-5;
    let h = cross_neon(&ray_direction, &edge2);
    let a = dot_neon(&edge1, &h);

    let mina = VMINV(a);
    if mina < TINY && mina > -TINY {
        return None;
    }
    let f = [1.; PACK_SIZE];
    let f = VLOAD(&f[0]);
    let f = VDIV(f, a);
    let s = [VSUB(ox, ax), VSUB(oy, ay), VSUB(oz, az)];

    let u = VMUL(f, dot_neon(&s, &h));
    let minu = VMINV(u);
    let maxu = VMAXV(u);
    if minu > 1. + Float::EPSILON || maxu < -Float::EPSILON {
        return None;
    }
    let q = cross_neon(&s, &edge1);
    let v = VMUL(f, dot_neon(&ray_direction, &q));
    let uv = VADD(u, v);
    let minuv = VMINV(uv);
    let maxuv = VMAXV(uv);
    if minuv > 1.0 + Float::EPSILON || maxuv < -Float::EPSILON {
        return None;
    }
    let t = VMUL(f, dot_neon(&edge2, &q));
    let maxt = VMAXV(t);
    if maxt < TINY {
        return None;
    }

    let us = destr(u);
    let vs = destr(v);
    let ts = destr(t);

    let mut any_intersect = false;
    let mut t = Float::MAX;
    let mut v = Float::MAX;
    let mut u = Float::MAX;
    let mut which_tri = usize::MAX;

    for (i, found_t) in ts.iter().enumerate() {
        let found_u = us[i];
        let found_v = vs[i];

        // If it is valid AND is closer than the other
        let is_valid = *found_t > TINY
            && found_u + found_v <= 1.
            && found_u > -Float::EPSILON
            && found_v > -Float::EPSILON;
        if is_valid && *found_t < t {
            any_intersect = true; // mark as found
            t = *found_t;
            u = found_u;
            v = found_v;
            which_tri = i;
        }
    }

    if any_intersect {
        Some((which_tri, ray.project(t), u, v))
    } else {
        None
    }
}

#[target_feature(enable = "neon")]
///    let dx = a[1] * b[2] - a[2] * b[1];
///     let dy = a[2] * b[0] - a[0] * b[2];
///     let dz = a[0] * b[1] - a[1] * b[0];
///     [dx, dy, dz]
unsafe fn cross_neon(a: &[BaseSimd; 3], b: &[BaseSimd; 3]) -> [BaseSimd; 3] {
    let dx1 = VMUL(a[1], b[2]);
    let dx2 = VMUL(a[2], b[1]);
    let dx = VSUB(dx1, dx2);

    let dy1 = VMUL(a[2], b[0]);
    let dy2 = VMUL(a[0], b[2]);
    let dy = VSUB(dy1, dy2);

    let dz1 = VMUL(a[0], b[1]);
    let dz2 = VMUL(a[1], b[0]);
    let dz = VSUB(dz1, dz2);

    [dx, dy, dz]
}

#[target_feature(enable = "neon")]
/// a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
unsafe fn dot_neon(a: &[BaseSimd; 3], b: &[BaseSimd; 3]) -> BaseSimd {
    let r0 = VMUL(a[0], b[0]); // a0*b0
    let r1 = VMUL(a[1], b[1]); // a1*b1
    let r2 = VMUL(a[2], b[2]); // a2*b2
    let r01 = VADD(r0, r1);
    VADD(r01, r2)
}
