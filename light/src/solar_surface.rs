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

use std::sync::Arc;

use crate::Float;

use matrix::Matrix;
use rendering::{colour_matrix::*, DCFactory, Ray, Scene};

use model::{
    Boundary, Fenestration, SimulationStateElement, SimulationStateHeader, Surface,
};

use geometry::{Point3D, Polygon3D, Ray3D, Triangulation3D, Vector3D};
use rendering::primitive_samplers::sample_triangle_surface;
use rendering::rand::*;

use crate::optical_info::IRViewFactorSet;

fn get_sampler(triangles_areas: Vec<Float>) -> impl Fn(&mut RandGen) -> usize {
    let total_area: Float = triangles_areas.iter().sum();

    move |rng: &mut RandGen| -> usize {
        let mut r: Float = rng.gen();
        r *= total_area;
        let mut acc = 0.0;
        for (i, area) in triangles_areas.iter().enumerate() {
            acc += area;
            if r <= acc {
                return i;
            }
        }
        unreachable!();
    }
}

/// Structure that can help calculate solar radiation
///
/// It contains the normal of the original Surface and the points
/// randomly sampled in each surface.
pub struct SolarSurface {
    points: Vec<Point3D>,
    pub normal: Vector3D,
    pub receives_sun_front: bool,
    pub receives_sun_back: bool,
}

impl SolarSurface {
    /// Offset for the starting point of the rays.
    const DELTA: Float = 0.001;

    /// Creates a new Solar Surface
    pub fn new(
        nrays: usize,
        polygon: &Polygon3D,
        receives_sun_front: bool,
        receives_sun_back: bool,
    ) -> Result<Self, String> {
        // Get polygon
        let normal = polygon.normal();

        // Triangulate the polygon
        let triangles = Triangulation3D::from_polygon(polygon)?.get_trilist();
        let triangles_areas: Vec<Float> = triangles.iter().map(|t| t.area()).collect();

        // Build a triangle sampler
        let sampler = get_sampler(triangles_areas);
        let mut rng = get_rng();

        // sample points
        let points: Vec<Point3D> = (0..nrays)
            .map(|_| {
                // choose the triangle
                let i = sampler(&mut rng);
                // choose a point in the triangle
                sample_triangle_surface(&triangles[i], &mut rng)
            })
            .collect();

        // return
        Ok(Self {
            normal,
            points,
            receives_sun_front,
            receives_sun_back,
        })
    }

    /// Receives sun if the boundary is either Space or Outdoor    
    pub(crate) fn boundary_receives_sun(boundary: &Boundary) -> bool {
        matches!(boundary, Boundary::Space { .. } | Boundary::Outdoor)
    }

    /// Receives a list of `SolarSurface` objects as well as the `Scene` containing them and
    /// calculates the Daylight Coefficient Matrix that can be used for
    /// estimating the incident solar radiation in W/m2. The options for this calculation are
    /// contained in the `DCFactory` used as input.
    pub fn calc_solar_dc_matrix(
        list: &[SolarSurface],
        scene: &Scene,
        dc_factory: &DCFactory,
        front_side: bool,
    ) -> Result<Matrix, String> {
        if list.is_empty() {
            return Ok(Matrix::empty());
        }

        let mut dcs: Vec<Matrix> = Vec::with_capacity(list.len());

        for s in list.iter() {
            // Skip front ones that do not receive front sun
            if front_side && !s.receives_sun_front {
                continue;
            }
            // Skip back ones that do not receive back side.
            if !front_side && !s.receives_sun_back {
                continue;
            }
            let rays = if front_side {
                s.front_rays()
            } else {
                s.back_rays()
            };
            dcs.push(s.solar_irradiance(&rays, scene, dc_factory))
        }
        if dcs.is_empty() {
            Ok(Matrix::empty())
        } else {
            let mut ret = dcs[0].clone();
            for dc in dcs.iter().skip(1) {
                ret.concat_rows(dc)?;
            }
            Ok(ret)
        }
    }

