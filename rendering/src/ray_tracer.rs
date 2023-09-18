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
use std::time::Instant;

use crate::camera::{Camera, CameraSample};
use crate::colour::Spectrum;
use crate::image::ImageBuffer;
use crate::material::Material;
use crate::rand::*;
use crate::ray::Ray;
use crate::scene::{Object, Scene};
use crate::Float;
use geometry::intersection::SurfaceSide;
use geometry::Ray3D;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

pub struct RayTracerHelper {
    pub rays: Vec<Ray>,
    pub nodes: Vec<usize>,
}

impl std::default::Default for RayTracerHelper {
    fn default() -> Self {
        Self {
            rays: vec![Ray::default(); 15],
            nodes: Vec::with_capacity(64),
        }
    }
}

impl RayTracerHelper {
    pub fn with_capacity(n: usize) -> Self {
        Self {
            rays: vec![Ray::default(); n],
            nodes: Vec::with_capacity(64),
        }
    }
}

pub struct RayTracer {
    pub max_depth: usize,
    pub n_shadow_samples: usize,
    pub n_ambient_samples: usize,

    pub limit_weight: Float,
    pub count_specular_bounce: Float,
}

impl Default for RayTracer {
    fn default() -> Self {
        Self {
            max_depth: 2,
            n_shadow_samples: 10,
            n_ambient_samples: 70,

            limit_weight: 1e-3,
            count_specular_bounce: 0.3,
        }
    }
}

