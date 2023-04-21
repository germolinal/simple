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
use serde::{Serialize,Deserialize};

/// Contains all the information in an EPW Weather line
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct EPWWeatherLine {
    /// Element N1 in the EPW dictionary
    pub year: usize,

    /// Element N2 in the EPW dictionary
    pub month: u8,

    /// Element N3 in the EPW dictionary
    pub day: u8,

    /// Element N4 in the EPW dictionary
    pub hour: Float,

    /// Element N4 in the EPW dictionary
    pub minute: u8,

    /// Element A1 in the EPW dictionary     
    pub uncertainty_flags: bool,

    /// Element N6 in the EPW dictionary
    /// in degrees C
    pub dry_bulb_temperature: Float,

    /// Element N7 in the EPW dictionary
    /// in degrees C
    pub dew_point_temperature: Float,

    /// Element N8 in the EPW dictionary
    /// in degrees C
    pub relative_humidity: Float,

    /// Element N9 in the EPW dictionary
    /// in Pa
    pub atmospheric_station_pressure: Float,

    /// Element N10 in the EPW dictionary
    /// in Wh/m2
    pub extraterrestrial_horizontal_radiation: Float,

    /// Element N11 in the EPW dictionary
    /// in Wh/m2
    pub extraterrestrial_direct_normal_radiation: Float,

    /// Element N12 in the EPW dictionary
    /// in Wh/m2
    pub horizontal_infrared_radiation_intensity: Float,

    /// Element N3 in the EPW dictionary
    /// in Wh/m2
    pub global_horizontal_radiation: Float,

    /// Element N14 in the EPW dictionary
    /// in Wh/m2
    pub direct_normal_radiation: Float,

    /// Element N15 in the EPW dictionary
    /// in Wh/m2
    pub diffuse_horizontal_radiation: Float,

    /// Element N16 in the EPW dictionary
    /// in lux
    pub global_horizontal_illuminance: Float,

    /// Element N17 in the EPW dictionary
    /// in lux
    pub direct_normal_illuminance: Float,

    /// Element N18 in the EPW dictionary
    /// in lux
    pub diffuse_horizontal_illuminance: Float,

    /// Element N19 in the EPW dictionary
    /// in Cd/m2
    pub zenith_luminance: Float,

    /// Element N20 in the EPW dictionary
    /// in degrees. (which is north?)
    pub wind_direction: Float,

    /// Element N21 in the EPW dictionary
    /// in m/s
    pub wind_speed: Float,

    /// Element N22 in the EPW dictionary    
    pub total_sky_cover: Float,

    /// Element N23 in the EPW dictionary
    /// EnergyPlus used this if IR Intensity is missing
    pub opaque_sky_cover: Float,

    /// Element N24 in the EPW dictionary
    /// in km
    pub visibility: Float,

    /// Element N25 in the EPW dictionary
    /// in m
    pub ceiling_height: Float,

    /// Element N26 in the EPW dictionary    
    pub present_weather_observation: Float,

    /// Element N27 in the EPW dictionary    
    pub present_weather_codes: Float,

    /// Element N28 in the EPW dictionary    
    /// in mm
    pub precipitable_water: Float,

    /// Element N29 in the EPW dictionary    
    /// in thousands
    pub aerosol_optical_depth: Float,

    /// Element N30 in the EPW dictionary    
    pub snow_depth: Float,

    /// Element N31 in the EPW dictionary    
    pub last_day_since_last_snowfall: usize,

    /// Element N32 in the EPW dictionary    
    pub albedo: Float,

    /// Element N33 in the EPW dictionary    
    /// in mm
    pub liquid_precipitation_depth: Float,

    /// Element N34 in the EPW dictionary    
    /// in hr
    pub liquid_precipitation_quantity: Float,
}

impl EPWWeatherLine {
    /// Gets the date corresponding to that
    pub fn date(&self) -> Date {
        Date {
            month: self.month,
            day: self.day,
            // We count hours from 0 to 23.9999, EPW files include
            // hours from 1 to 24.99
            hour: self.hour,// - 1.0,
        }
    }

