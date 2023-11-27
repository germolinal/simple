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

#![deny(missing_docs)]

//! This library is both an [EPW file parser](https://energyplus.net/weather) and
//! a trait that allows getting weather data for simulations

/// The kind of Floating point number used in the
/// library... the `"float"` feature means it becomes `f32`
/// and `f64` is used otherwise.
#[cfg(feature = "float")]
pub type Float = f32;

/// The kind of Floating point number used in the
/// library... the `"float"` feature means it becomes `f32`
/// and `f64` is used otherwise.
#[cfg(not(feature = "float"))]
pub type Float = f64;

#[cfg(feature = "float")]
const PI: Float = std::f32::consts::PI;

#[cfg(not(feature = "float"))]
const PI: Float = std::f64::consts::PI;

/// Solar calculations library. Based on Duffie and Beckman's excellent book.
///
/// We follow the convention of the book. This means that everything is in
/// international units, and times are solar. Angles (inputs and outputs) are
/// in Radians but can be converted into Degrees through the in_degrees() and
/// in_radiance() functions.
///
/// Solar azimuth angle is the angular displacement from south of the
/// projection of beam radiation on the horizontal plane (Figure 1.6.1 of the Book).
/// Displacements east of south are negative and west of south are positive.
///
/// North points in the Y direction. East points in the X direction. Up points in Z.
pub mod solar;
pub use self::solar::{PerezSky, ReinhartSky, SkyUnits, Solar, Time};

/// Data associated to a specific Location
pub mod location;
pub use crate::location::Location;

/// Data associated to the specific weather conditions at a particular moment
pub mod current_weather;
pub use crate::current_weather::CurrentWeather;

/// For handling EPW Files
pub mod epw;
pub use epw::{
    ground_temperature::EPWGroundTemperature, weather::EPWWeather, weather_line::EPWWeatherLine,
};
use serde::{Deserialize, Serialize};

/// Allows creating weathers that can be used for highly-specific
/// simulation. E.g., Having a sinusoidal exterior temperature with no
/// sun.
pub mod synthetic_weather;
pub use crate::synthetic_weather::SyntheticWeather;
pub use calendar::Date;

/// The basic trait defining a Weather that can be used in
/// Building Simulation
pub trait WeatherTrait: Sync {
    /// Retreives a [`CurrentWeather`] object based on the date.
    fn get_weather_data(&self, date: Date) -> CurrentWeather;
}

/// A structure containing weather data
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Weather {
    /// The weather data
    pub data: Vec<CurrentWeather>,

    /// Information about the location of the weather
    ///
    /// This is based on EPW Files information
    pub location: Location,
}

impl Weather {
    /// Gets a weather line corresponding to a specific [`Date`].
    ///
    /// It interpolates if necessary    
    pub fn find_weather_line(&self, date: Date) -> CurrentWeather {
        match self.data.binary_search_by(|x| x.date.cmp(&date)) {
            Ok(i) => {
                // Exact match.
                self.data[i]
            }
            Err(i) => {
                let n = self.data.len();
                if i == 0 || i == n {
                    // Border condition: Date is between the last and the first dates found in the data.
                    // This means we need to interpolate with element 0.
                    let last_date = self.data[n - 1].date;
                    let first_date = self.data[0].date;
                    let last_n = last_date.day_of_year();
                    let first_n = first_date.day_of_year() + 365.;
                    let date_n = date.day_of_year() + 365.;

                    let x = (date_n - last_n) / (first_n - last_n);
                    self.data.last().unwrap().interpolate(&self.data[0], x)
                } else {
                    let before = self.data[i - 1].date;
                    let after = self.data[i].date;

                    let before_n = before.day_of_year();
                    let after_n = after.day_of_year();
                    let date_n = date.day_of_year();

                    let x = (date_n - before_n) / (after_n - before_n);
                    self.data[i - 1].interpolate(&self.data[i], x)
                }
            }
        }
    }

    /// Sorts the data by date
    pub fn sort_data(&mut self) {
        self.data.sort_by(|a, b| a.date.cmp(&b.date));
    }

