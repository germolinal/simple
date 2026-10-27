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
        let n_bins = self.reinhart.n_bins;

        // Each sensor ray traces all its ambient samples, with its own random
        // number generator, adding their contributions into a single row.
        // (Rays are processed in parallel or in series, depending on the
        // compilation options.)
        let calc_row = |primary_ray: &Ray3D| -> ColourMatrix {
            let normal = primary_ray.direction;
            let origin = primary_ray.origin;
            let e2 = normal.get_perpendicular().unwrap();
            let e1 = e2.cross(normal);

            let mut rng = get_rng();
            let mut aux = [0; 32];
            let mut row = ColourMatrix::new(Spectrum::BLACK, 1, n_bins);
            for _ in 0..self.n_ambient_samples {
                let local_ray_dir = sample_cosine_weighted_horizontal_hemisphere(rng.random());
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
                // (with `f32`, 1e-7 is below rounding errors)
                debug_assert!(
                    (1. - new_ray_dir.length()).abs() < (1e-7 as Float).max(10. * Float::EPSILON),
                    "length is {}",
                    new_ray_dir.length()
                );

                let new_ray = Ray3D {
                    direction: new_ray_dir,
                    origin,
                };
                self.trace_ray(scene, new_ray, &mut row, &mut rng, &mut aux);
            }
            if let Some(progress) = &progress_bar {
                progress.tic_n(self.n_ambient_samples);
            }
            row
        };

        #[cfg(not(feature = "parallel"))]
        let rows: Vec<ColourMatrix> = rays.iter().map(calc_row).collect();
        #[cfg(feature = "parallel")]
        let rows: Vec<ColourMatrix> = rays.par_iter().map(calc_row).collect();

        // Write down the results
        let mut ret = ColourMatrix::new(Spectrum::BLACK, rays.len(), n_bins);
        for (row, dest) in rows.iter().zip(ret.as_mut_slice().chunks_exact_mut(n_bins)) {
            dest.copy_from_slice(row.as_slice());
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
            let aux: Float = rng.random();
            if aux < q {
                break;
            }
            beta /= 1. - q;
        }
    }
}

#[cfg(test)]
mod tests {}
