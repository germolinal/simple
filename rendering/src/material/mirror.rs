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

use crate::colour::Spectrum;
use crate::material::specular::{eval_mirror_bsdf, mirror_bsdf};
use crate::ray::Ray;
use geometry::{Point3D, Vector3D};

/// A mirror material
pub struct Mirror(pub Spectrum);

impl Mirror {
    pub fn id(&self) -> &str {
        "Mirror"
    }

    pub fn colour(&self) -> Spectrum {
        self.0
    }

    pub fn get_possible_paths(
        &self,
        normal: &Vector3D,
        intersection_pt: &Point3D,
        ray: &Ray,
    ) -> [Option<(Ray, Spectrum)>; 2] {
        // Calculate the ray direction and BSDF
        let mut ray = *ray;
        let v = mirror_bsdf(*intersection_pt, &mut ray, *normal);
        let cos_theta = (*normal * ray.geometry.direction).abs();
        [
            Some((ray, Spectrum::ONE * v * cos_theta)),
            None,
        ]
    }

    // pub fn sample_bsdf(
    //     &self,
    //     _normal: Vector3D,
    //     _e1: Vector3D,
    //     _e2: Vector3D,
    //     _intersection_pt: Point3D,
    //     _ray: &mut Ray,
    //     _rng: &mut RandGen,
    // ) -> (Spectrum, Float) {
    //     unreachable!();
    //     // let bsdf = mirror_bsdf(intersection_pt, ray, normal);
    //     // (self.0 * bsdf, 1.)
    // }

    pub fn eval_bsdf(
        &self,
        normal: Vector3D,
        _e1: Vector3D,
        _e2: Vector3D,
        ray: &Ray,
        vout: Vector3D,
    ) -> Spectrum {
        let vin = ray.geometry.direction;
        self.0 * eval_mirror_bsdf(normal, vin, vout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rand::*;
    use crate::Float;
    use geometry::{Ray3D, Vector3D};

    #[test]
    fn test_get_possible_paths_mirror() {
        let mirror = Mirror(Spectrum([0.1, 0.2, 0.3]));

        let mut rng = get_rng();

        for _ in 0..500 {
            let refraction_index: Float = rng.gen();
            let (x, y, z): (Float, Float, Float) = rng.gen();
            let direction = Vector3D::new(x, y, -z).get_normalized();

            let normal = Vector3D::new(0., 0., 1.);
            let intersection_pt = Point3D::new(0., 0., 0.);
            let ray = Ray {
                geometry: Ray3D {
                    origin: Point3D::new(0., 0., 2.),
                    direction,
                },
                refraction_index,
                ..Ray::default()
            };

            let paths = mirror.get_possible_paths(&normal, &intersection_pt, &ray);
            // Reflection
            if let Some((new_ray, bsdf)) = paths[0] {
                assert_eq!(
                    new_ray.refraction_index, refraction_index,
                    "Expecting the ray's refraction index to be {}... found {}",
                    refraction_index, ray.refraction_index
                );
                assert!(
                    bsdf.radiance().is_finite() && !bsdf.radiance().is_nan(),
                    "impossible BSDF --> {}",
                    bsdf
                );
                let new_dir = new_ray.geometry.direction;
                assert!(( (new_dir.x - direction.x).abs() < 1e-5 && (new_dir.y - direction.y).abs() < 1e-5 && (new_dir.z  + direction.z).abs() < 1e-5 ), "Expecting reflected direction to be mirrored against direction (ray.dir = {} | exp = {}).", ray.geometry.direction, direction);
            } else {
                panic!("Expecting a reflection path")
            }

            // Transmission
            if let Some(_) = paths[1] {
                panic!("Mirrors should not transmit!")
            }
        }
    }
}
