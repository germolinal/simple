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
use polynomial::{poly, Polynomial};

/// A structure containing the data that will describe the thermal
/// behaviour of a gas.
#[derive(Debug, Clone)]
pub struct Gas {
    /// The thermal conductivity ($`{W}/{m.K}`$) as a function of the
    /// temperature (in $`K`$)
    thermal_conductivity: Polynomial,

    /// The dynamic viscosity ( $`{N.s}/{m^2}`$) as a function of the
    /// temperature (in $`K`$)
    dynamic_viscosity: Polynomial,

    /// The specific heat capacity ($`{J}/{kg.K}`$) as a function of the
    /// temperature (in $`K`$)
    heat_capacity: Polynomial,

    /// THe Molecular Mass ($`{kg}/{Mol}`$)
    mass: Float,
}

/// Returns a gas with the properties of Air
pub const AIR: Gas = Gas {
    thermal_conductivity: poly![2.873e-3, 7.760e-5],
    dynamic_viscosity: poly![3.723e-6, 4.94e-8],
    heat_capacity: poly![1002.7370, 1.2324e-2],
    mass: 28.97,
};

/// Returns a gas with the properties of argon
pub const ARGON: Gas = Gas {
    thermal_conductivity: poly![2.285e-3, 5.149e-5],
    dynamic_viscosity: poly![3.379e-6, 6.451e-8],
    heat_capacity: poly![521.9285],
    mass: 39.948,
};

/// A gas with the properties of krypton
pub const KRYPTON: Gas = Gas {
    thermal_conductivity: poly![9.443e-4, 2.826e-5],
    dynamic_viscosity: poly![2.213e-6, 7.777e-8],
    heat_capacity: poly![248.0907],
    mass: 83.8,
};

/// A gas with the properties of xenon
pub const XENON: Gas = Gas {
    thermal_conductivity: poly![4.538e-4, 1.723e-5],
    dynamic_viscosity: poly![1.069e-6, 7.414e-8],
    heat_capacity: poly![158.3397],
    mass: 131.30,
};

impl Gas {
    /// Calculates the Raleigh number of a [`Gas`] cavity based on its
    /// `thickness` and its temperatures `t_front` and `t_back` (note that, for
    /// this particular function, these values are interchangeable)
    ///
    /// Source: Equation 40 of ISO15099/2003
    fn raleigh(&self, t_front: Float, t_back: Float, thickness: Float) -> Float {
        const G: Float = 9.81;

        if (t_front - t_back).abs() < 1e-10 {
            return 0.0000001;
        }

        // Gas mean temperature
        let temp = (in_kelvin(t_front) + in_kelvin(t_back)) / 2.;
        // Eq. 41 of iso15099/2003
        let beta = 1. / temp;

        // From Annex B
        let c_p = self.heat_capacity(temp);
        let mu = self.dynamic_viscosity(temp);
        let lambda = self.thermal_conductivity(temp);
        let rho = self.density(temp);

        // Eq. 40 of iso15099/2003
        rho.powi(2) * thickness.powi(3) * G * beta * c_p * (t_front - t_back).abs() / (mu * lambda)
    }

    /// Calculates the convective heat transfer coefficient within a gas-filled
    /// cavity based on its tilt  $`\gamma`$ (in radians), its outside temperature `t_out` and its
    /// interior temperature `t_in`.
    ///
    ///
    /// Adapted from Section 5.3.3.1 of ISO15099/2003:
    ///
    /// * $`\gamma = 0^o`$: is horizontal glazing
    /// * $`\gamma = 90^o`$: is vertical glazing
    ///
    /// # Note:
    ///
    /// The standard ISO15099/2003 states the following:
    ///
    /// "This categorization, as a function of $`\gamma`$, is based on the assumption **that
    /// the cavity is heated from the internal side** (i.e., $`T_{ft,i} > T_{b,i-1}`$). If the
    /// reverse is true it is necessary to seek the appropriate correlation on the basis of
    /// the complement of the tilt angle, $`180^o - \gamma`$, instead of $`\gamma`$
    /// when the calculation is carried out".        
    ///
    /// This conversion is handleded automatically by this function based
    /// on the inputs given to `t_front` and `t_back`
    pub fn cavity_convection(
        &self,
        height: Float,
        thickness: Float,
        mut gamma: Float,
        t_front: Float,
        t_back: Float,
    ) -> Float {
        debug_assert!(gamma >= 0.0);
        debug_assert!(gamma <= (180. as Float).to_radians());

        if t_front > t_back {
            gamma = (180. as Float).to_radians() - gamma;
        }

        // Eq. 42
        let a_gi = height / thickness;

        let ra = self.raleigh(t_front, t_back, thickness);
        let nu = nusselt(ra, gamma, a_gi);

        let temp = (in_kelvin(t_front) + in_kelvin(t_back)) / 2.;
        let lambda = self.thermal_conductivity(temp);

        // Equation 39 of ISO15099/2003
        nu * lambda / thickness
    }

