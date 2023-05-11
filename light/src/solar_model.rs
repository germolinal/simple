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
use crate::{solar_surface::SolarSurface, Float};
use calendar::Date;
use communication::{ErrorHandling, MetaOptions, SimulationModel};
use matrix::Matrix;
use model::{Boundary, Model, SimulationState, SimulationStateHeader, SolarOptions, print_warning};
use std::borrow::Borrow;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use weather::{CurrentWeather, WeatherTrait};
use weather::{PerezSky, SkyUnits, Solar};

use crate::optical_info::OpticalInfo;

/// The name of the module
pub(crate) const MODULE_NAME: &str = "Solar Model";

/// The memory used by this module during simulation
pub type SolarModelMemory = ();

/// The main model
pub struct SolarModel {
    // /// The scene that makes up this model from a lighting point of view.
    // lighting_scene: Scene,

    // Workplanes
    /// The scene that makes up this model from a radiation point of view.
    // solar_scene: Scene,

    // surfaces: Vec<SolarSurface>,

    /// The optical information from the model, containing
    /// DC matrices and view factors
    optical_info: OpticalInfo,

    /// The calculator for solar position and other solar variables
    solar: Solar,

    /// The MF discretization scheme for the sky.
    solar_sky_discretization: usize,
}

impl SolarModel {
    /// This function makes the IR heat transfer Zero... we will try to fix this soon enough,
    /// just not now    
    fn update_ir_radiation(
        &self,
        weather_data: &CurrentWeather,
        model: &Model,
        state: &mut SimulationState,
    ) -> Result<(), String> {
        pub const SIGMA: crate::Float = 5.670374419e-8;

        fn ir(temp: Float, emissivity: Float) -> Float {
            emissivity * SIGMA * (temp + 273.15).powi(4)
        }

        let db = weather_data.dry_bulb_temperature;
        let horizontal_ir = match weather_data.horizontal_infrared_radiation_intensity {
            Some(v) => v,
            None => weather_data.derive_horizontal_ir(),
        };

        let iter = model.surfaces.iter().enumerate();

        for (index, surface) in iter {
            // Deal with front
            match &surface.front_boundary {
                Boundary::Space { .. } => {
                    // Zero net IR exchange
                    let temp = surface.first_node_temperature(state).unwrap_or(22.);
                    surface.set_front_ir_irradiance(state, ir(temp, 1.0))?;
                }
                Boundary::AmbientTemperature { temperature } => {
                    // It depends on the ambient tempearture
                    surface.set_front_ir_irradiance(state, ir(*temperature, 1.0))?;
                }
                Boundary::Ground | Boundary::Adiabatic => {
                    // ignore ground and adiabatic
                }
                Boundary::Outdoor => {
                    // outdoor
                    let view_factors = &self.optical_info.front_surfaces_view_factors[index];
                    let ground_other = (view_factors.ground + view_factors.air) * ir(db, 1.0);
                    let sky = view_factors.sky * horizontal_ir;
                    surface.set_front_ir_irradiance(state, ground_other + sky)?;
                }
            }

            // Deal with Back
            match &surface.back_boundary {
                Boundary::Space { .. } => {
                    // Zero net IR exchange
                    let temp = surface.last_node_temperature(state).unwrap_or(22.);
                    surface.set_back_ir_irradiance(state, ir(temp, 1.0))?;
                }
                Boundary::AmbientTemperature { temperature } => {
                    surface.set_back_ir_irradiance(state, ir(*temperature, 1.0))?;
                }
                Boundary::Ground | Boundary::Adiabatic => {
                    // ignore ground and adiabatic
                }
                Boundary::Outdoor => {
                    // outdoor
                    let view_factors = &self.optical_info.back_surfaces_view_factors[index];
                    let ground_other = (view_factors.ground + view_factors.air) * ir(db, 1.0);
                    let sky = view_factors.sky * horizontal_ir;
                    surface.set_back_ir_irradiance(state, ground_other + sky)?;
                }
            }
        }

        let iter = model.fenestrations.iter().enumerate();
        for (index, surface) in iter {
            // Deal with front
            match &surface.front_boundary {
                Boundary::Space { .. } => {
                    // Zero net IR exchange
                    let temp = surface.first_node_temperature(state).unwrap_or(22.);
                    surface.set_front_ir_irradiance(state, ir(temp, 1.0))?;
                }
                Boundary::AmbientTemperature { temperature } => {
                    surface.set_front_ir_irradiance(state, ir(*temperature, 1.0))?;
                }
                Boundary::Ground | Boundary::Adiabatic => {}
                Boundary::Outdoor => {
                    let view_factors = &self.optical_info.front_fenestrations_view_factors[index];
                    let ground_other = (view_factors.ground + view_factors.air) * ir(db, 1.0);
                    let sky = view_factors.sky * horizontal_ir;
                    surface.set_front_ir_irradiance(state, ground_other + sky)?;
                }
            }

            // Deal with Back
            match &surface.back_boundary {
                Boundary::Space { .. } => {
                    // Zero net IR exchange
                    let temp = surface.last_node_temperature(state).unwrap_or(22.);
                    surface.set_back_ir_irradiance(state, ir(temp, 1.0))?;
                }
                Boundary::AmbientTemperature { temperature } => {
                    surface.set_back_ir_irradiance(state, ir(*temperature, 1.0))?;
                }
                Boundary::Ground | Boundary::Adiabatic => {}
                Boundary::Outdoor => {
                    // outdoor
                    let view_factors = &self.optical_info.back_fenestrations_view_factors[index];
                    let ground_other = (view_factors.ground + view_factors.air) * ir(db, 1.0);
                    let sky = view_factors.sky * horizontal_ir;
                    surface.set_back_ir_irradiance(state, ground_other + sky)?;
                }
            }
        }

        Ok(())
    }

