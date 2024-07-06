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

use crate::camera::{Camera, CameraSample};
use crate::colour::Spectrum;
use crate::image::ImageBuffer;
use crate::interaction::Interaction;
use crate::material::Material;
use crate::rand::*;
use crate::scene::{Object, Scene};
use crate::Float;
use geometry::intersection::SurfaceSide;
use geometry::Ray3D;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct RayTracer {
    pub max_depth: usize,
    pub n_shadow_samples: usize,
    pub n_ambient_samples: usize,
}

impl Default for RayTracer {
    fn default() -> Self {
        Self {
            max_depth: 100,
            n_shadow_samples: 1,
            n_ambient_samples: 70,
        }
    }
}

#[allow(dead_code)]
fn power_heuristic(na: usize, pdfa: Float, nb: usize, pdfb: Float) -> Float {
    let na = na as f64;
    let nb = nb as f64;
    let a = na * pdfa;
    let b = nb * pdfb;
    a * a / (a * a + b * b)
}

#[allow(dead_code)]
fn balance_heuristic(na: usize, pdfa: Float, nb: usize, pdfb: Float) -> Float {
    let na = na as f64;
    let nb = nb as f64;
    let a = na * pdfa;
    let b = nb * pdfb;
    a / (a + b)
}

impl RayTracer {
    /// Recursively traces a ray
    pub fn trace_ray<const N: usize>(
        &self,
        rng: &mut RandGen,
        scene: &Scene,
        mut ray: Ray3D,
        aux: &mut [usize; N],
    ) -> Spectrum {
        let original_ray = ray;
        let mut spectrum = Spectrum::BLACK;
        // If max_depth is Zero, then there is no point in
        // using multiple ambient samples
        let n_ambient_samples = if self.max_depth == 0 {
            1
        } else {
            self.n_ambient_samples
        };

        for _ in 0..n_ambient_samples {
            ray = original_ray;

            let mut ray_prob = 1.0;
            let mut beta = Spectrum::ONE;
            let mut depth = 0;
            let mut refraction_coefficient = 1.0;
            let mut specular_bounce = true;
            loop {
                let intersect = scene.cast_ray(ray, aux);
                if intersect.is_none() {
                    // Check distant lights
                    for light in &scene.distant_lights {
                        if let Some((light_colour, light_pdf)) =
                            intersect_light(scene, light, ray, aux)
                        {
                            if light_pdf > 1e-18 {
                                let w = if depth == 0 || specular_bounce {
                                    1.0
                                } else {
                                    power_heuristic(1, ray_prob, 1, light_pdf)
                                };
                                spectrum += beta * w * light_colour;
                            }
                        }
                    }

                    // and also the let's check the sky
                    if let Some(sky) = &scene.sky {
                        let sky_brightness = sky(ray.direction);
                        let colour = scene.sky_colour.unwrap_or_else(|| Spectrum::gray(1.0));
                        spectrum += beta * colour * sky_brightness;
                    }
                    // path is done
                    break;
                }

                let (triangle_index, mut interaction) = intersect.unwrap();
                let material = match interaction.geometry_shading.side {
                    SurfaceSide::Front => {
                        &scene.materials[scene.front_material_indexes[triangle_index]]
                    }
                    SurfaceSide::Back => {
                        &scene.materials[scene.back_material_indexes[triangle_index]]
                    }
                    SurfaceSide::NonApplicable => {
                        // Hit parallel to the surface...
                        break;
                    }
                };

                // We hit a light... lights do not reflect,
                // so break
                if material.emits_light() {
                    if specular_bounce {
                        spectrum += beta * material.colour();
                    }
                    break;
                }

                #[cfg(feature = "textures")]
                interaction.interpolate_normal(scene.normals[triangle_index]);

                // Direct lighting
                if !material.specular_only() {
                    let n_shadow_samples = if depth == 0 { self.n_shadow_samples } else { 1 };
                    let local = self.get_local_illumination(
                        scene,
                        material,
                        &interaction,
                        refraction_coefficient,
                        rng,
                        n_shadow_samples,
                        aux,
                    );
                    spectrum += beta * local;
                }

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
                    ray_prob = sample.pdf;
                    ray = Ray3D {
                        direction: sample.wi,
                        origin: interaction.point,
                    };
                    specular_bounce = sample.is_specular();
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

        spectrum / n_ambient_samples as Float
    }

    /// Calculates the luminance produced by the direct sources in the
    /// scene
    fn get_local_illumination<const N: usize>(
        &self,
        scene: &Scene,
        material: &Material,
        interaction: &Interaction,
        eta: Float,
        rng: &mut RandGen,
        n_shadow_samples: usize,
        node_aux: &mut [usize; N],
    ) -> Spectrum {
        let (mut intersection_pt, normal, e1, e2) = interaction.get_triad();
        intersection_pt += normal * 0.001; // prevent self-shading
        let mut local_illum = Spectrum::BLACK;

        let n = n_shadow_samples;
        let n_shadow_samples = n as Float;
        if let Some((light, p_light)) = scene.sample_light_uniform(rng) {
            let mut i = 0;
            while i < n {
                let wi = if n > 1 {
                    light.primitive.sample_direction(rng, intersection_pt)
                } else {
                    let (_, direction) = light.primitive.direction(intersection_pt);
                    direction
                };
                let shadow_ray = Ray3D {
                    origin: intersection_pt,
                    direction: wi,
                };

                if let Some((light_colour, light_pdf)) =
                    intersect_light(scene, light, shadow_ray, node_aux)
                {
                    i += 1;
                    if light_pdf < 1e-18 {
                        // The light is obstructed... don't add light, but count it.
                        continue;
                    }

                    let cos_theta = (normal * wi).abs();
                    let wo = interaction.wo;
                    let bsdf = material.eval_bsdf(normal, e1, e2, wo, wi, eta);
                    let fx = light_colour * cos_theta * bsdf;

                    let bsdf_pdf = material.pdf(normal, e1, e2, wi, wo, eta);

                    let weight = power_heuristic(n, light_pdf * p_light, 1, bsdf_pdf);

                    local_illum += weight * fx / (light_pdf * p_light);
                }
            } // end of iterating samples
        }

        local_illum / n_shadow_samples
    }

    // fn get_global_illumination<const N: usize>(
    //     &self,
    //     scene: &Scene,
    //     n_ambient_samples: usize,
    //     material: &Material,
    //     ray: &mut Ray,
    //     rng: &mut RandGen,
    //     aux: &mut [usize; N],
    // ) -> Spectrum {
    //     if n_ambient_samples == 0 {
    //         return Spectrum::BLACK;
    //     }

    //     let (intersection_pt, normal, e1, e2) = ray.get_triad();

    //     let mut global = Spectrum::BLACK;

    //     let depth = ray.depth;

    //     let mut count = 0;
    //     while count < n_ambient_samples {
    //         // Choose a direction.
    //         let sample = material
    //             .sample_bsdf(normal, e1, e2, intersection_pt, ray, rng)
    //             .expect("could not sample material");
    //         let new_ray_dir = ray.geometry.direction;
    //         debug_assert!(
    //             (1. - new_ray_dir.length()).abs() < 1e-2,
    //             "Length is {}",
    //             new_ray_dir.length()
    //         );
    //         debug_assert!(
    //             (1. - normal.length()).abs() < 1e-2,
    //             "normal Length is {}",
    //             normal.length()
    //         );

    //         let cos_theta = (normal * new_ray_dir).abs();
    //         let bsdf_rad = sample.spectrum.radiance();
    //         ray.depth += 1;
    //         ray.value *= bsdf_rad * cos_theta / sample.pdf;

    //         let li = self.trace_ray(rng, scene, ray, aux);

    //         count += 1;

    //         global += li * sample.spectrum * cos_theta / sample.pdf;
    //     }

    //     global / (count as Float)
    // }

    #[allow(clippy::needless_collect)]
    pub fn render(self, scene: &Scene, camera: &dyn Camera) -> ImageBuffer {
        let (width, height) = camera.film_resolution();

        let total_pixels = width * height;
        let mut pixels = vec![Spectrum::BLACK; total_pixels];

        let chunk_len = 128;
        let i: Vec<&mut [Spectrum]> = pixels.chunks_mut(chunk_len).collect();

        #[cfg(not(feature = "parallel"))]
        let i = i.into_iter();

        #[cfg(feature = "parallel")]
        let i = i.into_par_iter();

        let progress = utils::ProgressBar::new("Rendering".to_string(), total_pixels);

        let _ = &i.enumerate().for_each(|(first_p, chunk)| {
            let mut pindex = first_p * chunk_len;
            let mut aux = [0; 32];
            let mut rng = get_rng();

            for pixel in chunk {
                let y = (pindex as Float / width as Float).floor() as usize;
                let x = pindex - y * width;
                let (ray, _weight) = camera.gen_ray(&CameraSample { p_film: (x, y) });

                *pixel = self.trace_ray(&mut rng, scene, ray, &mut aux);

                progress.tic();
                pindex += 1;
            }
        });

        // println!("\nScene took {} seconds to render", now.elapsed().as_secs());
        progress.done();

        // return
        ImageBuffer::from_pixels(width, height, pixels)
    }
}

/// Sends a `shadow_ray` towards a `light`. Returns `None` if the ray misses
/// the light, returns `Some(Black, 0)` if obstructed; returns `Some(Color, pdf)`
/// if the light is hit.
pub fn intersect_light<const N: usize>(
    scene: &Scene,
    light: &Object,
    shadow_ray: Ray3D,
    node_aux: &mut [usize; N],
) -> Option<(Spectrum, Float)> {
    let light_direction = shadow_ray.direction;
    let origin = shadow_ray.origin;

    // Expect direction to be normalized
    debug_assert!((1. - light_direction.length()).abs() < 0.0001);

    let info = light.primitive.intersect(&shadow_ray)?;

    let light_distance_squared = (origin - info.p).length_squared();

    // If the light is not visible (this should not consider
    // transparent surfaces, yet.)
    if !scene.unobstructed_distance(shadow_ray, light_distance_squared, node_aux) {
        return Some((Spectrum::BLACK, 0.0));
    }

    let light_material = match info.side {
        SurfaceSide::Front => &scene.materials[light.front_material_index],
        SurfaceSide::Back => &scene.materials[light.back_material_index],
        SurfaceSide::NonApplicable => {
            // Hit parallel to the surface
            return Some((Spectrum::BLACK, 0.0));
        }
    };
    // let light_material = &scene.materials[light.front_material_index];

    let light_colour = light_material.colour();
    let light_pdf = light.primitive.solid_angle_pdf(&info, &shadow_ray);

    // let light_pdf = 1. / light.primitive.omega(origin);

    // return
    Some((light_colour, light_pdf))
}

#[cfg(test)]
mod tests {
    // use super::*;
}