    /// Derives the Thermal Conductivity at a certain Temperature (in $`K`$)
    pub fn thermal_conductivity(&self, temp: Float) -> Float {
        self.thermal_conductivity.eval(temp)
    }

    /// Derives the Dynamic Viscosity at a certain Temperature (in $`K`$)
    pub fn dynamic_viscosity(&self, temp: Float) -> Float {
        self.dynamic_viscosity.eval(temp)
    }

    /// Derives the Soecific Heat Capacity at a certain Temperature (in $`K`$)
    pub fn heat_capacity(&self, temp: Float) -> Float {
        self.heat_capacity.eval(temp)
    }

    /// Retreives the Molecular Mass
    pub fn mass(&self) -> Float {
        self.mass
    }

    /// Derives the density based on the temperature (in $`K`$)
    pub fn density(&self, temp: Float) -> Float {
        const R: Float = 8314.46261815324;
        // Eq. 55 of iso15099/2003
        101325. * self.mass / (R * temp)
    }
}

/// Transforms C into K
fn in_kelvin(t: Float) -> Float {
    t + 273.15
}

/// Calculates the Nusselt of a cavity number based on the Raleigh
/// number `ra`, the tilt of the cavity `gamma` and the aspect ratio `a_gi` (i.e., $`a_{gi}`$).
///
/// ```math
/// a_{gi} = \frac{H}{d}
/// ```
///
/// Where, according to the standard, "$`H`$ is the distance between the top and bottom of the fill
/// gas cavity which is usually the same as the height of the window view area.". Also, $`d`$ is the
/// thickness of the cavity.
fn nusselt(ra: Float, gamma: Float, a_gi: Float) -> Float {
    const THIRTY_RAD: Float = 30. * crate::PI / 180.;
    const EPSILON_RAD: Float = 0.5 * crate::PI / 180.;

    let gamma = gamma % crate::PI;

    if (0.0..2. * THIRTY_RAD - EPSILON_RAD).contains(&gamma) {
        // Between 0 and 60 degrees
        nu_0_60(ra, gamma, a_gi)
    } else if gamma < 2. * THIRTY_RAD + EPSILON_RAD {
        // 60 degrees
        nu_60(ra, a_gi)
    } else if gamma < 3. * THIRTY_RAD - EPSILON_RAD {
        // between 60 and 90 degrees
        nu_60_90(ra, gamma, a_gi)
    } else if gamma < 3. * THIRTY_RAD + EPSILON_RAD {
        // 90 degrees
        nu_90(ra, a_gi)
    } else if gamma < 6. * THIRTY_RAD {
        // between 90 and 180 degrees
        nu_90_180(ra, a_gi, gamma)
    } else {
        unreachable!("gamma is {}", gamma)
    }
}

