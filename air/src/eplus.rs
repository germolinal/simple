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
use model::{SimulationState, Space};
use weather::CurrentWeather;

/// Calculates an infiltration rate equal to that estimated by
/// EnergyPlus' `ZoneInfiltration:DesignFlowRate`.
///
/// The equation is $`\phi = \phi_{design} (A + B|T_{space} - T_{outside}| + C\times W_{speed} + D\times W^2_{speed})`$
#[allow(clippy::too_many_arguments)]
pub fn design_flow_rate(
    weather: &CurrentWeather,
    space: &Arc<Space>,
    state: &SimulationState,
    design_rate: Float,
    a: Float,
    b: Float,
    c: Float,
    d: Float,
    wind_speed_modifier: Float,
) -> Float {
    let t_space = space
        .dry_bulb_temperature(state)
        .expect("Space does not have Dry Bulb temperature");
    let t_out = weather.dry_bulb_temperature;
    let wind_speed = weather.wind_speed * wind_speed_modifier;

    design_rate * (a + b * (t_space - t_out).abs() + c * wind_speed + d * wind_speed * wind_speed)
}

/// Calculates the design flow rates using the BLAST defaults (reported in EnergyPlus' Input/Output reference)
pub fn blast_design_flow_rate(
    weather: &CurrentWeather,
    space: &Arc<Space>,
    state: &SimulationState,
    design_rate: Float,
    wind_speed_modifier: Float,
) -> Float {
    design_flow_rate(
        weather,
        space,
        state,
        design_rate,
        0.606,
        0.03636,
        0.1177,
        0.,
        wind_speed_modifier,
    )
}

/// Calculates the design flow rates using the DOE-2 defaults (reported in EnergyPlus' Input/Output reference)
pub fn doe2_design_flow_rate(
    weather: &CurrentWeather,
    space: &Arc<Space>,
    state: &SimulationState,
    design_rate: Float,
    wind_speed_modifier: Float,
) -> Float {
    design_flow_rate(
        weather,
        space,
        state,
        design_rate,
        0.,
        0.,
        0.224,
        0.,
        wind_speed_modifier,
    )
}

/// Calculates the infiltration flow in m3/s. Area should be defined in m2
pub fn effective_leakage_area(
    weather: &CurrentWeather,
    space: &Arc<Space>,
    state: &SimulationState,
    area: Float,
    cw: Float,
    cs: Float,
) -> Float {
    let outdoor_temp = weather.dry_bulb_temperature;
    let space_temp = space
        .dry_bulb_temperature(state)
        .expect("Space has no Dry-bulb temperature");
    let delta_t = (outdoor_temp - space_temp).abs();
    let ws = weather.wind_speed;

    let aux = cs * delta_t + cw * ws * ws;
    let aux = aux.sqrt();
    area * 10.0 * aux // m3/s
}

#[cfg(test)]
mod tests {

    use super::*;
    use calendar::Date;
    use schedule::ScheduleConstant;
    use weather::SyntheticWeather;
    use weather::WeatherTrait;

    #[test]
    fn test_design_blast_flow_rate() -> Result<(), String> {
        /* THIS COMES FROM ENERGY PLUS' INPUT OUTPUT REF */
        /*
            "These coefficients produce a value of 1.0 at 0◦C deltaT and
            3.35 m/s (7.5 mph) windspeed, which corresponds to a typical
            summer condition.

            At a winter condition of 40◦C deltaT and 6 m/s
            (13.4 mph) windspeed, these coefficients would increase the infiltration
            rate by a factor of 2.75."
        */

        // Summer
        let mut weather = SyntheticWeather::default();
        // 0 C of temperature difference
        weather.dry_bulb_temperature = Box::new(ScheduleConstant::new(2.));
        let state = vec![2.];
        //
        weather.wind_speed = Box::new(ScheduleConstant::new(3.35));

        let space = Space::new("some space".to_string());
        space.set_dry_bulb_temperature_index(0)?;
        let space = Arc::new(space);

        let date = Date {
            month: 1,
            day: 1,
            hour: 1.,
        };

        let design_rate = 1.;
        let current_weather = weather.get_weather_data(date);
        let flow = blast_design_flow_rate(&current_weather, &space, &state, design_rate, 1.0);
        assert!((1. - flow).abs() < 0.02);

        // WINTER
        let mut weather = SyntheticWeather::default();
        // 40 C of temperature difference
        weather.dry_bulb_temperature = Box::new(ScheduleConstant::new(-38.));
        let state = vec![2.];
        //
        weather.wind_speed = Box::new(ScheduleConstant::new(6.));

        let space = Space::new("some space".to_string());
        space.set_dry_bulb_temperature_index(0)?;
        let space = Arc::new(space);

        let date = Date {
            month: 1,
            day: 1,
            hour: 1.,
        };

        let design_rate = 1.;
        let current_weather = weather.get_weather_data(date);
        let flow = blast_design_flow_rate(&current_weather, &space, &state, design_rate, 1.0);
        assert!((2.75 - flow).abs() < 0.02);

        Ok(())
    }

