use crate::{Float, Scene};
use geometry::{intersection::IntersectionInfo, Point3D, Ray3D, Vector3D};

use super::Triangle;
use crate::{ax, ay, az, bx, by, bz, cx, cy, cz};

/// Tests the intersection between a `Ray3D` and a
/// [`Triangle`]. Returns the the point of intersection, and the `u`
/// and `v` baricentric coordinates of the intersection point.
#[allow(clippy::too_many_arguments)]
pub fn baricentric_coordinates(ray: Ray3D, t: &Triangle) -> Option<(Point3D, Float, Float)> {
    let edge1_x = bx![t] - ax![t];
    let edge1_y = by![t] - ay![t];
    let edge1_z = bz![t] - az![t];

    let edge2_x = cx![t] - ax![t];
    let edge2_y = cy![t] - ay![t];
    let edge2_z = cz![t] - az![t];

    let edge1 = Vector3D::new(edge1_x, edge1_y, edge1_z);
    let edge2 = Vector3D::new(edge2_x, edge2_y, edge2_z);

    const TINY: Float = 1e-5;
    let h = ray.direction.cross(edge2);
    let a = edge1 * h;

    if a < TINY && a > -TINY {
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
    if t > TINY {
        Some((ray.project(t), u, v))
    } else {
        None
    }
}

pub(crate) fn intersect_triangle_slice(
    scene: &Scene,
    ray: geometry::Ray3D,
    ini: usize,
    fin: usize,
) -> Option<(usize, IntersectionInfo)> {
    const MIN_T: Float = 0.0000001;
    let mut t_squared = Float::MAX;
    let mut ret = None;

    let it = scene.triangles.iter().enumerate().skip(ini).take(fin - ini);

    for (i, triangle) in it {
        // Calculate baricentric coordinates

        if let Some((point, u, v)) = baricentric_coordinates(ray, triangle) {
            // If hit, check the distance.
            let this_t_squared = (point - ray.origin).length_squared();
            // if the distance is less than the prevous one, update the info
            if this_t_squared > MIN_T && this_t_squared < t_squared {
                // If the distance is less than what we had, update return data
                t_squared = this_t_squared;
                let info = super::new_info(triangle, point, u, v, ray.direction);
                ret = Some((i, info));
            }
        }
    }

    ret
}

pub(crate) fn simple_intersect_triangle_slice(
    scene: &Scene,
    ray: geometry::Ray3D,
    ini: usize,
    fin: usize,
) -> Option<(usize, geometry::Point3D)> {
    const MIN_T: Float = 0.0000001;
    let mut t_squared = Float::MAX;
    let mut ret = None;

    let it = scene.triangles.iter().enumerate().skip(ini).take(fin - ini);

    for (i, t) in it {
        if let Some((point, ..)) = baricentric_coordinates(ray, t) {
            // If hit, check the distance.
            let this_t_squared = (point - ray.origin).length_squared();
            // if the distance is less than the prevous one, update the info
            if this_t_squared > MIN_T && this_t_squared < t_squared {
                // If the distance is less than what we had, update return data
                t_squared = this_t_squared;
                ret = Some((i, point));
            }
        }
    }

    ret
}
