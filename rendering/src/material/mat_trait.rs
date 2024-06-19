use super::bsdf_sample::BSDFSample;
use super::local_coordinates_utils::abs_cos_theta;
use crate::samplers::{sample_uniform_hemisphere, uniform_sample_hemisphere};
use crate::Float;
use crate::PI;
use crate::{ray::TransportMode, Spectrum};
use geometry::Vector3D;
use std::ops::BitAnd;

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub(crate) enum TransFlag {
    Unset = 0,
    Reflection = 1 << 0,
    Transmission = 1 << 1,
    All = 1 << 0 | 1 << 1,
}

const UNSET: u8 = 0;
const REFLECTION: u8 = 1 << 0;
const TRANSMISSION: u8 = 1 << 1;
const DIFFUSE: u8 = 1 << 2;
const GLOSSY: u8 = 1 << 3;
const SPECULAR: u8 = 1 << 4;
#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum MatFlag {
    Unset = UNSET,
    Reflection = REFLECTION,
    Transmission = TRANSMISSION,
    Diffuse = DIFFUSE,
    Glossy = GLOSSY,
    Specular = SPECULAR,
    DiffuseReflection = DIFFUSE | REFLECTION,
    DiffuseTransmission = DIFFUSE | TRANSMISSION,
    GlossyReflection = GLOSSY | REFLECTION,
    GlossyTransmission = GLOSSY | TRANSMISSION,
    SpecularReflection = SPECULAR | REFLECTION,
    SpecularTransmission = SPECULAR | TRANSMISSION,
    All = DIFFUSE | GLOSSY | SPECULAR | REFLECTION | TRANSMISSION,
}

impl MatFlag {
    pub fn is_reflective(self) -> bool {
        self & MatFlag::Reflection
    }
    pub fn is_transmissive(self) -> bool {
        self & MatFlag::Transmission
    }
    pub fn is_diffuse(self) -> bool {
        self & MatFlag::Diffuse
    }
    pub fn is_glossy(self) -> bool {
        self & MatFlag::Glossy
    }
    pub fn is_specular(self) -> bool {
        self & MatFlag::Specular
    }
    pub fn is_non_specular(self) -> bool {
        self as u8 & (MatFlag::Diffuse as u8 | MatFlag::Glossy as u8) > 0
    }
}

impl BitAnd for MatFlag {
    type Output = bool;

    // rhs is the "right-hand side" of the expression `a & b`
    fn bitand(self, rhs: Self) -> Self::Output {
        self as u8 & rhs as u8 > 0
    }
}

pub trait MaterialTrait: std::fmt::Debug {
    /// Queries the material behaviour
    fn flags(&self) -> MatFlag;

    /// Evaluates the BSDF based on the pair of incoming and outgoing vectors
    ///
    /// Transport mode indicates whether paths are being constructed
    /// towards the camera or towards light sources
    ///
    /// wo and wi are in local coordinates (which is why we do not need e1, e2, etc.)
    fn eval_bsdf(&self, wo: Vector3D, wi: Vector3D, transport_mode: TransportMode) -> Spectrum;

    /// Samples the BSDF
    ///
    /// if in doubt, set `transport_mode` to `TransportMode::Radiance`, and `trans_flags` to `TransFlag::All`.
    /// These are the defaults in [C++'s version of the code](https://pbr-book.org/4ed/Reflection_Models/BSDF_Representation#BxDF::Sample_f)
    fn sample_bsdf(
        &self,
        wo: Vector3D,
        uc: Float,
        u: (Float, Float),
        transport_mode: TransportMode,
        trans_flags: TransFlag,
    ) -> Option<BSDFSample>;

    /// Calculates the directional-hemispherical reflectance
    ///
    /// In its base implementation, it uses the Montecarlo method to integrate.
    fn directional_rho(&self, wo: Vector3D, uc: &[Float], u2: &[(Float, Float)]) -> Spectrum {
        let mut rho = Spectrum::BLACK;
        uc.into_iter().zip(u2.into_iter()).for_each(|(a, b)| {
            if let Some(sample) =
                self.sample_bsdf(wo, *a, *b, TransportMode::Radiance, TransFlag::All)
            {
                rho += sample.spectrum * abs_cos_theta(sample.wi) / sample.pdf;
            }
        });
        rho / uc.len() as Float
    }

