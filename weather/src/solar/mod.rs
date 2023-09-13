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

use geometry::Vector3D;

mod reinhart_sky;
pub use reinhart_sky::ReinhartSky;
mod perez;
use super::CurrentWeather;
use super::{Float, PI};
use calendar::Date;
pub use perez::{PerezSky, SkyUnits};

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
#[derive(Clone, Copy, Debug)]
pub struct Solar {
    /// Latitude in Radians. South is negative, North is positive.
    latitude: Float,

    /// Longitude (in Radians). East is negative, West is positive
    ///
    /// > Note that this is Radiance's conventions, which is the opposite of EPW files.
    longitude: Float,

    /// Standard meridian (in Radians). East is negative, West is positive.
    ///
    /// This value is essentially (in degrees) `-15.0*TimeZone` (e.g., GMT+1 becomes -15.0)
    ///
    ///  > Note that this is Radiance's conventions, which is the opposite of EPW files.
    standard_meridian: Float,
}

/// W/m2
const SOLAR_CONSTANT: Float = 1367.7;

/// Solar or Standard time, containing the day of the year 'n'
#[derive(Debug, Clone, Copy)]
pub enum Time {
    /// Time is in Solar time    
    Solar(Float),

    /// Time is in Standard time    
    Standard(Float),
}

impl std::default::Default for Time {
    fn default() -> Self {
        Self::Standard(0.0)
    }
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
            + 0.001480 * (3. * b).sin()
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

    /// Calculates the sun position based on a date, assumed ot be in Standard Time
    pub fn sun_position_from_standard_time(&self, date: Date) -> Option<Vector3D> {
        let n = Time::Standard(date.day_of_year());
        self.sun_position(n)
    }

    /// Calculates the sun position based on a date, assumed ot be in Solar Time
    pub fn sun_position_from_solar_time(&self, date: Date) -> Option<Vector3D> {
        let n = Time::Solar(date.day_of_year());
        self.sun_position(n)
    }

    /// Builds a vector that points towards the sun. Returns `None`if
    /// the sun is below the horizon.
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

    /// Calculates the Beam Atmosphere Transmittance (i.e., the ratio between
    /// clear-sky beam radiation and extraterrestrial radiation)
    ///
    /// It uses Equation 2.8.1a of the book. We do not correct
    /// based on climate.
    ///
    /// > Note: elevation in m
    pub fn beam_atmosphere_transmittance(
        &self,
        sundir: Vector3D,
        mut site_elevation: Float,
    ) -> Float {
        let cos_theta = sundir.z;
        site_elevation /= 1000.0; // equation is in km

        let a0 = 0.4237 - 0.00821 * (6.0 - site_elevation).powi(2);
        let a1 = 0.5055 + 0.00595 * (6.5 - site_elevation).powi(2);
        let k = 0.2711 + 0.01858 * (2.5 - site_elevation).powi(2);

        a0 + a1 * (-k / cos_theta).exp()
    }

    /// Calculates the global horizontal clear-sky radiation
    /// using equation 2.8.1a and 2.8.5 of the book
    ///
    /// altitude in m
    pub fn clear_sky_global_horizontal_rad(
        &self,
        n: Time,
        sundir: Vector3D,
        site_elevation: Float,
    ) -> Float {
        let tb = self.beam_atmosphere_transmittance(sundir, site_elevation);

        // Equation 2.8.6
        let td = 0.271 - 0.294 * tb;

        // Calculate
        let tau = tb + td;
        let extra = self.normal_extraterrestrial_radiation(self.unwrap_solar_time(n));

        extra * tau * sundir.z
    }

    /// Returns the `(direct_normal, diffuse_horizontal, global_horizontal)` radiation estimated
    /// purely based on the cloud cover.
    ///
    /// Inputs are in Celsius, m/s, % in fractions (0 to 1)
    pub fn direct_diffuse_from_cloud_generic(
        &self,
        sun_direction: Vector3D,
        current_data: CurrentWeather,
        next_hour_data: Option<CurrentWeather>,
        prev_hour_data: Option<CurrentWeather>,
        three_hours_prior_data: CurrentWeather,
    ) -> (Float, Float, Float) {
        // let global_horizontal = self.cloud_cover_to_global_rad_generic(n, sun_direction, site_elevation, cloud_cover);
        let global_horizontal = self.estimate_global_horizontal_radiation(
            sun_direction,
            current_data.opaque_sky_cover,
            current_data.dry_bulb_temperature,
            three_hours_prior_data.dry_bulb_temperature,
            current_data.wind_speed,
            current_data.relative_humidity,
        );
        let cos_theta = sun_direction.z;

        let direct_normal_radiation = self.perez_direct_normal_radiation(
            sun_direction,
            current_data,
            next_hour_data,
            prev_hour_data,
        );

        let diffuse_horizontal = (global_horizontal - direct_normal_radiation * cos_theta).max(0.0);

        (
            direct_normal_radiation,
            diffuse_horizontal,
            global_horizontal,
        )
    }

    /// Estimates the Global Solar Horizontal Radiation
    ///
    /// Equation 2 from: Bre, F., Lawrie, L. K., Crawley, D. B., & Lamberts, R. (2021). Assessment of solar radiation data quality in typical meteorological years and its influence on the building performance simulation. Energy and Buildings, 250, 111251.
    ///
    /// Inputs are in Celsius, m/s, % in fractions (0 to 1)
    pub fn estimate_global_horizontal_radiation(
        &self,
        sun_direction: Vector3D,
        cloud_cover: Float,
        air_temperature: Float,
        air_three_hours_prior: Float,
        wind_speed: Float,
        mut relative_humidity: Float,
    ) -> Float {
        // The equation receives this in percentage... we are asking for fraction
        debug_assert!(relative_humidity >= 0.0);
        debug_assert!(relative_humidity <= 1.0);
        relative_humidity *= 100.0;

        debug_assert!(cloud_cover >= 0.0);
        debug_assert!(cloud_cover <= 1.0);

        /*  TABLE 1 */
        /*
        let (c0,c1,c2,c3,c4,c5,d) = match climate {
            Af => (
                0.97136, 0.24936, -0.32165, 0.03768, -0.0076, 0.00794, -2.23175,
            ),
            Am => (
                0.71868, -0.11359, -0.07259, 0.01038, -0.00285, 0.00866, -8.42023,
            ),
            As => (
                0.8089, 0.07355, -0.40101, -0.00424, -0.00242, 0.00342, -8.395,
            ),
            Aw => (
                0.8089, 0.07355, -0.40101, -0.00424, -0.00242, 0.00342, -8.395,
            ),
            BSh => (
                0.68149, -0.04697, -0.2842, 0.01726, -0.00081, 0.00453, -8.91306,
            ),
            Cfa => (
                0.67839, 0.03646, -0.39075, 0.01359, -0.00148, 0.0073, -8.71373,
            ),
            Cfb => (
                0.7437, -0.02988, -0.26353, 0.02606, -0.00323, -0.00008, -1.97366,
            ),
            Cwa => (
                0.68175, 0.15988, -0.43455, 0.01972, -0.00303, 0.00201, -6.67731,
            ),
            Cwb => (
                0.65533, -0.00683, -0.13621, 0.0324, -0.00252, -0.0032, 0.17022,
            ),
            AVERAGE => (
                0.749833333,
                0.043947778,
                -0.28806125,
                0.016512222,
                -0.0029925,
                0.003777778,
                -5.949946667,
            ),
        };
        */
        // average
        let (c0, c1, c2, c3, c4, c5, d) = (
            0.749833333,
            0.043947778,
            -0.28806125,
            0.016512222,
            -0.0029925,
            0.003777778,
            -5.949946667,
        );

        let sin_alpha = sun_direction.z;
        let rad = SOLAR_CONSTANT
            * sin_alpha
            * (c0
                + c1 * cloud_cover
                + c2 * cloud_cover.powi(2)
                + c3 * (air_temperature - air_three_hours_prior)
                + c4 * relative_humidity
                + c5 * wind_speed)
            + d;

        if rad > 0.0 {
            rad
        } else {
            0.0
        }
    }

    /// Calculates the hourly clearness index
    pub fn hourly_clearness_index(self, n: Time, global_normal_radiation: Float) -> Float {
        let extra = self.normal_extraterrestrial_radiation(self.unwrap_solar_time(n));
        global_normal_radiation / extra
    }

