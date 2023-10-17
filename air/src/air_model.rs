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

// use crate::Float;
use crate::resolvers::*;
use calendar::Date;
use communication::{ErrorHandling, MetaOptions, SimulationModel};
use model::{Infiltration, Model, SimulationState, SimulationStateElement, SimulationStateHeader, SiteDetails};
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

    fn allocate_memory(&self) -> Result<Self::AllocType, String> {
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
            Some(d)=> d.clone(),
            None => SiteDetails::default()
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

#[cfg(test)]
mod tests {
    use super::*;
    use model::Space;
    use schedule::ScheduleConstant;
    use weather::SyntheticWeather;

    const META_OPTIONS: MetaOptions = MetaOptions {
        latitude: 0.,
        longitude: 0.,
        standard_meridian: 0.,
        elevation: 0.0,
    };

    #[test]
    fn test_infiltration() -> Result<(), String> {
        let mut model = Model::default();
        let mut state_header = SimulationStateHeader::new();

        let mut space = Space::new("some space".to_string());
        space.set_infiltration(Infiltration::Doe2 { flow: 1. });
        let i = state_header.push(SimulationStateElement::SpaceDryBulbTemperature(0), 22.)?;
        space.set_dry_bulb_temperature_index(i)?;
        let space = model.add_space(space);

        let air_model = AirFlowModel::new(&META_OPTIONS, (), &model, &mut state_header, 1)
            .expect("Could not build AirFlow model");
        let mut state = state_header
            .take_values()
            .expect("Could not take values form SimualationStateHeader");

        /*
        This test is essentially the same as in test_design_blast_flow_rate().

        "At a winter condition of 40◦C deltaT and 6 m/s
        (13.4 mph) windspeed, these coefficients would increase the infiltration
        rate by a factor of 2.75."
        */
        let space_temp = space
            .dry_bulb_temperature(&state)
            .expect("Could not get Dry BUlb Temp from space");
        let mut weather = SyntheticWeather::default();
        weather.dry_bulb_temperature = Box::new(ScheduleConstant::new(space_temp - 40.));
        weather.wind_speed = Box::new(ScheduleConstant::new(6.));

        let date = Date {
            month: 1,
            day: 1,
            hour: 10.,
        };

        // It should be initialized as Zero
        let inf = space
            .infiltration_volume(&state)
            .ok_or("No infiltration volume 1")?;
        assert!(inf < 1e-9);

        air_model.march(date, &weather, &model, &mut state, &mut ())?;

        // Check values.
        let inf = space
            .infiltration_volume(&state)
            .ok_or("No infiltration volume 2")?;
        assert!((1.34 - inf).abs() < 0.02);

        // ... A windspeed of 4.47 m/s (10 mph) gives a factor of 1.0.
        let mut weather = SyntheticWeather::default();
        weather.dry_bulb_temperature = Box::new(ScheduleConstant::new(space_temp - 40.));
        weather.wind_speed = Box::new(ScheduleConstant::new(4.47));
        air_model.march(date, &weather, &model, &mut state, &mut ())?;

        // Check values.
        let inf = space
            .infiltration_volume(&state)
            .ok_or("No infiltration volume 3")?;
        assert!((1. - inf).abs() < 0.02);

        Ok(())
    }
}