    fn update_solar_radiation(
        &self,
        date: Date,
        weather_data: CurrentWeather,
        model: &Model,
        state: &mut SimulationState,
    ) -> Result<(), String> {
        let direct_normal_irrad = weather_data.direct_normal_radiation;
        let diffuse_horizontal_irrad = weather_data.diffuse_horizontal_radiation;

        let is_day = direct_normal_irrad + diffuse_horizontal_irrad >= 1e-4;
        let vec = if is_day {
            // Build sky vector
            let albedo = 0.2;
            let add_sky = true;
            let add_sun = true;
            let units = SkyUnits::Solar;
            PerezSky::gen_sky_vec(
                self.solar_sky_discretization,
                &self.solar,
                date,
                weather_data,
                units,
                albedo,
                add_sky,
                add_sun,
            )?
        } else {
            Matrix::empty()
        };

        // Process Solar Irradiance in Surfaces
        if !self.optical_info.front_surfaces_dc.is_empty() {
            if is_day {
                let solar_irradiance = &self.optical_info.front_surfaces_dc * &vec;
                let mut i = 0;
                for s in model.surfaces.iter() {
                    if !SolarSurface::boundary_receives_sun(&s.front_boundary) {
                        continue;
                    }
                    // Average of the period
                    let mut v = solar_irradiance.get(i, 0)?;
                    if v < 0.0 {
                        v = 0.0
                    }
                    let old_v = s.front_incident_solar_irradiance(state).ok_or(
                        "Could not get previous front incident solar irradiance (surface)",
                    )?;
                    s.set_front_incident_solar_irradiance(state, (v + old_v) / 2.)?;
                    i += 1;
                }
            } else {
                for s in model.surfaces.iter() {
                    s.set_front_incident_solar_irradiance(state, 0.0)?;
                }
            }
        }
        if !self.optical_info.back_surfaces_dc.is_empty() {
            if is_day {
                let solar_irradiance = &self.optical_info.back_surfaces_dc * &vec;
                let mut i = 0;
                for s in model.surfaces.iter() {
                    if !SolarSurface::boundary_receives_sun(&s.back_boundary) {
                        continue;
                    }
                    // Average of the period
                    let mut v = solar_irradiance.get(i, 0)?;
                    if v < 0.0 {
                        v = 0.0
                    }
                    let old_v = s
                        .back_incident_solar_irradiance(state)
                        .ok_or("Could not get previous back incident solar irradiance (surface)")?;
                    s.set_back_incident_solar_irradiance(state, (v + old_v) / 2.)?;
                    i += 1;
                }
            } else {
                for s in model.surfaces.iter() {
                    s.set_front_incident_solar_irradiance(state, 0.0)?;
                }
            }
        }

        // Process Solar Irradiance in Fenestration
        if !self.optical_info.front_fenestrations_dc.is_empty() {
            if is_day {
                let solar_irradiance = &self.optical_info.front_fenestrations_dc * &vec;
                let mut i = 0;
                for s in model.fenestrations.iter() {
                    if !SolarSurface::boundary_receives_sun(&s.front_boundary) {
                        continue;
                    }
                    // Average of the period
                    let v = solar_irradiance.get(i, 0)?;
                    let old_v = s.front_incident_solar_irradiance(state).ok_or(
                        "Could not get previous front incident solar irradiance (fenestration)",
                    )?;
                    s.set_front_incident_solar_irradiance(state, (v + old_v) / 2.)?;
                    i += 1;
                }
            } else {
                for s in model.fenestrations.iter() {
                    s.set_front_incident_solar_irradiance(state, 0.0)?;
                }
            }
        }
        if !self.optical_info.back_fenestrations_dc.is_empty() {
            if is_day {
                let solar_irradiance = &self.optical_info.back_fenestrations_dc * &vec;
                let mut i = 0;
                for s in model.fenestrations.iter() {
                    if !SolarSurface::boundary_receives_sun(&s.back_boundary) {
                        continue;
                    }
                    // Average of the period
                    let v = solar_irradiance.get(i, 0)?;
                    let old_v = s.back_incident_solar_irradiance(state).ok_or(
                        "Could not get previous front incident solar irradiance (fenestration)",
                    )?;
                    s.set_back_incident_solar_irradiance(state, (v + old_v) / 2.)?;
                    i += 1;
                }
            } else {
                for s in model.fenestrations.iter() {
                    s.set_front_incident_solar_irradiance(state, 0.0)?;
                }
            }
        }
        Ok(())
    }
}

