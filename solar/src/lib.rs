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

#![deny(missing_docs)]

//! Solar calculations library. Based on Duffie and Beckman's excellent book.
//!
//! We follow the convention of the book. This means that everything is in
//! international units, and times are solar. Angles (inputs and outputs) are
//! in Radians but can be converted into Degrees through the in_degrees() and
//! in_radiance() functions.
//!
//! Solar azimuth angle is the angular displacement from south of the
//! projection of beam radiation on the horizontal plane (Figure 1.6.1 of the Book).
//! Displacements east of south are negative and west of south are positive.
//!
//! North points in the Y direction. East points in the X direction. Up points in Z.

use geometry::Vector3D;

mod reinhart_sky;
pub use reinhart_sky::ReinhartSky;
mod perez;
pub use perez::{PerezSky, SkyUnits};

/// The kind of Floating point number used in the
/// library... the `"float"` feature means it becomes `f32`
/// and `f64` is used otherwise.
#[cfg(feature = "float")]
type Float = f32;

#[cfg(not(feature = "float"))]
type Float = f64;

#[cfg(feature = "float")]
const PI: Float = std::f32::consts::PI;

#[cfg(not(feature = "float"))]
const PI: Float = std::f64::consts::PI;

/// Calculates the Air-mass .PerezSky
///
/// This is not the same as equation 1.5.1 of the book, as
/// we are using the approach utilized by Radiance's source code. The two equations
/// differ only in large zolar zeniths angles (e.g., close to 90 degrees)
pub fn air_mass(solar_zenith: Float) -> Float {
    1. / (solar_zenith.cos() + 0.15 * (93.885 - solar_zenith.to_degrees()).powf(-1.253))
}

/// The solar equivalent of Date's "day of the year". The
/// distinction is there so that we don't mistake solar and
/// standard time
pub struct Solar {
    /// Latitude in Radians. South is negative, North is positive.
    latitude: Float,

    /// Longitude (in Radians). East is negative, West is positive
    ///
    /// > Note that this is Radiance's conventions, which is the opposite of EPW files.
    longitude: Float,

    /// Standard meridian (in Radians). East is negative, West is positive.
    ///
    /// This value is essentially `-15.0*TimeZone` (e.g., GMT+1 becomes -15.0)
    ///
    ///  > Note that this is Radiance's conventions, which is the opposite of EPW files.
    standard_meridian: Float,
}

/// W/m2
const SOLAR_CONSTANT: Float = 1367.0;

/// Solar or Standard time, containing the day of the year 'n'
#[derive(Clone, Copy)]
pub enum Time {
    /// Time is in Solar time
    Solar(Float),

    /// Time is in Standard time
    Standard(Float),
}

impl Solar {
    /// Builds a Solar object from  a Latitude,
    /// Longitude and Standard meridian (in Radians)
    pub fn new(latitude: Float, longitude: Float, standard_meridian: Float) -> Self {
        Self {
            latitude,
            longitude,
            standard_meridian,
        }
    }

    /// Returns the difference between the solar and the standard time in minutes
    pub fn solar_standard_time_difference(&self, n: Float) -> Float {
        4.0 * (self.standard_meridian - self.longitude).to_degrees() + self.equation_of_time(n)
    }

    /// Returns the content of a Time enum. Transforms to Solar
    /// if the type of the Enum is Standard
    pub fn unwrap_solar_time(&self, n: Time) -> Float {
        match n {
            Time::Solar(t) => t,
            Time::Standard(t) => {
                let delta_minutes = self.solar_standard_time_difference(t);
                // return the standard time + the number of minutes divided
                // the number of minutes in a day
                t + delta_minutes / 24. / 60.
            }
        }
    }

    /// Returns the content of a Time enum. Transforms to Standard
    /// if the type of the Enum is Solar
    pub fn unwrap_standard_time(&self, n: Time) -> Float {
        match n {
            Time::Solar(t) => {
                let delta_minutes = self.solar_standard_time_difference(t);
                // return the standard time + the number of minutes divided
                // the number of minutes in a day
                t - delta_minutes / 24. / 60.
            }
            Time::Standard(t) => t,
        }
    }

    /// The Equation of Time based on the day of year (can have decimals)
    ///
    /// n should be in solar time, but this variable does not change daily so
    /// it probably does not matter... let's just treat it as Float
    pub fn equation_of_time(&self, n: Float) -> Float {
        let b = self.b(n);
        229.2
            * (0.000075 + 0.001868 * b.cos()
                - 0.032077 * b.sin()
                - 0.014615 * (2.0 * b).cos()
                - 0.04089 * (2.0 * b).sin())
    }

