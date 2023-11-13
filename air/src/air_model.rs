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

// use crate::Float;
use crate::resolvers::*;
use calendar::Date;
use communication::{ErrorHandling, MetaOptions, SimulationModel};
use model::{
    Infiltration, Model, SimulationState, SimulationStateElement, SimulationStateHeader,
    SiteDetails,
};
use std::borrow::Borrow;
use weather::{CurrentWeather, WeatherTrait};

pub type Resolver = Box<dyn Fn(&CurrentWeather, &mut SimulationState) -> Result<(), String>>;

pub struct AirFlowModel {
    infiltration_calcs: Vec<Resolver>,
}

impl ErrorHandling for AirFlowModel {
    fn module_name() -> &'static str {
        "Air-flow model"
    }
}

/// The memory needed to run this simulation
pub type AirFlowModelMemory = ();

impl SimulationModel for AirFlowModel {
    type OutputType = Self;
    type OptionType = ();
    type AllocType = AirFlowModelMemory;

    fn allocate_memory(&self, _state: &SimulationState) -> Result<Self::AllocType, String> {
        Ok(())
    }

    /// Creates a new AirFlowModel from a Model.    
    fn new<M: Borrow<Model>>(
        _meta_options: &MetaOptions,
        _options: (),
        model: M,
        state: &mut SimulationStateHeader,
        _n: usize,
    ) -> Result<Self, String> {
        let mut infiltration_calcs = Vec::with_capacity(model.borrow().spaces.len());

        let site_details = match &model.borrow().site_details {
            Some(d) => d.clone(),
            None => SiteDetails::default(),
        };

        for (i, space) in model.borrow().spaces.iter().enumerate() {
            // Should these initial values be different?
            let initial_vol = 0.0;
            let initial_temp = 0.0;
            let inf_vol_index = state.push(
                SimulationStateElement::SpaceInfiltrationVolume(i),
                initial_vol,
            )?;
            space.set_infiltration_volume_index(inf_vol_index)?;
            let inf_temp_index = state.push(
                SimulationStateElement::SpaceInfiltrationTemperature(i),
                initial_temp,
            )?;
            space.set_infiltration_temperature_index(inf_temp_index)?;

            // Pre-process infiltration calculations
            if let Ok(infiltration) = space.infiltration() {
                let infiltration_fn = match infiltration {
                    Infiltration::Constant { flow } => constant_resolver(space, *flow)?,
                    Infiltration::Blast { flow } => blast_resolver(space, &site_details, *flow)?,
                    Infiltration::Doe2 { flow } => doe2_resolver(space, &site_details, *flow)?,
                    Infiltration::DesignFlowRate { a, b, c, d, phi } => {
                        design_flow_rate_resolver(space, &site_details, *a, *b, *c, *d, *phi)?
                    }
                    Infiltration::EffectiveAirLeakageArea { area } => {
                        effective_air_leakage_resolver(space, model.borrow(), *area)?
                    }
                };
                infiltration_calcs.push(infiltration_fn);
            } else {
                // Does nothing
                infiltration_calcs.push(Box::new(
                    move |_current_weather: &CurrentWeather,
                          _state: &mut SimulationState|
                          -> Result<(), String> { Ok(()) },
                ));
            }
        }

        Ok(AirFlowModel { infiltration_calcs })
    }

    /// Advances one main_timestep through time. That is,
    /// it performs `self.dt_subdivisions` steps, advancing
    /// `self.dt` seconds in each of them.
    fn march<W: WeatherTrait, M: Borrow<Model>>(
        &self,
        date: Date,
        weather: &W,
        _model: M,
        state: &mut SimulationState,
        _alloc: &mut AirFlowModelMemory,
    ) -> Result<(), String> {
        // Process infiltration
        let current_weather = weather.get_weather_data(date);
        for func in self.infiltration_calcs.iter() {
            func(&current_weather, state)?;
        }

        Ok(())
    }
}
