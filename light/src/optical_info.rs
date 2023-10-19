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
use crate::solar_surface::SolarSurface;
use crate::Float;
use matrix::Matrix;
use model::{Model, SimulationStateHeader, SolarOptions};
use rendering::{DCFactory, Scene, Wavelengths};
use serde::{Deserialize, Serialize};
use weather::ReinhartSky;

/// A set of view factors as seen by a `ThermalSurface`.
#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct IRViewFactorSet {
    /// The fraction of the view that corresponds to the sky
    pub sky: Float,

    /// The fraction of the view that corresponds to the ground
    pub ground: Float,

    /// The fraction of the view that corresponds to other objects and
    /// surfaces (they are assumed to be at air temperature)
    pub air: Float,
}

/// Information about the solar radiation and other optical elements
/// of the whole model.
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct OpticalInfo {
    /// The [`IRViewFactorSet`] for the front side of each surface
    pub front_surfaces_view_factors: Vec<IRViewFactorSet>,

    /// The [`IRViewFactorSet`] for the back side of each surface
    pub back_surfaces_view_factors: Vec<IRViewFactorSet>,

    /// The [`IRViewFactorSet`] for the front side of each fenestration
    pub front_fenestrations_view_factors: Vec<IRViewFactorSet>,

    /// The [`IRViewFactorSet`] for the back side of each fenestration
    pub back_fenestrations_view_factors: Vec<IRViewFactorSet>,

    /// The Daylight Coefficients matrix for the front-side of the  surfaces in the scene
    pub front_surfaces_dc: Matrix,

    /// The Daylight Coefficients matrix for the back-side of the  surfaces in the scene
    pub back_surfaces_dc: Matrix,

    /// The Daylight Coefficients matrix for the front-side of the  fenestrations in the scene
    pub front_fenestrations_dc: Matrix,

    /// The Daylight Coefficients matrix for the back-side of the fenestrations in the scene
    pub back_fenestrations_dc: Matrix,
}

impl OpticalInfo {
    /// Calculates the new OpticalInformation.
    ///
    /// This will trigger ray-tracing processes, so it might be slow.
    pub fn new(
        options: &SolarOptions,
        model: &Model,
        state: &mut SimulationStateHeader,
    ) -> Result<Self, String> {
        // Collect calculation options
        let mf = options.solar_sky_discretization_or(crate::solar_model::MODULE_NAME, 1);
        let n_solar_rays =
            options.n_solar_irradiance_points_or(crate::solar_model::MODULE_NAME, 10);

        // Create Surfaces and Fenestrations
        let surfaces = SolarSurface::make_surfaces(&model.surfaces, state, n_solar_rays)?;
        let fenestrations =
            SolarSurface::make_fenestrations(&model.fenestrations, state, n_solar_rays)?;

        // build scene
        let mut solar_scene = Scene::from_simple_model(model, Wavelengths::Solar)?;
        solar_scene.build_accelerator();

        // calculator
        let solar_dc_factory = DCFactory {
            max_depth: 0,
            n_ambient_samples: options
                .solar_ambient_divitions_or(crate::solar_model::MODULE_NAME, 300),
            reinhart: ReinhartSky::new(mf),
            ..DCFactory::default()
        };

        // calculate
        let front_surfaces_dc =
            SolarSurface::calc_solar_dc_matrix(&surfaces, &solar_scene, &solar_dc_factory, true)?;

        let back_surfaces_dc =
            SolarSurface::calc_solar_dc_matrix(&surfaces, &solar_scene, &solar_dc_factory, false)?;

        let front_fenestrations_dc = SolarSurface::calc_solar_dc_matrix(
            &fenestrations,
            &solar_scene,
            &solar_dc_factory,
            true,
        )?;

        let back_fenestrations_dc = SolarSurface::calc_solar_dc_matrix(
            &fenestrations,
            &solar_scene,
            &solar_dc_factory,
            false,
        )?;

        let mut front_surfaces_view_factors = Vec::with_capacity(surfaces.len());
        for s in surfaces.iter() {
            front_surfaces_view_factors.push(s.calc_view_factors(&solar_scene, true)?)
        }
        let mut back_surfaces_view_factors = Vec::with_capacity(surfaces.len());
        for s in surfaces {
            back_surfaces_view_factors.push(s.calc_view_factors(&solar_scene, false)?)
        }
        let mut front_fenestrations_view_factors = Vec::with_capacity(fenestrations.len());
        for s in fenestrations.iter() {
            front_fenestrations_view_factors.push(s.calc_view_factors(&solar_scene, true)?);
        }
        let mut back_fenestrations_view_factors = Vec::with_capacity(fenestrations.len());
        for s in fenestrations.iter() {
            back_fenestrations_view_factors.push(s.calc_view_factors(&solar_scene, false)?)
        }

        Ok(Self {
            front_surfaces_view_factors,
            back_surfaces_view_factors,
            front_fenestrations_view_factors,
            back_fenestrations_view_factors,
            front_surfaces_dc,
            back_surfaces_dc,
            front_fenestrations_dc,
            back_fenestrations_dc,
        })
    }
}

