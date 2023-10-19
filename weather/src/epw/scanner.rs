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
use crate::Float;

use super::ground_temperature::EPWGroundTemperature;
use super::weather::EPWWeather;
use super::weather_line::EPWWeatherLine;
use std::fmt::Display;
use std::fs;
use std::path::Path;

pub(crate) struct EPWScanner<'a> {
    /// Indicates the position of current character being
    /// scanned
    current: usize,

    /// Indicates the position of the first character of the
    /// element being scanned
    start: usize,

    /// Indicates the line of the EPW file in which we are
    line: usize,

    /// The data source
    src: &'a Vec<u8>,

    /// Are we in a string? (i.e. within quotation marks, e.g. " STRING ")
    in_string: bool,
}

impl<'a> EPWScanner<'a> {
    /// Creates a new scanner.
    pub fn new(src: &'a Vec<u8>) -> Self {
        Self {
            current: 0,
            start: 0,
            line: 1,
            src,
            in_string: false,
        }
    }

    /// Builds a weather file starting from a file name
    pub fn from_file<P: AsRef<Path> + Display>(filename: P) -> Result<EPWWeather, String> {
        let src = match fs::read(&filename) {
            Ok(v) => v,
            Err(_) => return Err(format!("Could not read epw file '{}'", filename)),
        };

        EPWScanner::build_weather_file(&src)
    }

    /// Parses the EPW file and builds a proper EPWWeather
    pub fn build_weather_file(src: &'a Vec<u8>) -> Result<EPWWeather, String> {
        // build a scaner
        let mut scanner = EPWScanner::new(src);

        // create an empty .EPW
        let mut epw = EPWWeather::default();

        scanner.parse_file(&mut epw)?;

        Ok(epw)
    }

    /// Checks if the scanner is finished
    fn is_finished(&self) -> bool {
        self.current >= self.src.len()
    }

    /// Scans all the characters until reaching the Comma.
    /// Returns a slice    
    fn scan_element(&mut self) -> Option<&[u8]> {
        if self.is_finished() {
            return None;
        }

        loop {
            // Return if scanning is over
            if self.is_finished() {
                break;
            }

            // If we find a comma, and we are not in a string, break
            if !self.in_string && self.src[self.current] == b',' {
                break;
            }

            // Increase line number if required
            if self.src[self.current] == b'\n' {
                self.line += 1;
                break;
            }

            // Toggle string if we are in one.
            if self.src[self.current] == b'"' {
                self.in_string = !self.in_string;
            }
            self.current += 1;
        }

        // Ignore the \r thing that I hate
        let mut end = self.current;
        if self.src[end - 1] == b'\r' {
            end -= 1;
        }
        let (ini, fin) = (self.start, end);

        self.current += 1; // skip the comma
        self.start = self.current;

        self.src.get(ini..fin)
    }

    fn scan_string(slice: Option<&[u8]>) -> Result<String, String> {
        match slice {
            Some(v) => {
                let mut s: Vec<u8> = Vec::with_capacity(v.len());
                for b in v.iter() {
                    s.push(*b);
                }

                Ok(String::from_utf8(s).map_err(|e| e.to_string())?)
            }
            None => Err("Internal error... could not source text for element".to_string()),
        }
    }

    /// This function is meant to scan a string
    /// but not use it... returns a bool, but not needed
    fn skip_string(slice: Option<&[u8]>) -> Result<bool, String> {
        let _ = EPWScanner::scan_string(slice);
        Ok(true)
    }

    /// Scans an element and transforms it into a number
    fn scan_number(slice: Option<&[u8]>) -> Result<Float, String> {
        if let Ok(v) = EPWScanner::scan_string(slice) {
            let the_v = match v.parse::<Float>() {
                Ok(v) => v,
                Err(msg) => {
                    panic!("{}", msg);
                }
            };

            Ok(the_v)
        } else {
            Err("Could not scan number... scan_string() return error".to_string())
        }
    }

    /// Checks if the next element is empty
    fn next_is_empty(&self) -> bool {
        // If scanner is finished, is empty
        if self.is_finished() {
            return true;
        }

        // If not finished, check next one.
        match self.src[self.current] as char {
            ',' => {
                // if comma, empty.
                true
            }
            '\n' => {
                // if new line, we need to check the following
                self.src[self.current] as char == ','
            }
            _ => false,
        }
    }

