/*
MIT License
Copyright (c) 2021 Germán Molina
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

/// Information required for modelling Radiance's Plastic and Plastic
#[derive(Debug, Clone)]
pub struct Plastic {
    pub colour: Spectrum,
    pub specularity: Float,
    pub roughness: Float,
}

impl Plastic {
    pub fn id(&self) -> &str {
        "Plastic"
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

        // let bsdf = Spectrum::gray(direct) + self.colour * diffuse;
        let bsdf = self.colour * direct + self.colour * diffuse;


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

        // Spectrum::gray(direct) + self.colour * diffuse
        self.colour * direct + self.colour * diffuse

    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::colour::Spectrum;
    use geometry::Ray3D;

    // /// This test was developed by debugging RPICT in Radiance
    // #[test]
    // fn test_eval_plastic() {
    //     let plastic = Plastic {
    //         colour: Spectrum::<3>::from(0.5),
    //         specularity: 0.05,
    //         roughness: 0.1,
    //     };

    //     let origin = Point3D::new(2., 1., 1.);
    //     let direction = Vector3D::new(-0.446877862357762, 0.77495819368141017, 0.4469227832418069);
    //     let normal = Vector3D::new(1., 0., 0.);
    //     let vout = Vector3D::new(0.446877862357762, 0.77495819368141017, 0.4469227832418069);
    //     let ray = &Ray {
    //         geometry: Ray3D { origin, direction },
    //         .. Ray::default()
    //     };
    //     let e1 = Vector3D::new(0., 1., 0.);
    //     let e2 = Vector3D::new(0., 0., 1.);

    //     // alpha2	double	0.0099999999988358463
    //     // pdot	double	0.446877862357762
    //     // rdiff	double	0.95000000000291041
    //     // rspec	double	0.049999999997089616
    //     // scolor = [0.5, 0.5, 0.5] in ln 295
    //     // vrefl	FVECT
    //     // [0]	double	0.446877862357762
    //     // [1]	double	0.77495819368141017
    //     // [2]	double	0.4469227832418069
    //     // End of source.c / Direct
    //     // rcol	COLOR
    //     // [0]	COLORV	0.353318453
    //     // [1]	COLORV	0.353318453
    //     // [2]	COLORV	0.353318453

    //     plastic.eval_bsdf(normal, e1, e2, ray, vout);
    // }

    #[test]
    fn test_specular_plastic() {
        let plastic = Plastic {
            colour: Spectrum([0.2, 0.2, 0.2]),
            specularity: 0.1,
            roughness: 0.1,
        };

        let normal = Vector3D::new(0., 0., 1.);
        let e1 = Vector3D::new(1., 0., 0.);
        let e2 = Vector3D::new(0., 1., 0.);
        let intersection_pt = Point3D::new(0., 0., 0.);

        let mut ray = Ray {
            geometry: Ray3D {
                origin: Point3D::new(-1., 0., 1.),
                direction: Vector3D::new(1., 0., -1.).get_normalized(),
            },
            ..Ray::default()
        };

        let mut rng = crate::rand::get_rng();

        plastic.sample_bsdf(normal, e1, e2, intersection_pt, &mut ray, &mut rng);
    }
}