impl ErrorHandling for SolarModel {
    fn module_name() -> &'static str {
        MODULE_NAME
    }
}

impl SimulationModel for SolarModel {
    type OutputType = Self;
    type OptionType = SolarOptions;
    type AllocType = SolarModelMemory;

    fn allocate_memory(&self) -> Result<Self::AllocType, String> {
        Ok(())
    }

    fn new<M: Borrow<Model>>(
        meta_options: &MetaOptions,
        options: SolarOptions,
        model: M,
        state: &mut SimulationStateHeader,
        _n: usize,
    ) -> Result<Self::OutputType, String> {
        let model = model.borrow();
        // Make OpticalInfo, or read, as needed
        let optical_info = if let Ok(path_str) = options.optical_data_path() {
            let path = Path::new(path_str);
            if path.exists() {
                // read from file
                if !path.is_file() {
                    return Err(format!(
                        "Path '{}' is not a file",
                        path.to_str()
                            .expect("When !path.is_file... could not convert path into string")
                    ));
                }

                let data = match std::fs::read_to_string(path) {
                    Ok(v) => v,
                    Err(_) => {
                        return Err(format!("Unable to read optical_info file '{}'", path_str))
                    }
                };
                let info: OpticalInfo = match serde_json::from_str(&data) {
                    Ok(v) => v,
                    Err(_) => {
                        return Err(format!(
                            "Unable to patse optical_info object in file '{}'",
                            path_str
                        ))
                    }
                };

                info
            } else {
                // write into file
                let info = OpticalInfo::new(&options, model, state)?;
                let s = match serde_json::to_value(&info) {
                    Ok(v) => v,
                    Err(e) => return Err(format!("{}", e)),
                };
                let mut file = match File::create(path) {
                    Ok(v) => v,
                    Err(e) => return Err(format!("{}", e)),
                };
                if let Err(e) = writeln!(&mut file, "{}", s) {
                    return Err(format!("{}", e));
                }
                info
            }
        } else {
            // Forced calculation... not store
            OpticalInfo::new(&options, model, state)?
        };

        
        // Create the Solar object
        let latitude = meta_options.latitude;
        let longitude = -meta_options.longitude;
        let standard_meridian = -meta_options.standard_meridian;
        let solar = Solar::new(latitude, longitude, standard_meridian);

        if model.surfaces.is_empty(){
            print_warning(MODULE_NAME, "Model has no surfaces");
            return Ok(Self {
                optical_info,
                solar,
                solar_sky_discretization: 1, // not really relevant
            })
        }

        // derive MF
        let (.., ncols) = optical_info.back_surfaces_dc.size();
        if ncols == 0 {
            return Err(
                "optical data is corrupt: daylight coefficient matrix has zero columns."
                    .to_string(),
            );
        }
        let mut mf = 1;
        loop {
            if mf >= 9 || ncols == 0 {
                return Err(format!("sky discretization seems to be too high ({mf}... If this is a bug, please report it!"));
            }
            if weather::ReinhartSky::n_bins(mf) == ncols {
                break;
            } else {
                mf += 1;
            }
        }

        Ok(Self {
            optical_info,
            solar,
            solar_sky_discretization: mf,
        })
    }

