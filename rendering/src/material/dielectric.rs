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

use crate::material::bsdf_sample::BSDFSample;
use crate::material::specular::*;
use crate::material::TransportMode;
use crate::Float;
use crate::{
    colour::Spectrum,
    material::mat_trait::{MatFlag, MaterialTrait, TransFlag},
};
use geometry::Vector3D;
use rand::*;

use super::RandGen;

#[derive(Debug, Clone)]
pub struct Dielectric {
    pub colour: Spectrum,
    pub refraction_index: Float,
    // roughness...?
}

// /// From Radiance's Dielectric.c
// ///
// /// "special log for extinction coefficients"
// fn mylog(x: Float)->Float{
// 	if (x < 1e-40){
// 		-100.
//     }else if (x >= 1.){
// 		0.
//     }else{
//         x.ln()
//     }
// }

impl Dielectric {
    /// Gets the Reflected and Transmitted BSDF values
    pub fn refl_trans(
        &self,
        n1: Float,
        cos1: Float,
        n2: Float,
        cos2: Option<Float>,
    ) -> (Spectrum, Spectrum) {
        debug_assert!(cos1 > 0.0);
        // let mut ctrans = self.colour;
        // let mut cext = self.colour;

        // if (1. - n1).abs() < 1e-4 {
        //     // If we are getting in the dielectric
        //     ctrans.red = -mylog(ctrans.red);
        //     ctrans.green = -mylog(ctrans.green);
        //     ctrans.blue = -mylog(ctrans.blue);
        // }else{
        //     cext.red = -mylog(cext.red);
        //     cext.green = -mylog(cext.green);
        //     cext.blue = -mylog(cext.blue);
        // }

        if let Some(cos2) = cos2 {
            // There is refraction
            let refl = fresnel_reflectance(n1, cos1, n2, cos2);
            let refl_comp = refl;
            // This is one source of non-symmetrical BSDF
            // (check Eric Veach's thesis, chapter 5 )
            /* IF RADIANCE */
            let ratio = n2 / n1;
            let n_ratio2 = ratio * ratio;
            let t_comp = (1. - refl) * n_ratio2;
            /* IF IMPORTANCE */
            // let t_comp = (1. - refl) / cos1;

            // return
            (
                Spectrum::ONE * refl_comp / cos1,
                self.colour * t_comp / cos2,
            )
        } else {
            // pure reflection
            // (1. / cos1, 0.)
            (Spectrum::ONE / cos1, Spectrum::BLACK)
        }
    }
}

impl MaterialTrait for Dielectric {
    fn id(&self) -> &str {
        "Dielectric"
    }