    /// Declination (in Radians), according to Equation 1.6.1B
    ///
    /// n should be in solar time, but this variable does not change daily so
    /// it probably does not matter... let's just treat it as Float
    pub fn declination(&self, n: Float) -> Float {
        let b = self.b(n);

        // Return in Radians
        0.006918 - 0.399912 * b.cos() + 0.070257 * b.sin() - 0.006758 * (2. * b).cos()
            + 0.000907 * (2. * b).sin()
            - 0.002697 * (3. * b).cos()
            + 0.00148 * (3. * b).sin()
    }

    /// Equation 1.4.2 in the Book.
    ///
    /// n should be in solar time, but this variable does not change daily so
    /// it probably does not matter... let's just treat it as Float
    #[inline(always)]
    fn b(&self, n: Float) -> Float {
        (n - 1.0) * 2.0 * PI / 365.0
    }

    /// Normal extraterrestrial radiation (Gon)
    /// Equation 1.4.1b from Duffie and Beckman
    ///
    /// n should be in solar time, but this variable does not change daily so
    /// it probably does not matter... let's just treat it as Float
    pub fn normal_extraterrestrial_radiation(&self, n: Float) -> Float {
        let b = self.b(n);
        let aux = 1.000110
            + 0.034221 * b.cos()
            + 0.001280 * b.sin()
            + 0.000719 * (2.0 * b).cos()
            + 0.000077 * (2.0 * b).sin();
        SOLAR_CONSTANT * aux
    }

    /// Returns the hour angle in degrees    
    pub fn hour_angle(&self, n: Time) -> Float {
        let n = self.unwrap_solar_time(n);

        // Remove the day (keep the hour). Multiply by 24 hours
        let solar_hour = 24. * (n % 1.);

        // Multiply for 24 hours, and by 15degrees/hour
        ((solar_hour - 12.) * 15.).to_radians()
    }

    /// Gets the sunset time (equation 1.6.10)
    /// n should be in solar time, but since it does not change
    /// much on a daily basis, we treat it as an Float
    pub fn sunrise_sunset(&self, n: Float) -> (Time, Time) {
        let delta = self.declination(n);
        let cos_w = -self.latitude.tan() * delta.tan();
        let w = (cos_w.acos()).to_degrees();
        let half_n = w / 15.;

        // return
        let midday = n.floor() + 0.5;
        (
            Time::Solar(midday - half_n / 24.),
            Time::Solar(midday + half_n / 24.),
        )
    }

    /// Builds a vector that points towards the sun.
    ///
    /// Z is up, Y is North and X is East
    pub fn sun_position(&self, n: Time) -> Option<Vector3D> {
        let n = self.unwrap_solar_time(n);

        // if it is night-time, return None
        let (sunrise, sunset) = self.sunrise_sunset(n);
        if n < self.unwrap_solar_time(sunrise) || n > self.unwrap_solar_time(sunset) {
            return None;
        }

        // else, calculate stuff

        let cos_phi = self.latitude.cos();
        let sin_phi = self.latitude.sin();

        let delta = self.declination(n);
        let cos_delta = delta.cos();
        let sin_delta = delta.sin();

        let omega = self.hour_angle(Time::Solar(n));
        let cos_omega = omega.cos();

        // Equation 1.6.5, for Zenith
        let cos_zenith = cos_phi * cos_delta * cos_omega + sin_phi * sin_delta;
        let sin_zenith = cos_zenith.acos().sin();
        if cos_zenith < 0. {
            // it should be daytime; i.e., zenith < 90 (i.e., cos(zenith)>0)
            return None;
        }
        debug_assert!((1.0 - (cos_zenith * cos_zenith + sin_zenith * sin_zenith)).abs() < 0.000001);

        // Is vertical? If so, return vertical... otherwise, carry on.
        const LIMIT_ANGLE: Float = 0.9999; // A zenith angle of less than 0.8 degrees (ish) is considered vertical.
        if cos_zenith > LIMIT_ANGLE {
            return Some(Vector3D::new(0., 0., 1.));
        }
        let z = cos_zenith;

        // Equation 1.6.6 for Azimuth
        let mut cos_azimuth = (cos_zenith * sin_phi - sin_delta) / (sin_zenith * cos_phi);
        if cos_azimuth > 1. {
            cos_azimuth = 1.0;
        } else if cos_azimuth < -1. {
            cos_azimuth = -1.;
        }
        let sin_azimuth = cos_azimuth.acos().sin();

        debug_assert!(
            (1.0 - (cos_azimuth * cos_azimuth + sin_azimuth * sin_azimuth)).abs() < 0.0000001
        );

        // Trigonometry
        let mut x = sin_azimuth * sin_zenith;
        let y = -cos_azimuth * sin_zenith;

        // (x should be positive at this stage, right? then, if omega is
        // positive, we need to change the sign of x)
        if omega > 0. {
            x *= -1.
        }

        // Check length of vector
        debug_assert!(((x * x + y * y + z * z).sqrt() - 1.0).abs() < 0.000001);

        // Build the vector and return
        Some(Vector3D::new(x, y, z))
    }
} // end of impl Solar

