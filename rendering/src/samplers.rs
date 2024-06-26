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
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

use crate::Float;
use geometry::{Point3D, Vector3D};

pub fn uniform_sample_triangle(u: (Float, Float), a: Point3D, b: Point3D, c: Point3D) -> Point3D {
    let (rand1, rand2) = u;

    const TINY: Float = 1e-8;
    let rand1 = rand1.clamp(TINY, 1. - TINY);
    let rand2 = rand2.clamp(TINY, 1. - TINY);
    let aux = rand1.sqrt();
    let u = 1. - aux;
    let v = rand2 * aux;
    let v1 = b - a;
    let v2 = c - a;
    // return
    a + v1 * u + v2 * v
}

pub fn uniform_sample_horizontal_disc(u: (Float, Float), radius: Float) -> (Float, Float) {
    // let (ux, uy) = (2.0 * u.0 - 1.0, 2.0 * u.1 - 1.0);
    // if ux < 1e-9 && uy < 1e-9 {
    //     return (0., 0.);
    // }

    // let (theta, mut r) = if ux.abs() > uy.abs() {
    //     let r = ux;
    //     let theta = (crate::PI / 4.0) * (uy / ux);
    //     (theta, r)
    // } else {
    //     let r = uy;
    //     let theta = crate::PI / 2.0 - (crate::PI / 4.0) * (ux / uy);
    //     (theta, r)
    // };
    // r *= radius;
    // let (sin, cos) = theta.sin_cos();
    // (r * sin, r * cos)

    let (r, theta) = u;

    let r = radius * r.sqrt();
    let theta = 2. * crate::PI * theta;
    let (theta_sin, theta_cos) = theta.sin_cos();

    let local_x = r * theta_sin;
    let local_y = r * theta_cos;
    (local_x, local_y)
}

/// Transforms a Point from Local Coordinates (defined by the triad `local_e1`, `local_e2` and `normal`,
/// centered at `centre`) into world coordinates. For converting a vector, set `centre = Point3D::new(0.0, 0., 0.)`
pub fn local_to_world(
    local_e1: Vector3D,
    local_e2: Vector3D,
    normal: Vector3D,
    centre: Point3D,
    x_local: Float,
    y_local: Float,
    z_local: Float,
) -> (Float, Float, Float) {
    // Check that they are normalized
    debug_assert!((1. - local_e1.length_squared()).abs() < 1e-4);
    debug_assert!((1. - local_e2.length_squared()).abs() < 1e-4);
    debug_assert!((1. - normal.length_squared()).abs() < 1e-4);

    let x = centre.x + x_local * local_e1.x + y_local * local_e2.x + z_local * normal.x;
    let y = centre.y + x_local * local_e1.y + y_local * local_e2.y + z_local * normal.y;
    let z = centre.z + x_local * local_e1.z + y_local * local_e2.z + z_local * normal.z;

    debug_assert!(
        (1. - Vector3D::new(x, y, z).length()) < 1e-5,
        "Expecting a unit vector... found length {:.6}",
        Vector3D::new(x, y, z).length()
    );

    (x, y, z)
}

/// Gets a random `Vector3D`, distributed according to `cos(theta)` according
/// to a normal `Vector3D(0,0,1)`
pub fn sample_cosine_weighted_horizontal_hemisphere(u: (Float, Float)) -> Vector3D {
    let (local_x, local_y) = uniform_sample_horizontal_disc(u, 1.);
    let aux = (local_x * local_x + local_y * local_y).clamp(0., 1.);
    let local_z = (1. - aux).sqrt();
    Vector3D::new(local_x, local_y, local_z)
}

/// Samples a hemisphere looking up
pub fn sample_uniform_hemisphere(u: (Float, Float)) -> Vector3D {
    let rand1 = u.0;
    let rand2 = u.1;
    let z = rand1;
    let r = (1.0 - rand1 * rand1).sqrt();
    let pie2 = 2.0 * crate::PI * rand2;
    let (pie2_sin, pie2_cos) = pie2.sin_cos();
    let x = pie2_cos * r;
    let y = pie2_sin * r;

    Vector3D::new(x, y, z)
}

/// Samples a tilted hemisphere pointing in N direction
pub fn uniform_sample_tilted_hemisphere(
    u: (Float, Float),
    e1: Vector3D,
    e2: Vector3D,
    normal: Vector3D,
) -> Vector3D {
    // Calculate in
    let v = sample_uniform_hemisphere(u);

    // Take back to world normal
    let (x, y, z) = local_to_world(e1, e2, normal, Point3D::new(0., 0., 0.), v.x, v.y, v.z);
    debug_assert!(
        (Vector3D::new(x, y, z).length() - 1.).abs() < 1e-5,
        "length is {}",
        Vector3D::new(x, y, z).length()
    );
    Vector3D::new(x, y, z)
}

