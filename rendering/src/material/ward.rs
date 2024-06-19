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

use super::bsdf_sample::BSDFSample;
use super::mat_trait::{MatFlag, TransFlag};

const LOW_ROUGHNESS: Float = 1e-3;

/// Samples a Ward BSDF, changing the direction of a given Ray. Returns a tuple with the
/// Specular and Diffuse reflections, as well as the Weighting factor for the BSDF
///
/// This implementation is based on "A new Ward BRDF model with bounded albedo" (2010),
/// by David Geisler-Moroder and Arne Dür
#[allow(clippy::too_many_arguments)]
pub fn sample_ward_anisotropic(
    specularity: Float,
    mut alpha: Float,
    mut beta: Float,
    wo: Vector3D,
    uc: Float,
    u: (Float, Float),
    transport_mode: TransportMode,
    _trans_flags: TransFlag,
) -> Option<BSDFSample> {
    if alpha < LOW_ROUGHNESS {
        alpha = LOW_ROUGHNESS
    }
    if beta < LOW_ROUGHNESS {
        beta = LOW_ROUGHNESS
    }

    let prob_spec = uc;

    if prob_spec < specularity {
        loop {
            let (xi1, xi2): (Float, Float) = u;
            // incident direction
            let l = wo * -1.;

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

            let h = Vector3D::new(cosp * d, sinp * d, 1.0);
            d = (h * l) * (-2.) / (1. + d.powi(2));
            let v = (l + Vector3D::new(0., 0., 1.) * d).get_normalized();
            debug_assert!((1. - v.length()).abs() < 1e-5, "len of v = {}", v.length());

            let l_n = l.z;
            let v_n = v.z;

            // // let v_h = h * v;
            if v_n > 0.0 || l_n > 0.0 {
                // Here we want to evaluate the BSDF before we update the ray... otherwise the returned value would be incorrect
                let (spec, _diffuse) = evaluate_ward_anisotropic(
                    specularity,
                    alpha,
                    beta,
                    wo,
                    v * -1.,
                    transport_mode,
                );
                assert!(
                    !spec.is_nan(),
                    "incorrect (i.e., NaN) bsdf when calculating Ward aniso."
                );

                let ret = BSDFSample {
                    pdf: 2. / (1. + v_n / l_n), // Eq. 15
                    spectrum: Spectrum::gray(spec),
                    flags: MatFlag::Reflection,
                    wi: v,
                    ..Default::default()
                };

                return Some(ret);
            }
        } // end of loop. If we did not return, try again.
    } else {
        let ret = BSDFSample::new_diffuse(Spectrum::gray(1. - specularity), u);
        Some(ret)
    }
}

/// Evaluates a Ward BSDF
///
/// This implementation is based on "A new Ward BRDF model with bounded albedo" (2010),
/// by David Geisler-Moroder and Arne Dür
#[allow(clippy::too_many_arguments)]
pub fn evaluate_ward_anisotropic(
    specularity: Float,
    mut alpha: Float,
    mut beta: Float,
    wo: Vector3D,
    wi: Vector3D,
    _transport_mode: TransportMode,
) -> (Float, Float) {
    let wo = wo * -1.;
    let spec = if specularity > 1e-5 {
        if alpha < LOW_ROUGHNESS {
            alpha = LOW_ROUGHNESS;
        }
        if beta < LOW_ROUGHNESS {
            beta = LOW_ROUGHNESS;
        }

        let h = wo - wi;
        let i_n = wi.z;
        if i_n > 0. {
            return (0.0, 0.0);
        }

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
mod tests {}
