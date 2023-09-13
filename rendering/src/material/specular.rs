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

use crate::ray::Ray;
use crate::Float;
use geometry::{Point3D, Vector3D};

/// Calculates the parameters necessary for calculating the
/// Fresnel's equations. `cos2`—i.e., the cosine of the
/// angle between the normal and the transmitted ray—is wrapped in
/// an `Option` because it does not exist if the angle of incidence
/// is larger than the critical angle.
///
/// # Example
/// ```
/// use geometry::{Point3D,Vector3D, Ray3D};
/// use rendering::{Spectrum, Ray};
/// use rendering::material::cos_and_n;
/// use rendering::interaction::Interaction;
///
/// let mat_refraction_index = 1.52;
/// let normal = Vector3D::new(0., 0., 1.);
/// let ray = Ray{
///     geometry: Ray3D{
///         origin: Point3D::new(0., 0., 1.),
///         direction: Vector3D::new(0., 1., -2.).get_normalized()
///     },
///     .. Ray::default()
/// };
/// let (n1, cos1, n2, cos2) = cos_and_n(&ray, normal, mat_refraction_index);
/// ```
pub fn cos_and_n(
    ray: &Ray,
    normal: Vector3D,
    refraction_index: Float,
) -> (Float, Float, Float, Option<Float>) {
    let vin = ray.geometry.direction;

    let cos1 = (vin * normal).abs();
    let n1 = ray.refraction_index;
    let mut n2 = refraction_index;
    // If the ray already has this refraction index, assume
    // we are leaving a volume, entering air.
    if (n1 - n2).abs() < 1e-7 {
        n2 = 1.0;
    }
    // Calculate cos2
    let sin1_sq = (1. - cos1 * cos1).clamp(0., Float::MAX);
    let sin2_sq = n1 * n1 * sin1_sq / (n2 * n2); // Snell's law squared
    #[cfg(debug_assertions)]
    {
        let lhs = n1 * sin1_sq.sqrt();
        let rhs = n2 * sin2_sq.sqrt();
        debug_assert!((lhs - rhs).abs() < 1e-5, "rhs = {}, lhs = {}", lhs, rhs);
    }
    debug_assert!(sin2_sq >= 0.0);
    if sin2_sq > 1. {
        // Pure reflection...
        return (n1, cos1, n2, None);
    }

    let cos2 = (1. - sin2_sq).sqrt();

    (n1, cos1, n2, Some(cos2))
}

/// Fresnel Coefficient for TE-Polarized Light (i.e., perpendicular), according to PBR-book
///
/// `n1` is the index of refraction on the ray's side; `cos1` is the
/// cosine of the angle between the surface's normal and ray.
///
/// `n2` is the index of refraction on the side opposite to the ray; `cos2` is the
/// cosine of the angle between the surface's normal and transmitted ray
pub fn fresnel_te(n1: Float, cos1: Float, n2: Float, cos2: Float) -> Float {
    (n1 * cos1 - n2 * cos2) / (n1 * cos1 + n2 * cos2)
}

/// Fresnel Coefficient for TM-Polarized Light (i.e., parallel), according to PBR-book
///
/// `n1` is the index of refraction on the ray's side; `cos1` is the
/// cosine of the angle between the surface's normal and ray.
///
/// `n2` is the index of refraction on the side opposite to the ray; `cos2` is the
/// cosine of the angle between the surface's normal and transmitted ray
pub fn fresnel_tm(n1: Float, cos1: Float, n2: Float, cos2: Float) -> Float {
    // (n2 * cos1 - n1 * cos2) / (n2 * cos1 + n1 * cos2)
    (n1 / cos1 - n2 / cos2) / (n1 / cos1 + n2 / cos2)
}

pub fn fresnel_reflectance(n1: Float, cos1: Float, n2: Float, cos2: Float) -> Float {
    let parallel = fresnel_tm(n1, cos1, n2, cos2);
    let perpendicular = fresnel_te(n1, cos1, n2, cos2);
    0.5 * (parallel * parallel + perpendicular * perpendicular)
}