    /// Interpolates the data between to WeatherLines
    pub fn interpolate(&self, other: &Self, x: Float) -> Self {
        let interp = |a, b| a + x * (b - a);

        let self_date = Date {
            month: self.month,
            day: self.day,
            hour: self.hour,
        };
        let other_date = Date {
            month: other.month,
            day: other.day,
            hour: other.hour,
        };

        let date = self_date.interpolate(other_date, x);

        Self {
            year: self.year, // irrelevant, really
            month: date.month,
            day: date.day,
            hour: date.hour,
            minute: self.minute,                       // irrelevant, really
            uncertainty_flags: self.uncertainty_flags, // irrelevant, really
            dry_bulb_temperature: interp(self.dry_bulb_temperature, other.dry_bulb_temperature),
            dew_point_temperature: interp(self.dew_point_temperature, other.dew_point_temperature),
            relative_humidity: interp(self.relative_humidity, other.relative_humidity),
            atmospheric_station_pressure: interp(
                self.atmospheric_station_pressure,
                other.atmospheric_station_pressure,
            ),
            extraterrestrial_horizontal_radiation: interp(
                self.extraterrestrial_horizontal_radiation,
                other.extraterrestrial_horizontal_radiation,
            ),
            extraterrestrial_direct_normal_radiation: interp(
                self.extraterrestrial_direct_normal_radiation,
                other.extraterrestrial_direct_normal_radiation,
            ),
            horizontal_infrared_radiation_intensity: interp(
                self.horizontal_infrared_radiation_intensity,
                other.horizontal_infrared_radiation_intensity,
            ),
            global_horizontal_radiation: interp(
                self.global_horizontal_radiation,
                other.global_horizontal_radiation,
            ),
            direct_normal_radiation: interp(
                self.direct_normal_radiation,
                other.direct_normal_radiation,
            ),
            diffuse_horizontal_radiation: interp(
                self.diffuse_horizontal_radiation,
                other.diffuse_horizontal_radiation,
            ),
            global_horizontal_illuminance: interp(
                self.global_horizontal_illuminance,
                other.global_horizontal_illuminance,
            ),
            direct_normal_illuminance: interp(
                self.direct_normal_illuminance,
                other.direct_normal_illuminance,
            ),
            diffuse_horizontal_illuminance: interp(
                self.diffuse_horizontal_illuminance,
                other.diffuse_horizontal_illuminance,
            ),
            zenith_luminance: interp(self.zenith_luminance, other.zenith_luminance),
            wind_direction: interp(self.wind_direction, other.wind_direction),
            wind_speed: interp(self.wind_speed, other.wind_speed),
            total_sky_cover: interp(self.total_sky_cover, other.total_sky_cover),
            opaque_sky_cover: interp(self.opaque_sky_cover, other.opaque_sky_cover),
            visibility: interp(self.visibility, other.visibility),
            ceiling_height: interp(self.ceiling_height, other.ceiling_height),
            present_weather_observation: interp(
                self.present_weather_observation,
                other.present_weather_observation,
            ),
            present_weather_codes: interp(self.present_weather_codes, other.present_weather_codes),
            precipitable_water: interp(self.precipitable_water, other.precipitable_water),
            aerosol_optical_depth: interp(self.aerosol_optical_depth, other.aerosol_optical_depth),
            snow_depth: interp(self.snow_depth, other.snow_depth),
            last_day_since_last_snowfall: self.last_day_since_last_snowfall,
            albedo: interp(self.albedo, other.albedo),
            liquid_precipitation_depth: interp(
                self.liquid_precipitation_depth,
                other.liquid_precipitation_depth,
            ),
            liquid_precipitation_quantity: interp(
                self.liquid_precipitation_quantity,
                other.liquid_precipitation_quantity,
            ),
        }
    }
}
