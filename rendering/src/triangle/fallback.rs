use crate::{Float, Scene};
use geometry::{intersection::IntersectionInfo, Point3D, Ray3D, Vector3D};

use super::{Triangle, TriangleHit};
use crate::{ax, ay, az, bx, by, bz, cx, cy, cz};

/// Hits closer than this (as squared distance) are ignored, for avoiding
/// self-intersections
const MIN_T_SQUARED: Float = 0.0000001;

/// Tests the intersection between a `Ray3D` and a
/// [`Triangle`]. Returns the the point of intersection, and the `u`
/// and `v` baricentric coordinates of the intersection point.
pub fn baricentric_coordinates(ray: Ray3D, t: &Triangle) -> Option<(Point3D, Float, Float)> {
    intersect_triangle(ray, t).map(|(t, u, v)| (ray.project(t), u, v))
}

/// Tests the intersection between a `Ray3D` and a [`Triangle`]
/// (Möller–Trumbore). Returns `t` (the intersection is at
/// `ray.project(t)`), and the `u` and `v` baricentric coordinates of the
/// intersection point.
pub fn intersect_triangle(ray: Ray3D, t: &Triangle) -> Option<(Float, Float, Float)> {
    let edge1_x = bx![t] - ax![t];
    let edge1_y = by![t] - ay![t];
    let edge1_z = bz![t] - az![t];

    let edge2_x = cx![t] - ax![t];
    let edge2_y = cy![t] - ay![t];
    let edge2_z = cz![t] - az![t];

    let edge1 = Vector3D::new(edge1_x, edge1_y, edge1_z);
    let edge2 = Vector3D::new(edge2_x, edge2_y, edge2_z);

    // `a` is |edge1| |edge2| |direction| times the sine of the angle between
    // the ray and the triangle (roughly), so the ray is considered parallel
    // based on that sine; an absolute threshold missed all rays hitting
    // small triangles (e.g., 3 × 3 mm).
    const TINY: Float = 1e-5;
    let h = ray.direction.cross(edge2);
    let a = edge1 * h;
    let scale = (edge1 * edge1) * (edge2 * edge2) * (ray.direction * ray.direction);
    if a * a < TINY * TINY * scale {
        return None; // ray is parallel
    }
    let f = 1. / a;
    let s = Vector3D::new(
        ray.origin.x - ax![t],
        ray.origin.y - ay![t],
        ray.origin.z - az![t],
    );
    let u = f * (s * h);
    if !(-Float::EPSILON..=1. + Float::EPSILON).contains(&u) {
        return None;
    }
    let q = s.cross(edge1);
    let v = f * (ray.direction * q);
    if u + v > 1.0 + Float::EPSILON || v < -Float::EPSILON {
        return None; // intersection is outside
    }
    let t = f * (edge2 * q);
    // Hits too close to the origin are rejected by distance, by the callers
    if t > 0. { Some((t, u, v)) } else { None }
}

pub(crate) fn intersect_triangle_slice(
    scene: &Scene,
    ray: geometry::Ray3D,
    ini: usize,
    fin: usize,
) -> Option<(usize, IntersectionInfo)> {
    simple_intersect_triangle_slice(scene, ray, ini, fin).map(|hit| {
        let point = ray.project(hit.t);
        let triangle = &scene.triangles[hit.index];
        let info = super::new_info(triangle, point, hit.u, hit.v, ray.direction);
        (hit.index, info)
    })
}

/// Finds the closest triangle in `scene.triangles[ini..fin]` hit by `ray`
pub(crate) fn simple_intersect_triangle_slice(
    scene: &Scene,
    ray: geometry::Ray3D,
    ini: usize,
    fin: usize,
) -> Option<TriangleHit> {
    // The squared distance to `ray.project(t)` is t² times this
    let dir_squared = ray.direction * ray.direction;
    let mut ret: Option<TriangleHit> = None;
    let mut closest_t = Float::MAX;

    for (i, triangle) in scene.triangles[ini..fin].iter().enumerate() {
        if let Some((t, u, v)) = intersect_triangle(ray, triangle) {
            // If hit, check the distance.
            if t * t * dir_squared > MIN_T_SQUARED && t < closest_t {
                closest_t = t;
                ret = Some(TriangleHit {
                    index: ini + i,
                    t,
                    u,
                    v,
                });
            }
        }
    }

    ret
}