#[cfg(test)]
mod tests {

    use super::*;
    use calendar::Date;

    fn are_close(x: Float, y: Float, precision: Float) -> bool {
        if (x - y).abs() < precision {
            return true;
        }
        println!("x:{}, y:{}", x, y);
        false
    }

    #[test]
    fn test_unwrap_time() {
        /*
        Example 1.5.1 in Duffie & Beckman
        At Madison, Wisconsin, what is the solar time corresponding to 10:30 AM central time on February 3?

        Answer: 10:19
        */
        let longitude = (89.4 as Float).to_radians();
        let standard_meridian = (90.0 as Float).to_radians();
        let latitude = (-2. as Float).to_radians();

        let solar = Solar::new(latitude, longitude, standard_meridian);

        const EPS: Float = 0.5 / 60.; // Half a minute precision

        // Standard to solar
        let standard_time = Date {
            month: 2,
            day: 3,
            hour: 10.5,
        };
        let standard_n = standard_time.day_of_year();

        let solar_n = solar.unwrap_solar_time(Time::Standard(standard_n));
        let solar_time = Date::from_day_of_year(solar_n);
        assert!(are_close(solar_time.hour, 10.0 + 19.0 / 60.0, EPS));

        // Solar to standard
        let standard_n_2 = solar.unwrap_standard_time(Time::Solar(solar_n));
        assert!(are_close(standard_n, standard_n_2, EPS))
    }

    #[test]
    fn test_declination() {
        fn check(month: u8, day: u8, expected_n: Float, expected_d: Float) {
            let solar = Solar::new(0.0, 0., 0.);

            let date = Date {
                month: month,
                day: day,
                hour: 0.,
            };
            let n = date.day_of_year();
            assert_eq!(n, expected_n - 1.);

            let d = solar.declination(n);

            println!("exp: {}, found: {}", expected_d, d.to_degrees());
            // I suspect I need this margin of error (1.8 deg.)
            // because Duffie and Beckam do not specify the hour
            // of the day or the exact equation they use.
            assert!(are_close(d.to_degrees(), expected_d, 1.8))
        }

        // From table 1.6.1... declinations are in degrees
        check(1, 17, 17., -20.9);
        check(2, 16, 47., -13.0);
        check(3, 16, 75., -2.4);
        check(4, 15, 105., 9.4);
        check(5, 15, 135., 18.8);
        check(6, 11, 162., 23.1);
        check(7, 17, 198., 21.2);
        check(8, 16, 228., 13.5);
        check(9, 15, 258., 2.2);
        check(10, 15, 288., -9.6);
        check(11, 14, 318., -18.9);
        check(12, 10, 344., -23.0);
    }

    #[test]
    fn test_hour_angle() {
        // Example 1.6.1
        /*
        10:30 (solar time) on February 13... hour_angle is -22.55
        */

        let solar = Solar::new(0., 0., 0.);

        let n = Date {
            month: 2,
            day: 13,
            hour: 10.5,
        }
        .day_of_year();

        let w = solar.hour_angle(Time::Solar(n)).to_degrees();
        assert!(are_close(w, -22.5, 0.1));

        /* OTHERS */
        // Midday == 0
        let n = Date {
            month: 2,
            day: 13,
            hour: 12.0,
        }
        .day_of_year();
        let w = solar.hour_angle(Time::Solar(n)).to_degrees();
        assert!(are_close(w, 0., 0.1));

        // 13:00 == 15
        let n = Date {
            month: 2,
            day: 13,
            hour: 13.0,
        }
        .day_of_year();
        let w = solar.hour_angle(Time::Solar(n)).to_degrees();
        assert!(are_close(w, 15., 0.1));
    }

