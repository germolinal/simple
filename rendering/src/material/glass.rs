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
use crate::material::specular::*;
use crate::ray::Ray;
use crate::Float;
use geometry::{Point3D, Vector3D};

fn any_transmission(colour: &mut Spectrum) -> bool {
    const MIN_COLOUR: Float = 1e-15;
    if colour.max() < MIN_COLOUR {
        false
    } else {
        for v in colour.0.iter_mut() {
            if *v < MIN_COLOUR {
                *v = MIN_COLOUR;
            }
        }
        true
    }
}

#[derive(Debug, Clone)]
pub struct Glass {
    pub colour: Spectrum,
    pub refraction_index: Float,
}

impl Glass {
    /// Returns the Reflection and Transmission values for a glass. These values
    /// have already been modified by the colour of the glass
    pub fn refl_trans(
        &self,
        normal: Vector3D,
        direction: Vector3D,
        cos1: Float,
    ) -> (Spectrum, Spectrum) {
        debug_assert!(cos1 > 0.0);

        // Check if there is any transmission
        let mut colour = self.colour;
        let any_transmission = any_transmission(&mut colour);

        // Now calculate components
        // if let Some(cos2) = cos2 {
        let pdot = (normal * direction).abs();
        let cos2 = ((1. - 1. / self.refraction_index.powi(2))
            + (pdot / self.refraction_index).powi(2))
        .sqrt();

        let rindex = self.refraction_index;
        let mut r1e = (pdot - rindex * cos2) / (pdot + rindex * cos2);
        r1e *= r1e;
        let mut r1m = (1.0 / pdot - rindex / cos2) / (1.0 / pdot + rindex / cos2);
        r1m *= r1m;

        let d = if any_transmission {
            self.colour.powf(1. / cos2)
        } else {
            self.colour
        };

        // Process transmission
        let t_comp = if any_transmission {
            d * 0.5
                * ((1. - r1e).powi(2) / (1. - (d * r1e).powi(2))
                    + (1. - r1m).powi(2) / (1. - (d * r1m).powi(2)))
        } else {
            // Spectrum{red: 1., green: 0., blue: 0.}
            Spectrum::BLACK
        };

        // Process reflection
        let d = d.powi(2);
        let refl_comp = 0.5
            * (r1e * (1.0 + (1.0 - 2.0 * r1e) * d) / (1.0 - r1e * r1e * d)
                + r1m * (1.0 + (1.0 - 2.0 * r1m) * d) / (1.0 - r1m * r1m * d));

        // return
        (refl_comp, t_comp)
    }
}

impl Glass {
    pub fn id(&self) -> &str {
        "Glass"
    }

    pub fn colour(&self) -> Spectrum {
        let mut c = self.colour;
        _ = any_transmission(&mut c);
        c
    }

    pub fn get_possible_paths(
        &self,
        normal: &Vector3D,
        intersection_pt: &Point3D,
        ray: &Ray,
    ) -> [Option<(Ray, Spectrum)>; 2] {
        let normal = *normal;
        // Only two possible direction:

        let mirror_dir = mirror_direction(ray.geometry.direction, normal);

        debug_assert!(
            // some paranoia checks
            (1. - mirror_dir.length()).abs() < 1e-5,
            "length is {}",
            mirror_dir.length()
        );
        let (_n1, cos1, ..) = cos_and_n(ray, normal, self.refraction_index);
        let intersection_pt = *intersection_pt;
        let (refl, trans) = self.refl_trans(normal, ray.geometry.direction, cos1);

        // let total = refl.radiance() + trans.radiance();
        // let refl_fraction = refl.radiance()/total;
        // let trans_fraction = trans.radiance()/total;
        // dbg!(total, refl_fraction, trans_fraction);

        // Process reflection...
        let mut ray1 = *ray;
        ray1.geometry.direction = mirror_dir;
        ray1.geometry.origin = intersection_pt + normal * 0.00001;
        let pair1 = Some((ray1, refl));

        // process transmission
        let mut ray = *ray;
        let pair2 = if trans.radiance() > 0.0 {
            ray.geometry.origin = intersection_pt - normal * 0.00001;

            // ray.colour *= self.colour() * trans;
            Some((ray, trans))
        } else {
            None
        };

        [pair1, pair2]
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

    //     debug_assert!(
    //         (ray.geometry.direction.length() - 1.).abs() < 1e-5,
    //         "Length was {}",
    //         ray.geometry.direction.length()
    //     );
    //     debug_assert!((e1 * e2).abs() < 1e-8);
    //     debug_assert!((e1 * normal).abs() < 1e-8);
    //     debug_assert!((e2 * normal).abs() < 1e-8);

    //     let (n1, cos1, n2, cos2) = cos_and_n(ray, normal, self.refraction_index);
    //     let (refl, trans) = self.refl_trans(n1, cos1, n2, cos2);
    //     let ray_dir = ray.geometry.direction;
    //     let mirror_dir = mirror_direction(ray_dir, normal);
    //     debug_assert!(
    //         (1. - mirror_dir.length()).abs() < 1e-5,
    //         "length is {}",
    //         mirror_dir.length()
    //     );

    //     let r: Float = rng.gen();
    //     if r <= refl / (refl + trans) {
    //         // Reflection
    //         // avoid self shading
    //         ray.geometry.origin = intersection_pt + normal * 0.00001;

    //         ray.geometry.direction = mirror_dir;
    //         (Spectrum::ONE * refl, refl / (refl + trans))
    //     } else {
    //         // Transmission... keep same direction, dont change refraction
    //         // avoid self shading
    //         ray.geometry.origin = intersection_pt - normal * 0.00001;
    //         (self.colour * trans, trans / (refl + trans))
    //     }
    // }

    pub fn eval_bsdf(
        &self,
        normal: Vector3D,
        _e1: Vector3D,
        _e2: Vector3D,
        ray: &Ray,
        vout: Vector3D,
    ) -> Spectrum {
        let (_n1, cos1, _n2, cos2) = cos_and_n(ray, normal, self.refraction_index);
        let (refl, trans) = self.refl_trans(normal, ray.geometry.direction, cos1);
        let vin = ray.geometry.direction;
        let mirror_dir = mirror_direction(vin, normal);
        debug_assert!(
            (1. - mirror_dir.length()).abs() < 1e-5,
            "length is {}",
            mirror_dir.length()
        );

        // If reflection
        if vout.is_same_direction(mirror_dir) {
            return refl;
        }

        let mut colour = self.colour;
        if any_transmission(&mut colour) {
            // it is not refraction either
            return Spectrum::BLACK;
        }
        // Check transmission
        if let Some(_cos2) = cos2 {
            if vout.is_same_direction(vin) {
                return self.colour * trans;
            }
        }
        // panic!("Glass should never reach critical angle");
        Spectrum::ONE / cos1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rand::*;
    use geometry::{Ray3D, Vector3D};

    #[test]
    fn test_get_possible_paths_glass() {
        let glass = Glass {
            colour: Spectrum([0.1, 0.2, 0.3]),

            refraction_index: 1.52,
        };
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

            let paths = glass.get_possible_paths(&normal, &intersection_pt, &ray);
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
            if let Some((new_ray, bsdf)) = paths[1] {
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
                assert!(new_ray.geometry.direction.compare(direction), "Expecting transmitted direction to be the same as the original direction (ray.dir = {} | exp = {})... length of diff = {}", ray.geometry.direction, direction, (new_ray.geometry.direction - direction).length());
            }
        }
    }
}
