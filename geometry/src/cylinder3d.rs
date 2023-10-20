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

use crate::round_error::ApproxFloat;
use crate::{Float, PI};

use crate::RefCount;
use crate::{BBox3D, Point3D, Ray3D, Transform, Vector3D};

/// A Cylinder of radius `radius` aligned witht the Z axis, starting at a
/// Z value of `zmin` and ending at `zmax`.
///
/// It can be an 'open' cylinder, by setting a `phi_max` less than `2.*PI`
#[derive(Clone, Debug)]
pub struct Cylinder3D {
    /// The radius of the `Cylinder3D`
    radius: Float,

    /// The Z coordinate of the lower end of the [`Cylinder3D`]
    zmin: Float,

    /// The Z coordinate of the upper end of the [`Cylinder3D`]
    zmax: Float,

    /// The range of longitudes allowed (2*PI radians in a full [`Cylinder3D`], less in a partial one)
    phi_max: Float,

    /// A pointer to the [`Transform`] associated with this [`Cylinder3D`]
    transform: Option<RefCount<Transform>>,
    // Does the transform change the hand-ness of the coordinate system?
    // transform_reverses: bool,
}

impl Cylinder3D {
    /// Creates a new full `Cylinder3D`; i.e., it is a revolution of 360 degrees
    pub fn new(p0: Point3D, p1: Point3D, radius: Float) -> Self {
        Self::new_partial(p0, p1, radius, 360.)
    }

    /// Creates a new partial `Cylinder3D`; i.e., it does not have to be closed. For instance,
    /// it can only be a 180 degree revolution
    pub fn new_partial(p0: Point3D, p1: Point3D, radius: Float, phi_max: Float) -> Self {
        let l = p1 - p0;
        let (x, y, z) = (l.x, l.y, l.z);

        let mut transform = Transform::translate(p0.x, p0.y, p0.z);

        // Rotate...?
        let rot_y_degrees = (x * x + y * y).sqrt().atan2(z).to_degrees();
        let rot_z_degrees = y.atan2(x).to_degrees();

        transform *= Transform::rotate_y(rot_y_degrees);
        transform *= Transform::rotate_z(rot_z_degrees);

        Self::new_transformed(
            radius,
            0.,
            l.length(),
            phi_max,
            Some(RefCount::new(transform)),
        )
    }

    /// The angle `phi_max` is in degrees
    pub fn new_transformed(
        radius: Float,
        zmin: Float,
        zmax: Float,
        phi_max: Float,
        transform: Option<RefCount<Transform>>,
    ) -> Self {
        if zmin > zmax {
            panic!(
                "when creating a 'cylinder': given zmin > zmax (i.e., {} > {})",
                zmin, zmax
            );
        }
        let mut phi_max = phi_max;
        if !(-Float::EPSILON..=360. + Float::EPSILON).contains(&phi_max) {
            panic!("when creating a 'cylinder': given phi_max is not between 0 and 360 degrees (it was {})", phi_max);
        }
        phi_max = phi_max.clamp(0., 360.).to_radians();

        // let transform_reverses = match &transform {
        //     Some(e) => e.changes_hands(),
        //     None => false,
        // };

        Self {
            radius,
            zmin,
            zmax,
            phi_max,
            transform,
            // transform_reverses,
        }
    }

    /// Performs a basic intersection of a ray and a `Cylinder3D`, returning
    /// the point of intersection and the angle `phi` of that intersection
    pub fn basic_intersection(
        &self,
        ray: &Ray3D,
        o_error: Point3D,
        d_error: Point3D,
    ) -> Option<(Point3D, Float)> {
        // decompose ray
        let dx = ApproxFloat::from_value_and_error(ray.direction.x, d_error.x);
        let dy = ApproxFloat::from_value_and_error(ray.direction.y, d_error.y);
        // let dz = ApproxFloat::from_value_and_error(ray.direction.z, d_error.z);
        let ox = ApproxFloat::from_value_and_error(ray.origin.x, o_error.x);
        let oy = ApproxFloat::from_value_and_error(ray.origin.y, o_error.y);
        // let oz = ApproxFloat::from_value_and_error(ray.origin.z, o_error.z);

        let a = dx * dx + dy * dy;
        let b = (dx * ox + dy * oy) * 2.;
        let c = ox * ox + oy * oy - self.radius * self.radius;
        let (t0, t1) = ApproxFloat::solve_quadratic(a, b, c)?;
        debug_assert!(t1.as_float() >= t0.as_float());
        // t0 < t1... so, check if they are possitive
        if t1.low <= 0.0 {
            return None;
        }

        // We now know that t1 hits... check t0
        let (mut thit, hit_is_t1) = if t0.low > 0. {
            // if t0 is a valid hit, keep that
            (t0, false)
        } else {
            // else, use t1 (which we know works...)
            (t1, true)
        };

        // We might try to do the same with 'thit' = t1, later
        let calc_phit_and_phi = |thit: ApproxFloat| -> (Point3D, Float) {
            // Calculate point of intersection.
            let mut phit = ray.project(thit.as_float());
            // refine in order to avoid error accumulation
            let hit_rad = (phit.x * phit.x + phit.y * phit.y).sqrt();
            phit.x *= self.radius / hit_rad;
            phit.y *= self.radius / hit_rad;

            // calc phi
            let mut phi = phit.y.atan2(phit.x);
            if phi < 0. {
                phi += 2. * PI;
            }
            (phit, phi)
        };

        let (mut phit, mut phi) = calc_phit_and_phi(thit);

        // Check intersection against clipping parameters...
        // it is possible that the first hit misses, but the second
        // does not
        if phit.z < self.zmin || // zmin is limiting but 'thit' missed it
            phit.z > self.zmax || // zmax is limiting but 'thit' missed it
            phi > self.phi_max
        // 'thit' missed due to the phi limitation
        {
            // if this was already t1, then we missed the sphere
            if hit_is_t1 {
                return None;
            }

            // else, try with t1.
            thit = t1;

            // recalculate
            let (new_phit, new_phi) = calc_phit_and_phi(thit);

            if new_phit.z < self.zmin || // zmin is limiting but 'thit' missed it
                new_phit.z > self.zmax || // zmax is limiting but 'thit' missed it
                new_phi > self.phi_max
            // 'thit' missed due to the phi limitation
            {
                return None;
            }
            // update values
            phit = new_phit;
            phi = new_phi;
        }

        Some((phit, phi))
    }

