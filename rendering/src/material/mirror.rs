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

use crate::colour::Spectrum;

use crate::ray::TransportMode;
use crate::Float;
use geometry::Vector3D;

use super::bsdf_sample::BSDFSample;
use super::mat_trait::{MatFlag, MaterialTrait, TransFlag};
use super::{mirror_direction, RandGen};

/// A mirror material
#[derive(Debug, Clone)]
pub struct Mirror(pub Spectrum);

impl MaterialTrait for Mirror {
    fn id(&self) -> &str {
        "Mirror"
    }

    fn colour(&self) -> Spectrum {
        self.0
    }

    fn flags(&self) -> MatFlag {
        MatFlag::Specular
    }

    fn pdf(
        &self,
        _wo: Vector3D,
        _wi: Vector3D,
        _eta: Float,
        _transport_mode: TransportMode,
    ) -> Float {
        0.
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

    fn sample_bsdf(
        &self,
        wo: Vector3D,
        _eta: Float,
        _rng: &mut RandGen,
        _transport_mode: TransportMode,
        _trans_flags: TransFlag,
    ) -> Option<BSDFSample> {
        let wi = mirror_direction(wo);
        let spectrum = self.colour() / wo.z.abs();
        let ret = BSDFSample::new(spectrum, wi, 1., MatFlag::SpecularReflection);
        Some(ret)
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
}

#[cfg(test)]
mod tests {}