    fn flags(&self) -> MatFlag {
        if (1.0 - self.refraction_index).abs() < 1e-9 {
            MatFlag::Transmission
        } else {
            // if roughness < 1e-8 {
            //     MatFlag::GlossyTransmissionReflection
            // }else{
            MatFlag::SpecularTransmissionReflection
            // }
        }
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

    fn sample_bsdf(
        &self,
        wo: Vector3D,
        eta: Float,
        rng: &mut RandGen,
        transport_mode: crate::material::TransportMode,
        trans_flags: super::mat_trait::TransFlag,
    ) -> Option<super::bsdf_sample::BSDFSample> {
        let normal = Vector3D::new(0., 0., 1.);
        let (n1, cos1, n2, cos2) = cos_and_n(wo, eta, normal, self.refraction_index);
        let refl = match cos2 {
            Some(v) => fresnel_reflectance(n1, cos1, n2, v),
            None => 1.0,
        };

        let trans = 1.0 - refl;
        let mut pr = refl;
        let mut pt = trans;

        let n1_over_n2 = n1 / n2;

        // if n1_over_n2 == 1.0 || true {
        // if it is smooth or perectly transmissive
        if !(trans_flags & TransFlag::Reflection) {
            pr = 0.0;
        }
        if !(trans_flags & TransFlag::Transmission) {
            pt = 0.0;
        }
        let uc: Float = rng.gen();
        let ret = if uc < pr / (pr + pt) {
            // Prefect specular reflection
            // let wi = Vector3D::new(-wo.x, -wo.y, wo.z);
            let wi = mirror_direction(wo);
            // let spectrum = Spectrum::gray(0.2);
            let spectrum = Spectrum::gray(refl / cos1);
            let pdf = pr / (pr + pt);
            BSDFSample::new(spectrum, wi, pdf, MatFlag::SpecularReflection)
        } else {
            // transmission
            let cos2 = cos2.unwrap();
            let wi = fresnel_transmission_dir(wo, normal, n1, cos1, n2, cos2);
            let mut spectrum = Spectrum::gray(trans / cos2) * self.colour;
            if let TransportMode::Radiance = transport_mode {
                spectrum *= n1_over_n2.sqrt();
            }
            let pdf = pt / (pr + pt);
            BSDFSample::new(spectrum, wi, pdf, MatFlag::SpecularTransmission)
        };
        Some(ret)
        // } else {
        //     unreachable!("No support for non-smooth dielectrics")
        // }
    }

    fn eval_bsdf(
        &self,
        _wo: Vector3D,
        _wi: Vector3D,
        _eta: Float,
        _transport_mode: crate::material::TransportMode,
    ) -> Spectrum {
        Spectrum::BLACK
    }

    fn colour(&self) -> Spectrum {
        self.colour
    }
}

// impl Dielectric {

//     pub fn get_possible_paths(
//         &self,
//         normal: &Vector3D,
//         intersection_pt: &Point3D,
//         ray: &Ray,
//     ) -> [Option<(Ray, Spectrum)>; 2] {
//         let normal = *normal;
//         let intersection_pt = *intersection_pt;

//         let (n1, cos1, n2, cos2) = cos_and_n(
//             ray.geometry.direction,
//             ray.refraction_index,
//             normal,
//             self.refraction_index,
//         );
//         let (refl, trans) = self.refl_trans(n1, cos1, n2, cos2);
//         let ray_dir = ray.geometry.direction;
//         let mirror_dir = mirror_direction(ray_dir);
//         debug_assert!(
//             (1. - mirror_dir.length()).abs() < 1e-5,
//             "length is {}",
//             mirror_dir.length()
//         );

//         // Process reflection...
//         let mut ray1 = *ray;
//         ray1.geometry.direction = mirror_dir;
//         ray1.geometry.origin = intersection_pt + normal * 0.00001;
//         let pair1 = Some((ray1, refl * cos1));

//         let mut ray = *ray;
//         // process transmission
//         // let pair2 = if trans.radiance() > 0.0 && ray_dir * normal < 0.0 {
//         let pair2 = match cos2 {
//             Some(cos2) => {
//                 ray.geometry.origin = intersection_pt - normal * 0.00001;
//                 ray.refraction_index = n2;
//                 let trans_dir = fresnel_transmission_dir(ray_dir, normal, n1, cos1, n2, cos2);
//                 ray.geometry.direction = trans_dir;
//                 ray.colour *= self.colour;
//                 Some((ray, trans * cos2))
//             }
//             None => None,
//         };

//         [pair1, pair2]
//     }

//     // pub fn sample_bsdf(
//     //     &self,
//     //     normal: Vector3D,
//     //     e1: Vector3D,
//     //     e2: Vector3D,
//     //     intersection_pt: Point3D,
//     //     ray: &mut Ray,
//     //     rng: &mut RandGen,
//     // ) -> (Spectrum, Float) {

//     //     debug_assert!(
//     //         (ray.geometry.direction.length() - 1.).abs() < 1e-5,
//     //         "Length was {}",
//     //         ray.geometry.direction.length()
//     //     );
//     //     debug_assert!((e1 * e2).abs() < 1e-5, "e1*e2= {} ", (e1 * e2).abs());
//     //     debug_assert!(
//     //         (e1 * normal).abs() < 1e-5,
//     //         "e1*normal = {}",
//     //         e1 * normal.abs()
//     //     );

//     //     debug_assert!(
//     //         (e2 * normal).abs() < 1e-5,
//     //         "e2*normal = {}",
//     //         (e2 * normal).abs()
//     //     );

//     //     let (n1, cos1, n2, cos2) = cos_and_n(ray, normal, self.refraction_index);
//     //     let (refl, trans) = self.refl_trans(n1, cos1, n2, cos2);
//     //     let mirror_dir = mirror_direction(ray.geometry.direction, normal);
//     //     debug_assert!(
//     //         (1. - mirror_dir.length()).abs() < 1e-5,
//     //         "length is {}",
//     //         mirror_dir.length()
//     //     );

//     //     let r: Float = rng.gen();
//     //     if r <= refl / (refl + trans) {
//     //         // Reflection
//     //         // avoid self shading
//     //         ray.geometry.origin = intersection_pt + normal * 0.00001;

//     //         ray.geometry.direction = mirror_dir;
//     //         (self.colour * refl, refl / (refl + trans))
//     //     } else {
//     //         // Transmission
//     //         // avoid self shading
//     //         ray.geometry.origin = intersection_pt - normal * 0.00001;

//     //         ray.refraction_index = n2;
//     //         let trans_dir = fresnel_transmission_dir(
//     //             ray.geometry.direction,
//     //             normal,
//     //             n1,
//     //             cos1,
//     //             n2,
//     //             cos2.unwrap(),
//     //         );
//     //         ray.geometry.direction = trans_dir;
//     //         (self.colour * trans, trans / (refl + trans))
//     //     }
//     // }

//     pub fn eval_bsdf(
//         &self,
//         wo: Vector3D,
//         wi: Vector3D,
//         eta: Float,
//         transport_mode: TransportMode,
//     ) -> Spectrum {
//         // let normal = Vector3D::new(0., 0., 1.);
//         // let (n1, cos1, n2, cos2) = cos_and_n(wo, eta, normal, self.refraction_index);
//         // let (refl, trans) = self.refl_trans(n1, cos1, n2, cos2);
//         // let mirror_dir = mirror_direction(wi);
//         // debug_assert!(
//         //     (1. - mirror_dir.length()).abs() < 1e-5,
//         //     "length is {}",
//         //     mirror_dir.length()
//         // );

//         // // If reflection
//         // if same_heisphere(wi, wo) {
//         //     return self.colour * refl;
//         // }

//         // // We are transmitting.
//         // if let Some(cos2) = cos2 {
//         //     return self.colour * trans;
//         // }

//         // // there is no transmission
//         Spectrum::BLACK // specular materials evaluate to Zero
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::rand::*;

//     use crate::ray::Ray;
//     use geometry::{Point3D, Ray3D};
//     #[test]
//     fn test_normal_incidence() {
//         // Example found online, a glass_air transition
//         let n1 = 1.52; // glass
//         let n2 = 1.; // air

//         let normal = Vector3D::new(0., 0., 1.);

//         let mat = Dielectric {
//             colour: Spectrum::gray(0.1), //irrelevant for this test
//             refraction_index: n2,
//         };

//         // Perpendicular rays aren't deviated
//         let ray = Ray {
//             refraction_index: n1,
//             geometry: Ray3D {
//                 origin: Point3D::new(0., 0., 10.),
//                 direction: Vector3D::new(0., 0., -1.),
//             },
//             ..Ray::default()
//         };

//         let (np1, cos1, np2, cos2) = cos_and_n(
//             ray.geometry.direction,
//             ray.refraction_index,
//             normal,
//             mat.refraction_index,
//         );
//         assert!((n1 - np1).abs() < 1e-8, "np1 = {}, n1 = {}", np1, n1);
//         assert!((n2 - np2).abs() < 1e-8, "np2 = {}, n2 = {}", np2, n2);
//         assert!((1. - cos1).abs() < 1e-8, "cos1 = {}", cos1);
//         assert!(cos2.is_some());
//         let cos2 = cos2.unwrap();
//         assert!((1. - cos2).abs() < 1e-8, "cos2 = {}", cos2);
//     }

//     #[test]
//     fn test_critical_angle() {
//         // Example found online, a glass_air transition
//         let n1 = 1.52 as Float; // glass
//         let n2 = 1.003 as Float; // air

//         let normal = Vector3D::new(0., 0., 1.);

//         let mat = Dielectric {
//             colour: Spectrum::gray(0.1), //irrelevant for this test
//             refraction_index: n2,
//         };

//         let crit = (n2 / n1).asin();

//         let direction = |angle: Float| -> Vector3D {
//             let direction = Vector3D::new(0., angle.sin(), -angle.cos());
//             let found_angle = (direction * normal).abs().acos();
//             assert!(
//                 (found_angle - angle).abs() < 1e-4,
//                 "angle = {} | found_angle = {}",
//                 angle,
//                 found_angle
//             );
//             direction
//         };

//         // Check before critical angle
//         let mut angle = 0.;
//         let angle_d = 0.1;
//         while angle < crit {
//             let ray = Ray {
//                 refraction_index: n1,
//                 geometry: Ray3D {
//                     origin: Point3D::new(0., 0., 10.),
//                     direction: direction(angle.to_radians()),
//                 },
//                 ..Ray::default()
//             };

//             let (_np1, _cos1, _np2, cos2) = cos_and_n(
//                 ray.geometry.direction,
//                 ray.refraction_index,
//                 normal,
//                 mat.refraction_index,
//             );
//             assert!(cos2.is_some());
//             angle += angle_d;
//         }

//         // Check critical angle
//         angle = crit;
//         let ray = Ray {
//             refraction_index: n1,
//             geometry: Ray3D {
//                 origin: Point3D::new(0., 0., 10.),
//                 direction: direction(angle.to_radians()),
//             },
//             ..Ray::default()
//         };

//         let (_np1, _cos1, _np2, cos2) = cos_and_n(
//             ray.geometry.direction,
//             ray.refraction_index,
//             normal,
//             mat.refraction_index,
//         );
//         assert!(cos2.is_some());
//         angle += angle_d;

//         // Check beyond critical angle
//         while angle < crit {
//             let ray = Ray {
//                 refraction_index: n1,
//                 geometry: Ray3D {
//                     origin: Point3D::new(0., 0., 10.),
//                     direction: direction(angle.to_radians()),
//                 },
//                 ..Ray::default()
//             };

//             let (_np1, _cos1, _np2, cos2) = cos_and_n(
//                 ray.geometry.direction,
//                 ray.refraction_index,
//                 normal,
//                 mat.refraction_index,
//             );
//             assert!(cos2.is_some());
//             angle += angle_d;
//         }
//     }

//     #[test]
//     fn test_sin_cos_n() {
//         let n = 1.52;
//         let mat = Dielectric {
//             colour: Spectrum::gray(0.23), //irrelevant for this test
//             refraction_index: n,
//         };

//         let normal = Vector3D::new(0., 0., 1.);

//         let dir_zero = Vector3D::new(0., 1., -2.).get_normalized(); // going down

//         let ray = Ray {
//             geometry: Ray3D {
//                 origin: Point3D::new(0., 0., 0.),
//                 direction: dir_zero,
//             },
//             refraction_index: 2.9,
//             ..Ray::default()
//         };

//         let (n1, cos1, n2, cos2) = cos_and_n(
//             ray.geometry.direction,
//             ray.refraction_index,
//             normal,
//             mat.refraction_index,
//         );
//         let theta1 = cos1.acos();
//         let theta2 = cos2.unwrap().acos();
//         let fresnel_1 = n1 * theta1.sin();
//         let fresnel_2 = n2 * theta2.sin();

//         assert!(
//             (fresnel_1 - fresnel_2).abs() < 1e-5,
//             "n1*sin1 = {} | n2*sin2 = {}",
//             fresnel_1,
//             fresnel_2
//         );
//     }

//     // #[test]
//     // fn test_bsdf() {
//     //     let n = 1.52;
//     //     let mat = Dielectric {
//     //         colour: Spectrum::gray(0.23), //irrelevant for this test
//     //         refraction_index: n,
//     //     };

//     //     let mut rng = get_rng();
//     //     let normal = Vector3D::new(0., 0., 1.);
//     //     let e1 = Vector3D::new(1., 0., 0.);
//     //     let e2 = Vector3D::new(0., 1., 0.);

//     //     let dir_zero = Vector3D::new(0., 1., -2.).get_normalized(); // going down

//     //     let mut ray = Ray {
//     //         geometry: Ray3D {
//     //             origin: Point3D::new(0., 0., 0.),
//     //             direction: dir_zero,
//     //         },
//     //         refraction_index: 1.,
//     //         ..Ray::default()
//     //     };
//     //     println!("Before entering: {}", dir_zero);
//     //     let mirror_dir = mirror_direction(ray.geometry.direction, normal);
//     //     let mut trans_dir: Option<Vector3D> = None;
//     //     // Get INTO the material
//     //     for _ in 0..30 {
//     //         let mut new_ray = ray;
//     //         let (_bsdf, _pdf) = mat.sample_bsdf(
//     //             normal,
//     //             e1,
//     //             e2,
//     //             Point3D::new(0., 0., 0.),
//     //             &mut new_ray,
//     //             &mut rng,
//     //         );
//     //         println!("A -- PDF = {}", _pdf);

//     //         let new_dir = new_ray.geometry.direction;
//     //         if new_dir.z < 0. {
//     //             // We are still moving down... thus, refraction
//     //             assert!(
//     //                 new_ray.refraction_index == n,
//     //                 "Expeting n={}, found n={}",
//     //                 n,
//     //                 new_ray.refraction_index
//     //             );
//     //             trans_dir = Some(new_dir);
//     //         } else {
//     //             // reflection
//     //             assert!( (1. - new_dir * mirror_dir).abs() < 1e-5 );
//     //             assert!(
//     //                 new_ray.refraction_index == 1.0,
//     //                 "Expeting n={}, found n={}",
//     //                 1.,
//     //                 new_ray.refraction_index
//     //             );
//     //         }
//     //     }

//     //     println!("Inside: {:?}", trans_dir);

//     //     // Get OUT OF the material
//     //     ray.refraction_index = n;
//     //     ray.geometry.direction = trans_dir.unwrap();
//     //     for _ in 0..30 {
//     //         let mut new_ray = ray;
//     //         let (_bsdf, _pdf) = mat.sample_bsdf(
//     //             normal,
//     //             e1,
//     //             e2,
//     //             Point3D::new(0., 0., 0.),
//     //             &mut new_ray,
//     //             &mut rng,
//     //         );
//     //         println!("B -- PDF = {}", _pdf);
//     //         let new_dir = new_ray.geometry.direction;
//     //         if new_dir.z < 0. {
//     //             // We are still moving down... thus, refraction
//     //             assert!(
//     //                 new_ray.refraction_index == 1.,
//     //                 "Expeting n={}, found n={}",
//     //                 1,
//     //                 new_ray.refraction_index
//     //             );
//     //             assert!(
//     //                 (1. - new_dir * dir_zero).abs() < 1e-5,
//     //                 "ray_dir = {} | new_dir = {} | dir_zero = {}",
//     //                 new_ray.geometry.direction,
//     //                 new_dir,
//     //                 dir_zero
//     //             );
//     //             println!("After leaving : {}", new_dir);
//     //         }
//     //     }
//     // }

//     #[test]
//     fn test_get_possible_paths_dielectric() {
//         let dielectric = Dielectric {
//             colour: Spectrum::gray(0.23), //irrelevant for this test
//             refraction_index: 1.52,
//         };

//         let mut rng = get_rng();

//         for _ in 0..5000 {
//             let refraction_index: Float = rng.gen();
//             let (x, y, z): (Float, Float, Float) = rng.gen();
//             let direction = Vector3D::new(x, y, -z).get_normalized();

//             let normal = Vector3D::new(0., 0., 1.);
//             let intersection_pt = Point3D::new(0., 0., 0.);
//             let ray = Ray {
//                 geometry: Ray3D {
//                     origin: Point3D::new(0., 0., 2.),
//                     direction,
//                 },
//                 refraction_index,
//                 ..Ray::default()
//             };

//             let paths = dielectric.get_possible_paths(&normal, &intersection_pt, &ray);
//             // Reflection
//             if let Some((new_ray, bsdf)) = paths[0] {
//                 assert_eq!(
//                     new_ray.refraction_index, refraction_index,
//                     "Expecting the ray's refraction index to be {}... found {}",
//                     refraction_index, ray.refraction_index
//                 );
//                 assert!(
//                     bsdf.radiance().is_finite() && !bsdf.radiance().is_nan(),
//                     "impossible BSDF --> {}",
//                     bsdf
//                 );
//                 let new_dir = new_ray.geometry.direction;
//                 assert!(( (new_dir.x - direction.x).abs() < 1e-5 && (new_dir.y - direction.y).abs() < 1e-5 && (new_dir.z  + direction.z).abs() < 1e-5 ), "Expecting reflected direction to be mirrored against direction (ray.dir = {} | exp = {}).", ray.geometry.direction, direction);
//             } else {
//                 panic!("Expecting a reflection path")
//             }

//             // Transmission
//             if let Some((new_ray, bsdf)) = paths[1] {
//                 assert_eq!(
//                     new_ray.refraction_index, dielectric.refraction_index,
//                     "Expecting the ray's refraction index to be {}... found {}",
//                     refraction_index, ray.refraction_index
//                 );
//                 assert!(
//                     bsdf.radiance().is_finite() && !bsdf.radiance().is_nan(),
//                     "impossible BSDF --> {}",
//                     bsdf
//                 );
//                 // assert!(new_ray.geometry.direction.compare(direction), "Expecting transmitted direction to be the same as the original direction (ray.dir = {} | exp = {})... length of diff = {}", ray.geometry.direction, direction, (new_ray.geometry.direction - direction).length());
//                 assert!(
//                     new_ray.geometry.direction.z <= 0.0,
//                     "Expecting transmitted direction to be going down... found {}",
//                     new_ray.geometry.direction
//                 );
//             }
//         }
//     }
// }