    /// Builds a set of SolarSurfaces from Fenestrations
    ///
    /// Adds the necessary elements to the `SimulationStateHeader`
    pub fn make_fenestrations(
        list: &[Arc<Fenestration>],
        state: &mut SimulationStateHeader,
        n_rays: usize,
    ) -> Result<Vec<SolarSurface>, String> {
        let mut ret: Vec<SolarSurface> = Vec::with_capacity(list.len());
        for (i, s) in list.iter().enumerate() {
            if s.front_incident_solar_irradiance_index().is_none() {
                let i = state.push(
                    SimulationStateElement::FenestrationFrontSolarIrradiance(i),
                    0.0,
                )?;
                s.set_front_incident_solar_irradiance_index(i)?;
            }

            if s.back_incident_solar_irradiance_index().is_none() {
                let i = state.push(
                    SimulationStateElement::FenestrationBackSolarIrradiance(i),
                    0.0,
                )?;
                s.set_back_incident_solar_irradiance_index(i)?;
            }

            if s.front_ir_irradiance_index().is_none() {
                let i = state.push(
                    SimulationStateElement::FenestrationFrontIRIrradiance(i),
                    0.0,
                )?;
                s.set_front_ir_irradiance_index(i)?;
            }

            if s.back_ir_irradiance_index().is_none() {
                let i = state.push(SimulationStateElement::FenestrationBackIRIrradiance(i), 0.0)?;
                s.set_back_ir_irradiance_index(i)?;
            }

            let receives_sun_front = Self::boundary_receives_sun(&s.front_boundary);
            let receives_sun_back = Self::boundary_receives_sun(&s.back_boundary);

            ret.push(SolarSurface::new(
                n_rays,
                &s.vertices,
                receives_sun_front,
                receives_sun_back,
            )?)
        }

        Ok(ret)
    }

    /// Builds a set of SolarSurfaces from Surfaces
    ///
    /// Adds the necessary elements to the `SimulationStateHeader
    pub fn make_surfaces(
        list: &[Arc<Surface>],
        state: &mut SimulationStateHeader,
        n_rays: usize,
    ) -> Result<Vec<SolarSurface>, String> {
        let mut ret: Vec<SolarSurface> = Vec::with_capacity(list.len());

        for (i, s) in list.iter().enumerate() {
            if s.front_incident_solar_irradiance_index().is_none() {
                let i = state.push(SimulationStateElement::SurfaceFrontSolarIrradiance(i), 0.0)?;
                s.set_front_incident_solar_irradiance_index(i)?;
            }

            if s.back_incident_solar_irradiance_index().is_none() {
                let i = state.push(SimulationStateElement::SurfaceBackSolarIrradiance(i), 0.0)?;
                s.set_back_incident_solar_irradiance_index(i)?;
            }

            if s.front_ir_irradiance_index().is_none() {
                let i = state.push(SimulationStateElement::SurfaceFrontIRIrradiance(i), 0.0)?;
                s.set_front_ir_irradiance_index(i)?;
            }

            if s.back_ir_irradiance_index().is_none() {
                let i = state.push(SimulationStateElement::SurfaceBackIRIrradiance(i), 0.0)?;
                s.set_back_ir_irradiance_index(i)?;
            }

            let receives_sun_front = Self::boundary_receives_sun(&s.front_boundary);
            let receives_sun_back = Self::boundary_receives_sun(&s.back_boundary);

            // create
            ret.push(SolarSurface::new(
                n_rays,
                &s.vertices,
                receives_sun_front,
                receives_sun_back,
            )?)
        }

        Ok(ret)
    }

    /// Gets the front rays of a surface
    pub fn front_rays(&self) -> Vec<Ray3D> {
        self.points
            .iter()
            .map(|p| Ray3D {
                direction: self.normal,
                origin: *p + self.normal * Self::DELTA,
            })
            .collect()
    }

    /// Gets the back rays of a surface
    pub fn back_rays(&self) -> Vec<Ray3D> {
        self.points
            .iter()
            .map(|p| Ray3D {
                direction: self.normal * -1.,
                origin: *p - self.normal * Self::DELTA,
            })
            .collect()
    }

    /// Calculates the Daylight Coefficient matrix for the front of a `SolarSurface`
    pub fn solar_irradiance(&self, rays: &[Ray3D], scene: &Scene, factory: &DCFactory) -> Matrix {
        // let front_rays = self.front_rays();
        let dc = factory.calc_dc(rays, scene);
        let dc = colour_matrix_to_radiance(&dc);
        average_matrix(&dc)
    }

    /// Calculates an [`IRViewFactorSet`] for this surface
    pub fn calc_view_factors(
        &self,
        scene: &Scene,
        front_side: bool,
    ) -> Result<IRViewFactorSet, String> {
        let mut rng = rendering::rand::get_rng();

        let rays = if front_side {
            self.front_rays()
        } else {
            self.back_rays()
        };

        let mut ground = 0.0;
        let mut sky = 0.0;

        let n_samples = 10000;
        let mut node_aux = Vec::with_capacity(2);
        for r in &rays {
            let mut ray = Ray {
                geometry: *r,
                ..Ray::default()
            };
            let normal = r.direction;
            let e1 = normal.get_perpendicular()?;
            let e2 = normal.cross(e1);

            for _ in 0..n_samples {
                let dir = rendering::samplers::uniform_sample_hemisphere(&mut rng, e1, e2, normal);

                if scene.cast_ray(&mut ray, &mut node_aux).is_none() {
                    if dir.z > 0.0 {
                        sky += 1.0;
                    } else {
                        ground += 1.;
                    }
                }
            }
        }

        let n = n_samples as Float * rays.len() as Float;
        ground /= n;
        sky /= n;

        let beta = sky.sqrt();
        let air = sky * (1. - beta);
        sky *= beta;

        Ok(IRViewFactorSet { sky, ground, air })
    }
}