    /// This function is supposed to recursively parse the entire
    /// file. However, for now it only scans the location and the
    /// data
    fn parse_file(&mut self, epw: &mut EPWWeather) -> Result<(), String> {
        loop {
            // Scan
            let slice_option = self.scan_element();

            // Break if we are finished
            if slice_option.is_none() {
                break;
            }
            let keyword = EPWScanner::scan_string(slice_option)?;

            // Parse ground temperatures, if they are found
            if keyword == *"GROUND TEMPERATURES" {
                self.parse_ground_temperature(epw)?;
            }

            // Parse location, when found
            if keyword == *"LOCATION" {
                self.parse_location(epw)?;
            }

            if keyword == *"DATA PERIODS" {
                self.parse_data_periods(epw)?;
            }
        }

        Ok(())
    }

    /// Parse the actual data, starting from the description of the
    /// data period.
    /// For now, this only allows a single data period
    fn parse_data_periods(&mut self, epw: &mut EPWWeather) -> Result<(), String> {
        let n = EPWScanner::scan_number(self.scan_element())? as usize;
        if n != 1 {
            panic!("Only one data period per EPW file is allowed!");
        }

        let n_records_per_hour = EPWScanner::scan_number(self.scan_element())? as usize;
        if n_records_per_hour != 1 {
            panic!("Only one record per hour is allowed in EPW file!");
        }

        let _data_period_name = EPWScanner::scan_string(self.scan_element())?;
        let _start_day_of_the_week = EPWScanner::scan_string(self.scan_element())?;

        // this is in month/day data format
        let _data_period_start_day = EPWScanner::scan_string(self.scan_element())?;
        // this is in month/day data format
        let _data_period_end_day = EPWScanner::scan_string(self.scan_element())?;

        // Now scan until the file is finished

        while !self.is_finished() {
            epw.data.push(EPWWeatherLine {
                year: EPWScanner::scan_number(self.scan_element())? as usize,
                month: EPWScanner::scan_number(self.scan_element())? as u8,
                day: EPWScanner::scan_number(self.scan_element())? as u8,
                hour: EPWScanner::scan_number(self.scan_element())?,
                minute: EPWScanner::scan_number(self.scan_element())? as u8,
                uncertainty_flags: EPWScanner::skip_string(self.scan_element())?,
                dry_bulb_temperature: EPWScanner::scan_number(self.scan_element())?,
                dew_point_temperature: EPWScanner::scan_number(self.scan_element())?,
                relative_humidity: EPWScanner::scan_number(self.scan_element())?,
                atmospheric_station_pressure: EPWScanner::scan_number(self.scan_element())?,
                extraterrestrial_horizontal_radiation: EPWScanner::scan_number(
                    self.scan_element(),
                )?,
                extraterrestrial_direct_normal_radiation: EPWScanner::scan_number(
                    self.scan_element(),
                )?,
                horizontal_infrared_radiation_intensity: EPWScanner::scan_number(
                    self.scan_element(),
                )?,
                global_horizontal_radiation: EPWScanner::scan_number(self.scan_element())?,
                direct_normal_radiation: EPWScanner::scan_number(self.scan_element())?,
                diffuse_horizontal_radiation: EPWScanner::scan_number(self.scan_element())?,
                global_horizontal_illuminance: EPWScanner::scan_number(self.scan_element())?,
                direct_normal_illuminance: EPWScanner::scan_number(self.scan_element())?,
                diffuse_horizontal_illuminance: EPWScanner::scan_number(self.scan_element())?,
                zenith_luminance: EPWScanner::scan_number(self.scan_element())?,
                wind_direction: EPWScanner::scan_number(self.scan_element())?.max(360.), //Missing value is 999
                wind_speed: EPWScanner::scan_number(self.scan_element())?,
                total_sky_cover: EPWScanner::scan_number(self.scan_element())?,
                opaque_sky_cover: EPWScanner::scan_number(self.scan_element())?,
                visibility: EPWScanner::scan_number(self.scan_element())?,
                ceiling_height: EPWScanner::scan_number(self.scan_element())?,
                present_weather_observation: EPWScanner::scan_number(self.scan_element())?,
                present_weather_codes: EPWScanner::scan_number(self.scan_element())?,
                precipitable_water: EPWScanner::scan_number(self.scan_element())?,
                aerosol_optical_depth: EPWScanner::scan_number(self.scan_element())?,
                snow_depth: EPWScanner::scan_number(self.scan_element())?,
                last_day_since_last_snowfall: EPWScanner::scan_number(self.scan_element())?
                    as usize,
                albedo: EPWScanner::scan_number(self.scan_element())?,
                liquid_precipitation_depth: EPWScanner::scan_number(self.scan_element())?,
                liquid_precipitation_quantity: EPWScanner::scan_number(self.scan_element())?,
            })
        }
        Ok(())
    }

