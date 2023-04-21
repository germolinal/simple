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

use crate::camera::Camera;
use crate::colour::Spectrum;
use crate::rand::*;
use crate::ray::Ray;
use crate::scene::Scene;
use crate::Float;
use geometry::Ray3D;

use crate::backward_metropolis::path::{Path, PathNode};
use crate::backward_metropolis::BackwardMetropolis;

pub trait Mutation<'a> {
    fn mutate(
        &self,
        _x: &Path<'a>,
        scene: &'a Scene,
        camera: &dyn Camera,
        integrator: &BackwardMetropolis,
        rng: &mut SmallRng,
    ) -> Path<'a>;
}

/// A set of mutations and their respective probabilities.
#[derive(Default)]
pub struct MutationSet<'a> {
    /// A set of mutations and their respective probabilities
    mutations: Vec<(Float, Box<dyn Mutation<'a>>)>,

    // The total probability accumulated so far
    total_prob: Float,
}

impl<'a> MutationSet<'a> {
    /// Adds a mutation and its probability.
    ///
    /// Note that the probabilities are relative to each other, so they can add up to
    /// more than 1.
    pub fn push(&mut self, probability: Float, mutation: Box<dyn Mutation<'a>>) {
        self.total_prob += probability;
        // We store the accumulated probability
        self.mutations.push((self.total_prob, mutation));
    }

    /// Calculates the probability of accepting a mutation
    pub fn prob_of_accept(&self, fx1: Spectrum, fx2: Spectrum) -> Float {
        if fx1.is_black() {
            // Nothing can be worse... mutate
            return 1.;
        }
        let fx1 = fx1.radiance();
        let fx2 = fx2.radiance();
        // return
        (fx2 / fx1).min(1.)
    }
}

impl<'a> Mutation<'a> for MutationSet<'a> {
    fn mutate(
        &self,
        x: &Path<'a>,
        scene: &'a Scene,
        camera: &dyn Camera,
        integrator: &BackwardMetropolis,
        rng: &mut SmallRng,
    ) -> Path<'a> {
        let p: Float = rng.gen();
        let p = p * self.total_prob;
        for (acc_prob, mutation) in &self.mutations {
            if p < *acc_prob {
                return mutation.mutate(x, scene, camera, integrator, rng);
            }
        }
        unreachable!();
    }
}

pub struct RestartRay {}
impl<'a> Mutation<'a> for RestartRay {
    fn mutate(
        &self,
        _x: &Path<'a>,
        scene: &'a Scene,
        camera: &dyn Camera,
        integrator: &BackwardMetropolis,
        rng: &mut SmallRng,
    ) -> Path<'a> {
        let camera_sample = camera.gen_random_sample(rng);
        let (primary_ray, _weight) = camera.gen_ray(&camera_sample);

        // Create a random path
        Path::new_from_random_walk(&primary_ray, scene, integrator, rng)
    }
}

/// Chooses one node to modify and mutates it randomly.
pub struct LocalExploration {}
impl<'a> Mutation<'a> for LocalExploration {
    fn mutate(
        &self,
        x: &Path<'a>,
        scene: &'a Scene,
        camera: &dyn Camera,
        integrator: &BackwardMetropolis,
        rng: &mut SmallRng,
    ) -> Path<'a> {
        let mut ret = x.clone();

        if ret.nodes.is_empty() {
            // just restart
            let m = RestartRay {};
            return m.mutate(x, scene, camera, integrator, rng);
        }

        let mut i: usize = rng.gen();
        i = i % ret.nodes.len();

        let prev_pt = if i == 0 {
            ret.start
        } else {
            ret.nodes[i - 1].point
        };

        let next_point = ret.nodes[i].point;

        let aim_to = next_point - prev_pt;

        // Calculate new direction... sampling a disk in the next node.
        let radius: Float = aim_to.length() * 0.01;
        let target =
            crate::samplers::uniform_sample_disc(rng, radius, next_point, aim_to.get_normalized());

        let ray = Ray {
            geometry: Ray3D {
                origin: prev_pt,
                direction: (target - prev_pt).get_normalized(),
            },
            refraction_index: 1.,
        };

        if let Some(new_node) = PathNode::new(scene, &ray, integrator.n_shadow_samples, rng) {
            // update
            ret.nodes[i] = new_node;

            if i == 0 {
                // primary_dir has changed
                ret.primary_dir = Some((ret.nodes[i].point - ret.start).get_normalized());
            }

            ret
        } else {
            // did not work... try again
            self.mutate(x, scene, camera, integrator, rng)
        }
    }
}

// /// Extend path
// pub struct Extend {}
// impl <'a>Mutation<'a> for Extend {
//     fn mutate(&self, x: &Path<'a>,  scene: &'a Scene, camera: &dyn Camera, integrator: &BackwardMetropolis, rng: &mut SmallRng) -> Path<'a> {

//         let mut ret= x.clone();

//         if ret.nodes.is_empty(){
//             // just restart
//             let m = RestartRay{};
//             return m.mutate( x,  scene, camera, integrator, rng)
//         }

//         let mut i : usize = rng.gen();
//         i = i % ret.nodes.len();

//         let prev_pt = if i == 0 {
//             ret.start
//         } else {
//             ret.nodes[i-1].point
//         };

//         let next_point = ret.nodes[i].point;

//         let aim_to = next_point - prev_pt;

//         // Calculate new direction... sampling a disk in the next node.
//         let radius : Float = aim_to.length() * 0.001;
//         let target = crate::samplers::uniform_sample_disc(rng, radius, next_point, aim_to.get_normalized());

//         let ray = Ray{
//             geometry: Ray3D{
//                 origin: prev_pt,
//                 direction: (target - prev_pt).get_normalized(),
//             },
//             refraction_index: 1.
//         };

//         if let Some(new_node) = PathNode::new(scene, &ray, integrator.n_shadow_samples, rng){

//             // update
//             ret.nodes[i] = new_node;
//             ret
//         }else{
//             // did not work... try again
//             self.mutate(x, scene, camera, integrator, rng)
//         }

//     }
// }
