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
use crate::{colour::Spectrum, ray::TransportMode};
use geometry::Vector3D;

use super::bsdf_sample::BSDFSample;
use super::mat_trait::{MatFlag, MaterialTrait, TransFlag};
use super::ward::ward_pdf;
use super::RandGen;

/// Information required for modelling Radiance's Metal and Metal
#[derive(Debug, Clone)]
pub struct Metal {
    pub colour: Spectrum,
    pub specularity: Float,
    pub roughness: Float,
}

impl MaterialTrait for Metal {
    fn id(&self) -> &str {
        "Metal"
    }

    fn flags(&self) -> MatFlag {
        MatFlag::GlossyReflection
    }

    fn colour(&self) -> Spectrum {
        self.colour
    }

    fn pdf(&self, wo: Vector3D, wi: Vector3D, eta: Float, transport_mode: TransportMode) -> Float {
        ward_pdf(
            self.specularity,
            self.roughness,
            self.roughness,
            wo,
            wi,
            transport_mode,
        )
    }

    fn sample_bsdf(
        &self,
        wo: Vector3D,
        _eta: Float,
        rng: &mut RandGen,
        transport_mode: TransportMode,
        trans_flags: TransFlag,
    ) -> Option<BSDFSample> {
        let mut ret = crate::material::ward::sample_ward_anisotropic(
            self.specularity,
            self.roughness,
            self.roughness,
            wo,
            rng,
            transport_mode,
            trans_flags,
        );

        // Plastic differs from Metal in that the direct component is coloured
        if let Some(sample) = &mut ret {
            sample.spectrum *= self.colour();
        }
        ret
    }

    fn eval_bsdf(
        &self,
        wo: Vector3D,
        wi: Vector3D,
        _eta: Float,
        transport_mode: TransportMode,
    ) -> Spectrum {
        let (direct, diffuse) = crate::material::ward::evaluate_ward_anisotropic(
            self.specularity,
            self.roughness,
            self.roughness,
            wo,
            wi,
            transport_mode,
        );

        self.colour() * (direct + diffuse)
    }
}
