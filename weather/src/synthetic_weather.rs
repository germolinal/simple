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
use crate::Float;
use calendar::Date;
use schedule::EmptySchedule;
use schedule::Schedule;

use crate::current_weather::CurrentWeather;
use crate::Weather;

/// A Factory of CurrentWeather objects.
/// Each element is a Schedule that produces
/// the data.
pub struct SyntheticWeather {
    /// A schedule producing the drybulb temperature
    /// in C (in Float format)
    pub dry_bulb_temperature: Box<dyn Schedule<Float>>,

    /// A schedule producing the dew point temperature
    /// in C (in Float format)
    pub dew_point_temperature: Box<dyn Schedule<Float>>,

    /// A schedule producing the global horizontal radiation
    /// in Wh/m2 (in Float format)
    pub global_horizontal_radiation: Box<dyn Schedule<Float>>,

    /// A schedule producing the direct normal horizontal radiation
    /// in Wh/m2 (in Float format)
    pub direct_normal_radiation: Box<dyn Schedule<Float>>,

    /// A schedule producing the direct diffuse horizontal radiation
    /// in Wh/m2 (in Float format)
    pub diffuse_horizontal_radiation: Box<dyn Schedule<Float>>,

    /// A schedule producing the drybulb temperature
    /// in C (in Float format)
    pub wind_speed: Box<dyn Schedule<Float>>,

    /// Wind Direction in degrees
    ///
    /// From EnergyPlus documentation:
    /// > The convention is that North=0.0, East=90.0, South=180.0, West=270.0. (Wind direction in degrees at the time indicated. If calm, direction equals zero.) Values can range from 0 to 360
    pub wind_direction: Box<dyn Schedule<Float>>,

    /// Horizontal IR Radiation in Wh/m2
    pub horizontal_infrared_radiation_intensity: Box<dyn Schedule<Float>>,

    /// The opaque sky cover
    pub opaque_sky_cover: Box<dyn Schedule<Float>>,
}

impl std::default::Default for SyntheticWeather {
    fn default() -> Self {
        Self {
            dry_bulb_temperature: Box::new(EmptySchedule),
            dew_point_temperature: Box::new(EmptySchedule),
            global_horizontal_radiation: Box::new(EmptySchedule),
            direct_normal_radiation: Box::new(EmptySchedule),
            diffuse_horizontal_radiation: Box::new(EmptySchedule),
            wind_speed: Box::new(EmptySchedule),
            wind_direction: Box::new(EmptySchedule),
            horizontal_infrared_radiation_intensity: Box::new(EmptySchedule),
            opaque_sky_cover: Box::new(EmptySchedule),
        }
    }
}

impl Weather for SyntheticWeather {
    fn get_weather_data(&self, date: Date) -> CurrentWeather {
        CurrentWeather {
            dry_bulb_temperature: self.dry_bulb_temperature.get(date),
            dew_point_temperature: self.dew_point_temperature.get(date),
            global_horizontal_radiation: self.global_horizontal_radiation.get(date),
            direct_normal_radiation: self.direct_normal_radiation.get(date),
            diffuse_horizontal_radiation: self.diffuse_horizontal_radiation.get(date),
            wind_speed: self.wind_speed.get(date),
            wind_direction: self.wind_direction.get(date),
            horizontal_infrared_radiation_intensity: self
                .horizontal_infrared_radiation_intensity
                .get(date),
            opaque_sky_cover: self.opaque_sky_cover.get(date),
        }
    }
}
