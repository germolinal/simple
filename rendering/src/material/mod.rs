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
use crate::ray::Ray;
use crate::Float;
use crate::{colour::Spectrum, ray::TransportMode};
use bsdf_sample::BSDFSample;
use geometry::intersection::SurfaceSide;
use geometry::{Point3D, Vector3D};
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
            Self::Diffuse(m) => m.colour,
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

    pub fn get_possible_paths(
        &self,
        normal: &Vector3D,
        intersection_pt: &Point3D,
        ray: &Ray,
    ) -> [Option<(Ray, Spectrum)>; 2] {
        match self {
            Self::Mirror(m) => m.get_possible_paths(normal, intersection_pt, ray),
            Self::Dielectric(m) => m.get_possible_paths(normal, intersection_pt, ray),
            Self::Glass(m) => m.get_possible_paths(normal, intersection_pt, ray),
            _ => panic!("Trying to get possible paths in non-specular material"),
        }
    }

    fn to_local(&self, normal: Vector3D, e1: Vector3D, e2: Vector3D, v: Vector3D) -> Vector3D {
        Vector3D::new(v * e1, v * e2, v * normal)
    }
    fn to_world(&self, normal: Vector3D, e1: Vector3D, e2: Vector3D, v: Vector3D) -> Vector3D {
        e1 * v.x + e2 * v.y + normal * v.z
    }

    /// Samples the bsdf (returned by modifying the given `Ray`).
    /// Returns the value of the BSDF in that direction (as a Spectrum) and the probability
    pub fn sample_bsdf(
        &self,
        normal: Vector3D,
        e1: Vector3D,
        e2: Vector3D,
        intersection_pt: Point3D,
        ray: &mut Ray,
        rng: &mut RandGen,
    ) -> Option<BSDFSample> {
        // world to local
        ray.geometry.direction = self.to_local(normal, e1, e2, ray.geometry.direction);

        let uc: Float = rng.gen();
        let u: (Float, Float) = rng.gen();
        let transport_mode = TransportMode::default();
        let trans_flags = TransFlag::All;
        let wo = ray.geometry.direction;
        let eta = ray.refraction_index;

        let ret = match self {
            Self::Diffuse(m) => m.sample_bsdf(wo, eta, uc, u, transport_mode, trans_flags),
            Self::Plastic(m) => m.sample_bsdf(wo, eta, uc, u, transport_mode, trans_flags),
            Self::Metal(m) => m.sample_bsdf(wo, eta, uc, u, transport_mode, trans_flags),
            Self::Light(m) => panic!("Material '{}' has no BSDF", m.id()),
            Self::Mirror(_m) => panic!("Trying to sample the BSDF of a Mirror"),
            Self::Dielectric(m) => {
                let ret = m.sample_bsdf(wo, eta, uc, u, transport_mode, trans_flags);
                // if front or back?
                // dbg!("Fix the refraction index transition");
                match ray.interaction.geometry_shading.side {
                    // Going in
                    SurfaceSide::Front => ray.refraction_index = m.refraction_index,
                    // Going out... TODO?: create a stack of refraction indexes
                    SurfaceSide::Back => ray.refraction_index = 1.0,
                    _ => unreachable!(),
                }
                ret
            }
            Self::Glass(_m) => panic!("Trying to sample the BSDF of a Glass"),
        };

        if let Some(sample) = &ret {
            if sample.is_reflection() {
                ray.geometry.origin = intersection_pt + normal * 0.00001;
            } else if sample.is_transmission() {
                ray.geometry.origin = intersection_pt - normal * 0.00001;
            }
            ray.geometry.direction = sample.wi;
        }
        // local to world
        ray.geometry.direction = self.to_world(normal, e1, e2, ray.geometry.direction);

        ret
    }

    /// Evaluates a BSDF based on an input and outpt directions
    pub fn eval_bsdf(
        &self,
        normal: Vector3D,
        e1: Vector3D,
        e2: Vector3D,
        ray: &Ray,
        vout: Vector3D,
    ) -> Spectrum {
        let mut ray = ray.clone();
        // convert ray into local
        ray.geometry.direction = self.to_local(normal, e1, e2, ray.geometry.direction);
        let vout = self.to_local(normal, e1, e2, vout);
        let normal = Vector3D::new(0., 0., 1.);
        let e1 = Vector3D::new(1.0, 0., 0.);
        let e2 = Vector3D::new(0.0, 1., 0.);
        match self {
            Self::Diffuse(m) => m.eval_bsdf(ray.geometry.direction, vout, TransportMode::default()),
            Self::Plastic(m) => m.eval_bsdf(ray.geometry.direction, vout, TransportMode::default()),
            Self::Metal(m) => m.eval_bsdf(ray.geometry.direction, vout, TransportMode::default()),
            Self::Light(m) => panic!("Material '{}' has no BSDF", m.id()),
            Self::Mirror(m) => m.eval_bsdf(normal, e1, e2, &ray, vout),
            Self::Dielectric(m) => m.eval_bsdf(normal, e1, e2, &ray, vout),
            Self::Glass(m) => m.eval_bsdf(normal, e1, e2, &ray, vout),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn get_vectors(rng: &mut RandGen) -> (Vector3D, Vector3D, Vector3D, Ray, Vector3D) {
        let e1 = Vector3D::new(rng.gen(), rng.gen(), rng.gen()).get_normalized();
        let e2 = e1.get_perpendicular().unwrap();
        let normal = e1.cross(e2);
        let direction = geometry::Vector3D::new(rng.gen(), rng.gen(), rng.gen()).get_normalized();

        // We need the direction to be opposite to normal.
        if direction * normal < 0. {
            let ray = Ray {
                geometry: geometry::Ray3D {
                    direction,
                    origin: geometry::Point3D::new(rng.gen(), rng.gen(), rng.gen()),
                },
                refraction_index: rng.gen(),
                ..Ray::default()
            };
            let vout = Vector3D::new(1., 4., 12.).get_normalized();

            (normal, e1, e2, ray, vout)
        } else {
            get_vectors(rng)
        }
    }

    fn test_material(material: Material) {
        let mut rng = crate::rand::get_rng();
        for _ in 0..99999 {
            let (normal, e1, e2, mut ray, vout) = get_vectors(&mut rng);
            let old_ray = ray.clone();
            let sample =
                material.sample_bsdf(normal, e1, e2, Point3D::new(0., 0., 0.), &mut ray, &mut rng);
            let sample = sample.unwrap();

            assert!(sample.pdf.is_finite());
            assert!(sample.spectrum.radiance().is_finite());
            assert!(old_ray.geometry.direction.length().is_finite());
            let v: Vector3D = old_ray.geometry.origin.into();
            assert!(v.length().is_finite());
            let pdf = material.eval_bsdf(normal, e1, e2, &old_ray, vout);
            assert!(pdf.radiance().is_finite());
        }
    }

    #[test]
    fn test_sample_plastic() {
        let plastic = Material::Plastic(Plastic {
            colour: Spectrum([0.5, 0.2, 0.9]),
            specularity: 0.0,
            roughness: 0.0,
        });

        println!("{}", std::mem::size_of_val(&plastic));
        test_material(plastic)
    }

    #[test]
    fn test_sample_metal() {
        let metal = Material::Metal(Metal {
            colour: Spectrum([0.5, 0.2, 0.9]),
            specularity: 0.0,
            roughness: 0.0,
        });

        test_material(metal)
    }
}