#[cfg(test)]
mod testing {

    use json5;
    use model::{
        substance::Normal, Construction, Fenestration, Material, Model, SimulationStateHeader,
        SolarOptions, Surface,
    };

    use crate::OpticalInfo;

    #[test]
    fn test_new() -> Result<(), String> {
        // check that surfaces that do not receive sun are ignored
        let mut model = Model::default();
        let mut state = SimulationStateHeader::new();
        let mut options = SolarOptions::new();
        options.set_n_solar_irradiance_points(1);
        options.set_solar_ambient_divitions(1);
        options.set_solar_sky_discretization(1);

        let substance = Normal::new("the substance");
        model.add_substance(substance.wrap());

        let material = Material::new("the material", "the substance", 0.1);
        model.add_material(material);

        let mut construction = Construction::new("the construction");
        construction.materials.push("the material".into());
        model.add_construction(construction);

        let s: Surface = json5::from_str(
            "{
            name: 'the surface',
            construction:'the construction',
            vertices: [
                0, 0, 0, // X, Y and Z of Vertex 0
                1, 0, 0, // X, Y and Z of Vertex 1
                1, 1, 0, // X, Y and Z of Vertex 2
                0, 1, 0  // ...
            ]
         }",
        )
        .map_err(|e| e.to_string())?;
        model.add_surface(s);

