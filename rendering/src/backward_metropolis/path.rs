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

use crate::backward_metropolis::BackwardMetropolis;
use crate::colour::Spectrum;
use crate::interaction::Interaction;
use crate::material::Material;
use crate::rand::*;
use crate::ray::Ray;
use crate::ray_tracer::sample_light;
use crate::scene::Object;
use crate::scene::Scene;
use crate::Float;
use geometry::intersection::SurfaceSide;
use geometry::{Point3D, Ray3D, Vector3D};

#[derive(Clone)]
pub struct Path<'a> {
    pub nodes: Vec<PathNode<'a>>,
    pub start: Point3D,
    pub primary_dir: Option<Vector3D>,
}

impl<'a> Path<'a> {
    pub fn new(start: Point3D) -> Self {
        Self {
            nodes: Vec::new(),
            start,
            primary_dir: None,
        }
    }

    pub fn with_capacity(start: Point3D, capacity: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(capacity),
            start,
            primary_dir: None,
        }
    }

    pub fn new_from_random_walk(
        primary_ray: &Ray,
        scene: &'a Scene,
        integrator: &BackwardMetropolis,
        rng: &mut RandGen,
    ) -> Self {
        let mut ret =
            Self::with_capacity(primary_ray.geometry.origin, integrator.min_path_length * 2); // leave some room for future expansions
        let mut ray = *primary_ray;
        ret.primary_dir = Some(ray.geometry.direction);
        for _ in 0..integrator.min_path_length {
            if ret.extend(scene, &ray, integrator.n_shadow_samples, rng) {
                // Create new ray.
                let node = ret.nodes.last().ok_or("No last node?")?;
                let material = node.material;
                let normal = node.normal;
                let e1 = node.e1;
                let e2 = node.e2;
                let point = node.point;

                let (new_ray, _, _) = material.sample_bsdf(normal, e1, e2, point, ray, rng);
                ray = new_ray;
            } else {
                // Did not hit anything... no point in keeping going.
                return ret;
            }
        }

        // return
        ret
    }

    fn push(&mut self, node: PathNode<'a>) {
        if self.nodes.is_empty() {
            // Fill primary_dir
            let dir = (node.point - self.start).get_normalized();
            self.primary_dir = Some(dir);
        }
        self.nodes.push(node)
    }

    /// Adds one level to the Path by sending a `ray` through the `scene`.
    ///
    /// If the object that the ray hits has a fully specular material—e.g., Dielectric—
    /// then another ray will be sent to reflect the quasi-deterministic nature of
    /// specular reflections.
    #[must_use]
    pub fn extend(
        &mut self,
        scene: &'a Scene,
        ray: &Ray,
        n_shadow_samples: usize,
        rng: &mut RandGen,
    ) -> bool {
        if let Some(node) = PathNode::new(scene, ray, n_shadow_samples, rng) {
            self.push(node);
            true
        } else {
            false
        }
    }

    /// Walks from start to end, adding the contribution of the different
    /// nodes.
    pub fn eval_from_node(&self, i: usize, scene: &Scene) -> Spectrum {
        if self.nodes.is_empty() {
            return Spectrum::BLACK;
        }
        assert!(
            i < self.nodes.len(),
            "Trying to evaluate path of {} starting from node {}",
            self.nodes.len(),
            i
        );

        let prev_pt = if i == 0 {
            self.start
        } else {
            self.nodes[i - 1].point
        };

        // Add local
        let node = &self.nodes[i];

        let vin = (node.point - prev_pt).get_normalized();
        let ret = node.eval(vin);

        // Add next node
        if let Some(next_node) = self.nodes.get(i + 1) {
            let vout = next_node.point - node.point;
            let distance_squared = vout.length_squared();
            let vout = vout.get_normalized();
            // Ray frim this node to the next one
            let shadow_ray = Ray3D {
                origin: node.point,
                direction: vout,
            };

            // if next node is obstructed... then don't bother.
            if !scene.unobstructed_distance(&shadow_ray, distance_squared) {
                return ret;
            }

            // Ray from prev_point to this node
            let ray = Ray {
                geometry: Ray3D {
                    origin: prev_pt,
                    direction: vin,
                },
                refraction_index: 1.,
            };

            let bsdf = node
                .material
                .eval_bsdf(node.normal, node.e1, node.e2, &ray, vout);
            let cos_theta = (node.normal * vout).abs();

            // return this + the next node.
            ret + node.material.colour() * bsdf * cos_theta * self.eval_from_node(i + 1, scene)
        } else {
            // return
            ret
        }
    }
}

#[derive(Clone)]
pub struct PathNode<'a> {
    pub normal: Vector3D,
    pub e1: Vector3D,
    pub e2: Vector3D,
    pub point: Point3D,

    /// A vector containing the radiance and the direction of direct lighting
    /// reaching a point
    local_illuminance: Vec<(Vector3D, Spectrum)>,

    pub material: &'a Material,
}