    fn march<W: WeatherTrait, M: Borrow<Model>>(
        &self,
        date: Date,
        weather: &W,
        model: M,
        state: &mut SimulationState,
        _alloc: &mut SolarModelMemory,
    ) -> Result<(), String> {
        let model = model.borrow();
        // Handle the solar part

        let weather_data = weather.get_weather_data(date);

        self.update_ir_radiation(&weather_data, model, state)?;
        self.update_solar_radiation(date, weather_data, model, state)?;

        Ok(())
    }
}

#[cfg(test)]
mod testing {
    use super::*;
    use model::{substance::Normal, Construction, Fenestration, Material, Surface};
    use schedule::ScheduleConstant;
    use weather::SyntheticWeather;

    #[test]
    fn test_model_mf() {
        // cleanup
        let optical_data_path = "./tests/wall/optical_data.json";
        let path = Path::new(optical_data_path);
        if path.exists() {
            std::fs::remove_file(path).unwrap();
        }

        // Step 1: create the model, write the optical info data.
        let meta_options = MetaOptions {
            latitude: -33.,
            longitude: 72.,
            standard_meridian: 70.,
            elevation: 0.0,
        };
        let (model, mut state_header) = Model::from_file("./tests/wall/wall.spl").unwrap();
        let mut solar_options = model.solar_options.clone().unwrap();
        solar_options.set_optical_data_path(optical_data_path.to_string());

        let light_model =
            SolarModel::new(&meta_options, solar_options, &model, &mut state_header, 4).unwrap();
        assert_eq!(light_model.solar_sky_discretization, 1); //this comes in the model

        // Step 2: Run it again, with a different option
        let meta_options = MetaOptions {
            latitude: -33.,
            longitude: 72.,
            standard_meridian: 70.,
            elevation: 0.0,
        };
        let mut solar_options = model.solar_options.clone().unwrap();
        solar_options.set_solar_sky_discretization(2);
        solar_options.set_optical_data_path(optical_data_path.to_string());
        let light_model =
            SolarModel::new(&meta_options, solar_options, &model, &mut state_header, 4).unwrap();
        assert_eq!(light_model.solar_sky_discretization, 1); //this comes from the optical data.

        // cleanup
        let path = Path::new(optical_data_path);
        if path.exists() {
            std::fs::remove_file(path).unwrap();
        }
    }

