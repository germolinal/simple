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
use crate::colour_matrix::ColourMatrix;
use crate::rand::*;
use crate::ray::Ray;

use crate::samplers::sample_cosine_weighted_horizontal_hemisphere;
use crate::scene::Scene;
use crate::Float;
use geometry::intersection::SurfaceSide;
use geometry::Vector3D;
use geometry::{Point3D, Ray3D};
use weather::solar::ReinhartSky;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

/// A structure meant to calculate DC matrices
/// for Climate Daylight Simulations.
#[derive(Debug)]
pub struct DCFactory {
    pub reinhart: ReinhartSky,
    pub max_depth: usize,
    pub n_ambient_samples: usize,
    pub limit_weight: Float,
    pub count_specular_bounce: Float,
    // pub limit_reflections: usize,
}

impl Default for DCFactory {
    fn default() -> Self {
        Self {
            reinhart: ReinhartSky::new(1),
            max_depth: 0,
            n_ambient_samples: 10,
            count_specular_bounce: 0.5,

            limit_weight: 1e-4,
            // limit_reflections: 0,
        }
    }
}

impl DCFactory {
    pub fn calc_dc(&self, rays: &[Ray3D], scene: &Scene) -> ColourMatrix {
        // Initialize matrix
        let n_bins = self.reinhart.n_bins;

        // let counter = std::sync::Arc::new(std::sync::Mutex::new(0));
        // let last_progress = std::sync::Arc::new(std::sync::Mutex::new(0.0));

        // Process... This can be in parallel, or not.
        #[cfg(not(feature = "parallel"))]
        let aux_iter = rays.iter();
        #[cfg(feature = "parallel")]
        let aux_iter = rays.par_iter();
        // Iterate the rays
        let dcs: Vec<ColourMatrix> = aux_iter
            .map(|primary_ray| -> ColourMatrix {
                let normal = primary_ray.direction;
                let origin = primary_ray.origin;
                let e2 = normal.get_perpendicular().unwrap();
                let e1 = e2.cross(normal);

                // Run each spawned ray in parallel or series, depending on
                // the compilation options
                let mut rng = get_rng();
                #[allow(clippy::needless_collect)]
                let aux_iter: Vec<Vector3D> = (0..self.n_ambient_samples)
                    .map(|_| {
                        let u = rng.gen();
                        sample_cosine_weighted_horizontal_hemisphere(u)
                    })
                    .collect();

                #[cfg(not(feature = "parallel"))]
                let aux_iter = aux_iter.into_iter();

                #[cfg(feature = "parallel")]
                let aux_iter = aux_iter.into_par_iter();

                // Iterate primary rays
                let ray_contributions: Vec<ColourMatrix> = aux_iter
                    .map(|local_ray_dir: Vector3D| -> ColourMatrix {
                        let (x, y, z) = crate::samplers::local_to_world(
                            e1,
                            e2,
                            normal,
                            Point3D::new(0., 0., 0.),
                            local_ray_dir.x,
                            local_ray_dir.y,
                            local_ray_dir.z,
                        );
                        let new_ray_dir = Vector3D::new(x, y, z);

                        let mut this_ret = ColourMatrix::new(Spectrum::BLACK, 1, n_bins);

                        debug_assert!(
                            (1. - new_ray_dir.length()).abs() < 0.0000001,
                            "length is {}",
                            new_ray_dir.length()
                        );

                        let mut aux = [0; 32];
                        let mut new_ray = Ray {
                            // time: 0.,
                            geometry: Ray3D {
                                direction: new_ray_dir,
                                origin,
                            },
                            colour: Spectrum::gray(crate::PI),
                            ..Ray::default()
                        };

                        let mut rng = get_rng();
                        // let current_weight = cos_theta;
                        self.trace_ray(scene, &mut new_ray, &mut this_ret, &mut rng, &mut aux);

                        // let mut c = counter.lock().unwrap();
                        // *c += 1;
                        // let nrays = rays.len() * self.n_ambient_samples;
                        // let mut lp = last_progress.lock().unwrap();
                        // let progress = (100. * *c as Float / nrays as Float).round() as Float;
                        // if (*lp - progress.floor()) < 0.1 && (progress - *lp).abs() > 1. {
                        //     *lp = progress;
                        //     println!("... Done {:.0}%", progress);
                        // }

                        this_ret
                    })
                    .collect(); // End of iterating primary rays

                let mut ret = ColourMatrix::new(Spectrum::BLACK, 1, n_bins);
                ray_contributions.iter().for_each(|v| {
                    ret += v;
                });

                ret
                // ray_contributions.iter().sum();
            })
            .collect(); // End of iterating rays

        // Write down the results
        let mut ret = ColourMatrix::new(Spectrum::BLACK, rays.len(), n_bins);
        for (sensor_index, contribution) in dcs.iter().enumerate() {
            // add contribution
            for patch_index in 0..n_bins {
                let v = contribution.get(0, patch_index).unwrap();
                ret.set(sensor_index, patch_index, v).unwrap();
            }
        }

        ret
    }