    /// Calculates the hemispherical-hemispherical reflectance
    ///
    /// In its base implementation, it uses the Montecarlo method to integrate.
    fn hemispherical_rho(&self, uc: &[Float], u2: &[(Float, Float)]) -> Spectrum {
        let mut rho = Spectrum::BLACK;
        uc.into_iter().zip(u2.into_iter()).for_each(|(a, b)| {
            let wo = sample_uniform_hemisphere(*b);
            if let Some(sample) =
                self.sample_bsdf(wo, *a, *b, TransportMode::Radiance, TransFlag::All)
            {
                const UNIFORM_HEMISPHERE_PDF: Float = 0.5 / PI;
                rho += sample.spectrum * abs_cos_theta(sample.wi) * abs_cos_theta(wo)
                    / (sample.pdf * UNIFORM_HEMISPHERE_PDF);
            }
        });
        rho / (PI * uc.len() as Float)
    }

    fn is_reflective(&self) -> bool {
        self.flags().is_reflective()
    }
    fn is_transmissive(&self) -> bool {
        self.flags().is_transmissive()
    }
    fn is_diffuse(&self) -> bool {
        self.flags().is_diffuse()
    }
    fn is_glossy(&self) -> bool {
        self.flags().is_glossy()
    }
    fn is_specular(&self) -> bool {
        self.flags().is_specular()
    }
    fn is_non_specular(&self) -> bool {
        self.flags().is_non_specular()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rand::*;
    use crate::{material::get_rng, samplers::sample_cosine_weighted_horizontal_hemisphere};

    #[test]
    fn test_non_specular() {
        assert!(!MatFlag::Unset.is_non_specular());
        assert!(MatFlag::Reflection.is_non_specular());
        assert!(!MatFlag::Transmission.is_non_specular());
        assert!(MatFlag::Diffuse.is_non_specular()); // YES!
        assert!(MatFlag::Glossy.is_non_specular()); // YES!
        assert!(!MatFlag::Specular.is_non_specular());
        assert!(MatFlag::DiffuseReflection.is_non_specular()); // YES
        assert!(MatFlag::DiffuseTransmission.is_non_specular()); // YES
        assert!(MatFlag::GlossyReflection.is_non_specular()); // YES
        assert!(MatFlag::GlossyTransmission.is_non_specular()); // YES
        assert!(!MatFlag::SpecularReflection.is_non_specular());
        assert!(!MatFlag::SpecularTransmission.is_non_specular());
        assert!(MatFlag::All.is_non_specular()); // YES
    }

    #[test]
    fn test_lambertian_hemispherical_reflectance() {
        #[derive(Debug)]
        struct Lambert {
            rho: Float,
        }
        impl MaterialTrait for Lambert {
            fn flags(&self) -> MatFlag {
                MatFlag::Diffuse
            }
            fn eval_bsdf(
                &self,
                _wo: Vector3D,
                _wi: Vector3D,
                _transport_mode: TransportMode,
            ) -> Spectrum {
                Spectrum::gray(self.rho / PI)
            }

            fn sample_bsdf(
                &self,
                _wo: Vector3D,
                _uc: Float,
                _u: (Float, Float),
                _transport_mode: TransportMode,
                _trans_flags: TransFlag,
            ) -> Option<BSDFSample> {
                let mut rng = crate::rand::get_rng();
                let wi = sample_cosine_weighted_horizontal_hemisphere(&mut rng);
                let ret = BSDFSample::new(
                    Spectrum::gray(self.rho / PI),
                    wi,
                    abs_cos_theta(wi) / PI,
                    MatFlag::Diffuse,
                );
                Some(ret)
            }
        }

        let mut rng = get_rng();
        let n = 500000;
        let mut u: Vec<Float> = Vec::with_capacity(n);
        let mut u2: Vec<(Float, Float)> = Vec::with_capacity(n);
        for _ in 0..n {
            let su: Float = rng.gen();
            u.push(su);
            let su2: (Float, Float) = rng.gen();
            u2.push(su2);
        }

        let mat = Lambert { rho: 1.0 };

        let rho = mat.directional_rho(Vector3D::new(1., 0., 0.), &u, &u2);
        assert!(
            (rho.0[0] - 1.0).abs() < 1e-8,
            "Found rho to be {}... expecting 1.0",
            rho.0[0]
        );

        let rho = mat.hemispherical_rho(&u, &u2);
        assert!(
            (rho.0[0] - 1.0).abs() < 1e-2,
            "Found rho to be {}... expecting 1.0",
            rho.0[0]
        );
    }
}