        let s: Surface = json5::from_str(
            "{
            name: 'the surface 2',
            construction:'the construction',
            vertices: [
                0, 0, 0, // X, Y and Z of Vertex 0
                1, 0, 0, // X, Y and Z of Vertex 1
                1, 1, 0, // X, Y and Z of Vertex 2
                0, 1, 0  // ...
            ]
         }",
        )
        .map_err(|e| e.to_string())?;
        model.add_surface(s);

        let fen: Fenestration = json5::from_str(
            "{
            name: 'Window 1',
            construction: 'the construction',
            vertices: [
                0.548000,0,2.5000,  // X,Y,Z ==> Vertex 1 {m}
                0.548000,0,0.5000,  // X,Y,Z ==> Vertex 2 {m}
                5.548000,0,0.5000,  // X,Y,Z ==> Vertex 3 {m}
                5.548000,0,2.5000,   // X,Y,Z ==> Vertex 4 {m}
            ]
        }",
        )
        .map_err(|e| e.to_string())?;
        model.add_fenestration(fen)?;

        let fen: Fenestration = json5::from_str(
            "{
            name: 'Window 2',
            construction: 'the construction',
            vertices: [
                0.548000,0,2.5000,  // X,Y,Z ==> Vertex 1 {m}
                0.548000,0,0.5000,  // X,Y,Z ==> Vertex 2 {m}
                5.548000,0,0.5000,  // X,Y,Z ==> Vertex 3 {m}
                5.548000,0,2.5000,   // X,Y,Z ==> Vertex 4 {m}
            ]
        }",
        )
        .map_err(|e| e.to_string())?;
        model.add_fenestration(fen)?;

        let info = OpticalInfo::new(&options, &model, &mut state)?;
        assert_eq!(info.back_fenestrations_dc.size(), (2, 146)); // 2 fenestrations, 146 patches
        assert_eq!(info.front_fenestrations_dc.size(), (2, 146)); // 2 fenestrations, 146 patches
        assert_eq!(info.back_surfaces_dc.size(), (2, 146)); // 2 fenestrations, 146 patches
        assert_eq!(info.front_surfaces_dc.size(), (2, 146)); // 2 fenestrations, 146 patches
        assert_eq!(info.front_surfaces_view_factors.len(), 2);
        assert_eq!(info.back_surfaces_view_factors.len(), 2);
        assert_eq!(info.front_fenestrations_view_factors.len(), 2);
        assert_eq!(info.back_fenestrations_view_factors.len(), 2);

        Ok(())
    }

    #[test]
    fn test_ignore_no_sun() -> Result<(), String> {
        // check that surfaces that do not receive sun are ignored
        let mut model = Model::default();
        let mut state = SimulationStateHeader::new();
        let mut options = SolarOptions::new();
        options.set_n_solar_irradiance_points(1);
        options.set_solar_ambient_divitions(1);
        options.set_solar_sky_discretization(1);

        let substance = Normal::new("the substance");
        model.add_substance(substance.wrap());

        let material = Material::new("the material", "the substance", 0.1);
        model.add_material(material);

        let mut construction = Construction::new("the construction");
        construction.materials.push("the material".into());
        model.add_construction(construction);

        let s: Surface = json5::from_str(
            "{
            name: 'the surface',
            construction:'the construction',
            vertices: [
                0, 0, 0, // X, Y and Z of Vertex 0
                1, 0, 0, // X, Y and Z of Vertex 1
                1, 1, 0, // X, Y and Z of Vertex 2
                0, 1, 0  // ...
            ],
            front_boundary: {
                type: 'AmbientTemperature',
                temperature: 2.0,
            }
         }",
        )
        .map_err(|e| e.to_string())?;
        model.add_surface(s);

        let s: Surface = json5::from_str(
            "{
            name: 'the surface 2',
            construction:'the construction',
            vertices: [
                0, 0, 0, // X, Y and Z of Vertex 0
                1, 0, 0, // X, Y and Z of Vertex 1
                1, 1, 0, // X, Y and Z of Vertex 2
                0, 1, 0  // ...
            ]
         }",
        )
        .map_err(|e| e.to_string())?;
        model.add_surface(s);

        let fen: Fenestration = json5::from_str(
            "{
            name: 'Window 1',
            construction: 'the construction',
            vertices: [
                0.548000,0,2.5000,  // X,Y,Z ==> Vertex 1 {m}
                0.548000,0,0.5000,  // X,Y,Z ==> Vertex 2 {m}
                5.548000,0,0.5000,  // X,Y,Z ==> Vertex 3 {m}
                5.548000,0,2.5000,   // X,Y,Z ==> Vertex 4 {m}
            ]
        }",
        )
        .map_err(|e| e.to_string())?;
        model.add_fenestration(fen)?;

        let fen: Fenestration = json5::from_str(
            "{
            name: 'Window 2',
            construction: 'the construction',
            vertices: [
                0.548000,0,2.5000,  // X,Y,Z ==> Vertex 1 {m}
                0.548000,0,0.5000,  // X,Y,Z ==> Vertex 2 {m}
                5.548000,0,0.5000,  // X,Y,Z ==> Vertex 3 {m}
                5.548000,0,2.5000,   // X,Y,Z ==> Vertex 4 {m}
            ],
            back_boundary: {
                type: 'AmbientTemperature',
                temperature: 2.0
            }
        }",
        )
        .map_err(|e| e.to_string())?;
        model.add_fenestration(fen)?;

        let info = OpticalInfo::new(&options, &model, &mut state)?;
        assert_eq!(info.back_fenestrations_dc.size(), (1, 146)); // 1 fenestration has no solar radiation at the back
        assert_eq!(info.front_fenestrations_dc.size(), (2, 146)); // 2 fenestrations, 146 patches
        assert_eq!(info.back_surfaces_dc.size(), (2, 146)); // 2 fenestrations, 146 patches
        assert_eq!(info.front_surfaces_dc.size(), (1, 146)); // // 1 surface has no solar radiation at the front
        assert_eq!(info.front_surfaces_view_factors.len(), 2);
        assert_eq!(info.back_surfaces_view_factors.len(), 2);
        assert_eq!(info.front_fenestrations_view_factors.len(), 2);
        assert_eq!(info.back_fenestrations_view_factors.len(), 2);
        Ok(())
    }
}