    /// Calculates the solar data (direct_normal, diffuse_horizontal, global_horizontal)
    /// from an array of [`CurrentWeather`].
    pub fn fill_solar_radiation_data(&mut self) -> Result<(), String> {
        let solar = self.location.get_solar();

        let solar_data: Vec<(Float, Float, Float)> = self
            .data
            .iter()
            .enumerate()
            .map(|(line_index, current_data)| {
                let date = current_data.date;

                let pos = solar.sun_position_from_standard_time(date);
                if let Some(sun_direction) = pos {
                    let three_hours_prior_data = if line_index >= 3 {
                        self.data[line_index - 3]
                    } else {
                        *current_data
                    };
                    let prior_data: Option<CurrentWeather> = if line_index >= 1 {
                        let r = self.data[line_index - 1];
                        Some(r)
                    } else {
                        None
                    };

                    let next_data: Option<CurrentWeather> = if line_index + 1 < self.data.len() {
                        let r = self.data[line_index + 1];
                        Some(r)
                    } else {
                        None
                    };

                    let (direct_normal, diffuse_horizontal, global) = solar
                        .direct_diffuse_from_cloud_generic(
                            sun_direction,
                            *current_data,
                            next_data,
                            prior_data,
                            three_hours_prior_data,
                        );
                    (direct_normal, diffuse_horizontal, global)
                } else {
                    (0.0, 0.0, 0.0)
                }
            })
            .collect();

        // write results
        for (w, calculated) in self.data.iter_mut().zip(solar_data) {
            // dbg!(calculated, w.date);
            w.direct_normal_radiation = calculated.0;
            w.diffuse_horizontal_radiation = calculated.1;
            w.global_horizontal_radiation = calculated.2;
        }

        Ok(())
    }
}

impl std::ops::AddAssign<Self> for Weather {
    fn add_assign(&mut self, rhs: Self) {
        if self.location == rhs.location {
            self.data.extend_from_slice(&rhs.data)
        } else {
            panic!("Trying to concatenate climates from different origins")
        }
    }
}

impl WeatherTrait for Weather {
    fn get_weather_data(&self, date: Date) -> CurrentWeather {
        self.find_weather_line(date)
    }
}

#[cfg(test)]
mod tests {
    use super::epw::scanner::EPWScanner;
    use super::*;
    use calendar::Period;
    use validate::assert_close;

    #[test]
    fn test_find_weather_line() {
        let raw_source = "LOCATION,SANTIAGO,-,CHL,IWEC Data,855740,-33.38,-70.78,-4.0,476.0\nDESIGN CONDITIONS,1,Climate Design Data 2009 ASHRAE Handbook,,Heating,7,-1.1,0,-2.7,3.2,4.1,-1.4,3.6,4.4,8.3,9.6,6.5,10.7,0.9,30,Cooling,1,17.2,31.8,18,30.7,17.8,29.7,17.5,19.5,29,18.8,28.4,18.3,27.9,5.7,200,15.8,11.9,23.8,14.9,11.2,23,14.1,10.6,22,57.5,29.2,55.3,28.4,53.3,28,1149,Extremes,8.4,7.4,6.5,27.1,-3.5,34.5,1.3,1.1,-4.4,35.3,-5.2,35.9,-5.9,36.6,-6.8,37.4\nTYPICAL/EXTREME PERIODS,6,Summer - Week Nearest Max Temperature For Period,Extreme,1/20,1/26,Summer - Week Nearest Average Temperature For Period,Typical,12/ 8,12/14,Winter - Week Nearest Min Temperature For Period,Extreme,7/27,8/ 2,Winter - Week Nearest Average Temperature For Period,Typical,8/10,8/16,Autumn - Week Nearest Average Temperature For Period,Typical,4/12,4/18,Spring - Week Nearest Average Temperature For Period,Typical,10/27,11/ 2\nGROUND TEMPERATURES,3,.5,,,,18.03,20.05,20.54,19.99,17.11,13.95,11.03,8.95,8.41,9.49,11.96,15.03,2,,,,16.15,18.06,18.93,18.92,17.37,15.20,12.89,10.95,9.98,10.23,11.65,13.77,4,,,,14.90,16.39,17.29,17.55,16.95,15.67,14.11,12.60,11.61,11.40,12.03,13.28\nHOLIDAYS/DAYLIGHT SAVINGS,No,0,0,0\nCOMMENTS 1,\"IWEC- WMO#855740 - South America -- Original Source Data (c) 2001 American Society of Heating, Refrigerating and Air-Conditioning Engineers (ASHRAE), Inc., Atlanta, GA, USA.  www.ashrae.org  All rights reserved as noted in the License Agreement and Additional Conditions. DISCLAIMER OF WARRANTIES: The data is provided 'as is' without warranty of any kind, either expressed or implied. The entire risk as to the quality and performance of the data is with you. In no event will ASHRAE or its contractors be liable to you for any damages, including without limitation any lost profits, lost savings, or other incidental or consequential damages arising out of the use or inability to use this data.\"\nCOMMENTS 2, -- Ground temps produced with a standard soil diffusivity of 2.3225760E-03 {m**2/day}\nDATA PERIODS,1,1,Data,Sunday, 1/ 1,12/31\n1987,1,1,1,60,C9C9C9C9*0?9?9?9?9?9?9?9A7A7B8B8A7*0*0E8*0*0,16.7,9.6,63,95600,0,1415,326,0,0,0,0,0,0,0,150,1.5,0,0,9.9,77777,9,999999999,0,0.2680,0,88,0.000,0.0,0.0\n1987,1,1,2,60,C9C9C9C9*0?9?9?9?9?9?9?9A7A7A7A7A7A7*0E8*0*0,15.1,8.4,64,95700,0,1415,317,0,0,0,0,0,0,0,0,0.0,0,0,15.0,22000,9,999999999,0,0.2680,0,88,0.000,0.0,0.0\n1987,1,1,3,60,C9C9C9C9*0?9?9?9?9?9?9?9A7A7B8B8A7*0*0E8*0*0,13.8,7.6,66,95700,0,1415,311,0,0,0,0,0,0,0,0,0.0,0,0,9.9,22000,9,999999999,0,0.2680,0,88,0.000,0.0,0.0\n1987,1,1,4,60,C9C9C9C9*0?9?9?9?9?9?9?9A7A7B8B8A7*0*0E8*0*0,12.7,7.3,70,95700,0,1415,306,0,0,0,0,0,0,0,0,0.0,0,0,9.9,22000,9,999999999,0,0.2680,0,88,0.000,0.0,0.0".to_string();

        let source: Vec<u8> = raw_source.into_bytes();

        let epw: Weather = EPWScanner::build_weather_file(&source).unwrap().into();

        let ln = epw.find_weather_line(Date {
            month: 1,
            day: 1,
            hour: 1.,
        });

        assert_close!(
            ln.dew_point_temperature,
            epw.data[0].dew_point_temperature,
            1e-5
        );
        assert_close!(
            ln.dry_bulb_temperature,
            epw.data[0].dry_bulb_temperature,
            1e-5
        );

        let ln = epw.find_weather_line(Date {
            month: 1,
            day: 1,
            hour: 2.,
        });

        assert_close!(
            ln.dew_point_temperature,
            epw.data[1].dew_point_temperature,
            1e-5
        );
        assert_close!(
            ln.dry_bulb_temperature,
            epw.data[1].dry_bulb_temperature,
            1e-5
        );

        let ln = epw.find_weather_line(Date {
            month: 1,
            day: 1,
            hour: 1.5,
        });

        assert_close!(
            ln.dew_point_temperature,
            (epw.data[1].dew_point_temperature + epw.data[0].dew_point_temperature) / 2.,
            1e-5
        );
        assert_close!(
            ln.dry_bulb_temperature,
            (epw.data[1].dry_bulb_temperature + epw.data[0].dry_bulb_temperature) / 2.,
            1e-5
        );
    }

