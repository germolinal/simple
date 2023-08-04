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

use model::{Construction, Model, Substance};

use crate::Float;

/// An abstraction of a glazing layer for optical purposes.
///
/// All properties can be Solar or Visible spectral averages, or they
/// can be constrained to a specific wavelength. However, this library—because
/// it is all about heat transfer—only uses them in Solar purposes
#[derive(Debug, Clone, Copy)]
pub struct Glazing {
    /// Transmittance $`\tau`$
    tau: Float,

    /// Reflectance $`\rho_f`$ on the front side.
    rho_front: Float,

    /// Reflectance $`\rho_b`$ on the back side
    rho_back: Float,

    /// Absorbtance $`\alpha_f`$ on the front side
    alpha_front: Float,

    /// Absorbtance $`\alpha_b`$ on the back side
    alpha_back: Float,
}

impl Glazing {
    /// Creates a new `Glazing`
    pub fn new(tau: Float, rho_front: Float, rho_back: Float) -> Self {
        assert!(tau >= 0.0, "Found transmittance less than Zero");
        assert!(tau <= 1., "Found transmittance more than 1");
        assert!(rho_front >= 0.0, "Found front reflectance less than 0.");
        assert!(rho_back >= 0.0, "Found back reflectance less than 0");
        assert!(rho_front <= 1., "Found front reflectance more than 1.");
        assert!(rho_back <= 1., "Found back reflectance more than 1.");

        Self {
            tau,
            rho_back,
            rho_front,
            alpha_front: 1. - tau - rho_front,
            alpha_back: 1. - tau - rho_back,
        }
    }

    fn get_glazing_from_iter<T>(mut i: T, model: &Model, cap: usize) -> Result<Vec<Glazing>, String>
    where
        T: std::iter::Iterator<Item = String>,
    {
        let mut ret = Vec::with_capacity(cap);
        loop {
            // Get layer
            let mat_name = i.next().unwrap();
            let sub = model.get_material_substance(&mat_name)?;
            match sub {
                Substance::Gas(_) => {
                    // if it is a gas, something went wong.
                    panic!("NOT expecting a gas")
                }
                Substance::Normal(s) => {
                    // if it is a normal, the push it
                    let tau = s.solar_transmittance().unwrap_or(&0.0);
                    let alpha_front = s.front_solar_absorbtance().unwrap_or(&0.84);
                    let alpha_back = s.back_solar_absorbtance().unwrap_or(&0.84);
                    let rho_front = 1. - tau - alpha_front;
                    let rho_back = 1. - tau - alpha_back;
                    ret.push(Glazing::new(*tau, rho_front, rho_back));

                    // if not translucent, then we are done.
                    if *tau < 1e-9 {
                        break;
                    }
                }
            }
            // We only get here if we pushed somthing and need to continue... so, this should be a cavity... if any
            if let Some(mat_name) = i.next() {
                let sub = model.get_material_substance(&mat_name)?;
                if let Substance::Normal(_) = sub {
                    panic!("Expecting a Gas")
                }
            } else {
                // Else, we are done
                break;
            }
        }
        Ok(ret)
    }

    /// Gets the front translucent layers in a construction
    pub fn get_front_glazing_system(
        construction: &Construction,
        model: &Model,
    ) -> Result<Vec<Glazing>, String> {
        if construction.materials.is_empty() {
            return Err(format!(
                "Trying to get front_glazing_system of an empty construction, called '{}'",
                construction.name()
            ));
        }

        let i = construction.materials.iter().cloned();
        Self::get_glazing_from_iter(i, model, construction.materials.len())
    }

    /// Gets the front back translucent in a construction
    pub fn get_back_glazing_system(
        construction: &Construction,
        model: &Model,
    ) -> Result<Vec<Glazing>, String> {
        let i = construction.materials.iter().cloned().rev();
        Self::get_glazing_from_iter(i, model, construction.materials.len())
    }

    /// Gets the transmittance
    pub fn tau(&self) -> Float {
        self.tau
    }

    /// Gets the front reflectance
    pub fn rho_front(&self) -> Float {
        self.rho_front
    }

    /// Gets the back reflectance
    pub fn rho_back(&self) -> Float {
        self.rho_back
    }

    /// Gets the front absorbtance
    pub fn alpha_front(&self) -> Float {
        self.alpha_front
    }

    /// Gets the back absorbtance
    pub fn alpha_back(&self) -> Float {
        self.alpha_back
    }

