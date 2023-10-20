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
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Spectrum(pub [Float; crate::N_CHANNELS]);
const RADIANCE_COEFFICIENTS: [Float; crate::N_CHANNELS] = [0.265, 0.67, 0.065];
/// The standard Luminious Efficacy of equal white light energy
/// as defined in Radiance
pub const WHITE_EFFICACY: Float = 179.;

impl std::default::Default for Spectrum {
    fn default() -> Self {
        Self::BLACK
    }
}

impl matrix::traits::OneZero for Spectrum {
    fn one() -> Self {
        Self::ONE
    }

    fn zero() -> Self {
        Self::BLACK
    }
}

impl Spectrum {
    pub const ONE: Self = Self([1.0; crate::N_CHANNELS]);
    pub const BLACK: Self = Self([0.0; crate::N_CHANNELS]);

    /// Creates a new Spectrum full of equal values `v`
    pub fn gray(v: Float) -> Self {
        Self([v; crate::N_CHANNELS])
    }

    pub fn powi(&self, n: i32) -> Self {
        let mut data = [0.0; crate::N_CHANNELS];
        self.0
            .iter()
            .enumerate()
            .for_each(|(i, this)| data[i] = this.powi(n));
        Self(data)
    }
    pub fn powf(&self, n: Float) -> Self {
        let mut data = [0.0; crate::N_CHANNELS];
        self.0
            .iter()
            .enumerate()
            .for_each(|(i, this)| data[i] = this.powf(n));
        Self(data)
    }

    /// Checks whether `Spectrum::BLACK == self`
    pub fn is_black(&self) -> bool {
        for v in self.0 {
            if v >= 1e-24 {
                return false;
            }
        }
        true
    }

    /// Calculates a weighted average of RGB colours, returning
    /// a single value representing Radiance
    pub fn radiance(&self) -> Float {
        // self.0[0]*47.9 + self.0[1]*119.9 + self.0[2]*11.6
        self.0
            .iter()
            .zip(RADIANCE_COEFFICIENTS.iter())
            .map(|(a, b)| a * b)
            .sum()
    }

    /// Scales the chanels in order to make the
    /// radiance equals to 1
    pub fn normalize(&self) -> Self {
        *self / self.radiance()
    }

    /// Calculates a weighted average of RGB colours, returning
    /// a single value representing Radiance
    pub fn luminance(&self) -> Float {
        self.radiance() * WHITE_EFFICACY
    }

    /// Gets the maximum of the R, G, and B values
    pub fn max(&self) -> Float {
        let mut max = self.0[0];
        for v in self.0.iter().skip(1) {
            if *v > max {
                max = *v
            }
        }

        max
    }
}

impl std::fmt::Display for Spectrum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.5} {:.5} {:.5}", self.0[0], self.0[1], self.0[2])
    }
}

/* SPECTRUM | SPECTRUM OPERATIONS */
impl std::ops::Add for Spectrum {
    // Spectrum + Spectrum
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let mut data = [0.0; crate::N_CHANNELS];
        self.0
            .iter()
            .enumerate()
            .zip(other.0.iter())
            .for_each(|((i, this), other)| data[i] = this + other);
        Self(data)
    }
}
impl std::ops::Sub for Spectrum {
    // Spectrum - Spectrum
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        let mut data = [0.0; crate::N_CHANNELS];
        self.0
            .iter()
            .enumerate()
            .zip(other.0.iter())
            .for_each(|((i, this), other)| data[i] = this - other);
        Self(data)
    }
}

impl std::ops::AddAssign for Spectrum {
    // Spectrum += Spectrum
    fn add_assign(&mut self, other: Self) {
        for (i, this) in self.0.iter_mut().enumerate() {
            *this += other.0[i]
        }
    }
}

impl std::ops::SubAssign for Spectrum {
    // Spectrum -= Spectrum
    fn sub_assign(&mut self, other: Self) {
        for (i, this) in self.0.iter_mut().enumerate() {
            *this -= other.0[i]
        }
    }
}