#[cfg(test)]
mod testing {
    use super::*;
    use geometry::Loop3D;
    use validate::assert_close;

    #[test]
    fn test_view_factors_empty_scene_vertical() {
        let mut the_loop = Loop3D::new();
        the_loop.push(Point3D::new(0., 0., 0.)).unwrap();
        the_loop.push(Point3D::new(1., 0., 0.)).unwrap();
        the_loop.push(Point3D::new(1., 0., 1.)).unwrap();
        the_loop.push(Point3D::new(0., 0., 1.)).unwrap();
        the_loop.close().unwrap();

        let mut scene = Scene::new();
        scene.build_accelerator();
        let p = Polygon3D::new(the_loop).unwrap();
        let s = SolarSurface::new(10, &p, true, true).unwrap();

        let beta = (0.5 as Float).sqrt();

        // Front side
        let views = s.calc_view_factors(&scene, true).unwrap();

        assert_close!(views.ground, 0.5, 1e-2);
        assert_close!(views.sky, 0.5 * beta, 1e-2);
        assert_close!(views.air, 0.5 * (1. - beta), 1e-2);

        // back side
        let views = s.calc_view_factors(&scene, false).unwrap();

        assert_close!(views.ground, 0.5, 1e-2);
        assert_close!(views.sky, 0.5 * beta, 1e-2);
        assert_close!(views.air, 0.5 * (1. - beta), 1e-2);
    }

    #[test]
    fn test_view_factors_empty_scene_horizontal() {
        let mut the_loop = Loop3D::new();
        the_loop.push(Point3D::new(0., 0., 0.)).unwrap();
        the_loop.push(Point3D::new(1., 0., 0.)).unwrap();
        the_loop.push(Point3D::new(1., 1., 0.)).unwrap();
        the_loop.push(Point3D::new(0., 1., 0.)).unwrap();
        the_loop.close().unwrap();

        let mut scene = Scene::new();
        scene.build_accelerator();
        let p = Polygon3D::new(the_loop).unwrap();
        let s = SolarSurface::new(10, &p, true, true).unwrap();

        // Front side
        let views = s.calc_view_factors(&scene, true).unwrap();

        assert_close!(views.ground, 0.0, 1e-4);
        assert_close!(views.sky, 1.0, 1e-4);
        assert_close!(views.air, 0.0, 1e-4);

        // back side
        let views = s.calc_view_factors(&scene, false).unwrap();

        assert_close!(views.ground, 1.0, 1e-4);
        assert_close!(views.sky, 0.0, 1e-4);
        assert_close!(views.air, 0.0, 1e-4);
    }

    #[test]
    fn test_new_boundary_fenestrations() {
        // Check that the receives_sun is properly assigned

        // create geometry... they will all have this same one.
        let mut outer = Loop3D::new();
        outer.push(Point3D::new(0., 0., 0.)).unwrap();
        outer.push(Point3D::new(0., 1., 0.)).unwrap();
        outer.push(Point3D::new(0., 0., 1.)).unwrap();
        outer.close().unwrap();
        let poly = Polygon3D::new(outer).unwrap();

        // state header
        let mut state = SimulationStateHeader::new();

        // container
        let mut list = Vec::with_capacity(6);

        // Fen 0: outdoor on front and back
        let fen = Fenestration::new(
            "some fen",
            poly.clone(),
            "some construction",
            Boundary::Outdoor,
            Boundary::Outdoor,
        );
        list.push(Arc::new(fen));

        // Fen 1: outdoor on front, space at the back
        let fen = Fenestration::new(
            "some fen",
            poly.clone(),
            "some construction",
            Boundary::Outdoor,
            Boundary::Space {
                space: "some space".into(),
            },
        );
        list.push(Arc::new(fen));

        // Fen 2: outdoor on back, space at the front
        let fen = Fenestration::new(
            "some fen",
            poly.clone(),
            "some construction",
            Boundary::Space {
                space: "some space".into(),
            },
            Boundary::Outdoor,
        );
        list.push(Arc::new(fen));

        // Fen 3: Ambient Temp at the back, space at the front
        let fen = Fenestration::new(
            "some fen",
            poly.clone(),
            "some construction",
            Boundary::Space {
                space: "some space".into(),
            },
            Boundary::AmbientTemperature { temperature: 1. },
        );
        list.push(Arc::new(fen));

        // Fen 4: Ambient Temp at the back and front
        let fen = Fenestration::new(
            "some fen",
            poly.clone(),
            "some construction",
            Boundary::AmbientTemperature { temperature: 1. },
            Boundary::AmbientTemperature { temperature: 1. },
        );
        list.push(Arc::new(fen));

        // Fen 5: Ground at the back and front
        let fen = Fenestration::new(
            "some fen",
            poly.clone(),
            "some construction",
            Boundary::Ground,
            Boundary::Ground,
        );

        list.push(Arc::new(fen));

        // Calc
        let fens = SolarSurface::make_fenestrations(&list, &mut state, 1).unwrap();

        // check.
        assert!(fens[0].receives_sun_back);
        assert!(fens[0].receives_sun_front);

        assert!(fens[1].receives_sun_back);
        assert!(fens[1].receives_sun_front);

        assert!(fens[2].receives_sun_back);
        assert!(fens[2].receives_sun_front);

        assert!(!fens[3].receives_sun_back);
        assert!(fens[3].receives_sun_front);

        assert!(!fens[4].receives_sun_back);
        assert!(!fens[4].receives_sun_front);

        assert!(!fens[5].receives_sun_back);
        assert!(!fens[5].receives_sun_front);
    }

