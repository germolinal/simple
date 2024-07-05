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

use crate::ray::TransportMode;
use crate::Spectrum;
use crate::{Float, PI};
use geometry::Vector3D;
use rand::*;

use super::bsdf_sample::BSDFSample;
use super::local_coordinates_utils::same_heisphere;
use super::mat_trait::{MatFlag, TransFlag};
use super::RandGen;

const LOW_ROUGHNESS: Float = 1e-3;

/// Samples a Ward BSDF, changing the direction of a given Ray. Returns a tuple with the
/// Specular and Diffuse reflections, as well as the Weighting factor for the BSDF
///
/// This implementation is based on "A new Ward BRDF model with bounded albedo" (2010),
/// by David Geisler-Moroder and Arne Dür
pub fn sample_ward_anisotropic(
    specularity: Float,
    mut alpha: Float,
    mut beta: Float,
    wo: Vector3D,
    rng: &mut RandGen,
    transport_mode: TransportMode,
    _trans_flags: TransFlag,
) -> Option<BSDFSample> {
    alpha = alpha.max(LOW_ROUGHNESS);
    beta = beta.max(LOW_ROUGHNESS);

    let prob_spec: Float = rng.gen();

    if prob_spec < specularity {
        let mut attempts = 0;
        loop {
            attempts += 1;
            let (xi1, xi2): (Float, Float) = rng.gen();
            // incident direction

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
            };

            let h = Vector3D::new(cosp * d, sinp * d, 1.0);
            d = (h * wo) * 2. / (1. + d.powi(2));
            let mut v = -wo;
            v.z += d;
            v.normalize();
            debug_assert!((1. - v.length()).abs() < 1e-5, "len of v = {}", v.length());

            let l_n = wo.z;
            let v_n = v.z;
            if v_n < 0.0 || l_n < 0.0 {
                // Here we want to evaluate the BSDF before we update the ray... otherwise the returned value would be incorrect
                let (spec, _diffuse) =
                    evaluate_ward_anisotropic(specularity, alpha, beta, wo, v, transport_mode);
                assert!(
                    !spec.is_nan(),
                    "incorrect (i.e., NaN) bsdf when calculating Ward aniso."
                );

                // eq. 14 and 15
                let pdf = 0.5 * (1. + v_n / l_n) * spec / specularity;
                let mut spectrum = Spectrum::gray(spec);
                spectrum /= l_n.abs();
                if pdf < 1e-8 && spec < 1e-8 {
                    continue;
                }

                let ret = BSDFSample {
                    pdf,
                    spectrum,
                    flags: MatFlag::GlossyReflection,
                    wi: -v,
                    ..Default::default()
                };

                return Some(ret);
            } else if attempts > 50 {
                let ret = BSDFSample {
                    pdf: 1.0, // irrelevant, because spectrum is Zero
                    spectrum: Spectrum::BLACK,
                    flags: MatFlag::GlossyReflection,
                    wi: v,
                    ..Default::default()
                };
                return Some(ret);
            }
        } // end of loop. If we did not return, try again.
    } else {
        let ret = BSDFSample::new_diffuse(Spectrum::gray(1. - specularity), rng.gen());
        Some(ret)
    }
}

pub fn ward_pdf(
    specularity: Float,
    alpha: Float,
    beta: Float,
    wo: Vector3D,
    wi: Vector3D,
    transport_mode: TransportMode,
) -> Float {
    // Here we want to evaluate the BSDF before we update the ray... otherwise the returned value would be incorrect
    let (spec, _diffuse) =
        evaluate_ward_anisotropic(specularity, alpha, beta, wo, wi, transport_mode);
    assert!(
        !spec.is_nan(),
        "incorrect (i.e., NaN) bsdf when calculating Ward aniso."
    );

    // eq. 14 and 15
    0.5 * (1. + wo.z / wi.z) * spec / specularity
}

/// Evaluates a Ward BSDF
///
/// This implementation is based on "A new Ward BRDF model with bounded albedo" (2010),
/// by David Geisler-Moroder and Arne Dür
pub fn evaluate_ward_anisotropic(
    specularity: Float,
    mut alpha: Float,
    mut beta: Float,
    wo: Vector3D,
    wi: Vector3D,
    _transport_mode: TransportMode,
) -> (Float, Float) {
    // let wi = mirror_direction(wo);
    if !same_heisphere(wo, wi) {
        return (0.0, 0.0);
    }

    let spec = if specularity > 1e-5 {
        alpha = alpha.max(LOW_ROUGHNESS);
        beta = beta.max(LOW_ROUGHNESS);

        let h = (wo + wi).get_normalized();
        let h_n = h.z;
        // Eq. 17
        let c1 = specularity * (h * h) / (PI * alpha * beta * h_n.powi(4));
        let c2 = -((h.x / alpha).powi(2) + (h.y / beta).powi(2)) / (h_n.powi(2));
        c1 * c2.exp()
    } else {
        0.0
    };

    let diff = (1. - specularity) / PI;
    (spec, diff)
}

#[cfg(test)]
mod tests {

    use geometry::Vector3D;

    use crate::{
        material::{get_rng, mat_trait::TransFlag, mirror_direction},
        ray::TransportMode,
    };

    use super::*;

    #[test]
    fn sample_ward() {
        let mut rng = get_rng();
        let specularity = 0.95;
        let alpha = 0.02;
        let beta = 0.03;
        let wo = Vector3D::new(1., 0., 1.).get_normalized();
        let _wi = -mirror_direction(wo);

        for _ in 0..100 {
            let sample = sample_ward_anisotropic(
                specularity,
                alpha,
                beta,
                wo,
                &mut rng,
                TransportMode::Radiance,
                TransFlag::default(),
            )
            .unwrap();

            // println!("{} * {} ={}", sample.wi, wi, sample.wi * wi);
            // println!("{} - {}", sample.spectrum, sample.pdf);
            println!("{},{},{}", sample.wi.x, sample.wi.z, sample.pdf);
        }
    }
    #[test]
    fn evaluate_ward() {
        let specularity = 0.1;
        let alpha = 0.5;
        let beta = 0.5;
        let wo = Vector3D::new(1., 0., -1.).get_normalized();

        let n = 90;
        let delta = 9.0 / n as Float;
        for i in 0..n {
            let wi = Vector3D::new(-1., 0., -delta * i as Float).get_normalized();

            // println!("{}", wi)
            let (spec, _diff) = evaluate_ward_anisotropic(
                specularity,
                alpha,
                beta,
                wo,
                wi,
                crate::ray::TransportMode::Radiance,
            );

            let plot = wi * spec;

            println!("{:.6},{:.6}", plot.x, plot.z);
        }
    }
}
