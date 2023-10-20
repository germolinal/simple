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

/// The kind of Floating point number used in the
/// library... the `"float"` feature means it becomes `f32`
/// and `f64` is used otherwise.
#[cfg(feature = "float")]
type Float = f32;

/// The kind of Floating point number used in the
/// library... the `"float"` feature means it becomes `f32`
/// and `f64` is used otherwise.
#[cfg(not(feature = "float"))]
type Float = f64;

use model::{Model, SimulationState, SimulationStateHeader, SolarOptions};

use air::air_model::{AirFlowModel, AirFlowModelMemory};
use calendar::Date;
use communication::{ErrorHandling, MetaOptions, SimulationModel};
use heat::heat_model::{ThermalModel, ThermalModelMemory};
use light::solar_model::{SolarModel, SolarModelMemory};
use std::borrow::Borrow;
use weather::WeatherTrait;
// use acoustic_model::model::AcousticModel;

/// The memory to be allocated for running the simulation
pub struct MultiphysicsModelMemory {
    /// the memory of the [`ThermalModel`]
    thermal: ThermalModelMemory,
    solar: SolarModelMemory,
    air: AirFlowModelMemory,
}

/// The structure that connects all the SIMPLE simulation modules.
///
/// It focuses specifically on physics. Its state is contained in the `SimulationState`,
/// which is modified by another object or script.
pub struct MultiphysicsModel {
    /// The timestep of this model in Seconds
    pub dt: Float,

    /// The number of timestep per hour
    pub dt_subdivisions: usize,

    /// The model representing heat transfer and heat gains.
    thermal_model: ThermalModel,
    air_flow_model: AirFlowModel,
    solar_model: SolarModel,
    // acoustic_model: AcousticModel,
}

impl ErrorHandling for MultiphysicsModel {
    fn module_name() -> &'static str {
        "Multiphysics Model"
    }
}

impl SimulationModel for MultiphysicsModel {
    type OutputType = Self;
    type AllocType = MultiphysicsModelMemory;
    type OptionType = ();

    fn allocate_memory(&self) -> Result<Self::AllocType, String> {
        let thermal = self.thermal_model.allocate_memory()?;
        #[allow(clippy::let_unit_value)]
        let solar = self.solar_model.allocate_memory()?;
        #[allow(clippy::let_unit_value)]
        let air = self.air_flow_model.allocate_memory()?;

        let ret = MultiphysicsModelMemory {
            thermal,
            solar,
            air,
        };

        Ok(ret)
    }

    /// Creates a new simulation model.
    fn new<M: Borrow<Model>>(
        meta_options: &MetaOptions,
        _options: (),
        model: M,
        state: &mut SimulationStateHeader,
        n: usize,
    ) -> Result<Self::OutputType, String> {
        let thermal_model = match ThermalModel::new(meta_options, (), model.borrow(), state, n) {
            Ok(v) => v,
            Err(e) => return MultiphysicsModel::user_error(e),
        };

        let air_flow_model = match AirFlowModel::new(meta_options, (), model.borrow(), state, n) {
            Ok(v) => v,
            Err(e) => return MultiphysicsModel::user_error(e),
        };

        // let acoustic_model = match AcousticModel::new(building, state, n){
        //     Ok(v)=>v,
        //     Err(e)=>return MultiphysicsModel::user_error(e),
        // };
        let solar_options = match &model.borrow().solar_options {
            Some(options) => options.clone(),
            None => {
                let mut opt = SolarOptions::new();

                opt.set_n_solar_irradiance_points(10)
                    .set_solar_ambient_divitions(300)
                    .set_solar_sky_discretization(1)
                    .set_solar_sky_discretization(1);

                opt
            }
        };

        let solar_model =
            match SolarModel::new(meta_options, solar_options, model.borrow(), state, n) {
                Ok(v) => v,
                Err(e) => return MultiphysicsModel::user_error(e),
            };

        Ok(Self {
            thermal_model,
            // acoustic_model,
            solar_model,
            air_flow_model,

            dt_subdivisions: n,
            dt: 60. * 60. / n as Float,
        })
    }

    fn march<W: WeatherTrait, M: Borrow<Model>>(
        &self,
        date: Date,
        weather: &W,
        model: M,
        state: &mut SimulationState,
        alloc: &mut MultiphysicsModelMemory,
    ) -> Result<(), String> {
        // First solar,
        self.solar_model
            .march(date, weather, model.borrow(), state, &mut alloc.solar)?;

        // Then noise
        // self.acoustic_model.march(date, weather, building, state)?;

        // Then air flow
        self.air_flow_model
            .march(date, weather, model.borrow(), state, &mut alloc.air)?;

        // Then temperature
        self.thermal_model
            .march(date, weather, model.borrow(), state, &mut alloc.thermal)?;

        Ok(())
    }
}

impl MultiphysicsModel {
    /// Retrieves the thermal model
    pub fn thermal_model(&self) -> &ThermalModel {
        &self.thermal_model
    }
}

/***********/
/* TESTING */
/***********/

#[cfg(test)]
mod testing {}
