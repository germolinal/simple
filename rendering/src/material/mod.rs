use crate::interaction::Interaction;
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
use crate::Float;
use crate::{colour::Spectrum, ray::TransportMode};
use bsdf_sample::BSDFSample;
use geometry::intersection::SurfaceSide;
use geometry::Vector3D;
use mat_trait::MaterialTrait;

mod light;
pub use light::Light;

mod plastic;
use mat_trait::TransFlag;
pub use plastic::Plastic;

mod metal;
pub use metal::Metal;

mod diffuse;
pub use diffuse::Diffuse;

mod dielectric;
pub use dielectric::Dielectric;

mod mirror;
pub use mirror::Mirror;

mod glass;
pub use glass::Glass;

mod specular;
pub use specular::*;

pub mod bsdf_sample;
mod local_coordinates_utils;

mod mat_trait;
mod ward;

#[derive(Clone, Debug)]
pub enum Material {
    Plastic(Plastic),
    Metal(Metal),
    Diffuse(Diffuse),
    Light(Light),
    Mirror(Mirror),
    Dielectric(Dielectric),
    Glass(Glass),
}

impl Material {
    /// Returns an id, for debugging and error reporting purposes
    pub fn id(&self) -> &str {
        match self {
            Self::Diffuse(m) => m.id(),
            Self::Plastic(m) => m.id(),
            Self::Metal(m) => m.id(),
            Self::Light(m) => m.id(),
            Self::Mirror(m) => m.id(),
            Self::Dielectric(m) => m.id(),
            Self::Glass(m) => m.id(),
        }
    }

    /// Retrieves the Colour of the material. This will usually
    /// represent the values that will multiply the different
    /// elements of the [`Spectrum`]. E.g., the reflectance values.
    pub fn colour(&self) -> Spectrum {
        match self {
            Self::Diffuse(m) => m.colour(),
            Self::Plastic(m) => m.colour(),
            Self::Metal(m) => m.colour(),
            Self::Light(m) => m.colour(),
            Self::Mirror(m) => m.colour(),
            Self::Dielectric(m) => m.colour(),
            Self::Glass(m) => m.colour(),
        }
    }

    /// Should this material be tested for direct illumination?
    pub fn emits_direct_light(&self) -> bool {
        matches!(self, Self::Light(_))
    }

    /// Should this material emits light
    pub fn emits_light(&self) -> bool {
        matches!(self, Self::Light(_))
    }

    /// Does this material scatter (e.g., like [`Plastic`]) or does it
    /// only transmit/reflects specularly (e.g., like [`Mirror`])?
    ///
    /// Defaults to `false`
    pub fn specular_only(&self) -> bool {
        matches!(self, Self::Mirror(_) | Self::Glass(_) | Self::Dielectric(_))
    }

    // pub fn get_possible_paths(
    //     &self,
    //     normal: &Vector3D,
    //     intersection_pt: &Point3D,
    //     ray: &Ray,
    // ) -> [Option<(Ray, Spectrum)>; 2] {
    //     match self {
    //         Self::Mirror(m) => m.get_possible_paths(normal, intersection_pt, ray),
    //         Self::Dielectric(m) => m.get_possible_paths(normal, intersection_pt, ray),
    //         Self::Glass(m) => m.get_possible_paths(normal, intersection_pt, ray),
    //         _ => panic!("Trying to get possible paths in non-specular material"),
    //     }
    // }

    pub fn to_local(&self, normal: Vector3D, e1: Vector3D, e2: Vector3D, v: Vector3D) -> Vector3D {
        Vector3D::new(v * e1, v * e2, v * normal)
    }
    pub fn to_world(&self, normal: Vector3D, e1: Vector3D, e2: Vector3D, v: Vector3D) -> Vector3D {
        e1 * v.x + e2 * v.y + normal * v.z
    }

    /// Samples the bsdf (returned by modifying the given `Ray`).
    /// Returns the value of the BSDF in that direction (as a Spectrum) and the probability
    pub fn sample_bsdf(
        &self,
        wo: Vector3D,
        interaction: &mut Interaction,
        eta: &mut Float,
        rng: &mut RandGen,
    ) -> Option<BSDFSample> {
        // world to local
        let (intersection_pt, normal, e1, e2) = interaction.get_triad();
        let wo = self.to_local(normal, e1, e2, wo);

        let uc: Float = rng.gen();
        let u: (Float, Float) = rng.gen();
        let transport_mode = TransportMode::default();
        let trans_flags = TransFlag::All;

        let mut ret = match self {
            Self::Diffuse(m) => m.sample_bsdf(wo, *eta, uc, u, transport_mode, trans_flags),
            Self::Plastic(m) => m.sample_bsdf(wo, *eta, uc, u, transport_mode, trans_flags),
            Self::Metal(m) => m.sample_bsdf(wo, *eta, uc, u, transport_mode, trans_flags),
            Self::Light(_m) => None, //panic!("Material '{}' has no BSDF", m.id()),
            Self::Mirror(m) => None, //m.sample_bsdf(wo, *eta, uc, u, transport_mode, trans_flags),
            Self::Dielectric(m) => {
                let ret = m.sample_bsdf(wo, *eta, uc, u, transport_mode, trans_flags);
                // if front or back?
                // dbg!("Fxix the refraction index transition");
                match interaction.geometry_shading.side {
                    // Going in
                    SurfaceSide::Front => *eta = m.refraction_index,
                    // Going out... TODO?: create a stack of refraction indexes
                    SurfaceSide::Back => *eta = 1.0,
                    _ => unreachable!(),
                }
                ret
            }
            Self::Glass(m) => m.sample_bsdf(wo, *eta, uc, u, transport_mode, trans_flags),
        };

        if let Some(sample) = &mut ret {
            if sample.is_reflection() {
                interaction.point = intersection_pt + normal * 0.00001;
            } else if sample.is_transmission() {
                interaction.point = intersection_pt - normal * 0.00001;
            }
            sample.wi = self.to_world(normal, e1, e2, sample.wi);
        }

        ret
    }

    /// Evaluates a BSDF based on an input and outpt directions
    pub fn eval_bsdf(
        &self,
        normal: Vector3D,
        e1: Vector3D,
        e2: Vector3D,
        mut vin: Vector3D,
        vout: Vector3D,
        eta: Float,
    ) -> Spectrum {
        // convert ray into local
        vin = self.to_local(normal, e1, e2, vin);
        let vout = self.to_local(normal, e1, e2, vout);

        match self {
            Self::Diffuse(m) => m.eval_bsdf(vin, vout, eta, TransportMode::default()),
            Self::Plastic(m) => m.eval_bsdf(vin, vout, eta, TransportMode::default()),
            Self::Metal(m) => m.eval_bsdf(vin, vout, eta, TransportMode::default()),
            Self::Light(_) => Spectrum::BLACK,
            Self::Mirror(m) => m.eval_bsdf(vin, vout, TransportMode::default()),
            Self::Dielectric(m) => m.eval_bsdf(vin, vout, eta, TransportMode::default()),
            Self::Glass(m) => m.eval_bsdf(vin, vout, eta, TransportMode::default()),
        }
    }
}
