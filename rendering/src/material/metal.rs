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
use crate::rand::*;
use crate::ray::Ray;
use crate::Float;
use geometry::{Point3D, Vector3D};

/// Information required for modelling Radiance's Metal and Metal
#[derive(Debug, Clone)]
pub struct Metal {
    pub colour: Spectrum,
    pub specularity: Float,
    pub roughness: Float,
}

impl Metal {
    pub fn id(&self) -> &str {
        "Metal"
    }

    pub fn colour(&self) -> Spectrum {
        self.colour
    }

    pub fn sample_bsdf(
        &self,
        normal: Vector3D,
        e1: Vector3D,
        e2: Vector3D,
        intersection_pt: Point3D,
        ray: &mut Ray,
        rng: &mut RandGen,
    ) -> (Spectrum, Float) {
        let (direct, diffuse, weight) = crate::material::ward::sample_ward_anisotropic(
            normal,
            e1,
            e2,
            intersection_pt,
            self.specularity,
            self.roughness,
            self.roughness,
            ray,
            rng,
        );

        // Plastic differs from Metal in that the direct component is coloured
        // let bsdf = self.colour * direct + self.colour * diffuse;

        let diffuse_component = self.colour * diffuse;
        let specular_component = Spectrum::gray(direct);
        let bsdf =
            diffuse_component * (1.0 - self.specularity) + specular_component * self.specularity;

        (bsdf, weight)
    }

    pub fn eval_bsdf(
        &self,
        normal: Vector3D,
        e1: Vector3D,
        e2: Vector3D,
        ray: &Ray,
        vout: Vector3D,
    ) -> Spectrum {
        let vout = vout * -1.;
        let (direct, diffuse) = crate::material::ward::evaluate_ward_anisotropic(
            normal,
            e1,
            e2,
            self.specularity,
            self.roughness,
            self.roughness,
            ray,
            vout,
        );

        self.colour * direct + self.colour * diffuse
    }
}