/// Calculates the Nusselt number for cavities tilted between
/// $`0^o`$ and $`60^o`$
///
/// From Equation 43 and 44 of ISO15099/2003 (based on ref. 7 of that standard)
fn nu_0_60(ra: Float, gamma: Float, _a_gi: Float) -> Float {
    if ra > 1e5 || _a_gi < 20. {
        // This clause is written in section 5.3.3.2... does
        // not say what to do when this happens
        dbg!(ra, _a_gi, "Not expecting these values");
    }
    // Eq. 44 of ISO15099/2003
    let aux = |x: Float| -> Float { (x + x.abs()) / 2. };
    let cos_gamma = gamma.cos();

    // prepare return
    let a = aux(1. - 1708. / (ra * cos_gamma));
    let b = 1. - 1708. * ((1.8 * gamma).sin()).powf(1.6) / (ra * cos_gamma);
    let c = (ra * cos_gamma / 5830.).powf(1. / 3.) - 1.;

    // eq. 43
    1. + 1.44 * a * b + aux(c)
}

/// Calculates the Nusselt number for cavities tilted $`60^o`$
///
/// From Equations 45-48 of ISO15099/2003  (based on ref. 8 of that standard)
fn nu_60(ra: Float, a_gi: Float) -> Float {
    // Eq. 48
    let g = 0.5 / (1. + (ra / 3160.).powf(20.6)).powf(0.1);
    // Eq. 46
    let nu1 = (1. + (0.0936 * ra.powf(0.314) / (1. + g)).powi(7)).powf(1. / 7.);
    // Eq. 47
    let nu2 = (0.104 + 0.175 / a_gi) * ra.powf(0.283);

    // Eq. 45
    if nu1 > nu2 {
        nu1
    } else {
        nu2
    }
}

/// Calculates the Nusselt number for cavities tilted between $`60^o`$
/// and $`90^o`$
///
/// From section 5.3.3.4 of ISO15099/2003  (based on ref. 8 of that standard)
fn nu_60_90(ra: Float, gamma: Float, a_gi: Float) -> Float {
    if ra <= 1e2 || ra >= 2e7 || a_gi <= 5. || a_gi >= 100. {
        // This clause is written in section 5.3.3.5... does
        // not say what to do when this happens
        dbg!(ra, a_gi, "Not expecting these values");
    }
    let nu60 = nu_60(ra, a_gi); // PI/3 radians
    let nu90 = nu_90(ra, a_gi); // PI/2 radians
                                // interpolate
    let x = (gamma - crate::PI / 3.) / (crate::PI / 2. - crate::PI / 3.);
    nu60 + (nu90 - nu60) * x
}

/// Calculates the Nusselt number for cavities tilted $`90^o`$
///
/// From Equations 49-53 of ISO15099/2003  (based on ref. 8 of that standard)
fn nu_90(ra: Float, a_gi: Float) -> Float {
    let nu1 = if ra <= 1e4 {
        // Eq. 52
        1. + 1.7596678 * 1e-10 * ra.powf(2.2984755)
    } else if ra < 5e4 {
        // Eq. 51
        0.028154 * ra.powf(0.4134)
    } else if ra > 5e4 {
        // Eq. 50
        0.0673838 * ra.powf(1. / 3.)
    } else {
        unreachable!()
    };
    // Eq. 53
    let nu2 = 0.242 * (ra / a_gi).powf(0.272);

    // Eq. 49
    if nu1 > nu2 {
        nu1
    } else {
        nu2
    }
}

/// Calculates the Nusselt number for cavities tilted between $`90^o`$ and $`180^o`$
///
/// From Equation 54 of ISO15099/2003  (based on ref. 10 of that standard)
fn nu_90_180(ra: Float, a_gi: Float, gamma: Float) -> Float {
    let nu_v = nu_90(ra, a_gi);
    1. + (nu_v - 1.) * gamma.sin()
}

/***********/
/* TESTING */
/***********/

#[cfg(test)]
mod testing {
    use super::*;

    fn check_value(a: Float, b: Float) -> Result<(), String> {
        let err = (a - b).abs() / a.abs();
        if err > 1e-2 {
            //1% error max
            return Err(format!("a = {} | b = {} | err = {}", a, b, err));
        }
        Ok(())
    }