    /// Calculates an estimated solar position based on atmospheric pressure and
    /// the global normal radiation.
    ///
    /// Units are W/m2 and Pascals
    ///
    /// https://www.nrel.gov/grid/solar-resource/disc.html
    pub fn disc_direct_solar(
        &self,
        n: Time,
        sun_direction: Vector3D,
        global_normal_radiation: Float,
        pressure: Float,
    ) -> Float {
        let kt = self.hourly_clearness_index(n, global_normal_radiation);
        let extra_rad = global_normal_radiation / kt; // extraterrestrial;
                                                      // dbg!(extra_rad);
        if kt < 0.0 {
            return 0.0;
        }
        let kt = kt.clamp(0.0, 1.0);

        let solar_zenith = sun_direction.z.acos();
        // They check this in --> https://www.nrel.gov/grid/solar-resource/disc.html
        let air_mass = if solar_zenith > 80.0 * crate::PI / 180.0 {
            return 0.0;
        } else {
            air_mass(solar_zenith) * pressure / 101300.0
        };

        // A
        let a = if kt > 0.6 {
            -5.743 + 21.77 * kt - 27.49 * kt.powi(2) + 11.56 * kt.powi(3)
        } else {
            0.512 - 1.56 * kt + 2.286 * kt.powi(2) - 2.222 * kt.powi(3)
        };

        let b = if kt > 0.6 {
            41.4 - 118.5 * kt + 66.05 * kt.powi(2) + 31.9 * kt.powi(3)
        } else {
            0.37 + 0.962 * kt
        };

        let c = if kt > 0.6 {
            -47.01 + 184.2 * kt - 222.0 * kt.powi(2) + 73.81 * kt.powi(3)
        } else {
            -0.28 + 0.932 * kt - 2.048 * kt.powi(2)
        };

        let delta_kn = a + b * (c * air_mass).exp();

        let knc = 0.886 - 0.122 * air_mass + 0.0121 * (air_mass).powi(2)
            - 0.000653 * (air_mass).powi(3)
            + 0.000014 * air_mass.powi(4);

        let ret = extra_rad * (knc - delta_kn);
        if ret < 0.0 {
            0.0
        } else {
            ret
        }
    }

    /// Calculates the diffuse fraction from an hourly clearness index using Erb's correlation
    ///
    /// Equation 2.10.1 of the book
    pub fn diffuse_fraction(&self, kt: Float) -> Float {
        let ret = if kt <= 0.22 {
            1.0 - 0.09 * kt
        } else if kt > 0.8 {
            0.165
        } else {
            0.9511 - 0.1604 * kt + 4.388 * kt.powi(2) - 16.638 * kt.powi(3) + 12.336 * kt.powi(4)
        };
        debug_assert!(ret <= 1.0);
        debug_assert!(ret >= 0.0);
        ret
    }

    /// Calculates the direct normal radiation according to Perez's et al.
    /// Units are W/m2, C, Pa.
    ///
    /// `current_data`, `next_hour_data` and `prev_hour_data` are tuples containing
    /// horizontal_global_radiation and Time.
    ///
    /// > Note: this is not compatible with the `diffuse_fraction` function.
    ///
    /// Source: Ineichen, P., Perez, R. R., Seal, R. D., Maxwell, E. L., & Zalenka, A. J. A. T. (1992). Dynamic global-to-direct irradiance conversion models. ASHRAE transactions, 98(1), 354-369.    
    pub fn perez_direct_normal_radiation(
        &self,
        sun_direction: Vector3D,
        current_data: CurrentWeather,
        next_hour_data: Option<CurrentWeather>,
        prev_hour_data: Option<CurrentWeather>,
    ) -> Float {
        // Dimension 1:
        let cos_zenith = sun_direction.z;
        let zenith = cos_zenith.acos();

        // Eq 1.
        fn aux(kt: Float, m: Float) -> Float {
            kt / (1.031 * (-1.4 / (0.9 + 9.4 / m)).exp() + 0.1)
        }

        // Dimention 2
        let n = Time::Standard(current_data.date.day_of_year());
        let global_normal_radiation = current_data.global_horizontal_radiation / cos_zenith;
        let m = if zenith > 80.0 * crate::PI / 180.0 {
            return 0.0;
        } else {
            // air_mass(solar_zenith)
            air_mass(zenith) * current_data.pressure / 101300.0
        };
        let kt = self.hourly_clearness_index(n, global_normal_radiation);
        let kt = kt.clamp(0., 1.0);
        let kt_p = aux(kt, m);
        let kt_p = kt_p.clamp(0.0, 1.0);

        let next_kt_p = if let Some(next) = next_hour_data {
            let next_kt = self.hourly_clearness_index(
                Time::Standard(next.date.day_of_year()),
                next.global_horizontal_radiation / cos_zenith,
            );
            aux(next_kt, m)
        } else {
            kt_p
        };

        let prev_kt_p = if let Some(prev) = prev_hour_data {
            let prev_kt = self.hourly_clearness_index(
                Time::Standard(prev.date.day_of_year()),
                prev.global_horizontal_radiation / cos_zenith,
            );
            aux(prev_kt, m)
        } else {
            kt_p
        };

        // Dimension 3 - Eq 2
        let delta_kt_p = 0.5 * ((kt_p - next_kt_p).abs() + (kt_p - prev_kt_p).abs());
        let delta_kt_p = delta_kt_p.clamp(0.0, 1.0);

        // Dimention 4 - Eq 4
        let w = (0.07 * current_data.dew_point_temperature - 0.075).exp();

        let ktp_bin = perez_direct_rad_kt_bin(kt_p);
        let zeta_bin = perez_direct_rad_zeta_bin(zenith);
        let w_bin = perez_direct_rad_w_bin(w);
        let delta_ktp_bin = perez_direct_rad_delta_kt_bin(delta_kt_p);

        let x = perez_direct_rad_coefs(ktp_bin, zeta_bin, w_bin, delta_ktp_bin);

        x * self.disc_direct_solar(
            n,
            sun_direction,
            global_normal_radiation,
            current_data.pressure,
        )
    }
}

/// Table 1, upper
fn perez_direct_rad_kt_bin(kt_p: Float) -> usize {
    if !(0.0..=1.0).contains(&kt_p) {
        panic!("Perez KT coefficient out of bounds: {}", kt_p)
    } else if kt_p <= 0.24 {
        0
    } else if kt_p <= 0.4 {
        1
    } else if kt_p <= 0.56 {
        2
    } else if kt_p <= 0.7 {
        3
    } else if kt_p <= 0.8 {
        4
    } else if kt_p <= 1.0 {
        5
    } else {
        panic!("Perez KT coefficient out of bounds: {}", kt_p)
    }
}

/// Table 1, upper
fn perez_direct_rad_delta_kt_bin(delta_kt: Float) -> usize {
    if !(0.0..=1.0).contains(&delta_kt) {
        panic!("Perez Delta-KT coefficient out of bounds: {}", delta_kt)
    } else if delta_kt <= 0.015 {
        0
    } else if delta_kt <= 0.035 {
        1
    } else if delta_kt <= 0.070 {
        2
    } else if delta_kt <= 0.150 {
        3
    } else if delta_kt <= 0.300 {
        4
    } else if delta_kt <= 1.0 {
        5
    } else {
        panic!("Perez Delta-KT coefficient out of bounds: {}", delta_kt)
    }
}

/// Table 1, upper
fn perez_direct_rad_zeta_bin(zeta: Float) -> usize {
    if !(0.0..=crate::PI / 2.0).contains(&zeta) {
        panic!(
            "Perez Z coefficient out of bounds: {} ({} degrees)",
            zeta,
            zeta.to_degrees()
        )
    } else if zeta <= 25.0 * crate::PI / 180.0 {
        0
    } else if zeta <= 40.0 * crate::PI / 180.0 {
        1
    } else if zeta <= 55.0 * crate::PI / 180.0 {
        2
    } else if zeta <= 70.0 * crate::PI / 180.0 {
        3
    } else if zeta <= 80.0 * crate::PI / 180.0 {
        4
    } else if zeta <= 90.0 * crate::PI / 180.0 {
        5
    } else {
        panic!(
            "Perez Z coefficient out of bounds: {} ({} degrees)",
            zeta,
            zeta.to_degrees()
        )
    }
}

/// Table 1, upper
fn perez_direct_rad_w_bin(w: Float) -> usize {
    if w < 0.0 {
        panic!("Perez W coefficient out of bounds: {}", w)
    } else if w <= 1.0 {
        0
    } else if w <= 2.0 {
        1
    } else if w <= 3.0 {
        2
    } else {
        3
    }
}

