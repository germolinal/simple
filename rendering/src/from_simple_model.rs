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

use geometry::Triangulation3D;
use model::{FenestrationType, Model, Substance};

use crate::colour::Spectrum;
use crate::material::{Glass, Material, Plastic};
use crate::primitive::Primitive;
use crate::scene::{Scene, Wavelengths};

/// An auxiliar structure only meant to create a Scene from a Model
#[derive(Default)]
pub struct SimpleModelReader {
    /// A list of the modifiers already in the model
    modifiers: Vec<String>,
}

fn transmittance_to_transmissivity(tau: crate::Float) -> crate::Float {
    ((0.8402528435 + 0.0072522239 * tau.powi(2)).sqrt() - 0.9166530661) / 0.0036261119 / tau
}

/// Used for indicating whether a triangle in the [`Scene`]
/// was part of a fenestration or a surface.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SceneElement {
    /// Leads to a Surface
    Surface,
    /// Leads to a Fenestration
    Fenestration,
}

impl SimpleModelReader {
    /// Builds a scene, returning a [`Scene`] object and also a vector
    /// mapping each triangle in the scene to a surface
    pub fn build_scene(
        &mut self,
        model: &Model,
        wavelength: &Wavelengths,
    ) -> Result<(Scene, Vec<(SceneElement, usize)>), String> {
        if matches!(wavelength, Wavelengths::Visible) {
            unimplemented!()
        }

        let mut scene = Scene::new();
        let mut triangle_map = Vec::with_capacity(model.surfaces.len() * 3);

        // Add surfaces
        for (surface_i, s) in model.surfaces.iter().enumerate() {
            let polygon = &s.vertices;
            let c_name = &s.construction;
            let construction = model.get_construction(c_name)?;
            // Should not be empty, and should have been check before this
            assert!(
                !construction.materials.is_empty(),
                "Found an empty construction, called {}",
                construction.name()
            );

            let front_mat_name = &construction.materials[0];

            let front_substance = model.get_material_substance(front_mat_name)?;
            let front_mat_index = self
                .push_substance(&mut scene, &front_substance, wavelength)
                .unwrap_or_else(|| {
                    panic!(
                    "Front material of  Construction '{}' seems to be a gas. This is not supported",
                    construction.name()
                )
                });

            let last_mat_name = construction
                .materials
                .last()
                .ok_or("Could not get last material")?;
            let back_substance = model.get_material_substance(last_mat_name)?;
            let back_mat_index = self
                .push_substance(&mut scene, &back_substance, wavelength)
                .unwrap_or_else(|| {
                    panic!(
                    "Back material of  Construction '{}' seems to be a gas. This is not supported",
                    construction.name()
                )
                });

            // Add all the triangles necessary
            let t: Triangulation3D = polygon.try_into()?;

            let triangles = t.get_trilist();
            for tri in triangles {
                scene.push_object(front_mat_index, back_mat_index, Primitive::Triangle(tri));
                triangle_map.push((SceneElement::Surface, surface_i));
            }
        }

        // Add fenestrations
        for (fen_i, s) in model.fenestrations.iter().enumerate() {
            // Openings are not material surfaces.
            if let FenestrationType::Opening = s.category {
                continue;
            }

            let polygon = &s.vertices;
            let con_name = &s.construction;
            let construction = model.get_construction(con_name)?;

            assert!(
                !construction.materials.is_empty(),
                "Found an empty construction, called {}",
                construction.name()
            );

            let front_material_name = &construction.materials[0];
            let front_substance = model.get_material_substance(front_material_name)?;
            let front_mat_index = self
                .push_substance(&mut scene, &front_substance, wavelength)
                .unwrap_or_else(|| {
                    panic!(
                    "Front material of  Construction '{}' seems to be a gas. This is not supported",
                    construction.name()
                )
                });
            let back_material_name = construction
                .materials
                .last()
                .ok_or("Could not get last material")?;
            let back_substance = model.get_material_substance(back_material_name)?;
            let back_mat_index = self
                .push_substance(&mut scene, &back_substance, wavelength)
                .unwrap_or_else(|| {
                    panic!(
                    "Back material of  Construction '{}' seems to be a gas. This is not supported",
                    construction.name()
                )
                });

            // Add all the triangles necessary
            let t: Triangulation3D = polygon
                .try_into()
                .map_err(|e| format!("Could not transform polyton into triantilagion: {}", e))?;

            let triangles = t.get_trilist();
            for tri in triangles {
                scene.push_object(front_mat_index, back_mat_index, Primitive::Triangle(tri));
                triangle_map.push((SceneElement::Fenestration, fen_i));
            }
        }

        Ok((scene, triangle_map))
    }