    #[test]
    fn test_thermal_conductivity() {
        check_value(0.0241, crate::gas::AIR.thermal_conductivity(0. + 273.15)).unwrap();
        check_value(0.0248, crate::gas::AIR.thermal_conductivity(10. + 273.15)).unwrap();

        check_value(0.0163, crate::gas::ARGON.thermal_conductivity(0. + 273.15)).unwrap();
        check_value(0.0169, crate::gas::ARGON.thermal_conductivity(10. + 273.15)).unwrap();

        check_value(
            0.0087,
            crate::gas::KRYPTON.thermal_conductivity(0. + 273.15),
        )
        .unwrap();
        check_value(
            0.0089,
            crate::gas::KRYPTON.thermal_conductivity(10. + 273.15),
        )
        .unwrap();

        check_value(0.0052, crate::gas::XENON.thermal_conductivity(0. + 273.15)).unwrap();
        check_value(0.0053, crate::gas::XENON.thermal_conductivity(10. + 273.15)).unwrap();
    }

    #[test]
    fn test_dynamic_viscosity() {
        check_value(1.722e-5, crate::gas::AIR.dynamic_viscosity(0. + 273.15)).unwrap();
        check_value(1.771e-5, crate::gas::AIR.dynamic_viscosity(10. + 273.15)).unwrap();

        check_value(2.1e-5, crate::gas::ARGON.dynamic_viscosity(0. + 273.15)).unwrap();
        check_value(2.165e-5, crate::gas::ARGON.dynamic_viscosity(10. + 273.15)).unwrap();

        check_value(2.346e-5, crate::gas::KRYPTON.dynamic_viscosity(0. + 273.15)).unwrap();
        check_value(
            2.423e-5,
            crate::gas::KRYPTON.dynamic_viscosity(10. + 273.15),
        )
        .unwrap();

        check_value(2.132e-5, crate::gas::XENON.dynamic_viscosity(0. + 273.15)).unwrap();
        check_value(2.206e-5, crate::gas::XENON.dynamic_viscosity(10. + 273.15)).unwrap();
    }

    #[test]
    fn test_heat_capacity() {
        check_value(1006.1034, crate::gas::AIR.heat_capacity(0. + 273.15)).unwrap();
        check_value(1006.2265, crate::gas::AIR.heat_capacity(10. + 273.15)).unwrap();

        check_value(521.9285, crate::gas::ARGON.heat_capacity(0. + 273.15)).unwrap();
        check_value(521.9285, crate::gas::ARGON.heat_capacity(10. + 273.15)).unwrap();

        check_value(248.0907, crate::gas::KRYPTON.heat_capacity(0. + 273.15)).unwrap();
        check_value(248.0907, crate::gas::KRYPTON.heat_capacity(10. + 273.15)).unwrap();

        check_value(158.3397, crate::gas::XENON.heat_capacity(0. + 273.15)).unwrap();
        check_value(158.3397, crate::gas::XENON.heat_capacity(10. + 273.15)).unwrap();
    }

    #[test]
    fn test_mass() {
        check_value(28.97, crate::gas::AIR.mass()).unwrap();
        check_value(39.948, crate::gas::ARGON.mass()).unwrap();
        check_value(83.80, crate::gas::KRYPTON.mass()).unwrap();
        check_value(131.3, crate::gas::XENON.mass()).unwrap();
    }

    #[test]
    fn test_density() {
        let gas = crate::gas::AIR;
        let rho = gas.density(293.15);
        assert!((1.2041 - rho).abs() < 1e-3);
    }