    /// Calculates the overall transmittance of a system of two glazing layers
    ///
    /// Source: ISO-9050/2003, Equation 2
    ///
    /// ```math
    /// \tau_{1-2} = \frac{\tau_1 \tau_2}{1 - \rho'_1 \rho_2}
    /// ```
    ///
    /// This equation—along with the others in this module—can
    /// be used recursively, treating two layers combined as a single one
    /// and then attaching a third one. That is to say:
    ///
    /// ```math
    ///
    /// \tau_{1-3} = \frac{\tau_{1-2} \times \tau_3}{1 - \rho'_{1-2} \rho_3}
    /// ```
    pub fn combined_tau(&self, other: &Self) -> Float {
        self.tau * other.tau / (1. - self.rho_back * other.rho_front)
    }

    /// Calculates the overall front reflectance of a system of two glazing layers
    ///
    /// Source: ISO-9050/2003, Equation 5
    ///
    /// ```math
    /// \rho_{1-2} = \rho_1 + \frac{{\tau_1}^2 \rho_2}{1 - \rho'_1 \rho_2}
    /// ```
    pub fn combined_rho_front(&self, other: &Self) -> Float {
        self.rho_front + self.tau.powi(2) * other.rho_front / (1. - self.rho_back * other.rho_front)
    }

    /// Calculates the overall back reflectance of a system of two glazing layers
    ///
    /// This equation is not explicitly written on the standard I think, but we
    /// can derive it by drawing the system and assinging the corresponding values
    /// to Equation 5 of the same standard
    ///     
    /// ```math
    /// \rho'_{1-2} = \rho'_2 + \frac{{\tau_2}^2 \rho'_1}{1 - \rho_2 \rho'_1}
    /// ```
    pub fn combined_rho_back(&self, other: &Self) -> Float {
        other.rho_back + other.tau.powi(2) * self.rho_back / (1. - other.rho_front * self.rho_back)
    }

    /// Combines two `Glazing` into a new `Glazing`
    ///
    /// This method returns the equivalent glazing layer
    /// resulting after combining `self` with another `Glazing`,
    /// recalculating the reflectances and transmittance
    pub fn combine(&self, other: &Self) -> Self {
        let rho_back = self.combined_rho_back(other);
        let rho_front = self.combined_rho_front(other);
        let tau = self.combined_tau(other);
        Self::new(tau, rho_front, rho_back)
    }

    /// Combines several `Glazing` into a new `Glazing`
    pub fn combine_layers(layers: &[Glazing]) -> Self {
        if layers.len() == 1 {
            // if only one, then return.
            layers[0]
        } else {
            // otherwise, divide and conquer.
            let rest = Self::combine_layers(&layers[1..]);
            layers[0].combine(&rest)
        }
    }

    /// Calculates the front solar absorbtance of two `Glazing`
    /// according to Equations 17 and 18 of ISO9050/2003
    ///
    /// The resulting absorbtances are
    ///
    /// ```math
    /// \alpha_{e1} = \alpha_1 + \frac{\alpha'_1 \tau_1 \rho_2 }{1 - \rho'_1 \rho_2}
    ///
    /// ```
    ///
    /// and
    ///
    /// ```math
    /// \alpha_{e2} = \frac{\alpha_2 \tau_1}{1 - \rho'_1 \rho_2}
    /// ```
    pub fn combined_alphas(&self, other: &Self) -> (Float, Float) {
        let denom = 1. - self.rho_back * other.rho_front;
        let a1 = self.alpha_front + self.alpha_back * self.tau * other.rho_front / denom;
        let a2 = other.alpha_front * self.tau / denom;
        (a1, a2)
    }

    /// Calculates the absorbtances of each `Glazing` of the system, proportional
    /// to the incident radiation (i.e., they do not add up to 1.0)
    ///
    /// This function assumes that there is a layer of air (i.e., a cavity)
    /// between the glazing layers.
    pub fn alphas(layers: &[Glazing]) -> Vec<Float> {
        let mut ret = Vec::with_capacity(layers.len());

        // Trivial cases
        if layers.is_empty() {
            return ret;
        } else if layers.len() == 1 {
            ret.push(layers[0].alpha_front);
            return ret;
        }

        let mut acc_alpha = 0.0;

        for i in 1..layers.len() {
            let g0 = Self::combine_layers(&layers[0..i]);
            let g1 = Self::combine_layers(&layers[i..]);
            let (a0, _) = g0.combined_alphas(&g1);
            ret.push(a0 - acc_alpha);
            acc_alpha = a0;
        }

        // fill the last one
        let g0 = Self::combine_layers(&layers[0..layers.len() - 1]);
        let g1 = layers.last().unwrap();
        let (_, a1) = g0.combined_alphas(g1);
        ret.push(a1);
        ret
    }
}

#[cfg(test)]
mod testing {

    use super::*;
    use model::Material;

