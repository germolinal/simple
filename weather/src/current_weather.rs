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

use serde::{Serialize, Deserialize};
use calendar::Date;
use crate::Float;

/// A structure containing weather data necessary to simulate the performance
/// of buildings.
#[derive(Default, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CurrentWeather {

    /// The date of the current weather
    pub date: Date,

    /// Exterior dry bulb temperature in C
    pub dry_bulb_temperature: Float,

    /// Exterior dew point temperature in C
    pub dew_point_temperature: Float,

    /// in Wh/m2
    pub global_horizontal_radiation: Option<Float>,

    /// in Wh/m2
    pub direct_normal_radiation: Option<Float>,

    /// in Wh/m2
    pub diffuse_horizontal_radiation: Option<Float>,

    /// in m/s
    pub wind_speed: Float,

    /// in radians... north is zero?
    pub wind_direction: Float,

    /// in Wh/m2
    pub horizontal_infrared_radiation_intensity: Option<Float>,

    /// used this if IR Intensity is missing (in fraction, 0-1)
    pub opaque_sky_cover: Float,

    /// Relative humidity, in fractions (0-1)
    pub relative_humidity: Float,

    /// in Pa
    pub pressure: Float,
}

impl CurrentWeather {
    /// Utilized for deriving a value for the the `horizontal_infrared_radiation_intensity`
    /// field when it is not available.
    ///
    /// # The Math
    /// > *This comes from EnergyPlus Engineering Reference*
    ///
    /// The Horizontal Infrared Radiation Intensity ($`IR_h`$) should formally be calculated
    /// as $`\sigma \times T^4_{sky}`$ (where $`\sigma`$ is the Stefan-Boltzmann constant and $`T_{sky}`$
    /// is the sky temperature). This has historically been approximated through the following equation:
    ///
    /// ```math
    /// IR_h = \epsilon_{sky} \sigma  T^4_{dry bulb}
    /// ```
    ///
    /// Where $`\epsilon_{sky}`$ can be calculated from the clear-sky emmisivity ($`\epsilon_{sky, clear}`$)
    /// based on the opaque sky cover $`N`$ as follows
    ///
    /// ```math
    /// \epsilon_{sky} = \epsilon_{sky, clear} \left( 1 + 0.0224 N - 0.0035 N^2 + 0.00028 N^3 \right)
    /// ```
    ///
    /// Then, EnergyPlus offers 4 correlations... `SIMPLE` utilizes EnergyPlus' default one, which is
    /// the one called Clark & Allen in that reference. This correlation is the following:
    ///
    /// ```math
    /// \epsilon_{sky, clear} = 0.787 + 0.764 ln \left( \frac{T_{dewpoint}}{273} \right)
    /// ```
    /// # Examples 
    /// 
    /// ```rust
    /// 
    /// # use weather::current_weather::CurrentWeather;
    /// let cw = CurrentWeather {
    ///     // This method returns an error if this info is not available.
    ///     dry_bulb_temperature: 20.,
    ///     dew_point_temperature: 10.,
    ///     opaque_sky_cover: 0.,
    ///     .. CurrentWeather::default()
    /// };
    ///
    /// let expected = 341.2; // W/m2
    /// let found = cw.derive_horizontal_ir();
    /// assert!( (expected - found).abs() < 0.1, "expected = {} | found = {}", expected, found );
    ///
    /// // This example compares an actual value (in an EPW file) and a derived one
    /// 
    /// let cw = CurrentWeather {
    ///     // This method returns an error if this info is not available.
    ///     dry_bulb_temperature: 13.625,
    ///     dew_point_temperature: 8.325,
    ///     opaque_sky_cover: 5.,
    ///     .. CurrentWeather::default()
    /// };
    ///
    /// let expected = 329.25; // W/m2, from EPW file
    /// let found = cw.derive_horizontal_ir();
    /// assert!( (expected - found).abs() < 0.1, "expected = {} | found = {}", expected, found );
    /// ```
    pub fn derive_horizontal_ir(&self) -> Float {
        pub const SIGMA: Float = 5.670374419e-8;
        let n = self.opaque_sky_cover;
            
        let dp = self.dew_point_temperature + 273.15;

        let temp = self.dry_bulb_temperature + 273.15; 

        let e_sky_clear = 0.787 + 0.764 * (dp / 273.).ln();
        let e_sky = e_sky_clear * (1.0 + 0.0224 * n - 0.0035 * n.powi(2) + 0.00028 * n.powi(3));

        SIGMA * e_sky * (temp).powi(4)
    }

    /// Interpolates the data between to WeatherLines
    pub fn interpolate(&self, other: &Self, x: Float) -> Self {
        let interp_opt = |a, b| {
            if let (Some(a), Some(b)) = (a, b) {
                Some(a + x * (b - a))
            } else {
                None
            }
        };
        let interp = |a, b| {
            a + x * (b - a)
        };


        let date = self.date.interpolate(other.date, x);

        Self {            
            date,          
            dry_bulb_temperature: interp(self.dry_bulb_temperature, other.dry_bulb_temperature),
            dew_point_temperature: interp(self.dew_point_temperature, other.dew_point_temperature),
            relative_humidity: interp(self.relative_humidity, other.relative_humidity),
            pressure: interp(
                self.pressure,
                other.pressure,
            ),            
            horizontal_infrared_radiation_intensity: interp_opt(
                self.horizontal_infrared_radiation_intensity,
                other.horizontal_infrared_radiation_intensity,
            ),
            global_horizontal_radiation: interp_opt(
                self.global_horizontal_radiation,
                other.global_horizontal_radiation,
            ),
            direct_normal_radiation: interp_opt(
                self.direct_normal_radiation,
                other.direct_normal_radiation,
            ),
            diffuse_horizontal_radiation: interp_opt(
                self.diffuse_horizontal_radiation,
                other.diffuse_horizontal_radiation,
            ),            
            wind_direction: interp(self.wind_direction, other.wind_direction),
            wind_speed: interp(self.wind_speed, other.wind_speed),
            // total_sky_cover: interp(self.total_sky_cover, other.total_sky_cover),
            opaque_sky_cover: interp(self.opaque_sky_cover, other.opaque_sky_cover),            
        }
    }
}
