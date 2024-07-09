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

use crate::samplers::sample_cosine_weighted_horizontal_hemisphere;
use crate::scene::Scene;
use crate::Float;
use geometry::intersection::SurfaceSide;
use geometry::Vector3D;
use geometry::{Point3D, Ray3D};
use utils::ProgressBar;
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
}

impl Default for DCFactory {
    fn default() -> Self {
        Self {
            reinhart: ReinhartSky::new(1),
            max_depth: 190, // russian roulette takes care of this
            n_ambient_samples: 300,
        }
    }
}

impl DCFactory {
    pub fn calc_dc(
        &self,
        rays: &[Ray3D],
        scene: &Scene,
        progress_bar: Option<&ProgressBar>,
    ) -> ColourMatrix {
        // Initialize matrix
        let n_bins = self.reinhart.n_bins;

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

                        let new_ray = Ray3D {
                            direction: new_ray_dir,
                            origin,
                        };

                        let mut rng = get_rng();
                        self.trace_ray(scene, new_ray, &mut this_ret, &mut rng, &mut aux);

                        if let Some(progress) = &progress_bar {
                            progress.tic();
                        }
                        this_ret
                    })
                    .collect(); // End of iterating primary rays
                let mut ret = ColourMatrix::new(Spectrum::BLACK, 1, n_bins);
                ray_contributions.iter().for_each(|v| {
                    ret += v;
                });

                ret
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
        mut ray: Ray3D,
        contribution: &mut ColourMatrix,
        rng: &mut RandGen,
        aux: &mut [usize; N],
    ) {
        let mut beta = Spectrum::gray(crate::PI);
        let mut depth = 0;
        let mut refraction_coefficient = 1.0;

        loop {
            let intersect = scene.cast_ray(ray, aux);
            if intersect.is_none() {
                // Hit the sky.
                let bin_n = self.reinhart.dir_to_bin(ray.direction);

                let li = Spectrum::ONE;
                contribution
                    .add_to_element(0, bin_n, li * beta / self.n_ambient_samples as Float)
                    .unwrap();
                break;
            }

            let (triangle_index, mut interaction) = intersect.unwrap();
            let material = match interaction.geometry_shading.side {
                SurfaceSide::Front => {
                    &scene.materials[scene.front_material_indexes[triangle_index]]
                }
                SurfaceSide::Back => &scene.materials[scene.back_material_indexes[triangle_index]],
                SurfaceSide::NonApplicable => {
                    // Hit parallel to the surface...
                    break;
                }
            };
            #[cfg(feature = "textures")]
            interaction.interpolate_normal(scene.normals[triangle_index]);

            // reached limit.
            depth += 1;
            if depth > self.max_depth {
                break;
            }

            // Sample BSDF, and continue
            if let Some(sample) = material.sample_bsdf(
                ray.direction,
                &mut interaction,
                &mut refraction_coefficient,
                rng,
            ) {
                let cos_theta = (interaction.geometry_shading.normal * sample.wi).abs();
                beta *= sample.spectrum * cos_theta / sample.pdf;
                ray = Ray3D {
                    direction: sample.wi,
                    origin: interaction.point,
                };
            } else {
                break;
            };

            // Russian roulette
            let q = (1. - beta.radiance()).max(0.);
            let aux: Float = rng.gen();
            if aux < q {
                break;
            }
            beta /= 1. - q;
        }
    }
}

#[cfg(test)]
mod tests {}