    #[test]
    fn test_get_glazing_two_layers() {
        let mut model = Model::default();

        // Layer 0
        let mut s0 = model::substance::Normal::new("sub0");
        s0.set_solar_transmittance(0.2)
            .set_front_solar_absorbtance(0.3)
            .set_back_solar_absorbtance(0.4);
        let s0 = s0.wrap();
        let s0 = model.add_substance(s0);

        let m0 = Material::new("m0".to_string(), s0.name().clone(), 0.001);
        let m0 = model.add_material(m0);

        // Cavity
        let mut c0 = model::substance::gas::Gas::new("c0");
        c0.set_gas(model::substance::gas::GasSpecification::Air);
        let c0 = c0.wrap();
        let c0 = model.add_substance(c0);

        let c0 = Material::new("cavity".to_string(), c0.name().clone(), 0.01);
        let c0 = model.add_material(c0);

        // Layer 1
        let mut s1 = model::substance::Normal::new("sub1");
        s1.set_solar_transmittance(0.1)
            .set_front_solar_absorbtance(0.2)
            .set_back_solar_absorbtance(0.3);
        let s1 = s1.wrap();
        let s1 = model.add_substance(s1);

        let m1 = Material::new("m1".to_string(), s1.name().clone(), 0.001);
        let m1 = model.add_material(m1);

        let mut construction = Construction::new("constr");
        construction.materials.push(m0.name().clone());
        construction.materials.push(c0.name().clone());
        construction.materials.push(m1.name().clone());

        let construction = model.add_construction(construction);

        // Test
        let glazings = Glazing::get_front_glazing_system(&construction, &model).unwrap();
        assert_eq!(glazings.len(), 2);
        let props: Vec<(Float, Float, Float)> = glazings
            .iter()
            .map(|g| (g.tau, g.rho_front, g.rho_back))
            .collect();
        assert_eq!(
            props,
            vec![
                (0.2, 1.0 - 0.2 - 0.3, 1.0 - 0.2 - 0.4),
                (0.1, 1.0 - 0.1 - 0.2, 1.0 - 0.1 - 0.3),
            ]
        );

        let glazings = Glazing::get_back_glazing_system(&construction, &model).unwrap();
        assert_eq!(glazings.len(), 2);
        let props: Vec<(Float, Float, Float)> = glazings
            .iter()
            .map(|g| (g.tau, g.rho_front, g.rho_back))
            .collect();
        assert_eq!(
            props,
            vec![
                (0.1, 1.0 - 0.1 - 0.2, 1.0 - 0.1 - 0.3),
                (0.2, 1.0 - 0.2 - 0.3, 1.0 - 0.2 - 0.4),
            ]
        );
    }

    #[test]
    fn test_get_glazing_two_layers_2() {
        let mut model = Model::default();

        // Layer 0
        let mut s0 = model::substance::Normal::new("sub0".to_string());
        s0.set_solar_transmittance(0.0)
            .set_front_solar_absorbtance(0.1)
            .set_back_solar_absorbtance(0.2);
        let s0 = s0.wrap();
        let s0 = model.add_substance(s0);

        let m0 = Material::new("m0".to_string(), s0.name().clone(), 0.001);
        let m0 = model.add_material(m0);

        // Cavity
        let mut c0 = model::substance::gas::Gas::new("c0");
        c0.set_gas(model::substance::gas::GasSpecification::Air);
        let c0 = c0.wrap();
        let c0 = model.add_substance(c0);

        let c0 = Material::new("cavity".to_string(), c0.name().clone(), 0.01);
        let c0 = model.add_material(c0);

        // Layer 1
        let mut s1 = model::substance::Normal::new("sub1");
        s1.set_solar_transmittance(0.1)
            .set_front_solar_absorbtance(0.2)
            .set_back_solar_absorbtance(0.3);
        let s1 = s1.wrap();
        let s1 = model.add_substance(s1);

        let m1 = Material::new("m1".to_string(), s1.name().clone(), 0.001);
        let m1 = model.add_material(m1);

        let mut construction = Construction::new("constr");
        construction.materials.push(m0.name().clone());
        construction.materials.push(c0.name().clone());
        construction.materials.push(m1.name().clone());
        let construction = model.add_construction(construction);

        // Test
        let glazings = Glazing::get_front_glazing_system(&construction, &model).unwrap();
        assert_eq!(glazings.len(), 1);
        let props: Vec<(Float, Float, Float)> = glazings
            .iter()
            .map(|g| (g.tau, g.rho_front, g.rho_back))
            .collect();
        assert_eq!(props, vec![(0.0, 0.9, 0.8)]);
        let alphas = Glazing::alphas(&glazings);
        let exp = vec![0.1];
        for (found, exp) in alphas.iter().zip(exp.iter()) {
            validate::assert_close!(*exp, *found);
        }

        let glazings = Glazing::get_back_glazing_system(&construction, &model).unwrap();
        assert_eq!(glazings.len(), 2);
        let props: Vec<(Float, Float, Float)> = glazings
            .iter()
            .map(|g| (g.tau, g.rho_front, g.rho_back))
            .collect();
        let exp = vec![(0.1, 1.0 - 0.1 - 0.2, 1.0 - 0.1 - 0.3), (0.0, 0.9, 0.8)];
        assert_eq!(props, exp);
    }