    /// Recursively traces a ray until it excedes the `max_depth` of the
    /// `DCFactory` or the ray does not hit anything (i.e., it reaches either
    /// the sky or the ground)
    fn trace_ray<const N: usize>(
        &self,
        scene: &Scene,
        ray: &mut Ray,
        contribution: &mut ColourMatrix,
        rng: &mut RandGen,
        aux: &mut [usize; N],
    ) {
        // If hits an object
        if let Some((triangle_index, mut interaction)) = scene.cast_ray(ray.geometry, aux) {
            // Limit bounces
            if ray.depth > self.max_depth {
                return;
            }
            // NEARLY copied... except from the return statement
            let material = match interaction.geometry_shading.side {
                SurfaceSide::Front => {
                    &scene.materials[scene.front_material_indexes[triangle_index]]
                }
                SurfaceSide::Back => &scene.materials[scene.back_material_indexes[triangle_index]],
                SurfaceSide::NonApplicable => {
                    // Hit parallel to the surface...
                    return;
                }
            };

            // Limit bounces... also, emmiting materials don't reflect
            if ray.depth > self.max_depth || material.emits_direct_light() {
                return;
            }

            // let (intersection_pt, normal, ..) = ray.get_triad();
            #[cfg(feature = "textures")]
            ray.interaction
                .interpolate_normal(scene.normals[triangle_index]);

            let n_ambient_samples = ray.get_n_ambient_samples(
                self.n_ambient_samples,
                self.max_depth,
                self.limit_weight,
                rng,
            );

            // Spawn more rays
            // let depth = ray.depth;

            let (_pt, normal, ..) = ray.get_triad();
            (0..n_ambient_samples).for_each(|_| {
                let sample = material
                    .sample_bsdf(
                        ray.direction(),
                        &mut interaction,
                        &mut ray.refraction_index,
                        rng,
                    )
                    .unwrap();
                let new_ray_dir = ray.geometry.direction;
                debug_assert!(
                    (1. as Float - new_ray_dir.length()).abs() < 1e-5,
                    "Length is {}",
                    new_ray_dir.length()
                );

                // increase depth
                let cos_theta = (normal * new_ray_dir).abs();
                ray.colour *= sample.spectrum * cos_theta / sample.pdf;
                ray.depth += 1;

                self.trace_ray(scene, ray, contribution, rng, aux);
            }); // End the foreach spawned ray
        } else {
            let bin_n = self.reinhart.dir_to_bin(ray.geometry.direction);

            let li = Spectrum::ONE;
            let old_value = contribution.get(0, bin_n).unwrap();
            contribution
                .set(
                    0,
                    bin_n,
                    old_value + li * ray.colour / self.n_ambient_samples as Float, // accum_denom_samples as Float,
                )
                .unwrap();
        }
    }
}

#[cfg(test)]
mod tests {}
