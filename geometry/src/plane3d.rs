/*
MIT License

Copyright (c) 2021 Germán Molina

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

use crate::Float;

use crate::{Point3D, Ray3D, Vector3D};

/// Describes a plane based on a [`Point3D`] (`P`) contained in the plane and
/// a [`Vector3D`] (`N`) normal to the plane.
///
/// A vector of these characteristics can also be described by an equation
/// ```math
/// 0 = Ax+By+Cz+D
/// ```
/// Where $`A = \vec{N}_x`$, $`B = \vec{N}_y`$, $`C = \vec{N}_z`$ and $`D = \vec{N}\cdot\vec{P}`$,
#[derive(Clone, Debug)]
pub struct Plane3D {
    // A point contained in the plane
    //point: Point3D,
    /// Normal to the plane
    pub normal: Vector3D,

    /// The D coefficient in the equation explained earlier.    
    pub d: Float,
}

impl Plane3D {
    /// Creates a new plane
    pub fn new(point: Point3D, normal: Vector3D) -> Self {
        let mut normal = normal;
        normal.normalize();
        Self {
            normal,
            // point,
            d: normal * point,
        }
    }

    /// Tests whether a point is in a certain plane or not
    pub fn test_point(&self, point: Point3D) -> bool {
        (self.normal * point - self.d).abs() < Float::EPSILON
    }

    /// Intersects an object with a [`Ray3D]` (IN WORLD COORDINATES) traveling forward,
    /// returning the distance to the plane.
    pub fn intersect(&self, ray: &Ray3D) -> Option<Float> {
        let den = self.normal * ray.direction;
        // They do not intercect
        if den.abs() < Float::EPSILON {
            return None;
        }
        let t = (self.d - self.normal * ray.origin) / den;
        if t < 0. {
            None
        } else {
            // return
            Some(t)
        }
    }
}

// impl Intersect for Plane3D {

//     fn id(&self)->&'static str{
//         "plane"
//     }

//     fn intersect(&self, ray: &Ray3D) -> Option<Float> {
//         let den = self.normal * ray.direction;
//         // They do not intercect
//         if den.abs() < Float::EPSILON {
//             return None;
//         }
//         let t = (self.d - self.normal * ray.origin) / den;
//         if t < 0. {
//             None
//         } else {
//             // return
//             Some(t)
//         }
//     }

//     fn normal_at_intersection(&self, ray: &Ray3D, _t: Float) -> (Vector3D, SurfaceSide) {
//         let (side, normal) = SurfaceSide::get_side(self.normal, ray.direction);
//         (normal, side)
//     }
// }

#[cfg(test)]
mod testing {
    use super::*;

    #[test]
    fn test_new() {
        let p = Point3D::new(1.2, 2.2, 2.1);
        let mut n = Vector3D::new(1.2, 3.1, 2.3);
        n.normalize();
        let plane = Plane3D::new(p, n);
        assert!((plane.d - n * p).abs() < 1e-4);
        // assert_eq!(plane.d, n * p);
    }

    // #[test]
    // fn test_plane_intersect() {
    //     let p = Point3D::new(1.2, 2.2, 2.1);
    //     let mut n = Vector3D::new(0., 0., 1.);
    //     n.normalize();
    //     let plane = Plane3D::new(p, n);
    //     let ray = Ray3D {
    //         origin: Point3D::new(2.1, -3.1, 100.),
    //         direction: Vector3D::new(0., 0., -1.),
    //     };
    //     if let Some(t) = plane.intersect(&ray) {
    //         let (normal, side) = plane.normal_at_intersection(&ray, t);
    //         assert_eq!(side, SurfaceSide::Front);
    //         assert_eq!(normal, plane.normal);
    //         assert_eq!(t, 100. - p.z);
    //     } else {
    //         panic!("Did not intersect!")
    //     }
    // }
}