    /// Grabs the result of the `basic_intersection` and creates an [`IntersectionInfo`]
    pub fn intersection_info(
        &self,
        ray: &Ray3D,
        phit: Point3D,
        phi: Float,
    ) -> Option<IntersectionInfo> {
        // Calculate (u,v)
        let u = phi / self.phi_max;
        let v = (phit.z - self.zmin) / (self.zmax - self.zmin);

        // Calcuate first derivatives
        let dpdu = Vector3D::new(-self.phi_max * phit.y, self.phi_max * phit.x, 0.);
        let dpdv = Vector3D::new(0., 0., self.zmax - self.zmin);

        // Calculate second derivatives
        let d2p_duu = Vector3D::new(phit.x, phit.y, 0.) * (-self.phi_max * self.phi_max);
        let d2p_dvv = Vector3D::new(0., 0., 0.);
        let d2p_duv = Vector3D::new(0., 0., 0.);

        // return
        Some(IntersectionInfo::new(
            ray, phit, u, v, dpdu, dpdv, d2p_duu, d2p_dvv, d2p_duv,
        ))
    }

    /// Gets a `BBox3D` bounding the object, in local coordinates
    pub fn bounds(&self) -> BBox3D {
        let min = Point3D::new(-self.radius, -self.radius, self.zmin);
        let max = Point3D::new(self.radius, self.radius, self.zmax);
        BBox3D::new(min, max)
    }

    /// The name of the figure. Useful for debugging.
    pub fn id(&self) -> &'static str {
        "cylinder"
    }

    /// Gets the area of the object
    pub fn area(&self) -> Float {
        debug_assert!(self.zmax > self.zmin);
        (self.zmax - self.zmin) * self.radius * self.phi_max
    }

    /// Borrows the [`Transform`]
    pub fn transform(&self) -> &Option<RefCount<Transform>> {
        &self.transform
    }

    /// Like `intersect_local_ray` but simplified because there is not need
    /// for calcuating the paramtrisized elements
    pub fn simple_intersect_local_ray(
        &self,
        ray: &Ray3D,
        o_error: Point3D,
        d_error: Point3D,
    ) -> Option<Point3D> {
        // Do the first part
        let (phit, _) = self.basic_intersection(ray, o_error, d_error)?;
        Some(phit)
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
        let (phit, phi) = self.basic_intersection(ray, o_error, d_error)?;
        self.intersection_info(ray, phit, phi)
    }

    /// Intersects an object with a [`Ray3D]` (IN WORLD COORDINATES) traveling forward, returning the distance
    /// `t` and the normal [`Vector3D`] at that point. If the distance
    /// is negative (i.e., the object is behind the plane), it should return
    /// [`None`]. Returns detailed [`IntersectionInfo`] about the intersaction .
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

#[cfg(test)]
mod testing {
    use super::*;
    use crate::PI;
    // use crate::intersect_trait::SurfaceSide;

    #[test]
    fn test_cylinder_area() {
        let r = 2.1;
        let l = 3.2;
        let full_area = 2. * PI * r * l;

        // Full cylinder
        let c = Cylinder3D::new_transformed(r, 0., l, 360., None);
        assert_eq!(c.area(), full_area);

        // Half a cylinder due to phi
        let c = Cylinder3D::new_transformed(r, 0., l, 180., None);
        assert_eq!(c.area(), full_area / 2.);
    }
}
