use super::{
    bsdf_sample::BSDFSample,
    local_coordinates_utils::{abs_cos_theta, same_heisphere},
    mat_trait::{MatFlag, MaterialTrait, TransFlag},
    RandGen,
};
use rand::*;

use crate::{material::TransportMode, Float, Spectrum, PI};
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

    fn eval_bsdf(
        &self,
        wo: Vector3D,
        wi: Vector3D,
        _eta: Float,
        _transport_mode: TransportMode,
    ) -> Spectrum {
        if same_heisphere(wo, wi) {
            self.colour / PI
        } else {
            Spectrum::BLACK
        }
    }

    fn pdf(
        &self,
        wo: Vector3D,
        wi: Vector3D,
        _eta: Float,
        _transport_mode: TransportMode,
    ) -> Float {
        if same_heisphere(wo, wi) {
            abs_cos_theta(wi) / PI
        } else {
            0.0
        }
    }

    fn sample_bsdf(
        &self,
        _wo: Vector3D,
        _eta: Float,
        rng: &mut RandGen,
        _transport_mode: TransportMode,
        trans_flags: TransFlag,
    ) -> Option<BSDFSample> {
        let u = rng.gen();
        if trans_flags & TransFlag::Reflection {
            Some(BSDFSample::new_diffuse(self.colour, u))
        } else {
            None
        }
    }

    fn colour(&self) -> Spectrum {
        self.colour
    }
}
