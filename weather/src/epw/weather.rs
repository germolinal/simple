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

use super::ground_temperature::EPWGroundTemperature;
use super::scanner::EPWScanner;
use super::weather_line::EPWWeatherLine;
use crate::location::Location;
use crate::Weather;

use serde::{Deserialize, Serialize};
use std::{fmt::Display, path::Path};

/// A structure representing an EPW file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EPWWeather {
    /// The [`Location`] of the EPW file
    pub location: Location,

    /// The [`EPWGroundTemperature`] of the file
    pub ground_temperature: Vec<EPWGroundTemperature>,

    /// The weather data
    pub data: Vec<EPWWeatherLine>,
}

impl std::default::Default for EPWWeather {
    fn default() -> Self {
        Self {
            location: Location::default(),
            data: Vec::with_capacity(8670),
            ground_temperature: Vec::with_capacity(1),
        }
    }
}

impl EPWWeather {
    /// Creates an `EPWWeather` from a file
    pub fn from_file<P: AsRef<Path> + Display>(filename: P) -> Result<Self, String> {
        EPWScanner::from_file(filename)
    }
}

impl std::convert::From<EPWWeather> for Weather {
    fn from(epw: EPWWeather) -> Weather {
        let data = epw.data.iter().map(|ln| ln.into()).collect();

        Weather {
            data,
            location: epw.location,
        }
    }
}
