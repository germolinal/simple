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

use crate::{Float, solar::Solar};
use serde::{Serialize,Deserialize};

/// A Location
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct Location {
    /// The name of the City
    pub city: String,

    /// The name of the state (or something similar, varies with country)
    pub state: String,

    /// The name or ISO code of the country
    pub country: String,

    /// The source of the weather file (e.g., TMY, IWEC)
    pub source: String,

    /// The World Meteorological Organization Station Number.
    ///
    /// This comes with the EPW file, but we are unlikely to use it.
    pub wmo: String,

    /// The Latitude in Radians.
    ///
    /// South is negative, North is Positive.
    pub latitude: Float,

    /// The Longitude in Radians.
    ///
    /// West is Negative, East is Positive    
    pub longitude: Float,

    /// The Timezone of the location (GMT)
    pub timezone: i8,

    /// The elevation of the weather station
    pub elevation: Float,
}


impl Location {
    /// Builds a [`Solar`] object corresponding to 
    /// this location
    pub fn get_solar(&self)->Solar{
        let stdmer = ((self.timezone as Float)*15.0).to_radians();                
        Solar::new(self.latitude, -self.longitude, -stdmer)
    }
}