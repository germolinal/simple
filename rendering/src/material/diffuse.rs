// pub fn sample_

use super::{
    bsdf_sample::BSDFSample,
    local_coordinates_utils::same_heisphere,
    mat_trait::{MatFlag, MaterialTrait, TransFlag},
};

use crate::{ray::TransportMode, Float, Spectrum, PI};
use geometry::Vector3D;

// pub fn sample_diffuse()

#[derive(Clone, Debug)]
pub struct Diffuse {
    pub colour: Spectrum,
}
impl Diffuse {
    pub fn new(rho: Spectrum) -> Self {
        Self { colour: rho }
    }
}
impl MaterialTrait for Diffuse {
    fn id(&self) -> &str {
        "Diffuse"
    }
    fn flags(&self) -> MatFlag {
        MatFlag::Diffuse
    }
    fn eval_bsdf(&self, wo: Vector3D, wi: Vector3D, _transport_mode: TransportMode) -> Spectrum {
        if same_heisphere(wo, wi) {
            self.colour / PI
        } else {
            Spectrum::BLACK
        }
    }

    fn sample_bsdf(
        &self,
        _wo: Vector3D,
        _eta: Float,
        _uc: Float,
        u: (Float, Float),
        _transport_mode: TransportMode,
        trans_flags: TransFlag,
    ) -> Option<BSDFSample> {
        if trans_flags & TransFlag::Reflection {
            Some(BSDFSample::new_diffuse(self.colour, u))
        } else {
            None
        }
    }
}
