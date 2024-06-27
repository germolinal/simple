use super::{local_coordinates_utils::abs_cos_theta, mat_trait::MatFlag};
use crate::{
    samplers::{sample_cosine_weighted_horizontal_hemisphere, sample_uniform_hemisphere},
    Float, Spectrum, PI,
};
use geometry::Vector3D;

#[derive(Clone, Debug)]
pub struct BSDFSample {
    pub pdf: Float,
    pub eta: Float,
    pub pdf_is_proportional: bool,
    pub spectrum: Spectrum,
    pub flags: MatFlag,
    pub wi: Vector3D,
}

impl BSDFSample {
    pub fn new(f: Spectrum, wi: Vector3D, pdf: Float, flags: MatFlag) -> Self {
        Self {
            spectrum: f,
            wi,
            pdf,
            flags,
            ..Self::default()
        }
    }

    pub fn new_diffuse(f: Spectrum, u: (Float, Float)) -> Self {
        let mut wi = sample_cosine_weighted_horizontal_hemisphere(u);
        if wi.z < 0.0 {
            wi.z *= -1.0;
        }
        let pdf = abs_cos_theta(wi) / PI;
        let s = f / PI;
        BSDFSample::new(s, wi, pdf, MatFlag::DiffuseReflection)
    }

    pub fn with_eta(f: Spectrum, wi: Vector3D, pdf: Float, flags: MatFlag, eta: Float) -> Self {
        let mut ret = Self::new(f, wi, pdf, flags);
        ret.eta = eta;
        ret
    }
    pub fn with_is_proportional(
        f: Spectrum,
        wi: Vector3D,
        pdf: Float,
        flags: MatFlag,
        pdf_is_proportional: bool,
    ) -> Self {
        let mut ret = Self::new(f, wi, pdf, flags);
        ret.pdf_is_proportional = pdf_is_proportional;
        ret
    }

    pub fn with_eta_and_is_proportional(
        f: Spectrum,
        wi: Vector3D,
        pdf: Float,
        flags: MatFlag,
        eta: Float,
        pdf_is_proportional: bool,
    ) -> Self {
        let mut ret = Self::new(f, wi, pdf, flags);
        ret.pdf_is_proportional = pdf_is_proportional;
        ret.eta = eta;
        ret
    }

    pub fn is_reflection(&self) -> bool {
        self.flags.is_reflective()
    }
    pub fn is_transmission(&self) -> bool {
        self.flags.is_transmissive()
    }
    pub fn is_diffuse(&self) -> bool {
        self.flags.is_diffuse()
    }
    pub fn is_glossy(&self) -> bool {
        self.flags.is_glossy()
    }
    pub fn is_specular(&self) -> bool {
        self.flags.is_specular()
    }
}

impl std::default::Default for BSDFSample {
    fn default() -> Self {
        Self {
            pdf: 0.0,
            eta: 1.0,
            pdf_is_proportional: false,
            spectrum: Spectrum::BLACK,
            flags: MatFlag::Unset,
            wi: Vector3D::new(0., 0., 0.),
        }
    }
}
