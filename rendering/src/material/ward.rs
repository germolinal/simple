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

use crate::rand::*;
use crate::ray::Ray;
use crate::{Float, PI};
use geometry::{Point3D, Vector3D};

use crate::samplers::{local_to_world, sample_cosine_weighted_horizontal_hemisphere};

const LOW_ROUGHNESS: Float = 1e-3;

/// Samples a Ward BSDF, changing the direction of a given Ray. Returns a tuple with the
/// Specular and Diffuse reflections, as well as the Weighting factor for the BSDF
///
/// This implementation is based on "A new Ward BRDF model with bounded albedo" (2010),
/// by David Geisler-Moroder and Arne Dür
#[allow(clippy::too_many_arguments)]
pub fn sample_ward_anisotropic(
    normal: Vector3D,
    e1: Vector3D,
    e2: Vector3D,
    intersection_pt: Point3D,
    specularity: Float,
    mut alpha: Float,
    mut beta: Float,
    ray: &mut Ray,
    rng: &mut RandGen,
) -> (Float, Float, Float) {
    if alpha < LOW_ROUGHNESS {
        alpha = LOW_ROUGHNESS
    }
    if beta < LOW_ROUGHNESS {
        beta = LOW_ROUGHNESS
    }
    ray.geometry.origin = intersection_pt + normal * 0.00001;

    let prob_spec: Float = rng.gen();

    if prob_spec < specularity {
        loop {
            let (xi1, xi2): (Float, Float) = rng.gen();
            // incident direction
            let l = ray.geometry.direction * -1.;

            // From Radiance's https://github.com/NREL/Radiance/blob/2fcca99ace2f2435f32a09525ad31f2b3be3c1bc/src/rt/normal.c#L409
            let mut d = 2. * PI * xi1;
            let mut cosp = d.cos() * alpha;
            let mut sinp = d.sin() * beta;
            d = 1. / (cosp.powi(2) + sinp.powi(2)).sqrt();
            cosp *= d;
            sinp *= d;

            d = if xi2 < 1e-9 {
                1.
            } else {
                (-xi2.ln() / ((cosp / alpha).powi(2) + (sinp / beta).powi(2))).sqrt()
                // (alpha.powi(2) * -xi2.ln()).sqrt()
            };

            let h = normal + e1 * cosp * d + e2 * sinp * d;
            d = (h * l) * (-2.) / (1. + d.powi(2));
            let v = (l + normal * d).get_normalized();
            debug_assert!((1. - v.length()).abs() < 1e-5, "len of v = {}", v.length());

            let l_n = l * normal;
            let v_n = v * normal;

            // // let v_h = h * v;
            if v_n > 0.0 || l_n > 0.0 {
                // Here we want to evaluate the BSDF before we update the ray... otherwise the returned value would be incorrect
                let (spec, diffuse) = evaluate_ward_anisotropic(
                    normal,
                    e1,
                    e2,
                    specularity,
                    alpha,
                    beta,
                    ray,
                    v * -1.,
                );
                assert!(
                    !spec.is_nan(),
                    "incorrect (i.e., NaN) bsdf when calculating Ward aniso."
                );

                ray.geometry.direction = v; // update ray
                let weight = 2. / (1. + v_n / l_n); // Eq. 15
                return (spec, diffuse, weight);
            }
            // return (0.0, 0., 1.);
        } // end of loop. If we did not return, try again.
    } else {
        // Probability

        let diffuse = (1. - specularity) / PI;
        let new_dir = sample_cosine_weighted_horizontal_hemisphere(rng);
        let pdf = normal * new_dir / PI;
        ray.geometry.direction = new_dir;
        (0.0, diffuse, pdf)
    }
}

/// Evaluates a Ward BSDF
///
/// This implementation is based on "A new Ward BRDF model with bounded albedo" (2010),
/// by David Geisler-Moroder and Arne Dür
#[allow(clippy::too_many_arguments)]
pub fn evaluate_ward_anisotropic(
    normal: Vector3D,
    e1: Vector3D,
    e2: Vector3D,
    specularity: Float,
    mut alpha: Float,
    mut beta: Float,
    ray: &Ray,
    l: Vector3D,
) -> (Float, Float) {
    let spec = if specularity > 1e-5 {
        if alpha < LOW_ROUGHNESS {
            alpha = LOW_ROUGHNESS;
        }
        if beta < LOW_ROUGHNESS {
            beta = LOW_ROUGHNESS;
        }

        let i = ray.geometry.direction;
        let h = l - i;
        let i_n = i * normal;
        if i_n > 0. {
            return (0.0, 0.0);
        }

        let h_n = h * normal;
        // Eq. 17
        let c1 = specularity * (h * h) / (PI * alpha * beta * h_n.powi(4));
        let c2 = -((h * e1 / alpha).powi(2) + (h * e2 / beta).powi(2)) / (h_n.powi(2));
        c1 * c2.exp()
    } else {
        0.0
    };

    (spec, (1. - specularity) / PI)
}

#[cfg(test)]
mod tests {
    use geometry::{Point3D, Ray3D, Vector3D};

    use crate::{material::get_rng, Ray, PI};

    use super::sample_ward_anisotropic;

    // use super::*;
    #[test]
    fn test_diffuse_sampling() {
        let normal = Vector3D::new(0., 0., 1.);
        let e1 = Vector3D::new(1., 0., 0.);
        let e2 = Vector3D::new(0., 1., 0.);
        let intersection_pt = Point3D::new(0., 0., 0.);

        let specularity = 0.0;
        let alpha = 0.0;
        let beta = 0.0;

        let mut ray = Ray {
            geometry: Ray3D {
                direction: Vector3D::new(0., 0., -1.),
                origin: Point3D::new(0., 0., 1.),
            },
            ..Ray::default()
        };
        let mut rng = get_rng();

        for _ in 0..10 {
            let (direct, diffuse, pdf) = sample_ward_anisotropic(
                normal,
                e1,
                e2,
                intersection_pt,
                specularity,
                alpha,
                beta,
                &mut ray,
                &mut rng,
            );
            dbg!(direct, 1.0 / PI, diffuse, pdf, diffuse / pdf);
        }
    }
}