impl<'a> PathNode<'a> {
    pub fn new(
        scene: &'a Scene,
        ray: &Ray,
        n_shadow_samples: usize,
        rng: &mut RandGen,
    ) -> Option<Self> {
        if let Some(Interaction::Surface(data)) = scene.cast_ray(&ray) {
            let object = &scene.objects[data.prim_index];
            let material = match data.geometry_shading.side {
                SurfaceSide::Front => &scene.materials[object.front_material_index],
                SurfaceSide::Back => &scene.materials[object.back_material_index],
                SurfaceSide::NonApplicable => {
                    // Hit parallel to the surface...?
                    unreachable!();
                }
            };

            if material.emits_light() {
                return None;
                // todo!()
            }

            if material.specular_only() {
                todo!()
            }

            // let point = data.point;

            // GET LOCAL ILLUMINATION
            let normal = data.geometry_shading.normal;
            let e1 = data.geometry_shading.dpdu.get_normalized();
            let e2 = normal.cross(e1); //.get_normalized();
            let point = data.point;

            let n_lights = scene.count_all_lights();
            let mut local_illuminance: Vec<(Vector3D, Spectrum)> = Vec::with_capacity(n_lights);
            for light in &scene.lights {
                let (dir, colour) =
                    get_local_illumination(point, normal, n_shadow_samples, light, rng, scene);
                if !colour.is_black() {
                    local_illuminance.push((dir, colour))
                }
            }
            for light in &scene.distant_lights {
                let (dir, colour) =
                    get_local_illumination(point, normal, n_shadow_samples, light, rng, scene);
                if !colour.is_black() {
                    local_illuminance.push((dir, colour))
                }
            }

            // Build and push
            Some(PathNode {
                local_illuminance,
                normal,
                e1,
                e2,
                point,
                material,
            })
        } else {
            None
        }
    }

    /// Evaluates the local illumination of a node, as seen
    /// from a certain point.
    ///
    /// `vout` goes from the intersection point to the point of view
    #[must_use]
    pub fn eval(&self, vin: Vector3D) -> Spectrum {
        debug_assert!(
            (1. - vin.length()).abs() < 1e-5,
            "length is {}",
            vin.length()
        );

        // These variables relate to
        let normal = self.normal;

        let ray = Ray {
            geometry: Ray3D {
                origin: self.point,
                direction: vin,
            },
            refraction_index: 1.,
        };
        let mat_colour = self.material.colour();

        let mut ret = Spectrum::BLACK;

        // Denominator of the Balance Heuristic... I am assuming that

        for (direction, radiance) in &self.local_illuminance {
            let direction = *direction;
            let radiance = *radiance;

            let cos_theta = (normal * direction).abs();
            // let vout = shadow_ray.direction * -1.;

            let mat_bsdf_value =
                self.material
                    .eval_bsdf(normal, self.e1, self.e2, ray, direction * -1.);
            ret += (radiance * cos_theta) * (mat_colour * mat_bsdf_value);
        }

        ret
    }
}

fn get_local_illumination(
    mut point: Point3D,
    normal: Vector3D,
    n_shadow_samples: usize,
    light: &Object,
    rng: &mut RandGen,
    scene: &Scene,
) -> (Vector3D, Spectrum) {
    let mut ret_light = Spectrum::BLACK;
    let mut average_direction = Vector3D::new(0., 0., 0.);

    // prevent self-shading... this assumes we are reflecting
    point += normal * 0.001;

    let mut i = 0;
    while i < n_shadow_samples {
        let direction = light.primitive.sample_direction(rng, point);
        let shadow_ray = Ray3D {
            origin: point,
            direction,
        };
        if let Some((light_colour, light_pdf)) = sample_light(scene, light, &shadow_ray) {
            i += 1;
            if light_pdf < 1e-18 {
                // The light is obstructed... don't add light, but count it.
                continue;
            }

            average_direction += direction;
            ret_light += light_colour / (light_pdf * n_shadow_samples as Float);
        } else {
            // missed += 1;
            // eprintln!("Missed Light! {} (i = {})", missed, i)
        }
    }

    // return
    (average_direction, ret_light)
}

// primitive.intersect(ray) --> IntersectionInfo

#[cfg(test)]
mod tests {
    use super::*;

    use crate::material::PlasticMetal;
    use crate::primitive::Primitive;
    use crate::PI;
    use geometry::{DistantSource3D, Triangle3D};

    #[test]
    fn test_path_get_local_illumination() {
        let mut scene = Scene::new();
        let pt = Point3D::new(0., 0., 0.);

        let brightness = 100.;

        let bright_mat = scene.push_material(Material::Light(Spectrum {
            red: brightness,
            green: brightness,
            blue: brightness,
        }));

        let source = Primitive::Source(DistantSource3D::new(
            Vector3D::new(0., 0., 1.),   // direction
            (0.5 as Float).to_radians(), // angle
        ));
        let omega = source.omega(pt);
        scene.push_object(bright_mat, bright_mat, source);

        scene.build_accelerator();

        let normal = Vector3D::new(0., 0., 1.);
        let mut rng = get_rng();
        let (found_dir, found) =
            get_local_illumination(pt, normal, 1, &scene.distant_lights[0], &mut rng, &scene);

        let exp = brightness * omega;

        assert!((exp - found.radiance()).abs() < 1e-5);
        assert!((1. - normal * found_dir).abs() < 1e-5);

        println!(
            "found_dir = {} | found = {} | exp = {}",
            found_dir, found, exp
        );
    }