impl std::ops::Mul for Spectrum {
    // Spectrum * Spectrum
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        let mut data = [0.0; crate::N_CHANNELS];
        self.0
            .iter()
            .enumerate()
            .zip(other.0.iter())
            .for_each(|((i, this), other)| data[i] = this * other);
        Self(data)
    }
}

impl std::ops::Div for Spectrum {
    // Spectrum / Spectrum
    type Output = Self;

    fn div(self, other: Self) -> Self {
        let mut data = [0.0; crate::N_CHANNELS];
        self.0
            .iter()
            .enumerate()
            .zip(other.0.iter())
            .for_each(|((i, this), other)| data[i] = this / other);
        Self(data)
    }
}

impl std::ops::MulAssign for Spectrum {
    // Spectrum *= Spectrum
    fn mul_assign(&mut self, other: Self) {
        for (i, this) in self.0.iter_mut().enumerate() {
            *this *= other.0[i]
        }
    }
}

impl std::ops::DivAssign for Spectrum {
    // Spectrum /= Spectrum
    fn div_assign(&mut self, other: Self) {
        for (i, this) in self.0.iter_mut().enumerate() {
            *this /= other.0[i]
        }
    }
}

/* SPECTRUM | FLOAT OPERATIONS */

impl std::ops::Add<Float> for Spectrum {
    // Spectrum + Float
    type Output = Self;

    fn add(self, other: Float) -> Self {
        let mut data = [0.0; crate::N_CHANNELS];
        self.0
            .iter()
            .enumerate()
            .for_each(|(i, this)| data[i] = this + other);
        Self(data)
    }
}

impl std::ops::Sub<Float> for Spectrum {
    // Spectrum - Float
    type Output = Self;

    fn sub(self, other: Float) -> Self {
        // Spectrum - Float
        let mut data = [0.0; crate::N_CHANNELS];
        self.0
            .iter()
            .enumerate()
            .for_each(|(i, this)| data[i] = this - other);
        Self(data)
    }
}

impl std::ops::Mul<Float> for Spectrum {
    // Spectrum * Float
    type Output = Self;

    fn mul(self, other: Float) -> Self {
        let mut data = [0.0; crate::N_CHANNELS];
        self.0
            .iter()
            .enumerate()
            .for_each(|(i, this)| data[i] = this * other);
        Self(data)
    }
}

impl std::ops::Div<Float> for Spectrum {
    // Spectrum / Float
    type Output = Self;

    fn div(self, other: Float) -> Self {
        let mut data = [0.0; crate::N_CHANNELS];
        self.0
            .iter()
            .enumerate()
            .for_each(|(i, this)| data[i] = this / other);
        Self(data)
    }
}

impl std::ops::AddAssign<Float> for Spectrum {
    // Spectrum += Float
    fn add_assign(&mut self, other: Float) {
        for this in self.0.iter_mut() {
            *this += other
        }
    }
}

impl std::ops::SubAssign<Float> for Spectrum {
    // Spectrum -= Float
    fn sub_assign(&mut self, other: Float) {
        for this in self.0.iter_mut() {
            *this -= other
        }
    }
}

impl std::ops::MulAssign<Float> for Spectrum {
    // Spectrum *= Float
    fn mul_assign(&mut self, other: Float) {
        for this in self.0.iter_mut() {
            *this *= other
        }
    }
}

impl std::ops::DivAssign<Float> for Spectrum {
    // Spectrum /= Float
    fn div_assign(&mut self, other: Float) {
        for this in self.0.iter_mut() {
            *this /= other
        }
    }
}

/* FLOAT | SPECTRUM OPERATIONS */

impl std::ops::Add<Spectrum> for Float {
    // Float + Spectrum
    type Output = Spectrum;
    fn add(self, c: Spectrum) -> Spectrum {
        let mut data = [0.0; crate::N_CHANNELS];
        c.0.iter()
            .enumerate()
            .for_each(|(i, other)| data[i] = self + other);
        Spectrum(data)
    }
}