impl RayTracer {
    /// Recursively traces a ray
    pub fn trace_ray(
        &self,
        rng: &mut RandGen,
        scene: &Scene,
        ray: &mut Ray,
        aux: &mut RayTracerHelper,
    ) -> (Spectrum, Float) {
        if let Some(triangle_index) = scene.cast_ray(ray, &mut aux.nodes) {
            let material = match ray.interaction.geometry_shading.side {
                SurfaceSide::Front => {
                    &scene.materials[scene.front_material_indexes[triangle_index]]
                }
                SurfaceSide::Back => &scene.materials[scene.back_material_indexes[triangle_index]],
                SurfaceSide::NonApplicable => {
                    // Hit parallel to the surface...
                    // ray.colour = Spectrum::BLACK; // We won't use this, I think
                    return (Spectrum::BLACK, 0.0);
                }
            };

            let (intersection_pt, normal, ..) = ray.get_triad();

            // for now, emmiting materials don't reflect... but they
            // are visible when viewed directly from the camera
            if material.emits_light() {
                let light_pdf = crate::triangle::triangle_solid_angle_pdf(
                    scene.ax[triangle_index],
                    scene.ay[triangle_index],
                    scene.az[triangle_index],
                    scene.bx[triangle_index],
                    scene.by[triangle_index],
                    scene.bz[triangle_index],
                    scene.cx[triangle_index],
                    scene.cy[triangle_index],
                    scene.cz[triangle_index],
                    intersection_pt,
                    ray.interaction.geometry_shading.normal,
                    &ray.geometry,
                );

                return (material.colour(), light_pdf);
            }

            // Limit bounces
            if ray.depth > self.max_depth {
                return (Spectrum::BLACK, 0.0);
            }

            #[cfg(feature = "textures")]
            ray.interaction
                .interpolate_normal(scene.normals[triangle_index]);

            // let mut wt = ray.value;

            // Handle specular materials... we have 1 or 2 rays... spawn those.
            if material.specular_only() {
                let mut specular_li = Spectrum::BLACK;

                let paths = material.get_possible_paths(&normal, &intersection_pt, ray);

                for (new_ray, bsdf_value) in paths.iter().flatten() {
                    let mut new_ray = *new_ray;

                    ray.value *= bsdf_value.radiance();

                    let q: Float = rng.gen();
                    if q < self.count_specular_bounce {
                        new_ray.depth += 1;
                    }

                    let (li, _light_pdf) = self.trace_ray(rng, scene, &mut new_ray, aux);
                    specular_li += li * *bsdf_value
                }

                ray.colour *= specular_li;
                return (specular_li, 0.0);
            }

            let n_ambient_samples = ray.get_n_ambient_samples(
                self.n_ambient_samples,
                self.max_depth,
                self.limit_weight,
                rng,
            );

            // Calculate the number of direct samples

            let n_shadow_samples = if ray.depth == 0 {
                self.n_shadow_samples
            } else {
                1
            };

            /* DIRECT LIGHT */
            let local = self.get_local_illumination(
                scene,
                material,
                ray,
                rng,
                n_shadow_samples,
                &mut aux.nodes,
            );

            /* INDIRECT */
            let global =
                self.get_global_illumination(scene, n_ambient_samples, material, ray, rng, aux);

            ((local + global), 0.0)
        } else {
            // Did not hit... so, let's check the sky
            if let Some(sky) = &scene.sky {
                let sky_brightness = sky(ray.geometry.direction);
                let colour = scene.sky_colour.unwrap_or_else(|| Spectrum::gray(1.0));
                ray.colour *= colour * sky_brightness;
                (colour * sky_brightness, 1. / 2. / crate::PI)
            } else {
                (Spectrum::BLACK, 0.0)
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn sample_light_array(
        &self,
        scene: &Scene,
        material: &Material,
        ray: &Ray,
        rng: &mut RandGen,
        n_shadow_samples: usize,
        lights: &[Object],
        node_aux: &mut Vec<usize>,
    ) -> Spectrum {
        let (mut intersection_pt, normal, e1, e2) = ray.get_triad();
        intersection_pt += normal * 0.001; // prevent self-shading
        let mut local_illum = Spectrum::BLACK;

        let n = n_shadow_samples;
        // let n_ambient_samples = n_ambient_samples as Float;
        let n_shadow_samples = n_shadow_samples as Float;

        for light in lights.iter() {
            // let this_origin = this_origin + normal * 0.001;
            let mut i = 0;
            // let mut missed = 0;
            while i < n {
                let direction = light.primitive.sample_direction(rng, intersection_pt);
                // let (_,direction) = light.primitive.direction( point);
                let shadow_ray = Ray3D {
                    origin: intersection_pt,
                    direction,
                };

                if let Some((light_colour, light_pdf)) =
                    intersect_light(scene, light, &shadow_ray, node_aux)
                {
                    i += 1;
                    if light_pdf < 1e-18 {
                        // The light is obstructed... don't add light, but count it.
                        continue;
                    }

                    // Denominator of the Balance Heuristic... I am assuming that
                    // when one light has a pdf>0, then all the rest are Zero... is this
                    // correct?
                    let cos_theta = (normal * direction).abs();
                    let vout = shadow_ray.direction * -1.;

                    let mat_bsdf_value = material.eval_bsdf(normal, e1, e2, ray, vout);
                    // let denominator = light_pdf * n_shadow_samples
                    // + mat_bsdf_value.radiance() * n_ambient_samples;
                    let fx = light_colour * cos_theta * mat_bsdf_value;

                    // Return... light sources have a pdf equal to their 1/Omega (i.e. their size)
                    local_illum += fx / (n_shadow_samples * light_pdf); //denominator;
                } else {
                    #[cfg(debug_assertions)]
                    {
                        eprintln!(
                            "Missed Light... primitive '{}' (i = {})",
                            light.primitive.id(),
                            i
                        )
                    }
                }
                // ... missed light. Try again
            } // end of iterating samples
        } // end of iterating lights

        local_illum
    }

    /// Calculates the luminance produced by the direct sources in the
    /// scene
    fn get_local_illumination(
        &self,
        scene: &Scene,
        material: &Material, //&impl Material,
        ray: &Ray,
        rng: &mut RandGen,
        n_shadow_samples: usize,
        node_aux: &mut Vec<usize>,
    ) -> Spectrum {
        let close = self.sample_light_array(
            scene,
            material,
            ray,
            rng,
            n_shadow_samples,
            &scene.lights,
            node_aux,
        );
        let distant = self.sample_light_array(
            scene,
            material,
            ray,
            rng,
            n_shadow_samples,
            &scene.distant_lights,
            node_aux,
        );

        // return
        close + distant
    }

    fn get_global_illumination(
        &self,
        scene: &Scene,
        n_ambient_samples: usize,
        material: &Material,
        ray: &mut Ray,
        rng: &mut RandGen,
        aux: &mut RayTracerHelper,
    ) -> Spectrum {
        if n_ambient_samples == 0 {
            return Spectrum::BLACK;
        }

        let (intersection_pt, normal, e1, e2) = ray.get_triad();

        let mut global = Spectrum::BLACK;

        let depth = ray.depth;
        aux.rays[depth] = *ray; // store a copy.

        let mut count = 0;
        while count < n_ambient_samples {
            // Choose a direction.
            let (bsdf_value, ray_pdf) =
                material.sample_bsdf(normal, e1, e2, intersection_pt, ray, rng);
            let new_ray_dir = ray.geometry.direction;
            debug_assert!(
                (1. - new_ray_dir.length()).abs() < 1e-2,
                "Length is {}",
                new_ray_dir.length()
            );

            let cos_theta = (normal * new_ray_dir).abs();
            let bsdf_rad = bsdf_value.radiance();
            ray.depth += 1;
            ray.value *= bsdf_rad * cos_theta / ray_pdf;

            let (li, light_pdf) = self.trace_ray(rng, scene, ray, aux);

            if light_pdf > 0. {
                // ray hit a light... reset and try again
                *ray = aux.rays[depth];
                continue;
            }
            count += 1;

            let fx = li * cos_theta;

            global += fx / ray_pdf;

            // restore ray, because it was modified by trace_ray executions
            *ray = aux.rays[depth];
        }
        // return
        global / (n_ambient_samples as Float)
    }

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

        let now = Instant::now();
        // progress indicators
        let last_progress = std::sync::Arc::new(std::sync::Mutex::new(0.0));
        let counter = std::sync::Arc::new(std::sync::Mutex::new(0));

        let _ = &i.enumerate().for_each(|(first_p, chunk)| {
            let mut pindex = first_p * chunk_len;
            let mut aux = RayTracerHelper::with_capacity(self.max_depth + 1);
            let mut rng = get_rng();

            for pixel in chunk {
                let y = (pindex as Float / width as Float).floor() as usize;
                let x = pindex - y * width;
                let (mut ray, weight) = camera.gen_ray(&CameraSample { p_film: (x, y) });
                ray.value = weight;

                let (v, _) = self.trace_ray(&mut rng, scene, &mut ray, &mut aux);
                *pixel = v;

                // report
                let mut c = counter.lock().unwrap();
                *c += 1;

                let mut lp = last_progress.lock().unwrap();
                let progress = (100. * *c as Float / total_pixels as Float).round() as Float;
                if (*lp - progress.floor()) < 0.1 && (progress - *lp).abs() > 1. {
                    *lp = progress;
                    println!("... Done {:.0}%", progress);
                }

                pindex += 1;
            }
        });

        println!("Scene took {} seconds to render", now.elapsed().as_secs());

        // return
        ImageBuffer::from_pixels(width, height, pixels)
    }
}

/// Sends a `shadow_ray` towards a `light`. Returns `None` if the ray misses
/// the light, returns `Some(Black, 0)` if obstructed; returns `Some(Color, pdf)`
/// if the light is hit.
pub fn intersect_light(
    scene: &Scene,
    light: &Object,
    shadow_ray: &Ray3D,
    node_aux: &mut Vec<usize>,
) -> Option<(Spectrum, Float)> {
    let light_direction = shadow_ray.direction;
    let origin = shadow_ray.origin;

    // Expect direction to be normalized
    debug_assert!((1. - light_direction.length()).abs() < 0.0001);

    let info = light.primitive.intersect(shadow_ray)?;

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
    let light_pdf = light.primitive.solid_angle_pdf(&info, shadow_ray);

    // let light_pdf = 1. / light.primitive.omega(origin);

    // return
    Some((light_colour, light_pdf))
}

#[cfg(test)]
mod tests {
    // use super::*;
}