/// Calculates the direction of the transmision
pub fn fresnel_transmission_dir(
    vin: Vector3D,
    normal: Vector3D,
    n1: Float,
    cos1: Float,
    n2: Float,
    cos2: Float,
) -> Vector3D {
    // Check inputs
    debug_assert!(
        (1. - vin.length()).abs() < 1e-5,
        "length is {}",
        vin.length()
    );
    debug_assert!(
        (1. - normal.length()).abs() < 1e-5,
        "length is {}",
        normal.length()
    );

    debug_assert!(cos1 > 0.);
    debug_assert!(cos2 > 0.);
    debug_assert!(n1 > 0.);
    debug_assert!(n2 > 0.);
    if vin * normal > 0.0 {
        dbg!(vin * normal);
        debug_assert!(vin * normal < 0., "vin*normal = {}", vin * normal);
    }

    let n_ratio = n1 / n2;
    let ret = vin * n_ratio + normal * (n_ratio * cos1 - cos2);
    // check that it is normalized
    debug_assert!(
        (1. - ret.length()).abs() < 1e-5,
        "length is {}",
        ret.length()
    );
    // check that it is not pointing in the same direction as normal.
    debug_assert!(normal * ret < 0., "normal = {}, ret = {}", normal, ret);
    ret
}

/// Calculates the purely specular reflection direction.
pub fn mirror_direction(vin: Vector3D, normal: Vector3D) -> Vector3D {
    debug_assert!((vin.length() - 1.).abs() < 1e-6);
    debug_assert!((normal.length() - 1.).abs() < 1e-6);
    let vin_normal = vin * normal;
    let mut ret = vin - normal * (2. * vin_normal);
    ret.normalize();

    ret
}

/// Calculates the Mirror BSDF and modifies the given ray so that it now points in that direction
pub fn mirror_bsdf(intersection_pt: Point3D, ray: &mut Ray, normal: Vector3D) -> Float {
    // avoid self shading
    ray.geometry.origin = intersection_pt + normal * 0.00001;
    let ray_dir = ray.geometry.direction;
    let cos = (ray_dir * normal).abs();
    ray.geometry.direction = mirror_direction(ray_dir, normal);
    debug_assert!(
        (ray.geometry.direction.length() - 1.).abs() < 1e-5,
        "dir len is {}",
        ray.geometry.direction.length()
    );
    1. / cos
}

/// Evaluates the mirror BSDf
pub fn eval_mirror_bsdf(normal: Vector3D, vin: Vector3D, vout: Vector3D) -> Float {
    let mirror = mirror_direction(vin, normal);
    if vout.is_parallel(mirror) {
        let cos = (vin * normal).abs();
        1. / cos
    } else {
        0.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mirror_direction() -> Result<(), String> {
        fn check(v: Vector3D, normal: Vector3D, mirror: Vector3D) -> Result<(), String> {
            let v = v.get_normalized();
            let normal = normal.get_normalized();
            let mirror = mirror.get_normalized();

            let found = mirror_direction(v, normal);
            if !(mirror - found).is_zero() {
                return Err(format!(
                    "Expected mirror direction was {} | found {}",
                    mirror, found
                ));
            }
            Ok(())
        }

        check(
            Vector3D::new(0., 0., 1.),
            Vector3D::new(0., 0., 1.),
            Vector3D::new(0., 0., -1.),
        )?;
        check(
            Vector3D::new(0., 0., -1.),
            Vector3D::new(0., 0., -1.),
            Vector3D::new(0., 0., 1.),
        )?;
        check(
            Vector3D::new(1., 0., -1.).get_normalized(),
            Vector3D::new(0., 0., 1.),
            Vector3D::new(1., 0., 1.),
        )?;

        Ok(())
    }

    #[test]
    fn test_angles() {
        // Example found online
        let n1 = 1.; // air
        let n2 = 1.5; // glass

        // TE and TM polarized light are indistinguishble at 0 degrees
        let cos1 = 1.; // 0
        let cos2 = cos1;
        let te = fresnel_te(n1, cos1, n2, cos2);
        let tm = fresnel_tm(n1, cos1, n2, cos2);
        assert!(
            (te.abs() - tm.abs()).abs() < 1e-8,
            "te = {}, tm = {}",
            te,
            tm
        );
    }
}
