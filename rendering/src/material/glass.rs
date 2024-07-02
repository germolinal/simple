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

use crate::material::specular::*;

use crate::Float;
use crate::{
    colour::Spectrum,
    material::mat_trait::{MatFlag, MaterialTrait, TransFlag},
    ray::TransportMode,
};
use geometry::Vector3D;

use super::bsdf_sample::BSDFSample;

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
    fn transmitted_cos(&self, direction: Vector3D) -> Float {
        let rindex = self.refraction_index;
        let pdot = direction.z.abs();
        ((1. - 1. / rindex.powi(2)) + (pdot / rindex).powi(2)).sqrt()
    }
}

impl MaterialTrait for Glass {
    fn id(&self) -> &str {
        "Glass"
    }

    fn flags(&self) -> MatFlag {
        MatFlag::SpecularTransmissionReflection
    }

    fn colour(&self) -> Spectrum {
        let mut c = self.colour;
        _ = any_transmission(&mut c);
        c
    }

    fn sample_bsdf(
        &self,
        wo: Vector3D,
        eta: Float,
        uc: Float,
        _u: (Float, Float),
        _transport_mode: TransportMode,
        trans_flags: TransFlag,
    ) -> Option<BSDFSample> {
        // Get reflecte and transmitted components.
        let mut colour = self.colour;
        let any_transmission = any_transmission(&mut colour);

        // Now calculate components
        let rindex = self.refraction_index;
        let cos1 = wo.z.abs();
        let cos2 = self.transmitted_cos(wo);

        let mut fte = (cos1 - rindex * cos2) / (cos1 + rindex * cos2);
        fte *= fte;
        let mut ftm = (1.0 / cos1 - rindex / cos2) / (1.0 / cos1 + rindex / cos2);
        ftm *= ftm;

        let ct = if any_transmission {
            colour.powf(1. / cos2)
        } else {
            colour
        };

        // Process transmission
        let trans = if any_transmission {
            ct * 0.5
                * ((1. - fte).powi(2) / (1. - (ct * fte).powi(2))
                    + (1. - ftm).powi(2) / (1. - (ct * ftm).powi(2)))
        } else {
            Spectrum::BLACK
        };

        // Process reflection
        let ct = ct.powi(2);
        let refl = 0.5
            * (fte * (1. + (1. - 2. * fte) * ct) / (1. - fte.powi(2) * ct)
                + ftm * (1. + (1. - 2. * ftm) * ct) / (1. - ftm.powi(2) * ct));

        let wi = mirror_direction(wo);
        // let cos1 = wi.z.abs();

        // handle probabilities
        let mut pr = refl.radiance();
        let mut pt = trans.radiance();
        if !any_transmission {
            pr = 1.0;
            pt = 0.0;
        }
        if !(trans_flags & TransFlag::Reflection) {
            pr = 0.;
        }
        if !(trans_flags & TransFlag::Transmission) {
            pt = 0.;
        }
        if pr < 1e-19 && pt < 1e-19 {
            return None;
        }

        if uc <= pr / (pr + pt) {
            // Reflection

            Some(BSDFSample::new(
                refl / cos1,
                wi,
                pr / (pr + pt),
                MatFlag::SpecularReflection,
            ))
        } else {
            // Transmission
            Some(BSDFSample::new(
                trans / cos1,
                wo,
                pt / (pr + pt),
                MatFlag::SpecularTransmission,
            ))
        }
    }

    fn eval_bsdf(
        &self,
        _wo: Vector3D,
        _wi: Vector3D,
        _eta: Float,
        _transport_mode: TransportMode,
    ) -> Spectrum {
        Spectrum::BLACK
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
// }
