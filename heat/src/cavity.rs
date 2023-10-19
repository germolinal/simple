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

use crate::gas::Gas;
use crate::SIGMA;

/// Represents some gas enclosed by two solid
/// materials
#[derive(Debug, Clone)]
pub struct Cavity {
    /// The distance between the two materials, in $`m`$
    pub thickness: Float,

    /// Height of the `Cavity`. Defined by ISO15099/2003 as "the distance between the top and bottom of the fill
    /// gas cavity which is usually the same as the height of the window view area.". Also, $`d`$ is the
    /// thickness of the cavity."
    pub height: Float,

    /// The gas contained
    pub gas: Gas,

    /// The thermal emissivity of the material at the outer side
    /// of the cavity
    pub eout: Float,

    /// The thermal emissivity of the material at the inner side
    /// of the cavity
    pub ein: Float,

    /// The angle of the cavity in radians. $`0`$ is horizontal; $`\pi/2`$ (i.e., $`90^o`$) is vertical.
    pub angle: Float,
}

impl Cavity {
    /// Calculates the `U-value`—including convective and radiative heat transfer—of a
    /// cavity, so that $`U_{cavity}\times \Delta T = q`$
    ///
    /// ```math
    /// U_{cavity} = \frac{4*{T_m}^3 * \Sigma  \epsilon_1 \epsilon_2}{1-(1-\epsilon_1)(1-\epsilon_2)} + h_{conv}
    /// ```
    pub fn u_value(&self, t_front: Float, t_back: Float) -> Float {
        let conv =
            self.gas
                .cavity_convection(self.height, self.thickness, self.angle, t_front, t_back);
        let tm = (t_back + t_front) / 2. + 273.15;

        let rad = 4. * tm.powi(3) * SIGMA * self.ein * self.eout
            / (1. - (1. - self.ein) * (1. - self.eout));

        rad + conv
    }
}

#[cfg(test)]
mod testing {
    use super::*;
    // use model::Substance;

    #[test]
    fn test_u_value() {
        let gap_thickness = 0.0127;

        let gap = Cavity {
            thickness: gap_thickness,
            height: 1.,
            gas: crate::gas::AIR,
            eout: 0.84,
            ein: 0.84,
            angle: crate::PI / 2.,
        };
        let t_out = 259.116115 - 273.15;
        let t_in = 279.323983 - 273.15;
        let u = gap.u_value(t_out, t_in);
        let exp_u = 0.069446 / gap_thickness;
        dbg!(u, exp_u);
    }
}