    /// Parses a location... assumes that the LOCATION
    /// keyword has been consumed already
    fn parse_location(&mut self, epw: &mut EPWWeather) -> Result<(), String> {
        epw.location.city = EPWScanner::scan_string(self.scan_element())?;
        epw.location.state = EPWScanner::scan_string(self.scan_element())?;
        epw.location.country = EPWScanner::scan_string(self.scan_element())?;
        epw.location.source = EPWScanner::scan_string(self.scan_element())?;
        epw.location.wmo = EPWScanner::scan_string(self.scan_element())?;

        epw.location.latitude = EPWScanner::scan_number(self.scan_element())?.to_radians();
        epw.location.longitude = EPWScanner::scan_number(self.scan_element())?.to_radians();
        epw.location.timezone = EPWScanner::scan_number(self.scan_element())? as i8;
        epw.location.elevation = EPWScanner::scan_number(self.scan_element())?;

        Ok(())
    }

    /// Parses the ground temperature        
    #[allow(clippy::field_reassign_with_default)]
    fn parse_ground_temperature(&mut self, epw: &mut EPWWeather) -> Result<(), String> {
        // Number of ground temperatures to scan
        let n_values = EPWScanner::scan_number(self.scan_element())? as usize;

        for _ in 0..n_values {
            let mut g = EPWGroundTemperature::default();

            g.depth = EPWScanner::scan_number(self.scan_element())?;
            if self.next_is_empty() {
                self.scan_element();
            } else {
                g.soil_conductivity = Some(EPWScanner::scan_number(self.scan_element())?);
            }
            if self.next_is_empty() {
                self.scan_element();
            } else {
                g.soil_density = Some(EPWScanner::scan_number(self.scan_element())?);
            }
            if self.next_is_empty() {
                self.scan_element();
            } else {
                g.soil_specific_heat = Some(EPWScanner::scan_number(self.scan_element())?);
            }

            for month in 0..12 {
                g.average_monthly_temperature[month] =
                    EPWScanner::scan_number(self.scan_element())?;
            }

            epw.ground_temperature.push(g);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_element() -> Result<(), String> {
        let raw_source = "Hello,,how\n,are,you\n\"auto,con,ruedas\",tres".to_string();
        let source: Vec<u8> = raw_source.into_bytes();

        let mut s = EPWScanner::new(&source);

        assert_eq!(
            "Hello".to_string(),
            EPWScanner::scan_string(s.scan_element())?
        );
        assert_eq!(s.line, 1);
        assert!(s.next_is_empty());

        assert_eq!("".to_string(), EPWScanner::scan_string(s.scan_element())?);
        assert_eq!(s.line, 1);
        assert!(!s.next_is_empty());

        assert_eq!(
            "how".to_string(),
            EPWScanner::scan_string(s.scan_element())?
        );
        assert_eq!(s.line, 2);
        assert!(s.next_is_empty());

        assert_eq!("".to_string(), EPWScanner::scan_string(s.scan_element())?);
        assert_eq!(s.line, 2);
        assert!(!s.next_is_empty());

        assert_eq!(
            "are".to_string(),
            EPWScanner::scan_string(s.scan_element())?
        );
        assert_eq!(s.line, 2);
        assert!(!s.next_is_empty());

        assert_eq!(
            "you".to_string(),
            EPWScanner::scan_string(s.scan_element())?
        );
        assert_eq!(s.line, 3);
        assert!(!s.next_is_empty());

        assert_eq!(
            "\"auto,con,ruedas\"".to_string(),
            EPWScanner::scan_string(s.scan_element())?
        );
        assert_eq!(s.line, 3);
        assert!(!s.next_is_empty());

        assert_eq!(
            "tres".to_string(),
            EPWScanner::scan_string(s.scan_element())?
        );
        assert_eq!(s.line, 3);
        assert!(s.next_is_empty());

        Ok(())
    }

    #[test]
    fn test_location() -> Result<(), String> {
        let raw_source = "LOCATION,SANTIAGO,-,CHL,IWEC Data,855740,-33.38,-70.78,-4.0,476.0\nDESIGN CONDITIONS,1,Climate".to_string();
        let source: Vec<u8> = raw_source.into_bytes();

        let mut s = EPWScanner::new(&source);
        let mut epw = EPWWeather::default();

        // This is done by the the function that calls this
        s.scan_element().ok_or("no element")?;

        s.parse_location(&mut epw)?;

        assert_eq!(epw.location.city, "SANTIAGO".to_string());
        assert_eq!(epw.location.state, "-".to_string());
        assert_eq!(epw.location.country, "CHL".to_string());
        assert_eq!(epw.location.source, "IWEC Data".to_string());
        assert_eq!(epw.location.wmo, "855740".to_string());
        assert_eq!(epw.location.latitude, (-33.38 as Float).to_radians());
        assert_eq!(epw.location.longitude, (-70.78 as Float).to_radians());
        assert_eq!(epw.location.timezone, -4);
        assert_eq!(epw.location.elevation, 476.0);

        assert_eq!(
            EPWScanner::scan_string(s.scan_element())?,
            "DESIGN CONDITIONS".to_string()
        );

        Ok(())
    }

    #[test]
    fn test_ground_temperature() -> Result<(), String> {
        let raw_source = "GROUND TEMPERATURES,3,.5,,,,18.03,20.05,20.54,19.99,17.11,13.95,11.03,8.95,8.41,9.49,11.96,15.03,2,,,,16.15,18.06,18.93,18.92,17.37,15.20,12.89,10.95,9.98,10.23,11.65,13.77,4,,,,14.90,16.39,17.29,17.55,16.95,15.67,14.11,12.60,11.61,11.40,12.03,13.28\nHOLIDAYS/DAYLIGHT SAVINGS,".to_string();
        let source: Vec<u8> = raw_source.into_bytes();

        let mut s = EPWScanner::new(&source);
        let mut epw = EPWWeather::default();

        // This is done by the the function that calls this
        s.scan_element().ok_or("No element")?;

        s.parse_ground_temperature(&mut epw)?;

        assert_eq!(epw.ground_temperature.len(), 3);
        assert_eq!(epw.ground_temperature[0].depth, 0.5);
        assert_eq!(epw.ground_temperature[1].depth, 2.0);
        assert_eq!(epw.ground_temperature[2].depth, 4.0);

        Ok(())
    }

    #[test]
    fn test_data_period() -> Result<(), String> {
        let raw_source = "DATA PERIODS,1,1,Data,Sunday, 1/ 1,12/31\n1987,1,1,1,60,C9C9C9C9*0?9?9?9?9?9?9?9A7A7B8B8A7*0*0E8*0*0,16.7,9.6,63,95600,0,1415,326,0,0,0,0,0,0,0,150,1.5,0,0,9.9,77777,9,999999999,0,0.2680,0,88,0.000,0.0,0.0\n1987,1,1,2,60,C9C9C9C9*0?9?9?9?9?9?9?9A7A7A7A7A7A7*0E8*0*0,15.1,8.4,64,95700,0,1415,317,0,0,0,0,0,0,0,0,0.0,0,0,15.0,22000,9,999999999,0,0.2680,0,88,0.000,0.0,0.0\n1987,1,1,3,60,C9C9C9C9*0?9?9?9?9?9?9?9A7A7B8B8A7*0*0E8*0*0,13.8,7.6,66,95700,0,1415,311,0,0,0,0,0,0,0,0,0.0,0,0,9.9,22000,9,999999999,0,0.2680,0,88,0.000,0.0,0.0\n1987,1,1,4,60,C9C9C9C9*0?9?9?9?9?9?9?9A7A7B8B8A7*0*0E8*0*0,12.7,7.3,70,95700,0,1415,306,0,0,0,0,0,0,0,0,0.0,0,0,9.9,22000,9,999999999,0,0.2680,0,88,0.000,0.0,0.0".to_string();

        let source: Vec<u8> = raw_source.into_bytes();

        let mut s = EPWScanner::new(&source);
        let mut epw = EPWWeather::default();

        // This is done by the the function that calls this
        s.scan_element().ok_or("No element")?;

        s.parse_data_periods(&mut epw)?;

        assert_eq!(epw.data.len(), 4);
        Ok(())
    }

    #[test]
    fn test_parse_file() -> Result<(), String> {
        let raw_source = "LOCATION,SANTIAGO,-,CHL,IWEC Data,855740,-33.38,-70.78,-4.0,476.0\nDESIGN CONDITIONS,1,Climate Design Data 2009 ASHRAE Handbook,,Heating,7,-1.1,0,-2.7,3.2,4.1,-1.4,3.6,4.4,8.3,9.6,6.5,10.7,0.9,30,Cooling,1,17.2,31.8,18,30.7,17.8,29.7,17.5,19.5,29,18.8,28.4,18.3,27.9,5.7,200,15.8,11.9,23.8,14.9,11.2,23,14.1,10.6,22,57.5,29.2,55.3,28.4,53.3,28,1149,Extremes,8.4,7.4,6.5,27.1,-3.5,34.5,1.3,1.1,-4.4,35.3,-5.2,35.9,-5.9,36.6,-6.8,37.4\nTYPICAL/EXTREME PERIODS,6,Summer - Week Nearest Max Temperature For Period,Extreme,1/20,1/26,Summer - Week Nearest Average Temperature For Period,Typical,12/ 8,12/14,Winter - Week Nearest Min Temperature For Period,Extreme,7/27,8/ 2,Winter - Week Nearest Average Temperature For Period,Typical,8/10,8/16,Autumn - Week Nearest Average Temperature For Period,Typical,4/12,4/18,Spring - Week Nearest Average Temperature For Period,Typical,10/27,11/ 2\nGROUND TEMPERATURES,3,.5,,,,18.03,20.05,20.54,19.99,17.11,13.95,11.03,8.95,8.41,9.49,11.96,15.03,2,,,,16.15,18.06,18.93,18.92,17.37,15.20,12.89,10.95,9.98,10.23,11.65,13.77,4,,,,14.90,16.39,17.29,17.55,16.95,15.67,14.11,12.60,11.61,11.40,12.03,13.28\nHOLIDAYS/DAYLIGHT SAVINGS,No,0,0,0\nCOMMENTS 1,\"IWEC- WMO#855740 - South America -- Original Source Data (c) 2001 American Society of Heating, Refrigerating and Air-Conditioning Engineers (ASHRAE), Inc., Atlanta, GA, USA.  www.ashrae.org  All rights reserved as noted in the License Agreement and Additional Conditions. DISCLAIMER OF WARRANTIES: The data is provided 'as is' without warranty of any kind, either expressed or implied. The entire risk as to the quality and performance of the data is with you. In no event will ASHRAE or its contractors be liable to you for any damages, including without limitation any lost profits, lost savings, or other incidental or consequential damages arising out of the use or inability to use this data.\"\nCOMMENTS 2, -- Ground temps produced with a standard soil diffusivity of 2.3225760E-03 {m**2/day}\nDATA PERIODS,1,1,Data,Sunday, 1/ 1,12/31\n1987,1,1,1,60,C9C9C9C9*0?9?9?9?9?9?9?9A7A7B8B8A7*0*0E8*0*0,16.7,9.6,63,95600,0,1415,326,0,0,0,0,0,0,0,150,1.5,0,0,9.9,77777,9,999999999,0,0.2680,0,88,0.000,0.0,0.0\n1987,1,1,2,60,C9C9C9C9*0?9?9?9?9?9?9?9A7A7A7A7A7A7*0E8*0*0,15.1,8.4,64,95700,0,1415,317,0,0,0,0,0,0,0,0,0.0,0,0,15.0,22000,9,999999999,0,0.2680,0,88,0.000,0.0,0.0\n1987,1,1,3,60,C9C9C9C9*0?9?9?9?9?9?9?9A7A7B8B8A7*0*0E8*0*0,13.8,7.6,66,95700,0,1415,311,0,0,0,0,0,0,0,0,0.0,0,0,9.9,22000,9,999999999,0,0.2680,0,88,0.000,0.0,0.0\n1987,1,1,4,60,C9C9C9C9*0?9?9?9?9?9?9?9A7A7B8B8A7*0*0E8*0*0,12.7,7.3,70,95700,0,1415,306,0,0,0,0,0,0,0,0,0.0,0,0,9.9,22000,9,999999999,0,0.2680,0,88,0.000,0.0,0.0".to_string();

        let source: Vec<u8> = raw_source.into_bytes();

        let mut s = EPWScanner::new(&source);
        let mut epw = EPWWeather::default();

        s.parse_file(&mut epw)?;

        // Location
        assert_eq!(epw.location.city, "SANTIAGO".to_string());
        assert_eq!(epw.location.state, "-".to_string());
        assert_eq!(epw.location.country, "CHL".to_string());
        assert_eq!(epw.location.source, "IWEC Data".to_string());
        assert_eq!(epw.location.wmo, "855740".to_string());
        assert_eq!(epw.location.latitude, (-33.38 as Float).to_radians());
        assert_eq!(epw.location.longitude, (-70.78 as Float).to_radians());
        assert_eq!(epw.location.timezone, -4.0 as i8);
        assert_eq!(epw.location.elevation, 476.0);

        // Data
        assert_eq!(epw.data.len(), 4);

        // ground temperature
        assert_eq!(epw.ground_temperature.len(), 3);
        assert_eq!(epw.ground_temperature[0].depth, 0.5);
        assert_eq!(epw.ground_temperature[1].depth, 2.0);
        assert_eq!(epw.ground_temperature[2].depth, 4.0);

        Ok(())
    }
}