    #[test]
    fn test_design_doe2_flow_rate() -> Result<(), String> {
        /* THIS COMES FROM ENERGY PLUS' INPUT OUTPUT REF */
        /*
            "With these coefficients, the summer conditions above would
            give a factor of 0.75, and the winter conditions would give 1.34.
            A windspeed of 4.47 m/s (10 mph) gives a factor of 1.0.
        */

        // Summer
        let mut weather = SyntheticWeather::default();
        // 0 C of temperature difference
        weather.dry_bulb_temperature = Box::new(ScheduleConstant::new(2.));
        let state = vec![2.];
        //
        weather.wind_speed = Box::new(ScheduleConstant::new(3.35));

        let space = Space::new("some space".to_string());
        space.set_dry_bulb_temperature_index(0)?;
        let space = Arc::new(space);

        let date = Date {
            month: 1,
            day: 1,
            hour: 1.,
        };

        let design_rate = 1.;
        let current_weather = weather.get_weather_data(date);
        let flow = doe2_design_flow_rate(&current_weather, &space, &state, design_rate, 1.0);
        assert!((0.75 - flow).abs() < 0.02);

        // WINTER
        let mut weather = SyntheticWeather::default();
        // 40 C of temperature difference
        weather.dry_bulb_temperature = Box::new(ScheduleConstant::new(42.));
        let state = vec![2.];
        //
        weather.wind_speed = Box::new(ScheduleConstant::new(6.));

        let space = Space::new("some space".to_string());
        space.set_dry_bulb_temperature_index(0)?;
        let space = Arc::new(space);

        let date = Date {
            month: 1,
            day: 1,
            hour: 1.,
        };

        let design_rate = 1.;
        let current_weather = weather.get_weather_data(date);
        let flow = doe2_design_flow_rate(&current_weather, &space, &state, design_rate, 1.0);
        assert!((1.34 - flow).abs() < 0.02);

        // ... A windspeed of 4.47 m/s (10 mph) gives a factor of 1.0.
        let mut weather = SyntheticWeather::default();
        // 40 C of temperature difference
        weather.dry_bulb_temperature = Box::new(ScheduleConstant::new(42.));
        let state = vec![2.];
        //
        weather.wind_speed = Box::new(ScheduleConstant::new(4.47));

        let space = Space::new("some space".to_string());
        space.set_dry_bulb_temperature_index(0)?;
        let space = Arc::new(space);

        let date = Date {
            month: 1,
            day: 1,
            hour: 1.,
        };

        let design_rate = 1.;
        let current_weather = weather.get_weather_data(date);
        let flow = doe2_design_flow_rate(&current_weather, &space, &state, design_rate, 1.0);
        assert!((1. - flow).abs() < 0.02);
        Ok(())
    }

    #[test]
    fn test_effective_leakage_area() -> Result<(), String> {
        /* FROM ASHRAE FUNDAMENTALS 2001 - Chapter 26 */
        /*
        Estimate the infiltration at design conditions for a two-storey
        house in Lincoln, Nebraska. The house has an effective leakage area of
        500cm2 and a volume of 340m4, and the predominant wind is operpendicular
        to the street (shelter class 3). The indoor temp is 20C.

        > Assume outside is -19C; and wind speed is 6.7m/s
         */

        let t_out = -19.0;
        let wind_speed = 6.7;
        let t_in = 20.0;

        let mut weather = SyntheticWeather::default();
        weather.dry_bulb_temperature = Box::new(ScheduleConstant::new(t_out));
        let state = vec![t_in];
        //
        weather.wind_speed = Box::new(ScheduleConstant::new(wind_speed));

        let space = Space::new("some space".to_string());
        space.set_dry_bulb_temperature_index(0)?;
        let space = Arc::new(space);

        let date = Date {
            month: 1,
            day: 1,
            hour: 1.,
        };

        let current_weather = weather.get_weather_data(date);

        let a_cm2 = 500.0;
        let a_m2 = a_cm2 / 10000.0;

        assert!((a_m2 - 0.05 as f64).abs() < 1e-8);

        // for 1 storey
        let cs = 0.000290;
        let cw = 0.000231;

        let found = effective_leakage_area(&current_weather, &space, &state, a_m2, cw, cs);

        let exp = 0.0736;
        assert!(
            (exp - found).abs() < 1e-3,
            "Expecting {}... found {}",
            exp,
            found
        );

        Ok(())
    }
}
