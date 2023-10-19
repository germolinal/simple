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
use crate::image::ImageBuffer;
use crate::rand::*;
use crate::Float;
use geometry::Ray3D;

use crate::backward_metropolis::mutation::{LocalExploration, Mutation, MutationSet, RestartRay};
use crate::backward_metropolis::path::Path;

use crate::camera::Camera;
use crate::ray::Ray;
use crate::ray_tracer::RayTracer;
use crate::scene::Scene;

pub struct BackwardMetropolis {
    pub mutations_per_pixel: usize,
    pub min_path_length: usize,
    pub n_shadow_samples: usize,
}

impl BackwardMetropolis {
    fn evaluate_anchor(
        &self,
        scene: &Scene,
        ray: Ray,
        weight: Float,
        rng: &mut RandGen,
    ) -> Spectrum {
        let integrator = RayTracer {
            n_shadow_samples: 5,
            max_depth: 3,
            limit_weight: 0.001,
            n_ambient_samples: 129,
            ..RayTracer::default()
        };

        let (v, _) = integrator.trace_ray(rng, scene, ray, 0, weight);
        v
    }

    pub fn render<'a>(&self, scene: &'a Scene, camera: &dyn Camera) -> ImageBuffer {
        let (width, height) = camera.film_resolution();
        let total_pixels = width * height;
        let mut rng = get_rng();

        // Initialize the direction that will start it all.
        let camera_sample = camera.gen_random_sample(&mut rng);
        let (mut primary_ray, weight) = camera.gen_ray(&camera_sample);

        // Calculate an anchor that will be used later for calibrating
        // the whole image
        // let anchor_ray = primary_ray;
        let mut anchor_f = self.evaluate_anchor(scene, primary_ray, weight, &mut rng);
        let mut anchor_i = camera.pixel_index(camera_sample.p_film);
        while anchor_f.radiance() < 1e-5 {
            let camera_sample = camera.gen_random_sample(&mut rng);
            let (primary_ray2, _weight2) = camera.gen_ray(&camera_sample);
            anchor_f = self.evaluate_anchor(scene, primary_ray, weight, &mut rng);
            anchor_i = camera.pixel_index(camera_sample.p_film);
            primary_ray = primary_ray2;
        }

        // Initialize mutations
        let mut mutation = MutationSet::default();
        mutation.push(0.1, Box::new(RestartRay {}));
        mutation.push(0.9, Box::new(LocalExploration {}));

        // Run
        let n_samples = total_pixels * self.mutations_per_pixel;

        // Initialize values... path and its value
        let mut x1: Path = Path::new_from_random_walk(&primary_ray, scene, &self, &mut rng);
        let mut fx1 = x1.eval_from_node(0, scene);
        let mut pixel1 = anchor_i;

        let mut pixels = vec![Spectrum::BLACK; total_pixels];

        let mut lp = 0.0;
        // Now loop.
        let mut n = 0;
        while n < n_samples {
            let progress = (100. * n as Float / n_samples as Float).round() as Float;
            if (lp - progress.floor()) < 0.1 && (progress - lp).abs() > 1. {
                lp = progress;
                println!("... Done {:.0}%", progress);
            }

            // Mutate and evaluate
            let x2 = mutation.mutate(&x1, scene, camera, &self, &mut rng);
            let ray2 = Ray3D {
                origin: x2.start,
                direction: x2.primary_dir.unwrap(),
            };
            let (pixel2, weight) = camera.pixel_from_ray(&ray2);

            // the ray is outside of FOV
            if weight > 1e-22 {
                n += 1;
                // All good.
                let pixel2 = camera.pixel_index(pixel2);
                let fx2 = x2.eval_from_node(0, scene);

                // Calculate probability of accepting the mutation
                let a = mutation.prob_of_accept(fx1, fx2);

                // Add both contributions
                pixels[pixel1] += fx1.normalize() * (1. - a);
                pixels[pixel2] += fx2.normalize() * a;

                // Accept mutation... maybe
                let r: Float = rng.gen();
                if r < a {
                    // update values
                    x1 = x2;
                    fx1 = fx2;
                    pixel1 = pixel2;
                }
            }
        }

        // return
        // let found_in_anchor = pixels[anchor_i];
        // let scale_constant = anchor_f / found_in_anchor;
        // let scale_constant = 1./ n_samples as Float;
        // for p in pixels.iter_mut(){
        //     *p *= scale_constant;
        // }
        ImageBuffer::from_pixels(width, height, pixels)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use geometry::{Point3D, Vector3D};

    use crate::camera::{Film, Pinhole, View};
    use std::time::Instant;

    #[ignore]
    #[test]
    fn basic_test() {
        // cargo test --features parallel --release  -- --ignored --nocapture basic_test
        let file = "./tests/scenes/exterior_0_diffuse_plastic.rad";
        // let file = "./tests/scenes/room.rad";

        let mut scene = Scene::from_radiance(file.to_string());

        scene.build_accelerator();

        // Create film
        let film = Film {
            resolution: (512, 512),
        };

        // Create view
        // let view = View {
        //     view_direction: Vector3D::new(0., 1., 0.).get_normalized(),
        //     view_point: Point3D::new(2., 1., 1.),
        //     ..View::default()
        // };
        // Create view
        let view = View {
            view_direction: Vector3D::new(0., 1., 0.),
            view_point: Point3D::new(0., -13., 0.),
            ..View::default()
        };

        // Create camera
        let camera = Pinhole::new(view, film);

        let integrator = BackwardMetropolis {
            mutations_per_pixel: 1000,
            min_path_length: 3,
            n_shadow_samples: 1,
        };

        let now = Instant::now();

        let buffer = integrator.render(&scene, &camera);
        println!("Room took {} seconds to render", now.elapsed().as_secs());
        buffer.save_hdre(std::path::Path::new(
            "./tests/scenes/images/room_METROPOLIS.hdr",
        ));
    }
}