    /// Adds a Substance to the Scene, checking if it has been added before (by name).
    /// If a substance has already been added to the Scene, then it will not add it.
    ///
    /// Returns the index of the already existing or new Material in the Scene.
    fn push_substance(
        &mut self,
        scene: &mut Scene,
        substance: &Substance,
        wavelength: &Wavelengths,
    ) -> Option<usize> {
        let substance_name = substance.name().to_string();
        match self.get_modifier_index(&substance_name) {
            Some(i) => Some(i),
            None => {
                // Material is not there... add, then.
                let front_mat = Self::substance_to_material(substance, wavelength)?;
                Some(scene.push_material(front_mat))
            }
        }
    }

    fn get_modifier_index(&self, item: &str) -> Option<usize> {
        for (i, v) in self.modifiers.iter().enumerate() {
            if v == item {
                return Some(i);
            }
        }
        None // not found
    }

    /// Transformsa a Model Substance into a Material
    fn substance_to_material(substance: &Substance, wavelength: &Wavelengths) -> Option<Material> {
        match substance {
            Substance::Normal(s) => {
                let alpha = match *wavelength {
                    Wavelengths::Solar => match s.front_solar_absorbtance() {
                        Ok(v) => *v,
                        Err(_) => {
                            let v = 0.7;
                            // eprintln!("Substance '{}' does not have a Solar Absorbtance... assuming value of {}", s.name, v);
                            v
                        }
                    },
                    Wavelengths::Visible => match s.front_visible_reflectance() {
                        Ok(v) => *v,
                        Err(_) => {
                            let v = 0.7;
                            // eprintln!("Substance '{}' does not have a Solar Absorbtance... assuming value of {}", s.name, v);
                            v
                        }
                    },
                };
                let rho = 1. - alpha;
                let tau = match *wavelength {
                    Wavelengths::Solar => match s.solar_transmittance() {
                        Ok(v) => transmittance_to_transmissivity(*v),
                        Err(_) => {
                            let v = 0.;
                            // eprintln!("Substance '{}' does not have a Solar Absorbtance... assuming value of {}", s.name, v);
                            v
                        }
                    },
                    Wavelengths::Visible => match s.visible_transmissivity() {
                        Ok(v) => *v,
                        Err(_) => {
                            let v = 0.;
                            // eprintln!("Substance '{}' does not have a Solar Absorbtance... assuming value of {}", s.name, v);
                            v
                        }
                    },
                };

                // return
                if tau > 0.0 {
                    Some(Material::Glass(Glass {
                        colour: Spectrum::gray(tau),
                        refraction_index: 1.52,
                    }))
                } else {
                    Some(Material::Plastic(Plastic {
                        colour: Spectrum::gray(rho),
                        specularity: 0.0,
                        roughness: 0.0,
                    }))
                }
            }
            Substance::Gas(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::camera::{Film, Pinhole, View};
    use crate::material::Light;
    use crate::ray_tracer::RayTracer;
    use crate::Float;
    use geometry::{DistantSource3D, Loop3D, Point3D, Polygon3D, Vector3D};
    use model::{Construction, Fenestration, Surface};
    use std::time::Instant;
    use validate::assert_close;

    #[test]
    fn test_transmittance_to_transmissivity() {
        assert_close!(0.96, transmittance_to_transmissivity(0.88), 1e-2)
    }

    #[test]
    #[ignore]
    fn test_scene_from_model() -> Result<(), String> {
        // BUILD SCENE
        let (model, _state_header) = Model::from_file("./tests/scenes/room.spl".to_string())?;
        let mut reader = SimpleModelReader::default();
        let (mut scene, map) = reader.build_scene(&model, &Wavelengths::Solar)?;

        assert_eq!(map.len(), scene.triangles.len());

        let light_index = scene.push_material(Material::Light(Light(Spectrum::gray(10000.))));
        scene.push_object(
            light_index,
            light_index,
            Primitive::Source(DistantSource3D::new(
                Vector3D::new(1., 1., 1.),   // direction
                (0.5 as Float).to_radians(), // angle
            )),
        );

        // RENDER
        scene.build_accelerator();

        // Create film
        let film = Film {
            resolution: (512, 512),
        };

        // Create view
        let view = View {
            view_direction: Vector3D::new(0., 1., 0.).get_normalized(),
            view_point: Point3D::new(2., 1., 1.),
            ..View::default()
        };
        // Create camera
        let camera = Pinhole::new(view, film);

        let integrator = RayTracer {
            n_ambient_samples: 220,
            n_shadow_samples: 1,
            max_depth: 3,
            ..RayTracer::default()
        };

        let now = Instant::now();

        let buffer = integrator.render(&scene, &camera);
        println!("Room took {} seconds to render", now.elapsed().as_secs());
        buffer.save_hdre(std::path::Path::new(
            "./tests/scenes/images/simple_room.hdr",
        ))
    }

    #[test]
    fn test_map() -> Result<(), String> {
        let mut model = Model::default();
        let construction_name = "the construction";
        let sub = model::substance::Normal::new("the sub").wrap();
        let sub = model.add_substance(sub);
        let mat = model::Material::new("some_mat", sub.name(), 0.1);
        let mat = model.add_material(mat);
        let mut c = Construction::new(construction_name);
        c.materials.push(mat.name().clone());

        model.add_construction(c);

        /* SINGLE SURFACE -  A TRIANGLE */
        let mut the_l = Loop3D::with_capacity(3);
        the_l.push(Point3D::new(0., 0., 0.))?;
        the_l.push(Point3D::new(1., 0., 0.))?;
        the_l.push(Point3D::new(0., 1., 0.))?;
        the_l.close()?;

        let poly = Polygon3D::new(the_l)?;

        let s = Surface::new(
            "some surf",
            poly,
            construction_name,
            model::Boundary::default(),
            model::Boundary::default(),
        );
        model.add_surface(s);

        let mut r = SimpleModelReader::default();
        let (scene, map) = r.build_scene(&model, &Wavelengths::Solar)?;
        assert_eq!(scene.triangles.len(), map.len());

        assert_eq!(map.len(), 1);
        let (element_type, index) = map[0];
        assert_eq!(element_type, SceneElement::Surface);
        assert_eq!(0, index);

        /* A TRIANGLE AND A SQUARE */
        let mut the_l = Loop3D::with_capacity(3);
        the_l.push(Point3D::new(0., 0., 0.))?;
        the_l.push(Point3D::new(1., 0., 0.))?;
        the_l.push(Point3D::new(1., 1., 0.))?;
        the_l.push(Point3D::new(0., 1., 0.))?;
        the_l.close()?;

        let poly = Polygon3D::new(the_l)?;

        let s = Surface::new(
            "another surf",
            poly,
            construction_name,
            model::Boundary::default(),
            model::Boundary::default(),
        );
        model.add_surface(s);

        let mut r = SimpleModelReader::default();
        let (scene, map) = r.build_scene(&model, &Wavelengths::Solar)?;
        assert_eq!(scene.triangles.len(), map.len());

        assert_eq!(map.len(), 3);

        let (element_type, index) = map[0];
        assert_eq!(element_type, SceneElement::Surface);
        assert_eq!(0, index);
        let (element_type, index) = map[1];
        assert_eq!(element_type, SceneElement::Surface);
        assert_eq!(1, index);
        let (element_type, index) = map[2];
        assert_eq!(element_type, SceneElement::Surface);
        assert_eq!(1, index);

        /* A TRIANGLE AND A SQUARE AND A FENESTRATION */
        let mut the_l = Loop3D::with_capacity(3);
        the_l.push(Point3D::new(0., 0., 1.))?;
        the_l.push(Point3D::new(1., 0., 1.))?;
        the_l.push(Point3D::new(1., 1., 1.))?;
        the_l.push(Point3D::new(0., 1., 1.))?;
        the_l.close()?;

        let poly = Polygon3D::new(the_l)?;

        let s = Fenestration::new(
            "some fen",
            poly,
            construction_name,
            model::FenestrationType::Window,
            model::Boundary::default(),
            model::Boundary::default(),
        );
        model.add_fenestration(s)?;

        let mut r = SimpleModelReader::default();
        let (scene, map) = r.build_scene(&model, &Wavelengths::Solar)?;
        assert_eq!(scene.triangles.len(), map.len());

        assert_eq!(map.len(), 5);

        let (element_type, index) = map[0];
        assert_eq!(element_type, SceneElement::Surface);
        assert_eq!(0, index);
        let (element_type, index) = map[1];
        assert_eq!(element_type, SceneElement::Surface);
        assert_eq!(1, index);
        let (element_type, index) = map[2];
        assert_eq!(element_type, SceneElement::Surface);
        assert_eq!(1, index);
        let (element_type, index) = map[3];
        assert_eq!(element_type, SceneElement::Fenestration);
        assert_eq!(0, index);
        let (element_type, index) = map[4];
        assert_eq!(element_type, SceneElement::Fenestration);
        assert_eq!(0, index);

        Ok(())
    }
}