/// Gets the coefficients to be used in equation 5 of /// Ineichen, P., Perez, R. R., Seal, R. D., Maxwell, E. L., & Zalenka, A. J. A. T. (1992). Dynamic global-to-direct irradiance conversion models. ASHRAE transactions, 98(1), 354-369.    
fn perez_direct_rad_coefs(kt_p: usize, zenith: usize, prec_water: usize, delta_kt: usize) -> Float {
    const C00: [Float; 35] = [
        0.385230, 0.385230, 0.385230, 0.462880, 0.317440, 0.338390, 0.338390, 0.221270, 0.316730,
        0.503650, 0.235680, 0.235680, 0.241280, 0.157830, 0.269440, 0.830130, 0.830130, 0.171970,
        0.841070, 0.457370, 0.548010, 0.548010, 0.478000, 0.966880, 1.036370, 0.548010, 0.548010,
        1.000000, 3.012370, 1.976540, 0.582690, 0.582690, 0.229720, 0.892710, 0.569950,
    ];

    const C01: [Float; 35] = [
        0.131280, 0.131280, 0.385460, 0.511070, 0.127940, 0.223710, 0.223710, 0.193560, 0.304560,
        0.193940, 0.229970, 0.229970, 0.275020, 0.312730, 0.244610, 0.090100, 0.184580, 0.260500,
        0.687480, 0.579440, 0.131530, 0.131530, 0.370190, 1.380350, 1.052270, 1.116250, 1.116250,
        0.928030, 3.525490, 2.316920, 0.090100, 0.237000, 0.300040, 0.812470, 0.664970,
    ];

    const C02: [Float; 35] = [
        0.587510, 0.130000, 0.400000, 0.537210, 0.832490, 0.306210, 0.129830, 0.204460, 0.500000,
        0.681640, 0.224020, 0.260620, 0.334080, 0.501040, 0.350470, 0.421540, 0.753970, 0.750660,
        3.706840, 0.983790, 0.706680, 0.373530, 1.245670, 0.864860, 1.992630, 4.864400, 0.117390,
        0.265180, 0.359180, 3.310820, 0.392080, 0.493290, 0.651560, 1.932780, 0.898730,
    ];

    const C03: [Float; 35] = [
        0.126970, 0.126970, 0.126970, 0.126970, 0.126970, 0.810820, 0.810820, 0.810820, 0.810820,
        0.810820, 3.241680, 2.500000, 2.291440, 2.291440, 2.291440, 4.000000, 3.000000, 2.000000,
        0.975430, 1.965570, 12.494170, 12.494170, 8.000000, 5.083520, 8.792390, 21.744240,
        21.744240, 21.744240, 21.744240, 21.744240, 3.241680, 12.494170, 1.620760, 1.375250,
        2.331620,
    ];

    const C04: [Float; 35] = [
        0.126970, 0.126970, 0.126970, 0.126970, 0.126970, 0.810820, 0.810820, 0.810820, 0.810820,
        0.810820, 3.241680, 2.500000, 2.291440, 2.291440, 2.291440, 4.000000, 3.000000, 2.000000,
        0.975430, 1.965570, 12.494170, 12.494170, 8.000000, 5.083520, 8.792390, 21.744240,
        21.744240, 21.744240, 21.744240, 21.744240, 3.241680, 12.494170, 1.620760, 1.375250,
        2.331620,
    ];

    const C05: [Float; 35] = [
        0.126970, 0.126970, 0.126970, 0.126970, 0.126970, 0.810820, 0.810820, 0.810820, 0.810820,
        0.810820, 3.241680, 2.500000, 2.291440, 2.291440, 2.291440, 4.000000, 3.000000, 2.000000,
        0.975430, 1.965570, 12.494170, 12.494170, 8.000000, 5.083520, 8.792390, 21.744240,
        21.744240, 21.744240, 21.744240, 21.744240, 3.241680, 12.494170, 1.620760, 1.375250,
        2.331620,
    ];

    const C10: [Float; 35] = [
        0.337440, 0.337440, 0.969110, 1.097190, 1.116080, 0.337440, 0.337440, 0.969110, 1.116030,
        0.623900, 0.337440, 0.337440, 1.530590, 1.024420, 0.908480, 0.584040, 0.584040, 0.847250,
        0.914940, 1.289300, 0.337440, 0.337440, 0.310240, 1.435020, 1.852830, 0.337440, 0.337440,
        1.015010, 1.097190, 2.117230, 0.337440, 0.337440, 0.969110, 1.145730, 1.476400,
    ];

    const C11: [Float; 35] = [
        0.300000, 0.300000, 0.700000, 1.100000, 0.796940, 0.219870, 0.219870, 0.526530, 0.809610,
        0.649300, 0.386650, 0.386650, 0.119320, 0.576120, 0.685460, 0.746730, 0.399830, 0.470970,
        0.986530, 0.785370, 0.575420, 0.936700, 1.649200, 1.495840, 1.335590, 1.319670, 4.002570,
        1.276390, 2.644550, 2.518670, 0.665190, 0.678910, 1.012360, 1.199940, 0.986580,
    ];

    const C12: [Float; 35] = [
        0.378870, 0.974060, 0.500000, 0.491880, 0.665290, 0.105210, 0.263470, 0.407040, 0.553460,
        0.582590, 0.312900, 0.345240, 1.144180, 0.854790, 0.612280, 0.119070, 0.365120, 0.560520,
        0.793720, 0.802600, 0.781610, 0.837390, 1.270420, 1.537980, 1.292950, 1.152290, 1.152290,
        1.492080, 1.245370, 2.177100, 0.424660, 0.529550, 0.966910, 1.033460, 0.958730,
    ];

    const C13: [Float; 35] = [
        0.310590, 0.714410, 0.252450, 0.500000, 0.607600, 0.975190, 0.363420, 0.500000, 0.400000,
        0.502800, 0.175580, 0.196250, 0.476360, 1.072470, 0.490510, 0.719280, 0.698620, 0.657770,
        1.190840, 0.681110, 0.426240, 1.464840, 0.678550, 1.157730, 0.978430, 2.501120, 1.789130,
        1.387090, 2.394180, 2.394180, 0.491640, 0.677610, 0.685610, 1.082400, 0.735410,
    ];

    const C14: [Float; 35] = [
        0.597000, 0.500000, 0.300000, 0.310050, 0.413510, 0.314790, 0.336310, 0.400000, 0.400000,
        0.442460, 0.166510, 0.460440, 0.552570, 1.000000, 0.461610, 0.401020, 0.559110, 0.403630,
        1.016710, 0.671490, 0.400360, 0.750830, 0.842640, 1.802600, 1.023830, 3.315300, 1.510380,
        2.443650, 1.638820, 2.133990, 0.530790, 0.745850, 0.693050, 1.458040, 0.804500,
    ];

    const C15: [Float; 35] = [
        0.597000, 0.500000, 0.300000, 0.310050, 0.800920, 0.314790, 0.336310, 0.400000, 0.400000,
        0.237040, 0.166510, 0.460440, 0.552570, 1.000000, 0.581990, 0.401020, 0.559110, 0.403630,
        1.016710, 0.898570, 0.400360, 0.750830, 0.842640, 1.802600, 3.400390, 3.315300, 1.510380,
        2.443650, 1.638820, 2.508780, 0.204340, 1.157740, 2.003080, 2.622080, 1.409380,
    ];

    const C20: [Float; 35] = [
        1.242210, 1.242210, 1.242210, 1.242210, 1.242210, 0.056980, 0.056980, 0.656990, 0.656990,
        0.925160, 0.089090, 0.089090, 1.040430, 1.232480, 1.205300, 1.053850, 1.053850, 1.399690,
        1.084640, 1.233340, 1.151540, 1.151540, 1.118290, 1.531640, 1.411840, 1.494980, 1.494980,
        1.700000, 1.800810, 1.671600, 1.018450, 1.018450, 1.153600, 1.321890, 1.294670,
    ];

    const C21: [Float; 35] = [
        0.700000, 0.700000, 1.023460, 0.700000, 0.945830, 0.886300, 0.886300, 1.333620, 0.800000,
        1.066620, 0.902180, 0.902180, 0.954330, 1.126690, 1.097310, 1.095300, 1.075060, 1.176490,
        1.139470, 1.096110, 1.201660, 1.201660, 1.438200, 1.256280, 1.198060, 1.525850, 1.525850,
        1.869160, 1.985410, 1.911590, 1.288220, 1.082810, 1.286370, 1.166170, 1.119330,
    ];

    const C22: [Float; 35] = [
        0.600000, 1.029910, 0.859890, 0.550000, 0.813600, 0.604450, 1.029910, 0.859890, 0.656700,
        0.928840, 0.455850, 0.750580, 0.804930, 0.823000, 0.911000, 0.526580, 0.932310, 0.908620,
        0.983520, 0.988090, 1.036110, 1.100690, 0.848380, 1.035270, 1.042380, 1.048440, 1.652720,
        0.900000, 2.350410, 1.082950, 0.817410, 0.976160, 0.861300, 0.974780, 1.004580,
    ];

    const C23: [Float; 35] = [
        0.782110, 0.564280, 0.600000, 0.600000, 0.665740, 0.894480, 0.680730, 0.541990, 0.800000,
        0.669140, 0.487460, 0.818950, 0.841830, 0.872540, 0.709040, 0.709310, 0.872780, 0.908480,
        0.953290, 0.844350, 0.863920, 0.947770, 0.876220, 1.078750, 0.936910, 1.280350, 0.866720,
        0.769790, 1.078750, 0.975130, 0.725420, 0.869970, 0.868810, 0.951190, 0.829220,
    ];

    const C24: [Float; 35] = [
        0.791750, 0.654040, 0.483170, 0.409000, 0.597180, 0.566140, 0.948990, 0.971820, 0.653570,
        0.718550, 0.648710, 0.637730, 0.870510, 0.860600, 0.694300, 0.637630, 0.767610, 0.925670,
        0.990310, 0.847670, 0.736380, 0.946060, 1.117590, 1.029340, 0.947020, 1.180970, 0.850000,
        1.050000, 0.950000, 0.888580, 0.700560, 0.801440, 0.961970, 0.906140, 0.823880,
    ];

    const C25: [Float; 35] = [
        0.500000, 0.500000, 0.586770, 0.470550, 0.629790, 0.500000, 0.500000, 1.056220, 1.260140,
        0.658140, 0.500000, 0.500000, 0.631830, 0.842620, 0.582780, 0.554710, 0.734730, 0.985820,
        0.915640, 0.898260, 0.712510, 1.205990, 0.909510, 1.078260, 0.885610, 1.899260, 1.559710,
        1.000000, 1.150000, 1.120390, 0.653880, 0.793120, 0.903320, 0.944070, 0.796130,
    ];

    const C30: [Float; 35] = [
        1.000000, 1.000000, 1.050000, 1.170380, 1.178090, 0.960580, 0.960580, 1.059530, 1.179030,
        1.131690, 0.871470, 0.871470, 0.995860, 1.141910, 1.114600, 1.201590, 1.201590, 0.993610,
        1.109380, 1.126320, 1.065010, 1.065010, 0.828660, 0.939970, 1.017930, 1.065010, 1.065010,
        0.623690, 1.119620, 1.132260, 1.071570, 1.071570, 0.958070, 1.114130, 1.127110,
    ];

    const C31: [Float; 35] = [
        0.950000, 0.973390, 0.852520, 1.092200, 1.096590, 0.804120, 0.913870, 0.980990, 1.094580,
        1.042420, 0.737540, 0.935970, 0.999940, 1.056490, 1.050060, 1.032980, 1.034540, 0.968460,
        1.032080, 1.015780, 0.900000, 0.977210, 0.945960, 1.008840, 0.969960, 0.600000, 0.750000,
        0.750000, 0.844710, 0.899100, 0.926800, 0.965030, 0.968520, 1.044910, 1.032310,
    ];

    const C32: [Float; 35] = [
        0.850000, 1.029710, 0.961100, 1.055670, 1.009700, 0.818530, 0.960010, 0.996450, 1.081970,
        1.036470, 0.765380, 0.953500, 0.948260, 1.052110, 1.000140, 0.775610, 0.909610, 0.927800,
        0.987800, 0.952100, 1.000990, 0.881880, 0.875950, 0.949100, 0.893690, 0.902370, 0.875960,
        0.807990, 0.942410, 0.917920, 0.856580, 0.928270, 0.946820, 1.032260, 0.972990,
    ];

    const C33: [Float; 35] = [
        0.750000, 0.857930, 0.983800, 1.056540, 0.980240, 0.750000, 0.987010, 1.013730, 1.133780,
        1.038250, 0.800000, 0.947380, 1.012380, 1.091270, 0.999840, 0.800000, 0.914550, 0.908570,
        0.999190, 0.915230, 0.778540, 0.800590, 0.799070, 0.902180, 0.851560, 0.680190, 0.317410,
        0.507680, 0.388910, 0.646710, 0.794920, 0.912780, 0.960830, 1.057110, 0.947950,
    ];

    const C34: [Float; 35] = [
        0.750000, 0.833890, 0.867530, 1.059890, 0.932840, 0.979700, 0.971470, 0.995510, 1.068490,
        1.030150, 0.858850, 0.987920, 1.043220, 1.108700, 1.044900, 0.802400, 0.955110, 0.911660,
        1.045070, 0.944470, 0.884890, 0.766210, 0.885390, 0.859070, 0.818190, 0.615680, 0.700000,
        0.850000, 0.624620, 0.669300, 0.835570, 0.946150, 0.977090, 1.049350, 0.979970,
    ];

    const C35: [Float; 35] = [
        0.689220, 0.809600, 0.900000, 0.789500, 0.853990, 0.854660, 0.852840, 0.938200, 0.923110,
        0.955010, 0.938600, 0.932980, 1.010390, 1.043950, 1.041640, 0.843620, 0.981300, 0.951590,
        0.946100, 0.966330, 0.694740, 0.814690, 0.572650, 0.400000, 0.726830, 0.211370, 0.671780,
        0.416340, 0.297290, 0.498050, 0.843540, 0.882330, 0.911760, 0.898420, 0.960210,
    ];

    const C40: [Float; 35] = [
        1.054880, 1.075210, 1.068460, 1.153370, 1.069220, 1.000000, 1.062220, 1.013470, 1.088170,
        1.046200, 0.885090, 0.993530, 0.942590, 1.054990, 1.012740, 0.920000, 0.950000, 0.978720,
        1.020280, 0.984440, 0.850000, 0.908500, 0.839940, 0.985570, 0.962180, 0.800000, 0.800000,
        0.810080, 0.950000, 0.961550, 1.038590, 1.063200, 1.034440, 1.112780, 1.037800,
    ];

    const C41: [Float; 35] = [
        1.017610, 1.028360, 1.058960, 1.133180, 1.045620, 0.920000, 0.998970, 1.033590, 1.089030,
        1.022060, 0.912370, 0.949930, 0.979770, 1.020420, 0.981770, 0.847160, 0.935300, 0.930540,
        0.955050, 0.946560, 0.880260, 0.867110, 0.874130, 0.972650, 0.883420, 0.627150, 0.627150,
        0.700000, 0.774070, 0.845130, 0.973700, 1.006240, 1.026190, 1.071960, 1.017240,
    ];

    const C42: [Float; 35] = [
        1.028710, 1.017570, 1.025900, 1.081790, 1.024240, 0.924980, 0.985500, 1.014100, 1.092210,
        0.999610, 0.828570, 0.934920, 0.994950, 1.024590, 0.949710, 0.900810, 0.901330, 0.928830,
        0.979570, 0.913100, 0.761030, 0.845150, 0.805360, 0.936790, 0.853460, 0.626400, 0.546750,
        0.730500, 0.850000, 0.689050, 0.957630, 0.985480, 0.991790, 1.050220, 0.987900,
    ];

    const C43: [Float; 35] = [
        0.992730, 0.993880, 1.017150, 1.059120, 1.017450, 0.975610, 0.987160, 1.026820, 1.075440,
        1.007250, 0.871090, 0.933190, 0.974690, 0.979840, 0.952730, 0.828750, 0.868090, 0.834920,
        0.905510, 0.871530, 0.781540, 0.782470, 0.767910, 0.764140, 0.795890, 0.743460, 0.693390,
        0.514870, 0.630150, 0.715660, 0.934760, 0.957870, 0.959640, 0.972510, 0.981640,
    ];

    const C44: [Float; 35] = [
        0.965840, 0.941240, 0.987100, 1.022540, 1.011160, 0.988630, 0.994770, 0.976590, 0.950000,
        1.034840, 0.958200, 1.018080, 0.974480, 0.920000, 0.989870, 0.811720, 0.869090, 0.812020,
        0.850000, 0.821050, 0.682030, 0.679480, 0.632450, 0.746580, 0.738550, 0.668290, 0.445860,
        0.500000, 0.678920, 0.696510, 0.926940, 0.953350, 0.959050, 0.876210, 0.991490,
    ];

    const C45: [Float; 35] = [
        0.948940, 0.997760, 0.850000, 0.826520, 0.998470, 1.017860, 0.970000, 0.850000, 0.700000,
        0.988560, 1.000000, 0.950000, 0.850000, 0.606240, 0.947260, 1.000000, 0.746140, 0.751740,
        0.598390, 0.725230, 0.922210, 0.500000, 0.376800, 0.517110, 0.548630, 0.500000, 0.450000,
        0.429970, 0.404490, 0.539940, 0.960430, 0.881630, 0.775640, 0.596350, 0.937680,
    ];

    const C50: [Float; 35] = [
        1.030000, 1.040000, 1.000000, 1.000000, 1.049510, 1.050000, 0.990000, 0.990000, 0.950000,
        0.996530, 1.050000, 0.990000, 0.990000, 0.820000, 0.971940, 1.050000, 0.790000, 0.880000,
        0.820000, 0.951840, 1.000000, 0.530000, 0.440000, 0.710000, 0.928730, 0.540000, 0.470000,
        0.500000, 0.550000, 0.773950, 1.038270, 0.920180, 0.910930, 0.821140, 1.034560,
    ];

    const C51: [Float; 35] = [
        1.041020, 0.997520, 0.961600, 1.000000, 1.035780, 0.948030, 0.980000, 0.900000, 0.950360,
        0.977460, 0.950000, 0.977250, 0.869270, 0.800000, 0.951680, 0.951870, 0.850000, 0.748770,
        0.700000, 0.883850, 0.900000, 0.823190, 0.727450, 0.600000, 0.839870, 0.850000, 0.805020,
        0.692310, 0.500000, 0.788410, 1.010090, 0.895270, 0.773030, 0.816280, 1.011680,
    ];

    const C52: [Float; 35] = [
        1.022450, 1.004600, 0.983650, 1.000000, 1.032940, 0.943960, 0.999240, 0.983920, 0.905990,
        0.978150, 0.936240, 0.946480, 0.850000, 0.850000, 0.930320, 0.816420, 0.885000, 0.644950,
        0.817650, 0.865310, 0.742960, 0.765690, 0.561520, 0.700000, 0.827140, 0.643870, 0.596710,
        0.474460, 0.600000, 0.651200, 0.971740, 0.940560, 0.714880, 0.864380, 1.001650,
    ];

    const C53: [Float; 35] = [
        0.995260, 0.977010, 1.000000, 1.000000, 1.035250, 0.939810, 0.975250, 0.939980, 0.950000,
        0.982550, 0.876870, 0.879440, 0.850000, 0.900000, 0.917810, 0.873480, 0.873450, 0.751470,
        0.850000, 0.863040, 0.761470, 0.702360, 0.638770, 0.750000, 0.783120, 0.734080, 0.650000,
        0.600000, 0.650000, 0.715660, 0.942160, 0.919100, 0.770340, 0.731170, 0.995180,
    ];

    const C54: [Float; 35] = [
        0.952560, 0.916780, 0.920000, 0.900000, 1.005880, 0.928620, 0.994420, 0.900000, 0.900000,
        0.983720, 0.913070, 0.850000, 0.850000, 0.800000, 0.924280, 0.868090, 0.807170, 0.823550,
        0.600000, 0.844520, 0.769570, 0.719870, 0.650000, 0.550000, 0.733500, 0.580250, 0.650000,
        0.600000, 0.500000, 0.628850, 0.904770, 0.852650, 0.708370, 0.493730, 0.949030,
    ];

    const C55: [Float; 35] = [
        0.911970, 0.800000, 0.800000, 0.800000, 0.956320, 0.912620, 0.316930, 0.750000, 0.700000,
        0.950110, 0.653450, 0.659330, 0.700000, 0.600000, 0.856110, 0.648440, 0.600000, 0.641120,
        0.500000, 0.695780, 0.570000, 0.550000, 0.598800, 0.400000, 0.560150, 0.475230, 0.500000,
        0.518640, 0.339970, 0.520230, 0.743440, 0.592190, 0.603060, 0.316930, 0.794390,
    ];

    let i = prec_water * 5 + delta_kt;

    match (kt_p, zenith) {
        (0, 0) => C00[i],
        (0, 1) => C01[i],
        (0, 2) => C02[i],
        (0, 3) => C03[i],
        (0, 4) => C04[i],
        (0, 5) => C05[i],

        (1, 0) => C10[i],
        (1, 1) => C11[i],
        (1, 2) => C12[i],
        (1, 3) => C13[i],
        (1, 4) => C14[i],
        (1, 5) => C15[i],

        (2, 0) => C20[i],
        (2, 1) => C21[i],
        (2, 2) => C22[i],
        (2, 3) => C23[i],
        (2, 4) => C24[i],
        (2, 5) => C25[i],

        (3, 0) => C30[i],
        (3, 1) => C31[i],
        (3, 2) => C32[i],
        (3, 3) => C33[i],
        (3, 4) => C34[i],
        (3, 5) => C35[i],

        (4, 0) => C40[i],
        (4, 1) => C41[i],
        (4, 2) => C42[i],
        (4, 3) => C43[i],
        (4, 4) => C44[i],
        (4, 5) => C45[i],

        (5, 0) => C50[i],
        (5, 1) => C51[i],
        (5, 2) => C52[i],
        (5, 3) => C53[i],
        (5, 4) => C54[i],
        (5, 5) => C55[i],

        _ => panic!("Index {:?} out of range", (kt_p, zenith)),
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{EPWWeather, Weather};
    use calendar::{Date, Period};
    use validate::{valid, ScatterValidator, ValidFunc, Validator};

    #[test]
    fn test_cloud_cover_to_global_rad_generic() -> Result<(), String> {
        fn global(filename: &str) -> Result<ValidFunc, String> {
            let mut epw: Weather = EPWWeather::from_file(filename)?.into();

            let expected: Vec<Float> = epw
                .data
                .iter()
                .map(|line| line.global_horizontal_radiation)
                .collect();

            epw.fill_solar_radiation_data()?;

            let found: Vec<Float> = epw
                .data
                .iter()
                .map(|line| line.global_horizontal_radiation)
                .collect();

            let scatter = ScatterValidator {
                units: Some("W/m2"),
                expected_legend: Some("EPW Global Solar Radiation"),
                found_legend: Some("Calculated"),
                expected,
                found,
                ..ScatterValidator::default()
            };
            Ok(Box::new(scatter))
        }

        fn direct(filename: &str) -> Result<ValidFunc, String> {
            let mut epw: Weather = EPWWeather::from_file(filename)?.into();

            let expected: Vec<Float> = epw
                .data
                .iter()
                .map(|line| line.direct_normal_radiation)
                .collect();

            epw.fill_solar_radiation_data()?;

            let found: Vec<Float> = epw
                .data
                .iter()
                .map(|line| line.direct_normal_radiation)
                .collect();

            let scatter = ScatterValidator {
                units: Some("W/m2"),
                expected_legend: Some("EPW Direct Normal Solar Radiation"),
                found_legend: Some("Calculated"),
                expected,
                found,
                ..ScatterValidator::default()
            };
            Ok(Box::new(scatter))
        }

        fn diffuse(filename: &str) -> Result<ValidFunc, String> {
            let mut epw: Weather = EPWWeather::from_file(filename)?.into();

            let expected: Vec<Float> = epw
                .data
                .iter()
                .map(|line| line.diffuse_horizontal_radiation)
                .collect();

            epw.fill_solar_radiation_data()?;

            let found: Vec<Float> = epw
                .data
                .iter()
                .map(|line| line.diffuse_horizontal_radiation)
                .collect();

            let scatter = ScatterValidator {
                units: Some("W/m2"),
                expected_legend: Some("EPW Diffuse Horizontal Solar Radiation"),
                found_legend: Some("Calculated"),
                expected,
                found,
                ..ScatterValidator::default()
            };
            Ok(Box::new(scatter))
        }

        #[valid("Global Horizontal Radiation: Wellington")]
        /// Estimated based on cloud cover (in EPW file) vs global radiation (in EPW file) for Wellington
        fn wellington_global() -> Result<ValidFunc, String> {
            global("./test_data/wellington.epw")
        }
        #[valid("Global Horizontal Radiation: Barcelona")]
        /// Estimated based on cloud cover (in EPW file) vs global radiation (in EPW file) for Barcelona
        fn barcelona_global() -> Result<ValidFunc, String> {
            global("./test_data/barcelona.epw")
        }

        #[valid("Diffuse Horizontal Radiation: Wellington")]
        ///Estimated based on cloud cover (in EPW file) vs global radiation (in EPW file) for Wellington
        fn wellington_diffuse() -> Result<ValidFunc, String> {
            diffuse("./test_data/wellington.epw")
        }
        #[valid("Diffuse Horizontal Radiation: Barcelona")]
        /// Estimated based on cloud cover (in EPW file) vs global radiation (in EPW file) for Barcelona
        fn barcelona_diffuse() -> Result<ValidFunc, String> {
            diffuse("./test_data/barcelona.epw")
        }

        #[valid("Direct Normal Radiation: Wellington")]
        ///Estimated based on cloud cover (in EPW file) vs global radiation (in EPW file) for Wellington
        fn wellington_direct() -> Result<ValidFunc, String> {
            direct("./test_data/wellington.epw")
        }
        #[valid("Direct Normal Radiation: Barcelona")]
        /// Estimated based on cloud cover (in EPW file) vs global radiation (in EPW file) for Barcelona
        fn barcelona_direct() -> Result<ValidFunc, String> {
            direct("./test_data/barcelona.epw")
        }

        let mut validator = Validator::new(
            "Estimation of Global Horizontal Irradiance from Cloud Cover",
            "../docs/validation/global_horizontal_from_cloud_cover.html",
        );
        validator.push(wellington_global()?);
        validator.push(barcelona_global()?);

        validator.push(wellington_diffuse()?);
        validator.push(barcelona_diffuse()?);

        validator.push(wellington_direct()?);
        validator.push(barcelona_direct()?);

        validator.validate()
    }

    #[test]
    fn test_atmosphere_transmittance() -> Result<(), String> {
        // Example 2.8.1
        /*
        Calculate the transmittance for beam radiation of the standard clear
        atmosphere at Madison (altitude 270 m) on August 22 at 11:30AMsolar time.
        Estimate the intensity of beamradiation at that time and its component on
        a horizontal surface.
        */
        let site_elevation = 270.0;
        let date = Date {
            month: 8,
            day: 22,
            hour: 11.5,
        };

        let n = date.day_of_year();
        validate::assert_close!(n, 233.479, 0.01);
        let n = Time::Solar(n);
        let lat = (43.0730556 as Float).to_radians();
        let lon = (-89.40111 as Float).to_radians();

        let stdmed = 20.0; // we are working in solar time, so it is not relevant
        let solar = Solar::new(lat, lon, stdmed);

        let sundir = solar.sun_position(n).ok_or("No sun position")?;
        validate::assert_close!(sundir.z, 0.846, 0.01);

        let t = solar.beam_atmosphere_transmittance(sundir, site_elevation);
        validate::assert_close!(t, 0.62, 0.015);

        let extra = solar.normal_extraterrestrial_radiation(solar.unwrap_solar_time(n));
        validate::assert_close!(extra, 1339.0, 5.0);
        let beam_clear_sky_irr = extra * t * sundir.z;
        validate::assert_close!(beam_clear_sky_irr, 702.0, 22.0);

        // Example 2.8.2
        /*
        Estimate the standard clear-day radiation on a horizontal surface
        for Madison onAugust 22
        */
        let clear = solar.clear_sky_global_horizontal_rad(n, sundir, site_elevation);
        validate::assert_close!(clear, 702.0 + 101.0, 20.0);

        Ok(())
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
        validate::assert_close!(solar_time.hour, 10.0 + 19.0 / 60.0, EPS);

        // Solar to standard
        let standard_n_2 = solar.unwrap_standard_time(Time::Solar(solar_n));
        validate::assert_close!(standard_n, standard_n_2, EPS);
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
            validate::assert_close!(d.to_degrees(), expected_d, 1.8);
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
        validate::assert_close!(w, -22.5, 0.1);

        /* OTHERS */
        // Midday == 0
        let n = Date {
            month: 2,
            day: 13,
            hour: 12.0,
        }
        .day_of_year();
        let w = solar.hour_angle(Time::Solar(n)).to_degrees();
        validate::assert_close!(w, 0., 0.1);

        // 13:00 == 15
        let n = Date {
            month: 2,
            day: 13,
            hour: 13.0,
        }
        .day_of_year();
        let w = solar.hour_angle(Time::Solar(n)).to_degrees();
        validate::assert_close!(w, 15., 0.1);
    }

    #[test]
    fn test_sun_position() -> Result<(), String> {
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
        let dir = solar
            .sun_position(Time::Solar(n))
            .ok_or("No sun position")?;
        validate::assert_close!(dir.length(), 1.0, 0.00001);

        // check declination
        validate::assert_close!(solar.declination(n).to_degrees(), -14., 0.5);

        // check hour angle
        validate::assert_close!(solar.hour_angle(Time::Solar(n)).to_degrees(), -37.5, 0.5);

        // zenith
        let zenith = dir.z.acos().to_degrees();
        validate::assert_close!(zenith, 66.5, 0.5);

        // Azimuth
        let azimuth = (dir.x / dir.y).atan().to_degrees();
        validate::assert_close!(azimuth, -40., 0.5);

        // 6:30 PM on July 1
        // =================
        let n = Date {
            month: 7,
            day: 1,
            hour: 18.5,
        }
        .day_of_year();
        let dir = solar
            .sun_position(Time::Solar(n))
            .ok_or("No sun position 2")?;
        validate::assert_close!(dir.length(), 1.0, 0.00001);

        // check declination
        validate::assert_close!(solar.declination(n).to_degrees(), 23.1, 0.5);

        // check hour angle
        validate::assert_close!(solar.hour_angle(Time::Solar(n)).to_degrees(), 97.5, 0.5);

        // zenith
        let zenith = dir.z.acos().to_degrees();
        validate::assert_close!(zenith, 79.6, 0.5);

        // Azimuth
        let azimuth = (dir.x / dir.y).atan().to_degrees();
        validate::assert_close!(180.0 + azimuth, 112., 0.5); // This is working, but atan() returns -67 instead of 112

        Ok(())
    }

    #[test]
    fn test_angle_of_incidence() -> Result<(), String> {
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
        let solar_dir = solar
            .sun_position(Time::Solar(n))
            .ok_or("No sun position")?;
        // check declination
        validate::assert_close!(solar.declination(n).to_degrees(), -14., 0.5);

        // check hour angle
        validate::assert_close!(solar.hour_angle(Time::Solar(n)).to_degrees(), -22.5, 0.5);

        // surface
        let beta = (45. as Float).to_radians();
        let gamma = (15. as Float).to_radians();

        let x = -gamma.sin() * beta.sin();
        let y = -gamma.cos() * beta.sin();
        let z = beta.cos();
        let surface_dir = Vector3D::new(x, y, z);
        println!("{} | len = {}", surface_dir, surface_dir.length());

        let angle = (solar_dir * surface_dir).acos();

        validate::assert_close!(angle.to_degrees(), 35., 0.2);

        Ok(())
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
        validate::assert_close!(
            n_midday - 5.85 / 24.,
            solar.unwrap_solar_time(rise),
            1. / 24. / 60.0
        ); // one minute
        validate::assert_close!(
            n_midday + 5.85 / 24.,
            solar.unwrap_solar_time(set),
            1. / 24. / 60.
        ); // one minute
    }

    #[test]
    fn test_perez_direct_rad_coefs() {
        // 00
        validate::assert_close!(0.385230, perez_direct_rad_coefs(0, 0, 0, 0));
        validate::assert_close!(0.338390, perez_direct_rad_coefs(0, 0, 1, 1));
        validate::assert_close!(0.892710, perez_direct_rad_coefs(0, 0, 6, 3));
        // 01
        validate::assert_close!(0.131280, perez_direct_rad_coefs(0, 1, 0, 0));
        validate::assert_close!(0.223710, perez_direct_rad_coefs(0, 1, 1, 1));
        validate::assert_close!(0.812470, perez_direct_rad_coefs(0, 1, 6, 3));
        // 02
        validate::assert_close!(0.587510, perez_direct_rad_coefs(0, 2, 0, 0));
        validate::assert_close!(0.129830, perez_direct_rad_coefs(0, 2, 1, 1));
        validate::assert_close!(1.932780, perez_direct_rad_coefs(0, 2, 6, 3));
        // 03
        validate::assert_close!(0.126970, perez_direct_rad_coefs(0, 3, 0, 0));
        validate::assert_close!(0.810820, perez_direct_rad_coefs(0, 3, 1, 1));
        validate::assert_close!(1.375250, perez_direct_rad_coefs(0, 3, 6, 3));
        // O4
        validate::assert_close!(0.126970, perez_direct_rad_coefs(0, 4, 0, 0));
        validate::assert_close!(0.810820, perez_direct_rad_coefs(0, 4, 1, 1));
        validate::assert_close!(1.375250, perez_direct_rad_coefs(0, 4, 6, 3));
        // 05
        validate::assert_close!(0.126970, perez_direct_rad_coefs(0, 5, 0, 0));
        validate::assert_close!(0.810820, perez_direct_rad_coefs(0, 5, 1, 1));
        validate::assert_close!(1.375250, perez_direct_rad_coefs(0, 5, 6, 3));

        // 10
        validate::assert_close!(0.337440, perez_direct_rad_coefs(1, 0, 0, 0));
        validate::assert_close!(0.337440, perez_direct_rad_coefs(1, 0, 1, 1));
        validate::assert_close!(1.145730, perez_direct_rad_coefs(1, 0, 6, 3));
        // 11
        validate::assert_close!(0.300000, perez_direct_rad_coefs(1, 1, 0, 0));
        validate::assert_close!(0.219870, perez_direct_rad_coefs(1, 1, 1, 1));
        validate::assert_close!(1.199940, perez_direct_rad_coefs(1, 1, 6, 3));
        // 12
        validate::assert_close!(0.378870, perez_direct_rad_coefs(1, 2, 0, 0));
        validate::assert_close!(0.263470, perez_direct_rad_coefs(1, 2, 1, 1));
        validate::assert_close!(1.033460, perez_direct_rad_coefs(1, 2, 6, 3));
        // 13
        validate::assert_close!(0.310590, perez_direct_rad_coefs(1, 3, 0, 0));
        validate::assert_close!(0.363420, perez_direct_rad_coefs(1, 3, 1, 1));
        validate::assert_close!(1.082400, perez_direct_rad_coefs(1, 3, 6, 3));
        // 14
        validate::assert_close!(0.597000, perez_direct_rad_coefs(1, 4, 0, 0));
        validate::assert_close!(0.336310, perez_direct_rad_coefs(1, 4, 1, 1));
        validate::assert_close!(1.458040, perez_direct_rad_coefs(1, 4, 6, 3));
        // 15
        validate::assert_close!(0.597000, perez_direct_rad_coefs(1, 5, 0, 0));
        validate::assert_close!(0.336310, perez_direct_rad_coefs(1, 5, 1, 1));
        validate::assert_close!(2.622080, perez_direct_rad_coefs(1, 5, 6, 3));

        // 20
        validate::assert_close!(1.242210, perez_direct_rad_coefs(2, 0, 0, 0));
        validate::assert_close!(0.056980, perez_direct_rad_coefs(2, 0, 1, 1));
        validate::assert_close!(1.321890, perez_direct_rad_coefs(2, 0, 6, 3));
        // 21
        validate::assert_close!(0.700000, perez_direct_rad_coefs(2, 1, 0, 0));
        validate::assert_close!(0.886300, perez_direct_rad_coefs(2, 1, 1, 1));
        validate::assert_close!(1.166170, perez_direct_rad_coefs(2, 1, 6, 3));
        // 22
        validate::assert_close!(0.600000, perez_direct_rad_coefs(2, 2, 0, 0));
        validate::assert_close!(1.029910, perez_direct_rad_coefs(2, 2, 1, 1));
        validate::assert_close!(0.974780, perez_direct_rad_coefs(2, 2, 6, 3));
        // 23
        validate::assert_close!(0.782110, perez_direct_rad_coefs(2, 3, 0, 0));
        validate::assert_close!(0.680730, perez_direct_rad_coefs(2, 3, 1, 1));
        validate::assert_close!(0.951190, perez_direct_rad_coefs(2, 3, 6, 3));
        // 24
        validate::assert_close!(0.791750, perez_direct_rad_coefs(2, 4, 0, 0));
        validate::assert_close!(0.948990, perez_direct_rad_coefs(2, 4, 1, 1));
        validate::assert_close!(0.906140, perez_direct_rad_coefs(2, 4, 6, 3));
        // 25
        validate::assert_close!(0.500000, perez_direct_rad_coefs(2, 5, 0, 0));
        validate::assert_close!(0.500000, perez_direct_rad_coefs(2, 5, 1, 1));
        validate::assert_close!(0.944070, perez_direct_rad_coefs(2, 5, 6, 3));

        // 30
        validate::assert_close!(1.000000, perez_direct_rad_coefs(3, 0, 0, 0));
        validate::assert_close!(0.960580, perez_direct_rad_coefs(3, 0, 1, 1));
        validate::assert_close!(1.114130, perez_direct_rad_coefs(3, 0, 6, 3));
        // 31
        validate::assert_close!(0.950000, perez_direct_rad_coefs(3, 1, 0, 0));
        validate::assert_close!(0.913870, perez_direct_rad_coefs(3, 1, 1, 1));
        validate::assert_close!(1.044910, perez_direct_rad_coefs(3, 1, 6, 3));
        // 32
        validate::assert_close!(0.850000, perez_direct_rad_coefs(3, 2, 0, 0));
        validate::assert_close!(0.960010, perez_direct_rad_coefs(3, 2, 1, 1));
        validate::assert_close!(1.032260, perez_direct_rad_coefs(3, 2, 6, 3));
        // 33
        validate::assert_close!(0.750000, perez_direct_rad_coefs(3, 3, 0, 0));
        validate::assert_close!(0.987010, perez_direct_rad_coefs(3, 3, 1, 1));
        validate::assert_close!(1.057110, perez_direct_rad_coefs(3, 3, 6, 3));
        // 34
        validate::assert_close!(0.750000, perez_direct_rad_coefs(3, 4, 0, 0));
        validate::assert_close!(0.971470, perez_direct_rad_coefs(3, 4, 1, 1));
        validate::assert_close!(1.049350, perez_direct_rad_coefs(3, 4, 6, 3));
        // 35
        validate::assert_close!(0.689220, perez_direct_rad_coefs(3, 5, 0, 0));
        validate::assert_close!(0.852840, perez_direct_rad_coefs(3, 5, 1, 1));
        validate::assert_close!(0.898420, perez_direct_rad_coefs(3, 5, 6, 3));

        // 40
        validate::assert_close!(1.054880, perez_direct_rad_coefs(4, 0, 0, 0));
        validate::assert_close!(1.062220, perez_direct_rad_coefs(4, 0, 1, 1));
        validate::assert_close!(1.112780, perez_direct_rad_coefs(4, 0, 6, 3));
        // 41
        validate::assert_close!(1.017610, perez_direct_rad_coefs(4, 1, 0, 0));
        validate::assert_close!(0.998970, perez_direct_rad_coefs(4, 1, 1, 1));
        validate::assert_close!(1.071960, perez_direct_rad_coefs(4, 1, 6, 3));
        // 42
        validate::assert_close!(1.028710, perez_direct_rad_coefs(4, 2, 0, 0));
        validate::assert_close!(0.985500, perez_direct_rad_coefs(4, 2, 1, 1));
        validate::assert_close!(1.050220, perez_direct_rad_coefs(4, 2, 6, 3));
        // 43
        validate::assert_close!(0.992730, perez_direct_rad_coefs(4, 3, 0, 0));
        validate::assert_close!(0.987160, perez_direct_rad_coefs(4, 3, 1, 1));
        validate::assert_close!(0.972510, perez_direct_rad_coefs(4, 3, 6, 3));
        // 44
        validate::assert_close!(0.965840, perez_direct_rad_coefs(4, 4, 0, 0));
        validate::assert_close!(0.994770, perez_direct_rad_coefs(4, 4, 1, 1));
        validate::assert_close!(0.876210, perez_direct_rad_coefs(4, 4, 6, 3));
        // 45
        validate::assert_close!(0.948940, perez_direct_rad_coefs(4, 5, 0, 0));
        validate::assert_close!(0.970000, perez_direct_rad_coefs(4, 5, 1, 1));
        validate::assert_close!(0.596350, perez_direct_rad_coefs(4, 5, 6, 3));

        // 50
        validate::assert_close!(1.030000, perez_direct_rad_coefs(5, 0, 0, 0));
        validate::assert_close!(0.990000, perez_direct_rad_coefs(5, 0, 1, 1));
        validate::assert_close!(0.821140, perez_direct_rad_coefs(5, 0, 6, 3));
        // 51
        validate::assert_close!(1.041020, perez_direct_rad_coefs(5, 1, 0, 0));
        validate::assert_close!(0.980000, perez_direct_rad_coefs(5, 1, 1, 1));
        validate::assert_close!(0.816280, perez_direct_rad_coefs(5, 1, 6, 3));
        // 52
        validate::assert_close!(1.022450, perez_direct_rad_coefs(5, 2, 0, 0));
        validate::assert_close!(0.999240, perez_direct_rad_coefs(5, 2, 1, 1));
        validate::assert_close!(0.864380, perez_direct_rad_coefs(5, 2, 6, 3));
        // 53
        validate::assert_close!(0.995260, perez_direct_rad_coefs(5, 3, 0, 0));
        validate::assert_close!(0.975250, perez_direct_rad_coefs(5, 3, 1, 1));
        validate::assert_close!(0.731170, perez_direct_rad_coefs(5, 3, 6, 3));
        // 54
        validate::assert_close!(0.952560, perez_direct_rad_coefs(5, 4, 0, 0));
        validate::assert_close!(0.994420, perez_direct_rad_coefs(5, 4, 1, 1));
        validate::assert_close!(0.493730, perez_direct_rad_coefs(5, 4, 6, 3));
        // 55
        validate::assert_close!(0.911970, perez_direct_rad_coefs(5, 5, 0, 0));
        validate::assert_close!(0.316930, perez_direct_rad_coefs(5, 5, 1, 1));
        validate::assert_close!(0.316930, perez_direct_rad_coefs(5, 5, 6, 3));
    }

    #[test]
    fn test_perez_direct_rad_delta_kt_bin() {
        assert_eq!(perez_direct_rad_delta_kt_bin(0.0), 0);
        assert_eq!(perez_direct_rad_delta_kt_bin(0.01), 0);
        assert_eq!(perez_direct_rad_delta_kt_bin(0.015), 0);

        assert_eq!(perez_direct_rad_delta_kt_bin(0.01501), 1);
        assert_eq!(perez_direct_rad_delta_kt_bin(0.03), 1);
        assert_eq!(perez_direct_rad_delta_kt_bin(0.035), 1);

        assert_eq!(perez_direct_rad_delta_kt_bin(0.03501), 2);
        assert_eq!(perez_direct_rad_delta_kt_bin(0.050), 2);
        assert_eq!(perez_direct_rad_delta_kt_bin(0.070), 2);

        assert_eq!(perez_direct_rad_delta_kt_bin(0.0701), 3);
        assert_eq!(perez_direct_rad_delta_kt_bin(0.1), 3);
        assert_eq!(perez_direct_rad_delta_kt_bin(0.15), 3);

        assert_eq!(perez_direct_rad_delta_kt_bin(0.1501), 4);
        assert_eq!(perez_direct_rad_delta_kt_bin(0.2), 4);
        assert_eq!(perez_direct_rad_delta_kt_bin(0.3), 4);

        assert_eq!(perez_direct_rad_delta_kt_bin(0.301), 5);
        assert_eq!(perez_direct_rad_delta_kt_bin(0.5), 5);
        assert_eq!(perez_direct_rad_delta_kt_bin(1.0), 5);
    }

    #[test]
    #[should_panic]
    fn test_negative_delta_kt_bin() {
        perez_direct_rad_delta_kt_bin(-1212.);
    }

    #[test]
    #[should_panic]
    fn test_toobig_delta_kt_bin() {
        perez_direct_rad_delta_kt_bin(1212.);
    }

    #[test]
    fn test_perez_direct_rad_zeta_bin() {
        assert_eq!(perez_direct_rad_zeta_bin((0.0 as Float).to_radians()), 0);
        assert_eq!(perez_direct_rad_zeta_bin((20.0 as Float).to_radians()), 0);
        assert_eq!(perez_direct_rad_zeta_bin((25.0 as Float).to_radians()), 0);

        assert_eq!(perez_direct_rad_zeta_bin((25.01 as Float).to_radians()), 1);
        assert_eq!(perez_direct_rad_zeta_bin((32.0 as Float).to_radians()), 1);
        assert_eq!(perez_direct_rad_zeta_bin((40.0 as Float).to_radians()), 1);

        assert_eq!(perez_direct_rad_zeta_bin((40.01 as Float).to_radians()), 2);
        assert_eq!(perez_direct_rad_zeta_bin((50.0 as Float).to_radians()), 2);
        assert_eq!(perez_direct_rad_zeta_bin((55.0 as Float).to_radians()), 2);

        assert_eq!(perez_direct_rad_zeta_bin((55.01 as Float).to_radians()), 3);
        assert_eq!(perez_direct_rad_zeta_bin((60.0 as Float).to_radians()), 3);
        assert_eq!(perez_direct_rad_zeta_bin((70.0 as Float).to_radians()), 3);

        assert_eq!(perez_direct_rad_zeta_bin((70.001 as Float).to_radians()), 4);
        assert_eq!(perez_direct_rad_zeta_bin((75.0 as Float).to_radians()), 4);
        assert_eq!(perez_direct_rad_zeta_bin((80.0 as Float).to_radians()), 4);

        assert_eq!(perez_direct_rad_zeta_bin((80.01 as Float).to_radians()), 5);
        assert_eq!(perez_direct_rad_zeta_bin((85.0 as Float).to_radians()), 5);
        assert_eq!(perez_direct_rad_zeta_bin((90.0 as Float).to_radians()), 5);
    }

    #[test]
    #[should_panic]
    fn test_negative_zeta_bin() {
        perez_direct_rad_zeta_bin(-0.1);
    }

    #[test]
    #[should_panic]
    fn test_toobig_zeta_bin() {
        perez_direct_rad_zeta_bin((90.01 as Float).to_radians());
    }

    #[test]
    fn test_perez_direct_rad_w_bin() {
        assert_eq!(perez_direct_rad_w_bin(0.0), 0);
        assert_eq!(perez_direct_rad_w_bin(0.5), 0);
        assert_eq!(perez_direct_rad_w_bin(1.0), 0);

        assert_eq!(perez_direct_rad_w_bin(1.01), 1);
        assert_eq!(perez_direct_rad_w_bin(1.5), 1);
        assert_eq!(perez_direct_rad_w_bin(2.0), 1);

        assert_eq!(perez_direct_rad_w_bin(2.01), 2);
        assert_eq!(perez_direct_rad_w_bin(2.5), 2);
        assert_eq!(perez_direct_rad_w_bin(3.0), 2);

        assert_eq!(perez_direct_rad_w_bin(3.01), 3);
        assert_eq!(perez_direct_rad_w_bin(60.0), 3);
        assert_eq!(perez_direct_rad_w_bin(70.0), 3);
    }

    #[test]
    #[should_panic]
    fn test_negative_w_bin() {
        perez_direct_rad_w_bin(-0.1);
    }

    #[test]
    fn test_disc_direct_solar() {
        let pressure = 84000.0;
        let lat = (40.0 as Float).to_radians();
        let lon = (-75.0 as Float).to_radians();
        let stdmer = lon;
        let sol = Solar::new(lat, lon, stdmer);

        let start = Date {
            month: 1,
            day: 1,
            hour: 0.0,
        };
        let end = Date {
            month: 1,
            day: 3,
            hour: 0.0,
        };

        let dates = Period::new(start, end, 3600.0);
        const RAD_DATA: [Float; 48] = [
            0.00, -2.00, -3.20, -3.30, -3.20, -4.20, -3.20, 20.60, 86.10, 262.40, 391.20, 462.40,
            462.10, 410.60, 301.30, 162.60, 19.10, -4.40, -3.40, -4.80, -4.80, -3.80, -3.80, -3.80,
            -2.10, -2.90, -2.40, -2.00, -2.10, -3.40, -4.60, 9.60, 91.70, 213.70, 301.00, 466.30,
            468.20, 367.10, 269.50, 178.40, 22.70, -5.80, -4.60, -4.40, -4.10, -4.70, -4.00, -5.00,
        ];
        for (d, global_horizontal_rad) in dates.into_iter().zip(RAD_DATA) {
            let n = d.day_of_year();
            let n = Time::Solar(n);
            if global_horizontal_rad < 0.0 {
                // println!("{},0.0,0.0,0.0,0.0", d);
                println!(
                    "{},{},{},{},{},{},{},{},{}",
                    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0
                );
                continue;
            }
            if let Some(sun_direction) = sol.sun_position(n) {
                let _found = sol.disc_direct_solar(
                    n,
                    sun_direction,
                    global_horizontal_rad / sun_direction.z,
                    pressure,
                );
                // println!("{},{},{},0.0,0.0", d, global_horizontal_rad,found);
            } else {
                // println!("{},0.0,0.0,0.0,0.0", d);

                println!(
                    "{},{},{},{},{},{},{},{},{}",
                    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0
                );
            }
        }
    }
}