pub fn uniform_sample_sphere(u: (Float, Float)) -> Point3D {
    const TWO_PI: Float = 2. * crate::PI;
    const TINY: Float = 1e-7;
    // Sample a sphere of radius 1 centered at the origin
    let (rand1, rand2) = u;
    let rand1 = rand1.clamp(TINY, 1. - TINY);
    let rand2 = rand2.clamp(TINY, 1. - TINY);
    let z = 1. - 2. * rand1;
    let aux = (1. - z * z).sqrt() as Float;
    let z = z as Float;
    let x = (TWO_PI * rand2).cos() as Float * aux;
    let y = (TWO_PI * rand2).sin() as Float * aux;

    Point3D::new(x, y, z)
}

pub fn uniform_sample_tilted_disc(
    u: (Float, Float),
    radius: Float,
    centre: Point3D,
    normal: Vector3D,
) -> Point3D {
    let (x_local, y_local) = uniform_sample_horizontal_disc(u, radius);

    // Form the basis
    let e2 = normal.get_perpendicular().unwrap();
    let e1 = e2.cross(normal);
    let (x, y, z) = local_to_world(
        e1,
        e2,
        normal,
        centre,
        x_local as Float,
        y_local as Float,
        0.,
    );
    Point3D::new(x, y, z)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rand::get_rng;
    use rand::*;

    #[test]
    fn test_uniform_sample_disc() -> Result<(), String> {
        fn check(radius: Float, centre: Point3D, normal: Vector3D) -> Result<(), String> {
            let mut rng = crate::rand::get_rng();
            let normal = normal.get_normalized();
            let u = rng.gen();
            let p = uniform_sample_tilted_disc(u, radius, centre, normal);
            if ((p - centre) * normal).abs() > 100. * Float::EPSILON {
                return Err(format!(
                    "Point is not coplanar with circle. ((p-centre)*normal).abs() == {}",
                    ((p - centre) * normal).abs()
                ));
            }
            if (p - centre).length() > radius {
                return Err(format!(
                    "Sample out of circle. Point sampled was {} | p-centre = {} | radius = {}",
                    p,
                    (p - centre).length(),
                    radius
                ));
            }

            Ok(())
        }

        for _ in 0..100 {
            check(1.2, Point3D::new(0., 0., 0.), Vector3D::new(0., 0., 1.))?;
            check(4.2, Point3D::new(3., 0., 0.), Vector3D::new(0., 1., 1.))?;
            check(0.12, Point3D::new(0., 1., 0.), Vector3D::new(0., 1., 0.))?;
            check(23., Point3D::new(0., -10., -20.), Vector3D::new(1., 1., 0.))?;
            check(23., Point3D::new(0., -10., -20.), Vector3D::new(1., 0., 0.))?;
        }

        Ok(())
    }

    #[test]
    fn test_uniform_sample_hemisphere() -> Result<(), String> {
        fn check(normal: Vector3D) -> Result<(), String> {
            let normal = normal.get_normalized();
            let e2 = normal.get_perpendicular()?;
            let e1 = e2.cross(normal);

            let mut rng = get_rng();
            let u = rng.gen();
            let dir = uniform_sample_tilted_hemisphere(u, e1, e2, normal);

            if (1. - dir.length()).abs() > 1e-5 {
                return Err(format!("Sampled direction (from uniform_sample_hemisphere) was nor normalized... {} (length = {})", dir, dir.length()));
            }
            if dir * normal < -Float::EPSILON * 30. {
                return Err(format!("Sampled direction (from uniform_sample_hemisphere) is not in hemisphere... Normal = {} | Dir = {}", normal, dir));
            }

            Ok(())
        }

        for _ in 0..9999999 {
            check(Vector3D::new(1., 2., -1.))?;
            check(Vector3D::new(-1., 0., 0.))?;
            check(Vector3D::new(0., 0., 1.))?;
            check(Vector3D::new(0., 1., 0.))?;
            check(Vector3D::new(-1000., -1., 2.))?;
        }

        Ok(())
    }

    #[test]
    fn test_cosine_weighted_sampling() {
        let mut rng = get_rng();
        for _ in 0..99999999 {
            let u = rng.gen();
            let a = sample_cosine_weighted_horizontal_hemisphere(u);
            assert!(a.length().is_finite());
        }
    }

    #[test]
    fn test_uniform_sample_horizontal_disc() {
        let mut rng = get_rng();
        for _ in 0..100 {
            let u = rng.gen();
            let (x, y) = uniform_sample_horizontal_disc(u, 1.0);
            assert!(!x.is_nan());
            assert!(!y.is_nan());
            assert!(x * x + y * y <= 1.0);
            println!("{},{}", x, y);
        }
    }
}