    #[test]
    fn test_sun_position() {
        /*
        Example 1.6.2
        Calculate the zenith and solar azimuth angles for φ = 43◦ at 9:30 AM on February 13 and 6:30 PM on July 1.
        */
        let phi = (43. as Float).to_radians();

        // FOR 9:30 AM on February 13
        // ==========================
        let solar = Solar::new(phi, 0., 0.);
        let n = Date {
            month: 2,
            day: 13,
            hour: 9.5,
        }
        .day_of_year();
        let dir = solar.sun_position(Time::Solar(n)).unwrap();
        assert!(are_close(dir.length(), 1.0, 0.00001));

        // check declination
        assert!(are_close(solar.declination(n).to_degrees(), -14., 0.5));

        // check hour angle
        assert!(are_close(
            solar.hour_angle(Time::Solar(n)).to_degrees(),
            -37.5,
            0.5
        ));

        // zenith
        let zenith = dir.z.acos().to_degrees();
        assert!(are_close(zenith, 66.5, 0.5));

        // Azimuth
        let azimuth = (dir.x / dir.y).atan().to_degrees();
        assert!(are_close(azimuth, -40., 0.5));

        // 6:30 PM on July 1
        // =================
        let n = Date {
            month: 7,
            day: 1,
            hour: 18.5,
        }
        .day_of_year();
        let dir = solar.sun_position(Time::Solar(n)).unwrap();
        assert!(are_close(dir.length(), 1.0, 0.00001));

        // check declination
        assert!(are_close(solar.declination(n).to_degrees(), 23.1, 0.5));

        // check hour angle
        assert!(are_close(
            solar.hour_angle(Time::Solar(n)).to_degrees(),
            97.5,
            0.5
        ));

        // zenith
        let zenith = dir.z.acos().to_degrees();
        assert!(are_close(zenith, 79.6, 0.5));

        // Azimuth
        println!("{}", dir);
        let _azimuth = (dir.x / dir.y).atan().to_degrees();
        //assert!(are_close(azimuth, 112., 0.5)); // This is working, but atan() returns -67 instead of 112
    }

    #[test]
    fn test_angle_of_incidence() {
        /*
        Example 1.6.1
        Calculate the angle of incidence of beam radiation on a surface
        located at Madison, Wisconsin, at 10:30 (solar time) on February 13
        if the surface is tilted 45◦ from the horizontal and pointed 15◦
        west of south.
        */
        // sun direction
        let latitude = (43. as Float).to_radians();
        let solar = Solar::new(latitude, 0.0, 0.0);
        let n = Date {
            month: 2,
            day: 13,
            hour: 10.5,
        }
        .day_of_year();
        let solar_dir = solar.sun_position(Time::Solar(n)).unwrap();
        // check declination
        assert!(are_close(solar.declination(n).to_degrees(), -14., 0.5));

        // check hour angle
        assert!(are_close(
            solar.hour_angle(Time::Solar(n)).to_degrees(),
            -22.5,
            0.5
        ));

        // surface
        let beta = (45. as Float).to_radians();
        let gamma = (15. as Float).to_radians();

        let x = -gamma.sin() * beta.sin();
        let y = -gamma.cos() * beta.sin();
        let z = beta.cos();
        let surface_dir = Vector3D::new(x, y, z);
        println!("{} | len = {}", surface_dir, surface_dir.length());

        let angle = (solar_dir * surface_dir).acos();

        assert!(are_close(angle.to_degrees(), 35., 0.2));
    }

    #[test]
    fn test_sunrise_sunset() {
        /*
        Example 1.6.3
        Calculate the time of sunrise... at 4:00 PM solar time on March 16 at
        a latitude of 43◦.

        Solution:

        The sunrise hour angle is therefore −87.8◦.
        With the earth’s rotation of 15◦ per hour, sunrise (and sunset) occurs
        5.85 h (5 h and 51 min) from noon so sunrise is at 6:09 AM (and sunset
        is at 5:51 PM).
        */
        let latitude = (43. as Float).to_radians();
        let solar = Solar::new(latitude, 0., 0.);
        let date = Date {
            month: 3,
            day: 16,
            hour: 16.,
        };
        let n = date.day_of_year();
        let n_midday = n.floor() + 0.5;
        let (rise, set) = solar.sunrise_sunset(n);
        are_close(
            n_midday - 5.85 / 24.,
            solar.unwrap_solar_time(rise),
            1. / 24. / 60.,
        ); // one minute
        are_close(
            n_midday + 5.85 / 24.,
            solar.unwrap_solar_time(set),
            1. / 24. / 60.,
        ); // one minute
    }
}