    #[test]
    fn test_add_assign() {
        let fact = Period::new(
            Date {
                month: 1,
                day: 1,
                hour: 0.,
            },
            Date {
                month: 1,
                day: 25,
                hour: 0.,
            },
            60. * 60. / 2., // half an hour
        );

        let first_fact = fact;
        let second_fact = fact;

        let first_dates: Vec<CurrentWeather> = first_fact
            .into_iter()
            .take(5)
            .map(|date| CurrentWeather {
                date,
                ..CurrentWeather::default()
            })
            .collect();
        let second_dates: Vec<CurrentWeather> = second_fact
            .into_iter()
            .skip(5)
            .take(5)
            .map(|date| CurrentWeather {
                date,
                ..CurrentWeather::default()
            })
            .collect();

        let mut first = Weather {
            location: Location::default(),
            data: first_dates,
        };

        let second = Weather {
            location: Location::default(),
            data: second_dates,
        };

        first += second;

        assert_eq!(first.data.len(), 10);

        for (i, d) in fact.into_iter().take(10).enumerate() {
            assert_eq!(d, first.data[i].date)
        }
    }

    #[test]
    fn test_interpolate_line() {
        let one = CurrentWeather {
            date: Date {
                month: 1,
                day: 1,
                hour: 0.0,
            },
            dry_bulb_temperature: 22.,
            ..CurrentWeather::default()
        };

        let other = CurrentWeather {
            date: Date {
                month: 1,
                day: 1,
                hour: 0.0,
            },
            dry_bulb_temperature: 33.,
            ..CurrentWeather::default()
        };

        one.interpolate(&other, 0.5);
    }

    #[test]
    fn test_sort_data() {
        let w = Weather {
            location: Location::default(),
            data: vec![
                CurrentWeather {
                    date: Date {
                        month: 12,
                        day: 1,
                        hour: 0.0,
                    },
                    ..CurrentWeather::default()
                },
                CurrentWeather {
                    date: Date {
                        month: 1,
                        day: 1,
                        hour: 0.0,
                    },
                    ..CurrentWeather::default()
                },
                CurrentWeather {
                    date: Date {
                        month: 10,
                        day: 1,
                        hour: 0.0,
                    },
                    ..CurrentWeather::default()
                },
            ],
        };

        let mut wclone = w.clone();

        wclone.sort_data();

        assert_eq!(w.data[0].date, wclone.data[2].date);
        assert_eq!(w.data[1].date, wclone.data[0].date);
        assert_eq!(w.data[2].date, wclone.data[1].date);
    }
}