    #[test]
    fn test_nusselt() {
        // https://github.com/LBNL-ETA/Windows-CalcEngine/blob/main/src/Tarcog/tst/units/NusseltNumber.unit.cpp

        // Test 1
        let ra = 3638.21667064528;
        let a_gi = 83.3333333333333;

        let mut gamma = (30.0 as Float).to_radians();
        let nu = nusselt(ra, gamma, a_gi);
        let exp = 1.40474349200254;
        assert!((nu - exp).abs() < 1e-5, "Expecting {exp}... found {nu}");
        dbg!(nu, exp, (nu - exp).abs());

        gamma = (60.0 as Float).to_radians();
        let nu = nusselt(ra, gamma, a_gi);
        let exp = 1.08005742342789;
        assert!((nu - exp).abs() < 1e-5, "Expecting {exp}... found {nu}");
        dbg!(nu, exp, (nu - exp).abs());

        gamma = (73.0 as Float).to_radians();
        let nu = nusselt(ra, gamma, a_gi);
        let exp = 1.05703042079892;
        assert!((nu - exp).abs() < 1e-5, "Expecting {exp}... found {nu}");
        dbg!(nu, exp, (nu - exp).abs());

        gamma = (90.0 as Float).to_radians();
        let nu = nusselt(ra, gamma, a_gi);
        let exp = 1.02691818659179;
        assert!((nu - exp).abs() < 1e-5, "Expecting {exp}... found {nu}");
        dbg!(nu, exp, (nu - exp).abs());

        gamma = (134.0 as Float).to_radians();
        let nu = nusselt(ra, gamma, a_gi);
        let exp = 1.01936332296842;
        assert!((nu - exp).abs() < 1e-5, "Expecting {exp}... found {nu}");
        dbg!(nu, exp, (nu - exp).abs());

        // Test 2
        let ra = 140.779077041012;
        let a_gi = 200.;

        let mut gamma = (30.0 as Float).to_radians();
        let nu = nusselt(ra, gamma, a_gi);
        let exp = 1.;
        assert!((nu - exp).abs() < 1e-5, "Expecting {exp}... found {nu}");
        dbg!(nu, exp, (nu - exp).abs());

        gamma = (60.0 as Float).to_radians();
        let nu = nusselt(ra, gamma, a_gi);
        let exp = 1.00002777439094;
        assert!((nu - exp).abs() < 1e-5, "Expecting {exp}... found {nu}");
        dbg!(nu, exp, (nu - exp).abs());

        gamma = (73.0 as Float).to_radians();
        let nu = nusselt(ra, gamma, a_gi);
        let exp = 1.00002235511865;
        assert!((nu - exp).abs() < 1e-5, "Expecting {exp}... found {nu}");
        dbg!(nu, exp, (nu - exp).abs());

        gamma = (90.0 as Float).to_radians();
        let nu = nusselt(ra, gamma, a_gi);
        let exp = 1.00001526837795;
        assert!((nu - exp).abs() < 1e-5, "Expecting {exp}... found {nu}");
        dbg!(nu, exp, (nu - exp).abs());

        gamma = (134.0 as Float).to_radians();
        let nu = nusselt(ra, gamma, a_gi);
        let exp = 1.00001098315195;
        assert!((nu - exp).abs() < 1e-5, "Expecting {exp}... found {nu}");
        dbg!(nu, exp, (nu - exp).abs());

        // Test 3
        let ra = 4633340.8866717;
        let a_gi = 10.;

        let mut gamma = (30.0 as Float).to_radians();
        let nu = nusselt(ra, gamma, a_gi);
        let exp = 10.2680981545288;
        assert!((nu - exp).abs() < 1e-5, "Expecting {exp}... found {nu}");
        dbg!(nu, exp, (nu - exp).abs());

        gamma = (60.0 as Float).to_radians();
        let nu = nusselt(ra, gamma, a_gi);
        let exp = 11.5975502261096;
        assert!((nu - exp).abs() < 1e-5, "Expecting {exp}... found {nu}");
        dbg!(nu, exp, (nu - exp).abs());

        gamma = (73.0 as Float).to_radians();
        let nu = nusselt(ra, gamma, a_gi);
        let exp = 11.4398529673101;
        assert!((nu - exp).abs() < 1e-5, "Expecting {exp}... found {nu}");
        dbg!(nu, exp, (nu - exp).abs());

        gamma = (90.0 as Float).to_radians();
        let nu = nusselt(ra, gamma, a_gi);
        let exp = 11.2336334750340;
        assert!((nu - exp).abs() < 1e-5, "Expecting {exp}... found {nu}");
        dbg!(nu, exp, (nu - exp).abs());

        gamma = (134.0 as Float).to_radians();
        let nu = nusselt(ra, gamma, a_gi);
        let exp = 8.361460;
        assert!((nu - exp).abs() < 1e-5, "Expecting {exp}... found {nu}");
        dbg!(nu, exp, (nu - exp).abs());
    }
}