    #[test]
    fn test_skip_ambient_boundary() {
        // check that surfaces that do not receive sun are ignored
        let mut model = Model::default();

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
                type : 'AmbientTemperature',
                temperature: 2.0
            }
         }",
        )
        .unwrap();
        model.add_surface(s);

        let s: Surface = json5::from_str(
            "{
            name: 'the surface 2',
            construction:'the construction',
            vertices: [
                0, 0, 10, // X, Y and Z of Vertex 0
                1, 0, 10, // X, Y and Z of Vertex 1
                1, 1, 10, // X, Y and Z of Vertex 2
                0, 1, 10  // ...
            ]
         }",
        )
        .unwrap();
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
        .unwrap();
        model.add_fenestration(fen).unwrap();

        let fen: Fenestration = json5::from_str(
            "{
            name: 'Window 2',
            construction: 'the construction',
            vertices: [
                0.548000,10,2.5000,  // X,Y,Z ==> Vertex 1 {m}
                0.548000,10,0.5000,  // X,Y,Z ==> Vertex 2 {m}
                5.548000,10,0.5000,  // X,Y,Z ==> Vertex 3 {m}
                5.548000,10,2.5000,   // X,Y,Z ==> Vertex 4 {m}
            ],
            back_boundary: {
                type:'AmbientTemperature',
                temperature: 2.0
            }
        }",
        )
        .unwrap();

        model.add_fenestration(fen).unwrap();

        let meta_options = MetaOptions {
            latitude: (-41.3 as Float).to_radians(),
            longitude: (174.78 as Float).to_radians(),
            standard_meridian: (180. as Float).to_radians(),
            elevation: 0.0,
        };

        let mut state_header = SimulationStateHeader::new();
        let mut options = SolarOptions::new();
        options.set_n_solar_irradiance_points(10);
        options.set_solar_ambient_divitions(1000);
        options.set_solar_sky_discretization(1);

        let n: usize = 1;

        let solar_model =
            SolarModel::new(&meta_options, options, &model, &mut state_header, n).unwrap();

        let mut weather = SyntheticWeather::default();
        weather.dew_point_temperature = Box::new(ScheduleConstant::new(11.));
        weather.dry_bulb_temperature = Box::new(ScheduleConstant::new(24.));
        weather.opaque_sky_cover = Box::new(ScheduleConstant::new(0.));
        weather.direct_normal_radiation = Box::new(ScheduleConstant::new(400.));
        weather.diffuse_horizontal_radiation = Box::new(ScheduleConstant::new(200.));

        let mut state = state_header.take_values().unwrap();
        solar_model
            .march(
                Date {
                    month: 1,
                    day: 1,
                    hour: 12.,
                },
                &weather,
                &model,
                &mut state,
                &mut (),
            )
            .unwrap();

        // This surface should receive NO sun at the front but yes at the back
        let v = model.surfaces[0]
            .front_incident_solar_irradiance(&state)
            .unwrap()
            .abs();
        assert!(v < 1e-9, "v = {}", v);

        let v = model.surfaces[0]
            .back_incident_solar_irradiance(&state)
            .unwrap();
        assert!(v > 20., "v={}", v);

        // This surface should receive sun on both sides
        let v = model.surfaces[1]
            .front_incident_solar_irradiance(&state)
            .unwrap()
            .abs();
        assert!(v > 20., "v={}", v);

        let v = model.surfaces[1]
            .back_incident_solar_irradiance(&state)
            .unwrap();
        assert!(v > 20., "v={}", v);

        // This surface should receive sun on both sides
        let v = model.fenestrations[0]
            .front_incident_solar_irradiance(&state)
            .unwrap();
        assert!(v > 20., "v = {}", v);

        let v = model.fenestrations[0]
            .back_incident_solar_irradiance(&state)
            .unwrap();
        assert!(v > 20., "v = {}", v);

        // This surface should receive NO sun at the back but yes at the front
        let v = model.fenestrations[1]
            .front_incident_solar_irradiance(&state)
            .unwrap();
        assert!(v > 20., "v = {}", v);

        let v = model.fenestrations[1]
            .back_incident_solar_irradiance(&state)
            .unwrap()
            .abs();
        assert!(v < 1e-9, "v = {}", v);
    }
}
