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

use std::sync::Arc;

use crate::Float;

use matrix::Matrix;
use rendering::{DCFactory, Scene, colour_matrix::*};

use model::{Boundary, Fenestration, SimulationStateElement, SimulationStateHeader, Surface};

use geometry::{Point3D, Polygon3D, Ray3D, Triangulation3D, Vector3D};
use rendering::primitive_samplers::sample_triangle_surface;
use rendering::rand::*;
use utils::ProgressBar;

use crate::optical_info::IRViewFactorSet;

fn get_sampler(triangles_areas: Vec<Float>) -> impl Fn(&mut RandGen) -> usize {
    let total_area: Float = triangles_areas.iter().sum();

    move |rng: &mut RandGen| -> usize {
        let mut r: Float = rng.random();
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
    pub(crate) points: Vec<Point3D>,
    /// The normal of the surface (its front side)
    pub normal: Vector3D,
    /// Whether the front side receives sun
    pub receives_sun_front: bool,
    /// Whether the back side receives sun
    pub receives_sun_back: bool,

    /// The triangles of the surface, and their areas (for sampling points)
    triangles: Vec<geometry::Triangle3D>,
    triangle_areas: Vec<Float>,
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
        let t: Triangulation3D = polygon.try_into()?;
        let triangles = t.get_trilist();
        let triangle_areas: Vec<Float> = triangles.iter().map(|t| t.area()).collect();

        // Build a triangle sampler
        let sampler = get_sampler(triangle_areas.clone());
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
            triangles,
            triangle_areas,
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
        progress_bar: Option<&ProgressBar>,
    ) -> Result<Matrix, String> {
        if list.is_empty() {
            return Ok(Matrix::empty());
        }

        let mut dcs: Vec<Matrix> = Vec::with_capacity(list.len());

        for s in list.iter() {
            // Skip front ones that do not receive front sun
            if front_side && !s.receives_sun_front {
                dcs.push(Matrix::new(0.0, 1, dc_factory.reinhart.n_bins));
                continue;
            }
            // Skip back ones that do not receive back side.
            if !front_side && !s.receives_sun_back {
                dcs.push(Matrix::new(0.0, 1, dc_factory.reinhart.n_bins));
                continue;
            }
            let rays = if front_side {
                s.front_rays()
            } else {
                s.back_rays()
            };

            dcs.push(s.solar_irradiance(&rays, scene, dc_factory, progress_bar))
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
    pub fn solar_irradiance(
        &self,
        rays: &[Ray3D],
        scene: &Scene,
        factory: &DCFactory,
        progress_bar: Option<&ProgressBar>,
    ) -> Matrix {
        // let front_rays = self.front_rays();
        let dc = factory.calc_dc(rays, scene, progress_bar);
        let dc = colour_matrix_to_radiance(&dc);
        average_matrix(&dc)
    }

    /// Calculates an [`IRViewFactorSet`] for one side of this surface, sending
    /// about `n_rays` rays (rounded up to a square, as their directions are
    /// stratified on a grid), each from a new random point on the surface.
    ///
    /// Directions are cosine-weighted, so each ray weighs the same in the
    /// view factor. Rays that hit something in the scene count neither as sky
    /// nor as ground.
    pub fn calc_view_factors(
        &self,
        scene: &Scene,
        front_side: bool,
        n_rays: usize,
    ) -> Result<IRViewFactorSet, String> {
        let mut rng = rendering::rand::get_rng();
        let normal = if front_side {
            self.normal
        } else {
            self.normal * -1.
        };
        let e1 = normal.get_perpendicular()?;
        let e2 = normal.cross(e1);
        let sampler = get_sampler(self.triangle_areas.clone());

        let mut ground = 0.0;
        let mut sky = 0.0;

        // Directions are stratified: a jittered `side × side` grid
        let side = (n_rays.max(1) as Float).sqrt().ceil() as usize;
        let mut node_aux = [0; 32];
        for i in 0..side {
            for j in 0..side {
                // A random point on the surface (the view changes across it,
                // e.g., under an overhang)
                let triangle = &self.triangles[sampler(&mut rng)];
                let origin = sample_triangle_surface(triangle, &mut rng) + normal * Self::DELTA;

                let (du, dv): (Float, Float) = rng.random();
                let u = (
                    (i as Float + du) / side as Float,
                    (j as Float + dv) / side as Float,
                );
                let local = rendering::samplers::sample_cosine_weighted_horizontal_hemisphere(u);
                let dir = e1 * local.x + e2 * local.y + normal * local.z;
                if scene
                    .cast_ray(
                        Ray3D {
                            origin,
                            direction: dir,
                        },
                        &mut node_aux,
                    )
                    .is_some()
                {
                    // Sees the scene: neither sky nor ground
                    continue;
                }
                if dir.z > 0.0 {
                    sky += 1.0;
                } else {
                    ground += 1.;
                }
            }
        }

        let n = (side * side) as Float;
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

    /// A surface from its vertices
    fn surface(vertices: &[(Float, Float, Float)]) -> Result<SolarSurface, String> {
        let mut the_loop = Loop3D::new();
        for (x, y, z) in vertices {
            the_loop.push(Point3D::new(*x, *y, *z))?;
        }
        the_loop.close()?;
        SolarSurface::new(10, &Polygon3D::new(the_loop)?, true, true)
    }

    /// A scene made of `polygons` (Radiance syntax)
    fn scene(polygons: &str) -> Scene {
        let src = format!("void plastic grey 0 0 5 0.5 0.5 0.5 0 0\n{polygons}");
        let mut scene = Scene::from_radiance_source(src.as_bytes()).unwrap();
        scene.build_accelerator();
        scene
    }

    /// View factors used to be counted on directions uniformly distributed
    /// over the hemisphere (not cosine-weighted), and whether a point saw
    /// the outside was decided by its normal ray alone
    #[test]
    fn view_factors_tilted_and_obstructed() -> Result<(), String> {
        // Unobstructed roof tilted 45°: (1 ± cos 45°) / 2
        let roof = surface(&[(0., 0., 0.), (1., 0., 0.), (1., 1., 1.), (0., 1., 1.)])?;
        let front = roof.normal.z > 0.;
        let v = roof.calc_view_factors(&scene(""), front, 16384)?;
        let cos = (45. as Float).to_radians().cos();
        assert_close!(v.sky + v.air, (1. + cos) / 2., 0.005);
        assert_close!(v.ground, (1. - cos) / 2., 0.005);

        // A wall facing another one sees some sky (its normal ray hits the
        // other wall)
        let wall = surface(&[(0., 0., 0.), (0., 1., 0.), (0., 1., 1.), (0., 0., 1.)])?;
        let canyon = scene("grey polygon w 0 0 12 2 -50 0 2 50 0 2 50 3 2 -50 3\n");
        let v = wall.calc_view_factors(&canyon, wall.normal.x > 0., 16384)?;
        assert!(v.sky + v.air > 0.09 && v.sky + v.air < 0.13, "{v:?}");

        // A window under an overhang sees less sky than an unobstructed one
        // (its normal ray misses the overhang). Over 2000 runs, sky + air was
        // 0.2734 on average, with a standard deviation of 0.0019, so these
        // bounds are ~7 standard deviations away (0.27 used to fail 1 in 25)
        let window = surface(&[(0., 0., 0.5), (0., 1., 0.5), (0., 1., 1.5), (0., 0., 1.5)])?;
        let overhang = scene("grey polygon o 0 0 12 0 -0.5 1.6 1 -0.5 1.6 1 1.5 1.6 0 1.5 1.6\n");
        let v = window.calc_view_factors(&overhang, window.normal.x > 0., 16384)?;
        assert!(v.sky + v.air > 0.26 && v.sky + v.air < 0.29, "{v:?}");
        Ok(())
    }

    /// Bias, spread and time of the view factors for several numbers of rays
    #[test]
    #[ignore]
    fn view_factor_convergence() -> Result<(), String> {
        // cargo test --release -p light --lib view_factor_convergence -- --ignored --nocapture
        let cases = [
            (
                "A: roof tilted 45°, unobstructed",
                surface(&[(0., 0., 0.), (1., 0., 0.), (1., 1., 1.), (0., 1., 1.)])?,
                scene(""),
                Some((1. + (45. as Float).to_radians().cos()) / 2.),
            ),
            (
                "B: wall facing a 3 m wall, 2 m away",
                surface(&[(0., 0., 0.), (0., 1., 0.), (0., 1., 1.), (0., 0., 1.)])?,
                scene("grey polygon w 0 0 12 2 -50 0 2 50 0 2 50 3 2 -50 3\n"),
                None,
            ),
            (
                "C: window under a 1 m overhang",
                surface(&[(0., 0., 0.5), (0., 1., 0.5), (0., 1., 1.5), (0., 0., 1.5)])?,
                scene("grey polygon o 0 0 12 0 -0.5 1.6 1 -0.5 1.6 1 1.5 1.6 0 1.5 1.6\n"),
                None,
            ),
        ];
        for (name, surface, scene, analytic) in cases {
            // Which side faces the scene: the one whose normal points to +x
            // (or up, for the roof)
            let front = surface.normal.x > 0.5 || surface.normal.z > 0.5;
            let sky_view = |n: usize| -> Result<(Float, Float), String> {
                let v = surface.calc_view_factors(&scene, front, n)?;
                Ok((v.sky + v.air, v.ground))
            };
            let reference = match analytic {
                Some(sky) => sky,
                None => sky_view(2048 * 2048)?.0,
            };
            eprintln!("{name}: sky {reference:.4}");
            for n in [16, 256, 1024, 4096, 16384, 65536] {
                let reps = 30;
                let start = std::time::Instant::now();
                let values: Vec<Float> = (0..reps).map(|_| sky_view(n).unwrap().0).collect();
                let ms = start.elapsed().as_secs_f64() * 1000. / reps as f64;
                let mean = values.iter().sum::<Float>() / reps as Float;
                let std = (values.iter().map(|v| (v - mean).powi(2)).sum::<Float>()
                    / (reps - 1) as Float)
                    .sqrt();
                eprintln!(
                    "    {n:5} rays: bias {:+.4}, std {std:.4}, {ms:.2} ms",
                    mean - reference
                );
            }
        }
        Ok(())
    }
    use geometry::Loop3D;
    use validate::assert_close;

    #[test]
    fn test_view_factors_empty_scene_vertical() -> Result<(), String> {
        let mut the_loop = Loop3D::new();
        the_loop.push(Point3D::new(0., 0., 0.))?;
        the_loop.push(Point3D::new(1., 0., 0.))?;
        the_loop.push(Point3D::new(1., 0., 1.))?;
        the_loop.push(Point3D::new(0., 0., 1.))?;
        the_loop.close()?;

        let mut scene = Scene::new();
        scene.build_accelerator();
        let p = Polygon3D::new(the_loop)?;
        let s = SolarSurface::new(10, &p, true, true)?;

        let beta = (0.5 as Float).sqrt();

        // Front side
        let views = s.calc_view_factors(&scene, true, 16384)?;

        assert_close!(views.ground, 0.5, 1e-2);
        assert_close!(views.sky, 0.5 * beta, 1e-2);
        assert_close!(views.air, 0.5 * (1. - beta), 1e-2);

        // back side
        let views = s.calc_view_factors(&scene, false, 16384)?;

        assert_close!(views.ground, 0.5, 1e-2);
        assert_close!(views.sky, 0.5 * beta, 1e-2);
        assert_close!(views.air, 0.5 * (1. - beta), 1e-2);

        Ok(())
    }

    #[test]
    fn test_view_factors_empty_scene_horizontal() -> Result<(), String> {
        let mut the_loop = Loop3D::new();
        the_loop.push(Point3D::new(0., 0., 0.))?;
        the_loop.push(Point3D::new(1., 0., 0.))?;
        the_loop.push(Point3D::new(1., 1., 0.))?;
        the_loop.push(Point3D::new(0., 1., 0.))?;
        the_loop.close()?;

        let mut scene = Scene::new();
        scene.build_accelerator();
        let p = Polygon3D::new(the_loop)?;
        let s = SolarSurface::new(10, &p, true, true)?;

        // Front side
        let views = s.calc_view_factors(&scene, true, 16384)?;

        assert_close!(views.ground, 0.0, 1e-4);
        assert_close!(views.sky, 1.0, 1e-4);
        assert_close!(views.air, 0.0, 1e-4);

        // back side
        let views = s.calc_view_factors(&scene, false, 16384)?;

        assert_close!(views.ground, 1.0, 1e-4);
        assert_close!(views.sky, 0.0, 1e-4);
        assert_close!(views.air, 0.0, 1e-4);

        Ok(())
    }

    #[test]
    fn test_new_boundary_fenestrations() -> Result<(), String> {
        // Check that the receives_sun is properly assigned

        // create geometry... they will all have this same one.
        let mut outer = Loop3D::new();
        outer.push(Point3D::new(0., 0., 0.))?;
        outer.push(Point3D::new(0., 1., 0.))?;
        outer.push(Point3D::new(0., 0., 1.))?;
        outer.close()?;
        let poly = Polygon3D::new(outer)?;

        // state header
        let mut state = SimulationStateHeader::new();

        // container
        let mut list = Vec::with_capacity(6);

        // Fen 0: outdoor on front and back
        let fen = Fenestration::new(
            "some fen",
            poly.clone(),
            "some construction",
            model::FenestrationType::Window,
            Boundary::Outdoor,
            Boundary::Outdoor,
        );
        list.push(Arc::new(fen));

        // Fen 1: outdoor on front, space at the back
        let fen = Fenestration::new(
            "some fen",
            poly.clone(),
            "some construction",
            model::FenestrationType::Window,
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
            model::FenestrationType::Window,
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
            model::FenestrationType::Window,
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
            model::FenestrationType::Window,
            Boundary::AmbientTemperature { temperature: 1. },
            Boundary::AmbientTemperature { temperature: 1. },
        );
        list.push(Arc::new(fen));

        // Fen 5: Ground at the back and front
        let fen = Fenestration::new(
            "some fen",
            poly.clone(),
            "some construction",
            model::FenestrationType::Window,
            Boundary::Ground,
            Boundary::Ground,
        );

        list.push(Arc::new(fen));

        // Calc
        let fens = SolarSurface::make_fenestrations(&list, &mut state, 1)?;

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

        Ok(())
    }

    #[test]
    fn test_new_boundary_surfaces() -> Result<(), String> {
        // Check that the receives_sun is properly assigned

        // create geometry... they will all have this same one.
        let mut outer = Loop3D::new();
        outer.push(Point3D::new(0., 0., 0.))?;
        outer.push(Point3D::new(0., 1., 0.))?;
        outer.push(Point3D::new(0., 0., 1.))?;
        outer.close()?;
        let poly = Polygon3D::new(outer)?;

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
        let fens = SolarSurface::make_surfaces(&list, &mut state, 1)?;

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

        Ok(())
    }

    #[test]
    fn test_boundary_receives_sun() {
        // This should not receive
        assert!(!SolarSurface::boundary_receives_sun(&Boundary::Ground));
        assert!(!SolarSurface::boundary_receives_sun(
            &Boundary::AmbientTemperature { temperature: 12. }
        ));
        // These do receive
        assert!(SolarSurface::boundary_receives_sun(&Boundary::Outdoor));
        assert!(SolarSurface::boundary_receives_sun(&Boundary::Space {
            space: "the space".to_string()
        }));
    }
}