impl std::ops::Sub<Spectrum> for Float {
    // Float - Spectrum
    type Output = Spectrum;
    fn sub(self, c: Spectrum) -> Spectrum {
        let mut data = [0.0; crate::N_CHANNELS];
        c.0.iter()
            .enumerate()
            .for_each(|(i, other)| data[i] = self - other);
        Spectrum(data)
    }
}

impl std::ops::Mul<Spectrum> for Float {
    // Float * Spectrum
    type Output = Spectrum;
    fn mul(self, c: Spectrum) -> Spectrum {
        let mut data = [0.0; crate::N_CHANNELS];
        c.0.iter()
            .enumerate()
            .for_each(|(i, other)| data[i] = self * other);
        Spectrum(data)
    }
}

impl std::ops::Div<Spectrum> for Float {
    // FLoat / Spectrum
    type Output = Spectrum;
    fn div(self, c: Spectrum) -> Spectrum {
        let mut data = [0.0; crate::N_CHANNELS];
        c.0.iter()
            .enumerate()
            .for_each(|(i, other)| data[i] = self / other);
        Spectrum(data)
    }
}

impl std::convert::From<Float> for Spectrum {
    fn from(f: Float) -> Self {
        Spectrum([f; crate::N_CHANNELS])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use matrix::traits::OneZero;
    use validate::assert_close;

    #[test]
    fn test_default() {
        let c2 = Spectrum::default();
        for v in c2.0 {
            assert_close!(v, 0., 1e-9);
        }
        let zero = Spectrum::zero();
        assert_eq!(c2, zero);
    }

    #[test]
    fn test_one() {
        let c1 = Spectrum::ONE;
        for v in c1.0 {
            assert_close!(v, 1., 1e-9);
        }
        let one = Spectrum::one();
        assert_eq!(c1, one);
    }

    #[test]
    fn test_zero() {
        let c2 = Spectrum::BLACK;
        for v in c2.0 {
            assert_close!(v, 0., 1e-9);
        }
        let zero = Spectrum::zero();
        assert_eq!(c2, zero);
    }

    #[test]
    fn test_gray() {
        let exp = 1.123123;
        let c2 = Spectrum::gray(exp);
        for v in c2.0 {
            assert_close!(v, exp, 1e-9);
        }
    }

    #[test]
    fn test_powi() {
        let exp = 5;
        let c = Spectrum([1.23, 321., -5.12]);
        let c2 = c.powi(exp);

        assert_close!(c.0[0].powi(exp), c2.0[0], 1e-9);
        assert_close!(c.0[1].powi(exp), c2.0[1], 1e-9);
        assert_close!(c.0[2].powi(exp), c2.0[2], 1e-9);
    }

    #[test]
    fn test_powf() {
        let exp = 5.131241;
        let c = Spectrum([1.23, 321., -5.12]);
        let c2 = c.powf(exp);

        assert_close!(c.0[0].powf(exp), c2.0[0], 1e-9);
        assert_close!(c.0[1].powf(exp), c2.0[1], 1e-9);
        assert_close!(c.0[2].powf(exp), c2.0[2], 1e-9);
    }

    #[test]
    fn test_is_black() {
        let c = Spectrum([0., 0., 0.]);
        assert!(c.is_black());

        let c = Spectrum([0., 0., 1.]);
        assert!(!c.is_black());

        let c = Spectrum([0., 1., 0.]);
        assert!(!c.is_black());

        let c = Spectrum([1., 0., 0.]);
        assert!(!c.is_black());

        let c = Spectrum([1., 1., 0.]);
        assert!(!c.is_black());

        let c = Spectrum([1., 0., 1.]);
        assert!(!c.is_black());

        let c = Spectrum([1., 1., 1.]);
        assert!(!c.is_black());
    }

    #[test]
    fn test_radiance_luminance() {
        let c = Spectrum([1., 0., 0.]);
        let rad = c.radiance();
        assert_close!(rad, RADIANCE_COEFFICIENTS[0]);
        assert_close!(rad * WHITE_EFFICACY, c.luminance());

        let c = Spectrum([0., 1., 0.]);
        let rad = c.radiance();
        assert_close!(rad, RADIANCE_COEFFICIENTS[1]);
        assert_close!(rad * WHITE_EFFICACY, c.luminance());

        let c = Spectrum([0., 0., 1.]);
        let rad = c.radiance();
        assert_close!(rad, RADIANCE_COEFFICIENTS[2]);
        assert_close!(rad * WHITE_EFFICACY, c.luminance());

        let c = Spectrum([1., 1., 0.]);
        let rad = c.radiance();
        assert_close!(rad, RADIANCE_COEFFICIENTS[0] + RADIANCE_COEFFICIENTS[1]);
        assert_close!(rad * WHITE_EFFICACY, c.luminance());

        let c = Spectrum([0., 1., 1.]);
        let rad = c.radiance();
        assert_close!(rad, RADIANCE_COEFFICIENTS[1] + RADIANCE_COEFFICIENTS[2]);
        assert_close!(rad * WHITE_EFFICACY, c.luminance());

        let c = Spectrum([1., 0., 1.]);
        let rad = c.radiance();
        assert_close!(rad, RADIANCE_COEFFICIENTS[0] + RADIANCE_COEFFICIENTS[2]);
        assert_close!(rad * WHITE_EFFICACY, c.luminance());

        let c = Spectrum([1., 1., 1.]);
        let rad = c.radiance();
        assert_close!(
            rad,
            RADIANCE_COEFFICIENTS[0] + RADIANCE_COEFFICIENTS[1] + RADIANCE_COEFFICIENTS[2]
        );
        assert_close!(rad * WHITE_EFFICACY, c.luminance());

        let c = Spectrum([2., 3., 6.]);
        let rad = c.radiance();
        assert_close!(
            rad,
            2. * RADIANCE_COEFFICIENTS[0]
                + 3. * RADIANCE_COEFFICIENTS[1]
                + 6. * RADIANCE_COEFFICIENTS[2]
        );
        assert_close!(rad * WHITE_EFFICACY, c.luminance());
    }

    #[test]
    fn test_normalize() {
        let c = Spectrum([2., 3., 6.]);
        let c = c.normalize();
        assert_close!(c.radiance(), 1.0);
        assert_close!(c.luminance(), WHITE_EFFICACY);
    }

    #[test]
    fn test_max() {
        let c = Spectrum([2., 3., 6.]);
        assert_close!(c.max(), 6.);

        let c = Spectrum([2., 3., -6.]);
        assert_close!(c.max(), 3.);
    }

    #[test]
    fn test_from() {
        let exp = 4.12312;
        let c = Spectrum::from(exp);
        for v in c.0 {
            assert_close!(v, exp);
        }
    }

    /* SPECTRUM | SPECTRUM OPERATIONS */
    #[test]
    fn test_add() {
        // Spectrum + Spectrum
        let c1 = Spectrum([1.23, 5.321, 9.9719]);
        let c2 = Spectrum([21.23, 95.321, 0.9719]);
        let result = c1 + c2;
        assert_close!(result.0[0], c1.0[0] + c2.0[0]);
        assert_close!(result.0[1], c1.0[1] + c2.0[1]);
        assert_close!(result.0[2], c1.0[2] + c2.0[2]);
    }

    #[test]
    fn test_sub() {
        // Spectrum - Spectrum
        let c1 = Spectrum([1.23, 5.321, 9.9719]);
        let c2 = Spectrum([21.23, 95.321, 0.9719]);
        let result = c1 - c2;
        assert_close!(result.0[0], c1.0[0] - c2.0[0]);
        assert_close!(result.0[1], c1.0[1] - c2.0[1]);
        assert_close!(result.0[2], c1.0[2] - c2.0[2]);
    }

    #[test]
    fn test_mul() {
        // Spectrum * Spectrum
        let c1 = Spectrum([1.23, 5.321, 9.9719]);
        let c2 = Spectrum([21.23, 95.321, 0.9719]);
        let result = c1 * c2;
        assert_close!(result.0[0], c1.0[0] * c2.0[0]);
        assert_close!(result.0[1], c1.0[1] * c2.0[1]);
        assert_close!(result.0[2], c1.0[2] * c2.0[2]);
    }

    #[test]
    fn test_div() {
        // Spectrum / Spectrum
        let c1 = Spectrum([1.23, 5.321, 9.9719]);
        let c2 = Spectrum([21.23, 95.321, 0.9719]);
        let result = c1 / c2;
        assert_close!(result.0[0], c1.0[0] / c2.0[0]);
        assert_close!(result.0[1], c1.0[1] / c2.0[1]);
        assert_close!(result.0[2], c1.0[2] / c2.0[2]);
    }

    #[test]
    fn test_add_assign() {
        // Spectrum += Spectrum
        let c1 = Spectrum([1.23, 5.321, 9.9719]);
        let c2 = Spectrum([21.23, 95.321, 0.9719]);
        let mut result = c1;
        result += c2;

        assert_close!(result.0[0], c1.0[0] + c2.0[0]);
        assert_close!(result.0[1], c1.0[1] + c2.0[1]);
        assert_close!(result.0[2], c1.0[2] + c2.0[2]);
    }

    #[test]
    fn test_sub_assign() {
        // Spectrum -= Spectrum
        let c1 = Spectrum([1.23, 5.321, 9.9719]);
        let c2 = Spectrum([21.23, 95.321, 0.9719]);
        let mut result = c1;
        result -= c2;
        assert_close!(result.0[0], c1.0[0] - c2.0[0]);
        assert_close!(result.0[1], c1.0[1] - c2.0[1]);
        assert_close!(result.0[2], c1.0[2] - c2.0[2]);
    }

    #[test]
    fn test_mul_assign() {
        // Spectrum *= Spectrum
        let c1 = Spectrum([1.23, 5.321, 9.9719]);
        let c2 = Spectrum([21.23, 95.321, 0.9719]);
        let mut result = c1;
        result *= c2;
        assert_close!(result.0[0], c1.0[0] * c2.0[0]);
        assert_close!(result.0[1], c1.0[1] * c2.0[1]);
        assert_close!(result.0[2], c1.0[2] * c2.0[2]);
    }

    #[test]
    fn test_div_assign() {
        // Spectrum /= Spectrum
        let c1 = Spectrum([1.23, 5.321, 9.9719]);
        let c2 = Spectrum([21.23, 95.321, 0.9719]);
        let mut result = c1;
        result /= c2;

        assert_close!(result.0[0], c1.0[0] / c2.0[0]);
        assert_close!(result.0[1], c1.0[1] / c2.0[1]);
        assert_close!(result.0[2], c1.0[2] / c2.0[2]);
    }

    /* SPECTRUM | FLOAT OPERATIONS */

    #[test]
    fn test_add_float() {
        // Spectrum + Float
        let spectrum = Spectrum([1.23, 5.321, 9.9719]);
        let float = -5.123;
        let result = spectrum + float;
        assert_close!(result.0[0], spectrum.0[0] + float);
        assert_close!(result.0[1], spectrum.0[1] + float);
        assert_close!(result.0[2], spectrum.0[2] + float);
    }

    #[test]
    fn test_sub_float() {
        // Spectrum - Float
        let spectrum = Spectrum([1.23, 5.321, 9.9719]);
        let float = -5.123;
        let result = spectrum - float;
        assert_close!(result.0[0], spectrum.0[0] - float);
        assert_close!(result.0[1], spectrum.0[1] - float);
        assert_close!(result.0[2], spectrum.0[2] - float);
    }

    #[test]
    fn test_mul_float() {
        // Spectrum * Float
        let spectrum = Spectrum([1.23, 5.321, 9.9719]);
        let float = -5.123;
        let result = spectrum * float;
        assert_close!(result.0[0], spectrum.0[0] * float);
        assert_close!(result.0[1], spectrum.0[1] * float);
        assert_close!(result.0[2], spectrum.0[2] * float);
    }

    #[test]
    fn test_div_float() {
        // Spectrum /= Float
        let spectrum = Spectrum([1.23, 5.321, 9.9719]);
        let float = -5.123;
        let result = spectrum / float;
        assert_close!(result.0[0], spectrum.0[0] / float);
        assert_close!(result.0[1], spectrum.0[1] / float);
        assert_close!(result.0[2], spectrum.0[2] / float);
    }

    #[test]
    fn test_add_assign_float() {
        // Spectrum += Float
        let spectrum = Spectrum([1.23, 5.321, 9.9719]);
        let float = -5.123;
        let mut result = spectrum;
        result += float;

        assert_close!(result.0[0], spectrum.0[0] + float);
        assert_close!(result.0[1], spectrum.0[1] + float);
        assert_close!(result.0[2], spectrum.0[2] + float);
    }

    #[test]
    fn test_sub_assign_float() {
        // Spectrum -= Float
        let spectrum = Spectrum([1.23, 5.321, 9.9719]);
        let float = -5.123;
        let mut result = spectrum;
        result -= float;

        assert_close!(result.0[0], spectrum.0[0] - float);
        assert_close!(result.0[1], spectrum.0[1] - float);
        assert_close!(result.0[2], spectrum.0[2] - float);
    }

    #[test]
    fn test_mul_assign_float() {
        // Spectrum *= Float
        let spectrum = Spectrum([1.23, 5.321, 9.9719]);
        let float = -5.123;
        let mut result = spectrum;
        result *= float;

        assert_close!(result.0[0], spectrum.0[0] * float);
        assert_close!(result.0[1], spectrum.0[1] * float);
        assert_close!(result.0[2], spectrum.0[2] * float);
    }

    #[test]
    fn test_assign_div_float() {
        // Spectrum /= Float
        let spectrum = Spectrum([1.23, 5.321, 9.9719]);
        let float = -5.123;
        let mut result = spectrum;
        result /= float;

        assert_close!(result.0[0], spectrum.0[0] / float);
        assert_close!(result.0[1], spectrum.0[1] / float);
        assert_close!(result.0[2], spectrum.0[2] / float);
    }

    /* FLOAT | SPECTRUM OPERATIONS */
    #[test]
    fn test_add_float_2() {
        // Float + Spectrum
        let spectrum = Spectrum([1.23, 5.321, 9.9719]);
        let float = -5.123;
        let result = float + spectrum;
        assert_close!(result.0[0], float + spectrum.0[0]);
        assert_close!(result.0[1], float + spectrum.0[1]);
        assert_close!(result.0[2], float + spectrum.0[2]);
    }

    #[test]
    fn test_sub_float_2() {
        // Float - Spectrum
        let spectrum = Spectrum([1.23, 5.321, 9.9719]);
        let float = -5.123;
        let result = float - spectrum;
        assert_close!(result.0[0], float - spectrum.0[0]);
        assert_close!(result.0[1], float - spectrum.0[1]);
        assert_close!(result.0[2], float - spectrum.0[2]);
    }

    #[test]
    fn test_mul_float_2() {
        // Float * Spectrum
        let spectrum = Spectrum([1.23, 5.321, 9.9719]);
        let float = -5.123;
        let result = float * spectrum;
        assert_close!(result.0[0], float * spectrum.0[0]);
        assert_close!(result.0[1], float * spectrum.0[1]);
        assert_close!(result.0[2], float * spectrum.0[2]);
    }

    #[test]
    fn test_div_float_2() {
        // FLoat / Spectrum
        let spectrum = Spectrum([1.23, 5.321, 9.9719]);
        let float = -5.123;
        let result = float / spectrum;
        assert_close!(result.0[0], float / spectrum.0[0]);
        assert_close!(result.0[1], float / spectrum.0[1]);
        assert_close!(result.0[2], float / spectrum.0[2]);
    }
}