    #[test]
    fn test_path_build_and_eval_1node() -> Result<(),String> {
        // setup params
        let start = Point3D::new(2., 0., 0.);
        let exp_pt0 = Point3D::new(1., 0., 1.);
        let exp_pt1 = Point3D::new(0., 0., 0.);
        let dir_1 = Vector3D::new(-1., 0., 1.).get_normalized();
        let dir_2 = Vector3D::new(-1., 0., -1.).get_normalized();
        let floor_reflectance = 0.3;
        let ceiling_reflectance = 0.73;
        let light_brightness = 100.;

        let mut scene = Scene::new();
        let mut rng = get_rng();

        // ADD FLOOR
        let floor_mat = scene.push_material(Material::Plastic(PlasticMetal {
            color: Spectrum {
                red: floor_reflectance,
                green: floor_reflectance,
                blue: floor_reflectance,
            },
            specularity: 0.,
            roughness: 0.,
        }));
        scene.push_object(
            floor_mat,
            floor_mat,
            Primitive::Triangle(
                Triangle3D::new(
                    Point3D::new(-0.5, -0.5, 0.),
                    Point3D::new(0.5, -0.5, 0.),
                    Point3D::new(0.5, 0.5, 0.),
                )?,
            ),
        );
        scene.push_object(
            floor_mat,
            floor_mat,
            Primitive::Triangle(
                Triangle3D::new(
                    Point3D::new(0.5, 0.5, 0.),
                    Point3D::new(-0.5, 0.5, 0.),
                    Point3D::new(-0.5, -0.5, 0.),
                )?,
            ),
        );

        // ADD CEILING
        let ceiling_mat = scene.push_material(Material::Plastic(PlasticMetal {
            color: Spectrum {
                red: ceiling_reflectance,
                green: ceiling_reflectance,
                blue: ceiling_reflectance,
            },
            specularity: 0.,
            roughness: 0.,
        }));
        scene.push_object(
            ceiling_mat,
            ceiling_mat,
            Primitive::Triangle(
                Triangle3D::new(
                    Point3D::new(0.5, -0.5, 1.),
                    Point3D::new(1.5, -0.5, 1.),
                    Point3D::new(1.5, 0.5, 1.),
                )?,
            ),
        );
        scene.push_object(
            ceiling_mat,
            ceiling_mat,
            Primitive::Triangle(
                Triangle3D::new(
                    Point3D::new(1.5, 0.5, 1.),
                    Point3D::new(0.5, 0.5, 1.),
                    Point3D::new(0.5, -0.5, 1.),
                )?,
            ),
        );

        // Add light source
        let bright_mat = scene.push_material(Material::Light(Spectrum {
            red: light_brightness,
            green: light_brightness,
            blue: light_brightness,
        }));

        let source = DistantSource3D::new(
            Vector3D::new(-1., 0., 1.),  // direction
            (0.5 as Float).to_radians(), // angle
        );
        let omega = source.omega;
        let source = Primitive::Source(source);
        scene.push_object(bright_mat, bright_mat, source);

        // Scene done... compile
        scene.build_accelerator();

        // Start the test
        let mut path = Path::new(start);
        let ray = Ray {
            refraction_index: 1.,
            geometry: Ray3D {
                origin: start,
                direction: dir_1,
            },
        };
        // Extend
        assert!(path.extend(&scene, &ray, 1, &mut rng));
        // Check node position
        assert!(
            (path.nodes[0].point - exp_pt0).length() < 1e-20,
            "expecting {} ... found {}",
            exp_pt0,
            path.nodes[0].point
        );

        let ray = Ray {
            refraction_index: 1.,
            geometry: Ray3D {
                origin: path.nodes[0].point,
                direction: dir_2,
            },
        };
        // Extend
        assert!(path.extend(&scene, &ray, 1, &mut rng));
        // Check node position
        assert!(
            (path.nodes[1].point - exp_pt1).length() < 1e-20,
            "expecting {} ... found {}",
            exp_pt1,
            path.nodes[1].point
        );

        let found_v1 = path.eval_from_node(1, &scene);
        let exp_v1 =
            (light_brightness * omega) * (45. as Float).to_radians().cos() * floor_reflectance / PI; //
        assert!(
            (exp_v1 - found_v1.radiance()).abs() < 1e-5,
            "found_v1 = {} | exp_v1 = {}",
            found_v1.radiance(),
            exp_v1
        );

        let found_v0 = path.eval_from_node(0, &scene);
        let exp_v0 = exp_v1 * (45. as Float).to_radians().cos() * ceiling_reflectance / PI;
        assert!(
            (exp_v0 - found_v0.radiance()).abs() < 1e-5,
            "found_v0 = {} | exp_v0 = {}",
            found_v0.radiance(),
            exp_v0
        );
    }
}
