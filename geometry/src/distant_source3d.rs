/*
MIT License

Copyright (c)  Germán Molina

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERefCountHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

use crate::intersection::IntersectionInfo;
use crate::{BBox3D, Disk3D, Point3D, Ray3D, Transform, Vector3D};

use crate::RefCount;
use crate::{Float, PI};

/// Represents a solid angle pointing from any [`Point3D`] in the scene towards
/// a certain `direction` (i.e., a [`Vector3D`]). So, this is not really a surface
/// because it is at an infinite distance from any point.
#[derive(Debug, Clone)]
pub struct DistantSource3D {
    /// The direction pointing to the souRefCounte
    pub direction: Vector3D,

    /// The solid angle in Sr
    pub omega: Float,

    /// The angle in Radians
    pub angle: Float,

    /// Cos half angle
    pub cos_half_alpha: Float,

    /// Tan of half angle
    pub tan_half_alpha: Float,

    /// A pointer to the [`Transform`] associated with this `DistantSouRefCounte3D`
    transform: Option<RefCount<Transform>>,
    // Does the transform change the hand-ness of the coordinate system?
    // transform_reverses: bool,
}

impl DistantSource3D {
    /// Creates a new `DistantSouRefCounte3D` geometry.
    ///
    /// # Inputs:
    /// * direction: A [`Vector3D`] pointing to the souRefCounte
    /// * angle: The flat angle in Radians (e.g., a hemisphere would be $`\pi`$)
    pub fn new(direction: Vector3D, angle: Float) -> Self {
        let tan_half_alpha = (angle / 2.0).tan();
        let omega = tan_half_alpha * tan_half_alpha * PI;
        Self {
            omega,
            angle,
            tan_half_alpha,
            cos_half_alpha: (angle / 2.).cos(),
            direction: direction.get_normalized(),
            // transform_reverses: false,
            transform: None,
        }
    }

    /// Creates a proxy [`Disk3D`] that can be used for sampling and
    /// other operations
    pub fn get_proxy_disk(&self, t: Float) -> Disk3D {
        // Simulate a Disk3D and get that value.
        let center = (self.direction * t).as_point3d();
        let normal = self.direction;
        let radius = t * self.tan_half_alpha;

        Disk3D::new(center, normal, radius)
    }

    /// The name of the figure. Useful for debugging.
    pub fn id(&self) -> &'static str {
        "source"
    }

    /// Gets a `BBox3D` bounding the object, in local coordinates
    pub fn bounds(&self) -> BBox3D {
        panic!("Trying to get the bounds of a {}", self.id())
    }

    /// Gets the area of the object
    pub fn area(&self) -> Float {
        Float::MAX
    }

    /// Borrows the [`Transform`]
    pub fn transform(&self) -> &Option<RefCount<Transform>> {
        &self.transform
    }

    /// Intersects an object with a [`Ray3D]` (IN LOCAL COORDINATES) traveling forward, returning the distance
    /// `t` and the normal [`Vector3D`] at that point. If the distance
    /// is negative (i.e., the object is behind the plane), it should return
    /// [`None`].    
    ///
    ///  The steps should be:
    /// * Calculate the point of intersection
    /// * Calculate normal and side of intersection
    /// * Calculate (u,v)
    /// * Calculate first derivative (i.e., dp_du, dp_dv)
    /// * Calculate second derivative (i.e., d2p_duu, d2p_dvv, d2p_duv)
    /// * Call `IntersectionInfo::new(ray,p, u, v, dpdu, dpdv, d2p_duu, d2p_dvv, d2p_duv)`
    pub fn intersect_local_ray(
        &self,
        ray: &Ray3D,
        o_error: Point3D,
        d_error: Point3D,
    ) -> Option<IntersectionInfo> {
        let phit = self.simple_intersect_local_ray(ray, o_error, d_error)?;
        let t = 10.;
        let phi = 0.5; // I don't think this value is relevant
        let disk = self.get_proxy_disk(t);
        // Get basic info and update
        let mut info = disk.intersection_info(ray, ray.project(t), phi)?;
        info.p = phit;
        Some(info)
    }

    /// Like `intersect_local_ray` but simplified because there is not need
    /// for calcuating the paramtrisized elements
    pub fn simple_intersect_local_ray(
        &self,
        ray: &Ray3D,
        _o_error: Point3D,
        _d_error: Point3D,
    ) -> Option<Point3D> {
        // intersects as long as it is in the same direction...
        let cos_angle = ray.direction.get_normalized() * self.direction;
        if cos_angle >= self.cos_half_alpha {
            Some(ray.project(Float::MAX))
        } else {
            None
        }
    }

    /// Intersects an object with a [`Ray3D]` (IN WORLD COORDINATES) traveling forward,
    /// returning a detailed [`IntersectionInfo`] about the intersaction .
    pub fn intersect(&self, ray: &Ray3D) -> Option<IntersectionInfo> {
        // Transform ray into object space, if needed
        let (local_ray, o_error, d_error) = if let Some(t) = self.transform() {
            t.inv_transform_ray(ray)
        } else {
            (*ray, Point3D::new(0., 0., 0.), Point3D::new(0., 0., 0.))
        };

        // Intersect, and return transformed back... if needed
        match self.intersect_local_ray(&local_ray, o_error, d_error) {
            None => None,
            Some(info) => {
                if let Some(t) = self.transform() {
                    Some(info.transform(t))
                } else {
                    Some(info)
                }
            }
        }
    }

    /// Intersects an object with a [`Ray3D]` (IN WORLD COORDINATES) traveling forward,
    /// returning the point of intersection, if any.
    pub fn simple_intersect(&self, ray: &Ray3D) -> Option<Point3D> {
        // Transform ray into object space, if needed
        let (local_ray, o_error, d_error) = if let Some(t) = self.transform() {
            t.inv_transform_ray(ray)
        } else {
            let t = Transform::new();
            t.inv_transform_ray(ray)
        };

        // Intersect, and return transformed back... if needed
        match self.simple_intersect_local_ray(&local_ray, o_error, d_error) {
            None => None,
            Some(phit) => {
                if let Some(t) = self.transform() {
                    Some(t.transform_pt(phit))
                } else {
                    Some(phit)
                }
            }
        }
    }

    /// Gets a `BBox3D` bounding the object, in world's coordinates.
    pub fn world_bounds(&self) -> BBox3D {
        let local_b = self.bounds();
        match self.transform() {
            Some(t) => t.transform_bbox(local_b),
            None => local_b,
        }
    }
}