    #[test]
    fn test_9050() {
        let tau1 = 0.1;
        let rho_b1 = 0.3;
        let rho_f1 = 0.13;
        let g1 = Glazing::new(tau1, rho_f1, rho_b1);

        let tau2 = 0.21;
        let rho_b2 = 0.34;
        let rho_f2 = 0.1123;
        let g2 = Glazing::new(tau2, rho_f2, rho_b2);

        // Eq. 2 of ISO9050/2003
        let tau12 = g1.combined_tau(&g2);
        let exp = tau1 * tau2 / (1. - rho_b1 * rho_f2);
        assert!((tau12 - exp).abs() < 1e-15);

        // Eq. 5 of ISO9050/2003
        let rho_f12 = g1.combined_rho_front(&g2);
        let exp = rho_f1 + tau1 * tau1 * rho_f2 / (1. - rho_b1 * rho_f2);
        assert!((rho_f12 - exp).abs() < 1e-15);

        let tau3 = 0.21;
        let rho_b3 = 0.34;
        let rho_f3 = 0.1123;
        let g3 = Glazing::new(tau3, rho_f3, rho_b3);

        let g12 = g1.combine(&g2);
        let g13 = g12.combine(&g3);

        // Eq. 3 of ISO9050/2003
        let exp = tau1 * tau2 * tau3
            / ((1. - rho_b1 * rho_f2) * (1. - rho_b2 * rho_f3) - tau2.powi(2) * rho_b1 * rho_f3);
        assert!((exp - g13.tau()).abs() < 1e-15);

        // Eq. 6 of ISO9050/2003
        let exp = rho_f1
            + (tau1 * tau1 * rho_f2 * (1. - rho_b2 * rho_f3) + tau1 * tau1 * tau2 * tau2 * rho_f3)
                / ((1. - rho_b1 * rho_f2) * (1. - rho_b2 * rho_f3) - tau2 * tau2 * rho_b1 * rho_f3);
        assert!((exp - g13.rho_front()).abs() < 1e-15);

        // test integration
        let other_g13 = Glazing::combine_layers(&[g1, g2, g3]);
        assert!((g13.tau - other_g13.tau).abs() < 1e-15);
        assert!((g13.rho_front - other_g13.rho_front).abs() < 1e-15);
        assert!((g13.rho_back - other_g13.rho_back).abs() < 1e-15);
        assert!((g13.alpha_back - other_g13.alpha_back).abs() < 1e-15);
        assert!((g13.alpha_front - other_g13.alpha_front).abs() < 1e-15);

        // Test alphas
        let alphas = Glazing::alphas(&[g1, g2, g3]);
        let found: Float = alphas.iter().sum();
        assert!(
            (found - g13.alpha_front).abs() < 1e-15,
            "expecting {}, found {}",
            g13.alpha_front,
            found
        );

        let a_f1 = g1.alpha_front;
        let a_b1 = g1.alpha_back;
        let a_f2 = g2.alpha_front;
        let a_b2 = g2.alpha_back;
        let a_f3 = g3.alpha_front;

        // Equations 23-25 of ISO9050/2003
        let denom = (1. - rho_b1 * rho_f2) * (1. - rho_b2 * rho_f3) - tau2 * tau2 * rho_b1 * rho_f3;
        let exp_a1 = a_f1
            + (tau1 * a_b1 * rho_f2 * (1. - rho_b2 * rho_f3) + tau1 * tau2 * tau2 * a_b1 * rho_f3)
                / denom;
        let exp_a2 = (tau1 * a_f2 * (1. - rho_b2 * rho_f3) + tau1 * tau2 * a_b2 * rho_f3) / denom;
        let exp_a3 = (tau1 * tau2 * a_f3) / denom;

        #[cfg(feature = "float")]
        const SMOL: Float = 1e-10;
        #[cfg(not(feature = "float"))]
        const SMOL: Float = 1e-15;

        assert!(
            (alphas[0] - exp_a1).abs() < SMOL,
            "Expecting {} ... found {}",
            exp_a1,
            alphas[0]
        );
        assert!(
            (alphas[1] - exp_a2).abs() < SMOL,
            "Expecting {} ... found {}",
            exp_a2,
            alphas[1]
        );
        assert!(
            (alphas[2] - exp_a3).abs() < SMOL,
            "Expecting {} ... found {}",
            exp_a3,
            alphas[2]
        );
    }
}