    #[test]
    fn test_new_boundary_surfaces() {
        // Check that the receives_sun is properly assigned

        // create geometry... they will all have this same one.
        let mut outer = Loop3D::new();
        outer.push(Point3D::new(0., 0., 0.)).unwrap();
        outer.push(Point3D::new(0., 1., 0.)).unwrap();
        outer.push(Point3D::new(0., 0., 1.)).unwrap();
        outer.close().unwrap();
        let poly = Polygon3D::new(outer).unwrap();

        // state header
        let mut state = SimulationStateHeader::new();

        // container
        let mut list = Vec::with_capacity(6);

        // Fen 0: outdoor on front and back
        let fen = Surface::new(
            "some fen",
            poly.clone(),
            "some construction",
            Boundary::Outdoor,
            Boundary::Outdoor,
        );
        list.push(Arc::new(fen));

        // Fen 1: outdoor on front, space at the back
        let fen = Surface::new(
            "some fen",
            poly.clone(),
            "some construction",
            Boundary::Outdoor,
            Boundary::Space {
                space: "some space".into(),
            },
        );
        list.push(Arc::new(fen));

        // Fen 2: outdoor on back, space at the front
        let fen = Surface::new(
            "some fen",
            poly.clone(),
            "some construction",
            Boundary::Space {
                space: "some space".into(),
            },
            Boundary::Outdoor,
        );

        list.push(Arc::new(fen));

        // Fen 3: Ambient Temp at the back, space at the front
        let fen = Surface::new(
            "some fen",
            poly.clone(),
            "some construction",
            Boundary::Space {
                space: "some space".into(),
            },
            Boundary::AmbientTemperature { temperature: 1. },
        );

        list.push(Arc::new(fen));

        // Fen 4: Ambient Temp at the back and front
        let fen = Surface::new(
            "some fen",
            poly.clone(),
            "some construction",
            Boundary::AmbientTemperature { temperature: 1. },
            Boundary::AmbientTemperature { temperature: 1. },
        );
        list.push(Arc::new(fen));

        // Fen 5: Ground at the back and front
        let fen = Surface::new(
            "some fen",
            poly.clone(),
            "some construction",
            Boundary::Ground,
            Boundary::Ground,
        );

        list.push(Arc::new(fen));

        // Calc
        let fens = SolarSurface::make_surfaces(&list, &mut state, 1).unwrap();

        // check.
        assert!(fens[0].receives_sun_back);
        assert!(fens[0].receives_sun_front);

        assert!(fens[1].receives_sun_back);
        assert!(fens[1].receives_sun_front);

        assert!(fens[2].receives_sun_back);
        assert!(fens[2].receives_sun_front);

        assert!(!fens[3].receives_sun_back);
        assert!(fens[3].receives_sun_front);

        assert!(!fens[4].receives_sun_back);
        assert!(!fens[4].receives_sun_front);

        assert!(!fens[5].receives_sun_back);
        assert!(!fens[5].receives_sun_front);
    }

    #[test]
    fn test_boundary_receives_sun() {
        // This should not receive
        assert!(!SolarSurface::boundary_receives_sun(&Boundary::Ground));
        assert!(!SolarSurface::boundary_receives_sun(&Boundary::AmbientTemperature {
            temperature: 12.
        }));
        // These do receive
        assert!(SolarSurface::boundary_receives_sun(&Boundary::Outdoor));
        assert!(SolarSurface::boundary_receives_sun(&Boundary::Space {
            space: "the space".to_string()
        }));
    }
}
