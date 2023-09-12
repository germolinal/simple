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

use crate::{Float, PI};
use geometry::Vector3D;

/// The number of patches in teach of Tregenza's (i.e.,
/// Reinhart's with MF=1) sky subdivisions. The angle
/// must come in Radians.
///
/// The most basic Reinhart's subdivision is equivalent to
/// Tregenza's one. It has 7 rows of patches (going from bottom,
/// i.e., altitude of 0.0, to top, i.e., alt. of almost 90 deg)
/// plus a Cap at the top.
const TNAZ: [usize; 7] = [30, 30, 24, 24, 18, 12, 6];

/// Calculates the solid angle of a Cone.
fn cone_solid_angle(angle: Float) -> Float {
    2.0 * PI * (1. - angle.cos())
}

fn patch_solid_angle(top_angle: Float, low_angle: Float, bins_in_row: usize) -> Float {
    // This comes from genskymtx's code
    2. * PI * (top_angle.sin() - low_angle.sin()) / bins_in_row as Float
    /*
    In my Master's thesis I had a different equation for this:
    let sin_top = top_angle.sin();
    let sin_low = low_angle.sin();
    PI * ( sin_top * sin_top - sin_low * sin_low ) / bins_in_row as Float
    */
}

/// Returns the number of elements in row `row`, starting from
/// 0.
fn bins_in_row(mf: usize, row: usize) -> usize {
    // This is missing the cap, which has only one element
    let nrows = 7 * mf;
    match row.cmp(&nrows) {
        std::cmp::Ordering::Equal => 1,
        std::cmp::Ordering::Greater => {
            panic!(
                "The value received is larger than the number of rows! row = {}, nrows = {}",
                row, nrows
            )
        }
        _ => {
            // Somewhere in between
            let i = ((row as f32 + 0.5) / mf as f32).floor() as usize;
            mf * TNAZ[i]
        }
    }
}

/// Returns the height of a full row (the cap is only half
/// this height)
fn row_height(mf: usize) -> Float {
    PI / 2. / ((TNAZ.len() * mf) as Float + 0.5)
}

/// A structure that helps creating discretized Skies, using Reinhart's discretization
pub struct ReinhartSky {
    /// Subdivition scheme
    pub mf: usize,

    /// The number of rows, including Cap
    n_rows: usize,

    /// An array with the Sin(max angle) for every row.
    ///
    /// This is useful for transforming a vector into a bin,
    /// as it avoids the use of slow trigonometric functions.
    row_max_sin: Vec<Float>,

    /// An array with the number of bins accumulated before each row.
    acc_bins: Vec<usize>,

    /// The number of elements in this sky discretization (including ground)
    pub n_bins: usize,
}

impl ReinhartSky {
    /// Calculates the number of total bins in a Reinhart's discretization
    /// including the ground
    pub fn n_bins(mf: usize) -> usize {
        // 144 Tregenza divided in MF rows and cols; + Ground + Cap
        144 * mf.pow(2) + 2

        // This below is what is written in Radiance's cal files, I think
        // fn raccum(mf: usize, row: usize) -> usize {
        //     if row == 0 {
        //         0
        //     } else {
        //         bins_in_row(mf, row - 1) + raccum(mf, row - 1)
        //     }
        // }

        // raccum(mf, 7 * mf + 1) + 1
    }

    /// Calculates the row in which a certain bin is located
    fn bins_row(&self, nbin: usize) -> usize {
        if nbin > self.n_bins {
            panic!("Trying to get the row of a bin out of bounds... found {} but only {} are available", nbin, self.n_bins);
        }

        let mut row = self.n_rows - 1; // If not found, this is the value
        for (this_row, acc) in self.acc_bins.iter().enumerate() {
            if nbin < *acc + 1 {
                row = this_row - 1;
                break;
            }
        }
        row
    }

    /// Calculates te solid angle of a certain bin
    pub fn bin_solid_angle(&self, nbin: usize) -> Float {
        if nbin == 0 {
            // Ground... it is half a sphere
            return 2. * PI;
        }
        if nbin == self.n_bins - 1 {
            // Cap... a small cone
            let height = row_height(self.mf);
            return cone_solid_angle(height / 2.);
        }
        // Error
        if nbin > self.n_bins {
            panic!("Trying to get direction to a bin out of bounds... found {} but only {} are available", nbin, self.n_bins);
        }
        // All others

        let row = self.bins_row(nbin);
        let (low, high) = self.row_altitude(row);
        let n_bins = bins_in_row(self.mf, row);
        patch_solid_angle(high, low, n_bins)
    }

    /// The number of sky elements accumulated up to row `row` (inclusive).
    /// Rows are Zero-indexed (i.e., the first one is 0)
    fn raccum(&self, row: usize) -> usize {
        match row.cmp(&self.n_bins) {
            std::cmp::Ordering::Equal => self.n_bins,
            std::cmp::Ordering::Greater => {
                panic!(
                    "Trying to access an accumulated number of bins in row {}... only {} available",
                    row, self.n_rows
                )
            }
            _ => self.acc_bins[row],
        }
    }

    /// Returns the minimum and maximum altitude of a row, in Radians
    fn row_altitude(&self, row: usize) -> (Float, Float) {
        let row_height = row_height(self.mf);
        if row == self.n_rows - 1 {
            // Cap
            (PI / 2. - row_height / 2., PI / 2.)
        } else {
            // others
            let row = row as Float;
            (row * row_height, (row + 1.) * row_height)
        }
    }

    /// Gets the sin of an altitude
    ///
    /// E.g., Receives the `z` component of a direction
    /// (if the vector is normalized) and returns the row towards which that
    /// vector is pointing.
    fn sin_altitude_to_row(&self, sin_altitude: Float) -> usize {
        if sin_altitude < 0. {
            // Points ot the ground... This should be handled in the caller function.
            panic!("Trying to call sin_altitude_to_row() by using a negative sin_altitude (e.g., does not point to the sky")
        }
        for (row, max_sin) in self.row_max_sin.iter().enumerate() {
            if sin_altitude < *max_sin {
                return row;
            }
        }

        self.n_rows - 1
    }

    /// Gets the position of the bin within a row.
    ///
    /// E.g., gets the `x` and `y` components of a vector pointing
    /// to the Sky and returns the bin pointed within that row
    fn xy_to_bin(&self, row: usize, x: Float, y: Float) -> usize {
        // If it is the cap, don't think about it and return 1
        if row == self.n_rows {
            return 1;
        }
        // Get the angle in Radians
        // atan(x/y), not atan(y/x) because azimuth 0 is pointing towards Y
        let mut azimuth = x.atan2(y);
        if azimuth < 0. {
            azimuth += 2. * PI;
        }

        // Size of a Bin in Radians
        let in_row = bins_in_row(self.mf, row);
        let bin_size = 2. * PI / in_row as Float;

        // Y direction points towards the centre of the first patch...
        // so we need to add Half a bin size.
        ((azimuth + bin_size / 2.) / bin_size).floor() as usize % in_row
    }

    /// Returns the bin number pointed by direction `dir`
    pub fn dir_to_bin(&self, dir: Vector3D) -> usize {
        // Check that this is normalized in debug mode
        debug_assert!((1. - dir.length()).abs() < 1e-5);

        // Pointing the ground.
        if dir.z < 0. {
            return 0;
        }

        // Get row
        let row = self.sin_altitude_to_row(dir.z);
        if row == self.n_rows - 1 {
            return self.n_bins - 1;
        }

        let previous_bins = self.raccum(row);
        // Get current bin within row
        let bin = self.xy_to_bin(row, dir.x, dir.y);

        // Return (count the ground.)
        bin + previous_bins + 1
    }

    /// Gets a `Vector3D` pointing to the centre the bin.
    pub fn bin_dir(&self, nbin: usize) -> Vector3D {
        if nbin == 0 {
            // Ground
            return Vector3D::new(0., 0., -1.);
        }
        if nbin == self.n_bins - 1 {
            // Cap
            return Vector3D::new(0., 0., 1.);
        }
        // Error
        if nbin > self.n_bins {
            panic!("Trying to get direction to a bin out of bounds... found {} but only {} are available", nbin, self.n_bins);
        }

        /*******************/
        /* ALL OTHER CASES */
        /*******************/

        // First, find the row in which this bin is located
        // and then extract the altitude from it.
        let row = self.bins_row(nbin);

        // Calculate the altitude angle.
        let (min_alt, max_alt) = self.row_altitude(row);
        let alt = (min_alt + max_alt) / 2.;

        // Now, calculate the azimuth
        let prev_bins = self.raccum(row);
        let n_bins = bins_in_row(self.mf, row);
        let bin_width = 2. * PI / n_bins as Float;
        let azi = bin_width * (nbin - prev_bins - 1) as Float;

        // Compute
        let cos_ralt = alt.cos();
        let dx = azi.sin() * cos_ralt;
        let dy = azi.cos() * cos_ralt;
        let dz = alt.sin();

        // if (solidAngle != NULL) {
        //     if (row == (n_rows-1)) {
        //         // if polar cap
        //         *solidAngle = coneSolidAngle(RAH/2.0);
        //     }
        //     else {
        //         *solidAngle = 2.0 * PI * (sin(alt + RAH/2.0)-sin(alt - RAH/2.0)) / (double)nBins;
        //     }
        // }

        Vector3D::new(dx, dy, dz)
    }

    /// Creates  a new Reinhart sky discretization
    pub fn new(mf: usize) -> Self {
        if mf == 0 {
            panic!("When creating ReinhartSky: mf must be 1 or greater")
        }

        let n_rows = 7 * mf + 1;

        let height = row_height(mf);
        let mut acc = 0.0;
        let mut row_max_sin = Vec::with_capacity(n_rows - 1);
        let mut acc_bins = Vec::with_capacity(n_rows - 1);
        for row in 0..n_rows {
            // Fill the sin of the max angle of each row
            acc += height;
            row_max_sin.push(acc.sin());

            // Fill the accumulated number of bins after each line.
            if row == 0 {
                acc_bins.push(0)
            } else {
                acc_bins.push(acc_bins[row - 1] + bins_in_row(mf, row - 1))
            }
        }

        // Cap + Ground + others
        let n_bins = 1 + acc_bins[n_rows - 1] + 1;

        // Return
        Self {
            mf,
            n_rows,
            n_bins,
            row_max_sin,
            acc_bins,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dir_to_bin() {
        // These were produced by running the following command:
        // `echo 123 | rcalc -e MF:1 -e Dx:0.2 -e Dy:1 -e Dz:0 -f $RAYPATH/reinhart.cal -o '${rbin - 1}'`

        let r = ReinhartSky::new(1);

        let dir = Vector3D::new(0., 1., -1.).get_normalized();
        let bin = r.dir_to_bin(dir);
        assert_eq!(bin, 0);

        let dir = Vector3D::new(0., 1., 0.).get_normalized();
        let bin = r.dir_to_bin(dir);
        assert_eq!(bin, 1);

        let dir = Vector3D::new(0.1, 1., 0.).get_normalized();
        let bin = r.dir_to_bin(dir);
        assert_eq!(bin, 1);

        let dir = Vector3D::new(0.112, 1., 0.).get_normalized();
        let bin = r.dir_to_bin(dir);
        assert_eq!(bin, 2);

        let dir = Vector3D::new(0.2, 1., 0.).get_normalized();
        let bin = r.dir_to_bin(dir);
        assert_eq!(bin, 2);

        let dir = Vector3D::new(0.4, 1., 0.).get_normalized();
        let bin = r.dir_to_bin(dir);
        assert_eq!(bin, 3);

        let dir = Vector3D::new(-0.4, 1., 0.).get_normalized();
        let bin = r.dir_to_bin(dir);
        assert_eq!(bin, 29);

        // MF = 2
        let r = ReinhartSky::new(2);
        let dir = Vector3D::new(-0.4, 1., 0.).get_normalized();
        let bin = r.dir_to_bin(dir);
        assert_eq!(bin, 57);

        let dir = Vector3D::new(0.4, 1., 0.).get_normalized();
        let bin = r.dir_to_bin(dir);
        assert_eq!(bin, 5);

        let dir = Vector3D::new(-0.4, 1., 0.9999).get_normalized();
        let bin = r.dir_to_bin(dir);
        assert_eq!(bin, 382);

        let dir = Vector3D::new(0., 1., -1.).get_normalized();
        let bin = r.dir_to_bin(dir);
        assert_eq!(bin, 0);
    }

    #[test]
    fn test_row_altitude() -> Result<(), String> {
        fn check(mf: usize) -> Result<(), String> {
            let r = ReinhartSky::new(mf);
            let height = PI / 2. / ((7 * mf) as Float + 0.5);
            let mut acc_alt = 0.;
            for row in 0..r.n_rows {
                let (exp_min, exp_max) = if row == r.n_rows - 1 {
                    (acc_alt, PI / 2.)
                } else {
                    (acc_alt, acc_alt + height)
                };

                // Tests the values
                let (min, max) = r.row_altitude(row);

                if (min - exp_min).abs() > 1e-9 {
                    return Err(format!(
                        "MIN does not match: row = {} | Exp_min = {} min = {} ",
                        row,
                        exp_min * 180. / PI,
                        min * 180. / PI
                    ));
                }
                if (max - exp_max).abs() > 1e-9 {
                    return Err(format!(
                        "MAX does not match: row = {} | Exp_max = {} | max = {}",
                        row,
                        exp_max * 180. / PI,
                        max * 180. / PI
                    ));
                }

                // Test reverse function
                let r = ReinhartSky::new(mf);
                let steps = 100;
                for i in 0..steps {
                    let angle = exp_min + (i as Float / steps as Float) * (exp_max - exp_min);
                    let z = angle.sin();
                    let found_row = r.sin_altitude_to_row(z);
                    if row != found_row {
                        return Err(format!("Row does not match... Exp_min = {}, Exp_max = {}, angle = {} ... expected_row = {}, but found row = {}", exp_min*180./PI, exp_max*180./PI, angle*180./PI, row, found_row));
                    }
                }

                // advance
                acc_alt += height;
            }
            Ok(())
        }

        check(1)?;
        check(2)?;
        check(9)?;
        Ok(())
    }

    #[test]
    fn test_cone_solid_angles() {
        assert!((2. * PI - cone_solid_angle(PI / 2.)).abs() < 1e-9);
        assert!(cone_solid_angle(0.) < 1e-9);
    }

    #[test]
    fn test_rnaz() {
        let test_rnaz = |mf: usize| {
            const TNAZ: [usize; 7] = [30, 30, 24, 24, 18, 12, 6];
            for main_row in 0..=7 {
                for i in 0..mf {
                    let row = mf * main_row + i;
                    if main_row == 7 {
                        assert_eq!(bins_in_row(mf, row), 1);
                        return; // stop test here
                    } else {
                        assert_eq!(bins_in_row(mf, row), TNAZ[main_row] * mf);
                    }
                }
            }
        };

        test_rnaz(1);
        test_rnaz(2);
        test_rnaz(3);
        test_rnaz(4);
    }

    #[test]
    fn test_raccum() {
        let r = ReinhartSky::new(1);
        assert_eq!(r.raccum(0), 0);
        assert_eq!(r.raccum(1), 30);
        assert_eq!(r.raccum(2), 60);
        assert_eq!(r.raccum(3), 84);
        assert_eq!(r.raccum(4), 84 + 24);
        assert_eq!(r.raccum(5), 84 + 24 + 18);
        assert_eq!(r.raccum(6), 84 + 24 + 18 + 12);
        assert_eq!(r.raccum(7), 144);

        let r = ReinhartSky::new(2);
        assert_eq!(r.raccum(0), 0);
        assert_eq!(r.raccum(1), 60);
        assert_eq!(r.raccum(2), 120);
        assert_eq!(r.raccum(3), 180);
        assert_eq!(r.raccum(4), 240);
        assert_eq!(r.raccum(5), 240 + 48);
    }

    #[test]
    fn test_n_bins() {
        let r = ReinhartSky::new(1);
        assert_eq!(r.n_bins, 146);
        assert_eq!(ReinhartSky::n_bins(1), 146);

        let r = ReinhartSky::new(2);
        assert_eq!(r.n_bins, 578);
        assert_eq!(ReinhartSky::n_bins(2), 578);

        let r = ReinhartSky::new(3);
        assert_eq!(r.n_bins, 1298);
        assert_eq!(ReinhartSky::n_bins(3), 1298);

        let r = ReinhartSky::new(4);
        assert_eq!(r.n_bins, 2306);
        assert_eq!(ReinhartSky::n_bins(4), 2306);
    }

    #[test]
    fn test_n_rows() {
        let r = ReinhartSky::new(1);
        assert_eq!(r.n_rows, 8);
        let r = ReinhartSky::new(2);
        assert_eq!(r.n_rows, 15);
        let r = ReinhartSky::new(3);
        assert_eq!(r.n_rows, 22);
        let r = ReinhartSky::new(4);
        assert_eq!(r.n_rows, 29);
    }

    #[test]
    fn test_bin_dir() {
        let r = ReinhartSky::new(1);

        assert!(r.bin_dir(0).compare(Vector3D::new(0., 0., -1.)));

        // Test automatically produced by using command:
        // cnt 145 | rcalc -e MF:1 -e Rbin=recno -f $RAYPATH/reinsrc.cal -o 'assert!( (r.bin_dir(${recno}) - Vector3D::new(${Dx}, ${Dy}, ${Dz})).length() < 1e-6);assert_eq!(${recno}, r.dir_to_bin(Vector3D::new(${Dx}, ${Dy}, ${Dz})) );'
        assert!((r.bin_dir(1) - Vector3D::new(0., 0.994522, 0.104528)).length() < 1e-6);
        assert_eq!(1, r.dir_to_bin(Vector3D::new(0., 0.994522, 0.104528)));
        assert!((r.bin_dir(2) - Vector3D::new(0.206773, 0.972789, 0.104528)).length() < 1e-6);
        assert_eq!(2, r.dir_to_bin(Vector3D::new(0.206773, 0.972789, 0.104528)));
        assert!((r.bin_dir(3) - Vector3D::new(0.404508, 0.908541, 0.104528)).length() < 1e-6);
        assert_eq!(3, r.dir_to_bin(Vector3D::new(0.404508, 0.908541, 0.104528)));
        assert!((r.bin_dir(4) - Vector3D::new(0.584565, 0.804585, 0.104528)).length() < 1e-6);
        assert_eq!(4, r.dir_to_bin(Vector3D::new(0.584565, 0.804585, 0.104528)));
        assert!((r.bin_dir(5) - Vector3D::new(0.739074, 0.665465, 0.104528)).length() < 1e-6);
        assert_eq!(5, r.dir_to_bin(Vector3D::new(0.739074, 0.665465, 0.104528)));
        assert!((r.bin_dir(6) - Vector3D::new(0.861281, 0.497261, 0.104528)).length() < 1e-6);
        assert_eq!(6, r.dir_to_bin(Vector3D::new(0.861281, 0.497261, 0.104528)));
        assert!((r.bin_dir(7) - Vector3D::new(0.945847, 0.307324, 0.104528)).length() < 1e-6);
        assert_eq!(7, r.dir_to_bin(Vector3D::new(0.945847, 0.307324, 0.104528)));
        assert!((r.bin_dir(8) - Vector3D::new(0.989074, 0.103956, 0.104528)).length() < 1e-6);
        assert_eq!(8, r.dir_to_bin(Vector3D::new(0.989074, 0.103956, 0.104528)));
        assert!((r.bin_dir(9) - Vector3D::new(0.989074, -0.103956, 0.104528)).length() < 1e-6);
        assert_eq!(
            9,
            r.dir_to_bin(Vector3D::new(0.989074, -0.103956, 0.104528))
        );
        assert!((r.bin_dir(10) - Vector3D::new(0.945847, -0.307324, 0.104528)).length() < 1e-6);
        assert_eq!(
            10,
            r.dir_to_bin(Vector3D::new(0.945847, -0.307324, 0.104528))
        );
        assert!((r.bin_dir(11) - Vector3D::new(0.861281, -0.497261, 0.104528)).length() < 1e-6);
        assert_eq!(
            11,
            r.dir_to_bin(Vector3D::new(0.861281, -0.497261, 0.104528))
        );
        assert!((r.bin_dir(12) - Vector3D::new(0.739074, -0.665465, 0.104528)).length() < 1e-6);
        assert_eq!(
            12,
            r.dir_to_bin(Vector3D::new(0.739074, -0.665465, 0.104528))
        );
        assert!((r.bin_dir(13) - Vector3D::new(0.584565, -0.804585, 0.104528)).length() < 1e-6);
        assert_eq!(
            13,
            r.dir_to_bin(Vector3D::new(0.584565, -0.804585, 0.104528))
        );
        assert!((r.bin_dir(14) - Vector3D::new(0.404508, -0.908541, 0.104528)).length() < 1e-6);
        assert_eq!(
            14,
            r.dir_to_bin(Vector3D::new(0.404508, -0.908541, 0.104528))
        );
        assert!((r.bin_dir(15) - Vector3D::new(0.206773, -0.972789, 0.104528)).length() < 1e-6);
        assert_eq!(
            15,
            r.dir_to_bin(Vector3D::new(0.206773, -0.972789, 0.104528))
        );
        assert!((r.bin_dir(16) - Vector3D::new(1.21794e-16, -0.994522, 0.104528)).length() < 1e-6);
        assert_eq!(
            16,
            r.dir_to_bin(Vector3D::new(1.21794e-16, -0.994522, 0.104528))
        );
        assert!((r.bin_dir(17) - Vector3D::new(-0.206773, -0.972789, 0.104528)).length() < 1e-6);
        assert_eq!(
            17,
            r.dir_to_bin(Vector3D::new(-0.206773, -0.972789, 0.104528))
        );
        assert!((r.bin_dir(18) - Vector3D::new(-0.404508, -0.908541, 0.104528)).length() < 1e-6);
        assert_eq!(
            18,
            r.dir_to_bin(Vector3D::new(-0.404508, -0.908541, 0.104528))
        );
        assert!((r.bin_dir(19) - Vector3D::new(-0.584565, -0.804585, 0.104528)).length() < 1e-6);
        assert_eq!(
            19,
            r.dir_to_bin(Vector3D::new(-0.584565, -0.804585, 0.104528))
        );
        assert!((r.bin_dir(20) - Vector3D::new(-0.739074, -0.665465, 0.104528)).length() < 1e-6);
        assert_eq!(
            20,
            r.dir_to_bin(Vector3D::new(-0.739074, -0.665465, 0.104528))
        );
        assert!((r.bin_dir(21) - Vector3D::new(-0.861281, -0.497261, 0.104528)).length() < 1e-6);
        assert_eq!(
            21,
            r.dir_to_bin(Vector3D::new(-0.861281, -0.497261, 0.104528))
        );
        assert!((r.bin_dir(22) - Vector3D::new(-0.945847, -0.307324, 0.104528)).length() < 1e-6);
        assert_eq!(
            22,
            r.dir_to_bin(Vector3D::new(-0.945847, -0.307324, 0.104528))
        );
        assert!((r.bin_dir(23) - Vector3D::new(-0.989074, -0.103956, 0.104528)).length() < 1e-6);
        assert_eq!(
            23,
            r.dir_to_bin(Vector3D::new(-0.989074, -0.103956, 0.104528))
        );
        assert!((r.bin_dir(24) - Vector3D::new(-0.989074, 0.103956, 0.104528)).length() < 1e-6);
        assert_eq!(
            24,
            r.dir_to_bin(Vector3D::new(-0.989074, 0.103956, 0.104528))
        );
        assert!((r.bin_dir(25) - Vector3D::new(-0.945847, 0.307324, 0.104528)).length() < 1e-6);
        assert_eq!(
            25,
            r.dir_to_bin(Vector3D::new(-0.945847, 0.307324, 0.104528))
        );
        assert!((r.bin_dir(26) - Vector3D::new(-0.861281, 0.497261, 0.104528)).length() < 1e-6);
        assert_eq!(
            26,
            r.dir_to_bin(Vector3D::new(-0.861281, 0.497261, 0.104528))
        );
        assert!((r.bin_dir(27) - Vector3D::new(-0.739074, 0.665465, 0.104528)).length() < 1e-6);
        assert_eq!(
            27,
            r.dir_to_bin(Vector3D::new(-0.739074, 0.665465, 0.104528))
        );
        assert!((r.bin_dir(28) - Vector3D::new(-0.584565, 0.804585, 0.104528)).length() < 1e-6);
        assert_eq!(
            28,
            r.dir_to_bin(Vector3D::new(-0.584565, 0.804585, 0.104528))
        );
        assert!((r.bin_dir(29) - Vector3D::new(-0.404508, 0.908541, 0.104528)).length() < 1e-6);
        assert_eq!(
            29,
            r.dir_to_bin(Vector3D::new(-0.404508, 0.908541, 0.104528))
        );
        assert!((r.bin_dir(30) - Vector3D::new(-0.206773, 0.972789, 0.104528)).length() < 1e-6);
        assert_eq!(
            30,
            r.dir_to_bin(Vector3D::new(-0.206773, 0.972789, 0.104528))
        );
        assert!((r.bin_dir(31) - Vector3D::new(0., 0.951057, 0.309017)).length() < 1e-6);
        assert_eq!(31, r.dir_to_bin(Vector3D::new(0., 0.951057, 0.309017)));
        assert!((r.bin_dir(32) - Vector3D::new(0.197736, 0.930274, 0.309017)).length() < 1e-6);
        assert_eq!(
            32,
            r.dir_to_bin(Vector3D::new(0.197736, 0.930274, 0.309017))
        );
        assert!((r.bin_dir(33) - Vector3D::new(0.38683, 0.868833, 0.309017)).length() < 1e-6);
        assert_eq!(33, r.dir_to_bin(Vector3D::new(0.38683, 0.868833, 0.309017)));
        assert!((r.bin_dir(34) - Vector3D::new(0.559017, 0.769421, 0.309017)).length() < 1e-6);
        assert_eq!(
            34,
            r.dir_to_bin(Vector3D::new(0.559017, 0.769421, 0.309017))
        );
        assert!((r.bin_dir(35) - Vector3D::new(0.706773, 0.636381, 0.309017)).length() < 1e-6);
        assert_eq!(
            35,
            r.dir_to_bin(Vector3D::new(0.706773, 0.636381, 0.309017))
        );
        assert!((r.bin_dir(36) - Vector3D::new(0.823639, 0.475528, 0.309017)).length() < 1e-6);
        assert_eq!(
            36,
            r.dir_to_bin(Vector3D::new(0.823639, 0.475528, 0.309017))
        );
        assert!((r.bin_dir(37) - Vector3D::new(0.904508, 0.293893, 0.309017)).length() < 1e-6);
        assert_eq!(
            37,
            r.dir_to_bin(Vector3D::new(0.904508, 0.293893, 0.309017))
        );
        assert!((r.bin_dir(38) - Vector3D::new(0.945847, 0.0994125, 0.309017)).length() < 1e-6);
        assert_eq!(
            38,
            r.dir_to_bin(Vector3D::new(0.945847, 0.0994125, 0.309017))
        );
        assert!((r.bin_dir(39) - Vector3D::new(0.945847, -0.0994125, 0.309017)).length() < 1e-6);
        assert_eq!(
            39,
            r.dir_to_bin(Vector3D::new(0.945847, -0.0994125, 0.309017))
        );
        assert!((r.bin_dir(40) - Vector3D::new(0.904508, -0.293893, 0.309017)).length() < 1e-6);
        assert_eq!(
            40,
            r.dir_to_bin(Vector3D::new(0.904508, -0.293893, 0.309017))
        );
        assert!((r.bin_dir(41) - Vector3D::new(0.823639, -0.475528, 0.309017)).length() < 1e-6);
        assert_eq!(
            41,
            r.dir_to_bin(Vector3D::new(0.823639, -0.475528, 0.309017))
        );
        assert!((r.bin_dir(42) - Vector3D::new(0.706773, -0.636381, 0.309017)).length() < 1e-6);
        assert_eq!(
            42,
            r.dir_to_bin(Vector3D::new(0.706773, -0.636381, 0.309017))
        );
        assert!((r.bin_dir(43) - Vector3D::new(0.559017, -0.769421, 0.309017)).length() < 1e-6);
        assert_eq!(
            43,
            r.dir_to_bin(Vector3D::new(0.559017, -0.769421, 0.309017))
        );
        assert!((r.bin_dir(44) - Vector3D::new(0.38683, -0.868833, 0.309017)).length() < 1e-6);
        assert_eq!(
            44,
            r.dir_to_bin(Vector3D::new(0.38683, -0.868833, 0.309017))
        );
        assert!((r.bin_dir(45) - Vector3D::new(0.197736, -0.930274, 0.309017)).length() < 1e-6);
        assert_eq!(
            45,
            r.dir_to_bin(Vector3D::new(0.197736, -0.930274, 0.309017))
        );
        assert!((r.bin_dir(46) - Vector3D::new(1.16471e-16, -0.951057, 0.309017)).length() < 1e-6);
        assert_eq!(
            46,
            r.dir_to_bin(Vector3D::new(1.16471e-16, -0.951057, 0.309017))
        );
        assert!((r.bin_dir(47) - Vector3D::new(-0.197736, -0.930274, 0.309017)).length() < 1e-6);
        assert_eq!(
            47,
            r.dir_to_bin(Vector3D::new(-0.197736, -0.930274, 0.309017))
        );
        assert!((r.bin_dir(48) - Vector3D::new(-0.38683, -0.868833, 0.309017)).length() < 1e-6);
        assert_eq!(
            48,
            r.dir_to_bin(Vector3D::new(-0.38683, -0.868833, 0.309017))
        );
        assert!((r.bin_dir(49) - Vector3D::new(-0.559017, -0.769421, 0.309017)).length() < 1e-6);
        assert_eq!(
            49,
            r.dir_to_bin(Vector3D::new(-0.559017, -0.769421, 0.309017))
        );
        assert!((r.bin_dir(50) - Vector3D::new(-0.706773, -0.636381, 0.309017)).length() < 1e-6);
        assert_eq!(
            50,
            r.dir_to_bin(Vector3D::new(-0.706773, -0.636381, 0.309017))
        );
        assert!((r.bin_dir(51) - Vector3D::new(-0.823639, -0.475528, 0.309017)).length() < 1e-6);
        assert_eq!(
            51,
            r.dir_to_bin(Vector3D::new(-0.823639, -0.475528, 0.309017))
        );
        assert!((r.bin_dir(52) - Vector3D::new(-0.904508, -0.293893, 0.309017)).length() < 1e-6);
        assert_eq!(
            52,
            r.dir_to_bin(Vector3D::new(-0.904508, -0.293893, 0.309017))
        );
        assert!((r.bin_dir(53) - Vector3D::new(-0.945847, -0.0994125, 0.309017)).length() < 1e-6);
        assert_eq!(
            53,
            r.dir_to_bin(Vector3D::new(-0.945847, -0.0994125, 0.309017))
        );
        assert!((r.bin_dir(54) - Vector3D::new(-0.945847, 0.0994125, 0.309017)).length() < 1e-6);
        assert_eq!(
            54,
            r.dir_to_bin(Vector3D::new(-0.945847, 0.0994125, 0.309017))
        );
        assert!((r.bin_dir(55) - Vector3D::new(-0.904508, 0.293893, 0.309017)).length() < 1e-6);
        assert_eq!(
            55,
            r.dir_to_bin(Vector3D::new(-0.904508, 0.293893, 0.309017))
        );
        assert!((r.bin_dir(56) - Vector3D::new(-0.823639, 0.475528, 0.309017)).length() < 1e-6);
        assert_eq!(
            56,
            r.dir_to_bin(Vector3D::new(-0.823639, 0.475528, 0.309017))
        );
        assert!((r.bin_dir(57) - Vector3D::new(-0.706773, 0.636381, 0.309017)).length() < 1e-6);
        assert_eq!(
            57,
            r.dir_to_bin(Vector3D::new(-0.706773, 0.636381, 0.309017))
        );
        assert!((r.bin_dir(58) - Vector3D::new(-0.559017, 0.769421, 0.309017)).length() < 1e-6);
        assert_eq!(
            58,
            r.dir_to_bin(Vector3D::new(-0.559017, 0.769421, 0.309017))
        );
        assert!((r.bin_dir(59) - Vector3D::new(-0.38683, 0.868833, 0.309017)).length() < 1e-6);
        assert_eq!(
            59,
            r.dir_to_bin(Vector3D::new(-0.38683, 0.868833, 0.309017))
        );
        assert!((r.bin_dir(60) - Vector3D::new(-0.197736, 0.930274, 0.309017)).length() < 1e-6);
        assert_eq!(
            60,
            r.dir_to_bin(Vector3D::new(-0.197736, 0.930274, 0.309017))
        );
        assert!((r.bin_dir(61) - Vector3D::new(0., 0.866025, 0.5)).length() < 1e-6);
        assert_eq!(61, r.dir_to_bin(Vector3D::new(0., 0.866025, 0.5)));
        assert!((r.bin_dir(62) - Vector3D::new(0.224144, 0.836516, 0.5)).length() < 1e-6);
        assert_eq!(62, r.dir_to_bin(Vector3D::new(0.224144, 0.836516, 0.5)));
        assert!((r.bin_dir(63) - Vector3D::new(0.433013, 0.75, 0.5)).length() < 1e-6);
        assert_eq!(63, r.dir_to_bin(Vector3D::new(0.433013, 0.75, 0.5)));
        assert!((r.bin_dir(64) - Vector3D::new(0.612372, 0.612372, 0.5)).length() < 1e-6);
        assert_eq!(64, r.dir_to_bin(Vector3D::new(0.612372, 0.612372, 0.5)));
        assert!((r.bin_dir(65) - Vector3D::new(0.75, 0.433013, 0.5)).length() < 1e-6);
        assert_eq!(65, r.dir_to_bin(Vector3D::new(0.75, 0.433013, 0.5)));
        assert!((r.bin_dir(66) - Vector3D::new(0.836516, 0.224144, 0.5)).length() < 1e-6);
        assert_eq!(66, r.dir_to_bin(Vector3D::new(0.836516, 0.224144, 0.5)));
        assert!((r.bin_dir(67) - Vector3D::new(0.866025, 5.30288e-17, 0.5)).length() < 1e-6);
        assert_eq!(67, r.dir_to_bin(Vector3D::new(0.866025, 5.30288e-17, 0.5)));
        assert!((r.bin_dir(68) - Vector3D::new(0.836516, -0.224144, 0.5)).length() < 1e-6);
        assert_eq!(68, r.dir_to_bin(Vector3D::new(0.836516, -0.224144, 0.5)));
        assert!((r.bin_dir(69) - Vector3D::new(0.75, -0.433013, 0.5)).length() < 1e-6);
        assert_eq!(69, r.dir_to_bin(Vector3D::new(0.75, -0.433013, 0.5)));
        assert!((r.bin_dir(70) - Vector3D::new(0.612372, -0.612372, 0.5)).length() < 1e-6);
        assert_eq!(70, r.dir_to_bin(Vector3D::new(0.612372, -0.612372, 0.5)));
        assert!((r.bin_dir(71) - Vector3D::new(0.433013, -0.75, 0.5)).length() < 1e-6);
        assert_eq!(71, r.dir_to_bin(Vector3D::new(0.433013, -0.75, 0.5)));
        assert!((r.bin_dir(72) - Vector3D::new(0.224144, -0.836516, 0.5)).length() < 1e-6);
        assert_eq!(72, r.dir_to_bin(Vector3D::new(0.224144, -0.836516, 0.5)));
        assert!((r.bin_dir(73) - Vector3D::new(1.06058e-16, -0.866025, 0.5)).length() < 1e-6);
        assert_eq!(73, r.dir_to_bin(Vector3D::new(1.06058e-16, -0.866025, 0.5)));
        assert!((r.bin_dir(74) - Vector3D::new(-0.224144, -0.836516, 0.5)).length() < 1e-6);
        assert_eq!(74, r.dir_to_bin(Vector3D::new(-0.224144, -0.836516, 0.5)));
        assert!((r.bin_dir(75) - Vector3D::new(-0.433013, -0.75, 0.5)).length() < 1e-6);
        assert_eq!(75, r.dir_to_bin(Vector3D::new(-0.433013, -0.75, 0.5)));
        assert!((r.bin_dir(76) - Vector3D::new(-0.612372, -0.612372, 0.5)).length() < 1e-6);
        assert_eq!(76, r.dir_to_bin(Vector3D::new(-0.612372, -0.612372, 0.5)));
        assert!((r.bin_dir(77) - Vector3D::new(-0.75, -0.433013, 0.5)).length() < 1e-6);
        assert_eq!(77, r.dir_to_bin(Vector3D::new(-0.75, -0.433013, 0.5)));
        assert!((r.bin_dir(78) - Vector3D::new(-0.836516, -0.224144, 0.5)).length() < 1e-6);
        assert_eq!(78, r.dir_to_bin(Vector3D::new(-0.836516, -0.224144, 0.5)));
        assert!((r.bin_dir(79) - Vector3D::new(-0.866025, -1.59086e-16, 0.5)).length() < 1e-6);
        assert_eq!(
            79,
            r.dir_to_bin(Vector3D::new(-0.866025, -1.59086e-16, 0.5))
        );
        assert!((r.bin_dir(80) - Vector3D::new(-0.836516, 0.224144, 0.5)).length() < 1e-6);
        assert_eq!(80, r.dir_to_bin(Vector3D::new(-0.836516, 0.224144, 0.5)));
        assert!((r.bin_dir(81) - Vector3D::new(-0.75, 0.433013, 0.5)).length() < 1e-6);
        assert_eq!(81, r.dir_to_bin(Vector3D::new(-0.75, 0.433013, 0.5)));
        assert!((r.bin_dir(82) - Vector3D::new(-0.612372, 0.612372, 0.5)).length() < 1e-6);
        assert_eq!(82, r.dir_to_bin(Vector3D::new(-0.612372, 0.612372, 0.5)));
        assert!((r.bin_dir(83) - Vector3D::new(-0.433013, 0.75, 0.5)).length() < 1e-6);
        assert_eq!(83, r.dir_to_bin(Vector3D::new(-0.433013, 0.75, 0.5)));
        assert!((r.bin_dir(84) - Vector3D::new(-0.224144, 0.836516, 0.5)).length() < 1e-6);
        assert_eq!(84, r.dir_to_bin(Vector3D::new(-0.224144, 0.836516, 0.5)));
        assert!((r.bin_dir(85) - Vector3D::new(0., 0.743145, 0.669131)).length() < 1e-6);
        assert_eq!(85, r.dir_to_bin(Vector3D::new(0., 0.743145, 0.669131)));
        assert!((r.bin_dir(86) - Vector3D::new(0.19234, 0.717823, 0.669131)).length() < 1e-6);
        assert_eq!(86, r.dir_to_bin(Vector3D::new(0.19234, 0.717823, 0.669131)));
        assert!((r.bin_dir(87) - Vector3D::new(0.371572, 0.643582, 0.669131)).length() < 1e-6);
        assert_eq!(
            87,
            r.dir_to_bin(Vector3D::new(0.371572, 0.643582, 0.669131))
        );
        assert!((r.bin_dir(88) - Vector3D::new(0.525483, 0.525483, 0.669131)).length() < 1e-6);
        assert_eq!(
            88,
            r.dir_to_bin(Vector3D::new(0.525483, 0.525483, 0.669131))
        );
        assert!((r.bin_dir(89) - Vector3D::new(0.643582, 0.371572, 0.669131)).length() < 1e-6);
        assert_eq!(
            89,
            r.dir_to_bin(Vector3D::new(0.643582, 0.371572, 0.669131))
        );
        assert!((r.bin_dir(90) - Vector3D::new(0.717823, 0.19234, 0.669131)).length() < 1e-6);
        assert_eq!(90, r.dir_to_bin(Vector3D::new(0.717823, 0.19234, 0.669131)));
        assert!((r.bin_dir(91) - Vector3D::new(0.743145, 4.55045e-17, 0.669131)).length() < 1e-6);
        assert_eq!(
            91,
            r.dir_to_bin(Vector3D::new(0.743145, 4.55045e-17, 0.669131))
        );
        assert!((r.bin_dir(92) - Vector3D::new(0.717823, -0.19234, 0.669131)).length() < 1e-6);
        assert_eq!(
            92,
            r.dir_to_bin(Vector3D::new(0.717823, -0.19234, 0.669131))
        );
        assert!((r.bin_dir(93) - Vector3D::new(0.643582, -0.371572, 0.669131)).length() < 1e-6);
        assert_eq!(
            93,
            r.dir_to_bin(Vector3D::new(0.643582, -0.371572, 0.669131))
        );
        assert!((r.bin_dir(94) - Vector3D::new(0.525483, -0.525483, 0.669131)).length() < 1e-6);
        assert_eq!(
            94,
            r.dir_to_bin(Vector3D::new(0.525483, -0.525483, 0.669131))
        );
        assert!((r.bin_dir(95) - Vector3D::new(0.371572, -0.643582, 0.669131)).length() < 1e-6);
        assert_eq!(
            95,
            r.dir_to_bin(Vector3D::new(0.371572, -0.643582, 0.669131))
        );
        assert!((r.bin_dir(96) - Vector3D::new(0.19234, -0.717823, 0.669131)).length() < 1e-6);
        assert_eq!(
            96,
            r.dir_to_bin(Vector3D::new(0.19234, -0.717823, 0.669131))
        );
        assert!((r.bin_dir(97) - Vector3D::new(9.1009e-17, -0.743145, 0.669131)).length() < 1e-6);
        assert_eq!(
            97,
            r.dir_to_bin(Vector3D::new(9.1009e-17, -0.743145, 0.669131))
        );
        assert!((r.bin_dir(98) - Vector3D::new(-0.19234, -0.717823, 0.669131)).length() < 1e-6);
        assert_eq!(
            98,
            r.dir_to_bin(Vector3D::new(-0.19234, -0.717823, 0.669131))
        );
        assert!((r.bin_dir(99) - Vector3D::new(-0.371572, -0.643582, 0.669131)).length() < 1e-6);
        assert_eq!(
            99,
            r.dir_to_bin(Vector3D::new(-0.371572, -0.643582, 0.669131))
        );
        assert!((r.bin_dir(100) - Vector3D::new(-0.525483, -0.525483, 0.669131)).length() < 1e-6);
        assert_eq!(
            100,
            r.dir_to_bin(Vector3D::new(-0.525483, -0.525483, 0.669131))
        );
        assert!((r.bin_dir(101) - Vector3D::new(-0.643582, -0.371572, 0.669131)).length() < 1e-6);
        assert_eq!(
            101,
            r.dir_to_bin(Vector3D::new(-0.643582, -0.371572, 0.669131))
        );
        assert!((r.bin_dir(102) - Vector3D::new(-0.717823, -0.19234, 0.669131)).length() < 1e-6);
        assert_eq!(
            102,
            r.dir_to_bin(Vector3D::new(-0.717823, -0.19234, 0.669131))
        );
        assert!(
            (r.bin_dir(103) - Vector3D::new(-0.743145, -1.36513e-16, 0.669131)).length() < 1e-6
        );
        assert_eq!(
            103,
            r.dir_to_bin(Vector3D::new(-0.743145, -1.36513e-16, 0.669131))
        );
        assert!((r.bin_dir(104) - Vector3D::new(-0.717823, 0.19234, 0.669131)).length() < 1e-6);
        assert_eq!(
            104,
            r.dir_to_bin(Vector3D::new(-0.717823, 0.19234, 0.669131))
        );
        assert!((r.bin_dir(105) - Vector3D::new(-0.643582, 0.371572, 0.669131)).length() < 1e-6);
        assert_eq!(
            105,
            r.dir_to_bin(Vector3D::new(-0.643582, 0.371572, 0.669131))
        );
        assert!((r.bin_dir(106) - Vector3D::new(-0.525483, 0.525483, 0.669131)).length() < 1e-6);
        assert_eq!(
            106,
            r.dir_to_bin(Vector3D::new(-0.525483, 0.525483, 0.669131))
        );
        assert!((r.bin_dir(107) - Vector3D::new(-0.371572, 0.643582, 0.669131)).length() < 1e-6);
        assert_eq!(
            107,
            r.dir_to_bin(Vector3D::new(-0.371572, 0.643582, 0.669131))
        );
        assert!((r.bin_dir(108) - Vector3D::new(-0.19234, 0.717823, 0.669131)).length() < 1e-6);
        assert_eq!(
            108,
            r.dir_to_bin(Vector3D::new(-0.19234, 0.717823, 0.669131))
        );
        assert!((r.bin_dir(109) - Vector3D::new(0., 0.587785, 0.809017)).length() < 1e-6);
        assert_eq!(109, r.dir_to_bin(Vector3D::new(0., 0.587785, 0.809017)));
        assert!((r.bin_dir(110) - Vector3D::new(0.201034, 0.552337, 0.809017)).length() < 1e-6);
        assert_eq!(
            110,
            r.dir_to_bin(Vector3D::new(0.201034, 0.552337, 0.809017))
        );
        assert!((r.bin_dir(111) - Vector3D::new(0.377821, 0.45027, 0.809017)).length() < 1e-6);
        assert_eq!(
            111,
            r.dir_to_bin(Vector3D::new(0.377821, 0.45027, 0.809017))
        );
        assert!((r.bin_dir(112) - Vector3D::new(0.509037, 0.293893, 0.809017)).length() < 1e-6);
        assert_eq!(
            112,
            r.dir_to_bin(Vector3D::new(0.509037, 0.293893, 0.809017))
        );
        assert!((r.bin_dir(113) - Vector3D::new(0.578855, 0.102068, 0.809017)).length() < 1e-6);
        assert_eq!(
            113,
            r.dir_to_bin(Vector3D::new(0.578855, 0.102068, 0.809017))
        );
        assert!((r.bin_dir(114) - Vector3D::new(0.578855, -0.102068, 0.809017)).length() < 1e-6);
        assert_eq!(
            114,
            r.dir_to_bin(Vector3D::new(0.578855, -0.102068, 0.809017))
        );
        assert!((r.bin_dir(115) - Vector3D::new(0.509037, -0.293893, 0.809017)).length() < 1e-6);
        assert_eq!(
            115,
            r.dir_to_bin(Vector3D::new(0.509037, -0.293893, 0.809017))
        );
        assert!((r.bin_dir(116) - Vector3D::new(0.377821, -0.45027, 0.809017)).length() < 1e-6);
        assert_eq!(
            116,
            r.dir_to_bin(Vector3D::new(0.377821, -0.45027, 0.809017))
        );
        assert!((r.bin_dir(117) - Vector3D::new(0.201034, -0.552337, 0.809017)).length() < 1e-6);
        assert_eq!(
            117,
            r.dir_to_bin(Vector3D::new(0.201034, -0.552337, 0.809017))
        );
        assert!((r.bin_dir(118) - Vector3D::new(7.19829e-17, -0.587785, 0.809017)).length() < 1e-6);
        assert_eq!(
            118,
            r.dir_to_bin(Vector3D::new(7.19829e-17, -0.587785, 0.809017))
        );
        assert!((r.bin_dir(119) - Vector3D::new(-0.201034, -0.552337, 0.809017)).length() < 1e-6);
        assert_eq!(
            119,
            r.dir_to_bin(Vector3D::new(-0.201034, -0.552337, 0.809017))
        );
        assert!((r.bin_dir(120) - Vector3D::new(-0.377821, -0.45027, 0.809017)).length() < 1e-6);
        assert_eq!(
            120,
            r.dir_to_bin(Vector3D::new(-0.377821, -0.45027, 0.809017))
        );
        assert!((r.bin_dir(121) - Vector3D::new(-0.509037, -0.293893, 0.809017)).length() < 1e-6);
        assert_eq!(
            121,
            r.dir_to_bin(Vector3D::new(-0.509037, -0.293893, 0.809017))
        );
        assert!((r.bin_dir(122) - Vector3D::new(-0.578855, -0.102068, 0.809017)).length() < 1e-6);
        assert_eq!(
            122,
            r.dir_to_bin(Vector3D::new(-0.578855, -0.102068, 0.809017))
        );
        assert!((r.bin_dir(123) - Vector3D::new(-0.578855, 0.102068, 0.809017)).length() < 1e-6);
        assert_eq!(
            123,
            r.dir_to_bin(Vector3D::new(-0.578855, 0.102068, 0.809017))
        );
        assert!((r.bin_dir(124) - Vector3D::new(-0.509037, 0.293893, 0.809017)).length() < 1e-6);
        assert_eq!(
            124,
            r.dir_to_bin(Vector3D::new(-0.509037, 0.293893, 0.809017))
        );
        assert!((r.bin_dir(125) - Vector3D::new(-0.377821, 0.45027, 0.809017)).length() < 1e-6);
        assert_eq!(
            125,
            r.dir_to_bin(Vector3D::new(-0.377821, 0.45027, 0.809017))
        );
        assert!((r.bin_dir(126) - Vector3D::new(-0.201034, 0.552337, 0.809017)).length() < 1e-6);
        assert_eq!(
            126,
            r.dir_to_bin(Vector3D::new(-0.201034, 0.552337, 0.809017))
        );
        assert!((r.bin_dir(127) - Vector3D::new(0., 0.406737, 0.913545)).length() < 1e-6);
        assert_eq!(127, r.dir_to_bin(Vector3D::new(0., 0.406737, 0.913545)));
        assert!((r.bin_dir(128) - Vector3D::new(0.203368, 0.352244, 0.913545)).length() < 1e-6);
        assert_eq!(
            128,
            r.dir_to_bin(Vector3D::new(0.203368, 0.352244, 0.913545))
        );
        assert!((r.bin_dir(129) - Vector3D::new(0.352244, 0.203368, 0.913545)).length() < 1e-6);
        assert_eq!(
            129,
            r.dir_to_bin(Vector3D::new(0.352244, 0.203368, 0.913545))
        );
        assert!((r.bin_dir(130) - Vector3D::new(0.406737, 2.49054e-17, 0.913545)).length() < 1e-6);
        assert_eq!(
            130,
            r.dir_to_bin(Vector3D::new(0.406737, 2.49054e-17, 0.913545))
        );
        assert!((r.bin_dir(131) - Vector3D::new(0.352244, -0.203368, 0.913545)).length() < 1e-6);
        assert_eq!(
            131,
            r.dir_to_bin(Vector3D::new(0.352244, -0.203368, 0.913545))
        );
        assert!((r.bin_dir(132) - Vector3D::new(0.203368, -0.352244, 0.913545)).length() < 1e-6);
        assert_eq!(
            132,
            r.dir_to_bin(Vector3D::new(0.203368, -0.352244, 0.913545))
        );
        assert!((r.bin_dir(133) - Vector3D::new(4.98109e-17, -0.406737, 0.913545)).length() < 1e-6);
        assert_eq!(
            133,
            r.dir_to_bin(Vector3D::new(4.98109e-17, -0.406737, 0.913545))
        );
        assert!((r.bin_dir(134) - Vector3D::new(-0.203368, -0.352244, 0.913545)).length() < 1e-6);
        assert_eq!(
            134,
            r.dir_to_bin(Vector3D::new(-0.203368, -0.352244, 0.913545))
        );
        assert!((r.bin_dir(135) - Vector3D::new(-0.352244, -0.203368, 0.913545)).length() < 1e-6);
        assert_eq!(
            135,
            r.dir_to_bin(Vector3D::new(-0.352244, -0.203368, 0.913545))
        );
        assert!(
            (r.bin_dir(136) - Vector3D::new(-0.406737, -7.47163e-17, 0.913545)).length() < 1e-6
        );
        assert_eq!(
            136,
            r.dir_to_bin(Vector3D::new(-0.406737, -7.47163e-17, 0.913545))
        );
        assert!((r.bin_dir(137) - Vector3D::new(-0.352244, 0.203368, 0.913545)).length() < 1e-6);
        assert_eq!(
            137,
            r.dir_to_bin(Vector3D::new(-0.352244, 0.203368, 0.913545))
        );
        assert!((r.bin_dir(138) - Vector3D::new(-0.203368, 0.352244, 0.913545)).length() < 1e-6);
        assert_eq!(
            138,
            r.dir_to_bin(Vector3D::new(-0.203368, 0.352244, 0.913545))
        );
        assert!((r.bin_dir(139) - Vector3D::new(0., 0.207912, 0.978148)).length() < 1e-6);
        assert_eq!(139, r.dir_to_bin(Vector3D::new(0., 0.207912, 0.978148)));
        assert!((r.bin_dir(140) - Vector3D::new(0.180057, 0.103956, 0.978148)).length() < 1e-6);
        assert_eq!(
            140,
            r.dir_to_bin(Vector3D::new(0.180057, 0.103956, 0.978148))
        );
        assert!((r.bin_dir(141) - Vector3D::new(0.180057, -0.103956, 0.978148)).length() < 1e-6);
        assert_eq!(
            141,
            r.dir_to_bin(Vector3D::new(0.180057, -0.103956, 0.978148))
        );
        assert!((r.bin_dir(142) - Vector3D::new(2.54618e-17, -0.207912, 0.978148)).length() < 1e-6);
        assert_eq!(
            142,
            r.dir_to_bin(Vector3D::new(2.54618e-17, -0.207912, 0.978148))
        );
        assert!((r.bin_dir(143) - Vector3D::new(-0.180057, -0.103956, 0.978148)).length() < 1e-6);
        assert_eq!(
            143,
            r.dir_to_bin(Vector3D::new(-0.180057, -0.103956, 0.978148))
        );
        assert!((r.bin_dir(144) - Vector3D::new(-0.180057, 0.103956, 0.978148)).length() < 1e-6);
        assert_eq!(
            144,
            r.dir_to_bin(Vector3D::new(-0.180057, 0.103956, 0.978148))
        );
        assert!((r.bin_dir(145) - Vector3D::new(-0., -1.60812e-16, 1.)).length() < 1e-6);
        assert_eq!(145, r.dir_to_bin(Vector3D::new(-0., -1.60812e-16, 1.)));

        // Automatically produced using command:
        // cnt 577 | rcalc -e MF:2 -e Rbin=recno -f $RAYPATH/reinsrc.cal -o 'assert!( (r.bin_dir(${recno}) - Vector3D::new(${Dx}, ${Dy}, ${Dz})).length() < 1e-6);assert_eq!(${recno}, r.dir_to_bin(Vector3D::new(${Dx}, ${Dy}, ${Dz})) );'
        let r = ReinhartSky::new(2);

        assert!((r.bin_dir(1) - Vector3D::new(0., 0.998533, 0.0541389)).length() < 1e-6);
        assert_eq!(1, r.dir_to_bin(Vector3D::new(0., 0.998533, 0.0541389)));
        assert!((r.bin_dir(2) - Vector3D::new(0.104375, 0.993063, 0.0541389)).length() < 1e-6);
        assert_eq!(
            2,
            r.dir_to_bin(Vector3D::new(0.104375, 0.993063, 0.0541389))
        );
        assert!((r.bin_dir(3) - Vector3D::new(0.207607, 0.976713, 0.0541389)).length() < 1e-6);
        assert_eq!(
            3,
            r.dir_to_bin(Vector3D::new(0.207607, 0.976713, 0.0541389))
        );
        assert!((r.bin_dir(4) - Vector3D::new(0.308564, 0.949662, 0.0541389)).length() < 1e-6);
        assert_eq!(
            4,
            r.dir_to_bin(Vector3D::new(0.308564, 0.949662, 0.0541389))
        );
        assert!((r.bin_dir(5) - Vector3D::new(0.40614, 0.912206, 0.0541389)).length() < 1e-6);
        assert_eq!(5, r.dir_to_bin(Vector3D::new(0.40614, 0.912206, 0.0541389)));
        assert!((r.bin_dir(6) - Vector3D::new(0.499267, 0.864755, 0.0541389)).length() < 1e-6);
        assert_eq!(
            6,
            r.dir_to_bin(Vector3D::new(0.499267, 0.864755, 0.0541389))
        );
        assert!((r.bin_dir(7) - Vector3D::new(0.586923, 0.807831, 0.0541389)).length() < 1e-6);
        assert_eq!(
            7,
            r.dir_to_bin(Vector3D::new(0.586923, 0.807831, 0.0541389))
        );
        assert!((r.bin_dir(8) - Vector3D::new(0.668149, 0.742055, 0.0541389)).length() < 1e-6);
        assert_eq!(
            8,
            r.dir_to_bin(Vector3D::new(0.668149, 0.742055, 0.0541389))
        );
        assert!((r.bin_dir(9) - Vector3D::new(0.742055, 0.668149, 0.0541389)).length() < 1e-6);
        assert_eq!(
            9,
            r.dir_to_bin(Vector3D::new(0.742055, 0.668149, 0.0541389))
        );
        assert!((r.bin_dir(10) - Vector3D::new(0.807831, 0.586923, 0.0541389)).length() < 1e-6);
        assert_eq!(
            10,
            r.dir_to_bin(Vector3D::new(0.807831, 0.586923, 0.0541389))
        );
        assert!((r.bin_dir(11) - Vector3D::new(0.864755, 0.499267, 0.0541389)).length() < 1e-6);
        assert_eq!(
            11,
            r.dir_to_bin(Vector3D::new(0.864755, 0.499267, 0.0541389))
        );
        assert!((r.bin_dir(12) - Vector3D::new(0.912206, 0.40614, 0.0541389)).length() < 1e-6);
        assert_eq!(
            12,
            r.dir_to_bin(Vector3D::new(0.912206, 0.40614, 0.0541389))
        );
        assert!((r.bin_dir(13) - Vector3D::new(0.949662, 0.308564, 0.0541389)).length() < 1e-6);
        assert_eq!(
            13,
            r.dir_to_bin(Vector3D::new(0.949662, 0.308564, 0.0541389))
        );
        assert!((r.bin_dir(14) - Vector3D::new(0.976713, 0.207607, 0.0541389)).length() < 1e-6);
        assert_eq!(
            14,
            r.dir_to_bin(Vector3D::new(0.976713, 0.207607, 0.0541389))
        );
        assert!((r.bin_dir(15) - Vector3D::new(0.993063, 0.104375, 0.0541389)).length() < 1e-6);
        assert_eq!(
            15,
            r.dir_to_bin(Vector3D::new(0.993063, 0.104375, 0.0541389))
        );
        assert!((r.bin_dir(16) - Vector3D::new(0.998533, 6.11425e-17, 0.0541389)).length() < 1e-6);
        assert_eq!(
            16,
            r.dir_to_bin(Vector3D::new(0.998533, 6.11425e-17, 0.0541389))
        );
        assert!((r.bin_dir(17) - Vector3D::new(0.993063, -0.104375, 0.0541389)).length() < 1e-6);
        assert_eq!(
            17,
            r.dir_to_bin(Vector3D::new(0.993063, -0.104375, 0.0541389))
        );
        assert!((r.bin_dir(18) - Vector3D::new(0.976713, -0.207607, 0.0541389)).length() < 1e-6);
        assert_eq!(
            18,
            r.dir_to_bin(Vector3D::new(0.976713, -0.207607, 0.0541389))
        );
        assert!((r.bin_dir(19) - Vector3D::new(0.949662, -0.308564, 0.0541389)).length() < 1e-6);
        assert_eq!(
            19,
            r.dir_to_bin(Vector3D::new(0.949662, -0.308564, 0.0541389))
        );
        assert!((r.bin_dir(20) - Vector3D::new(0.912206, -0.40614, 0.0541389)).length() < 1e-6);
        assert_eq!(
            20,
            r.dir_to_bin(Vector3D::new(0.912206, -0.40614, 0.0541389))
        );
        assert!((r.bin_dir(21) - Vector3D::new(0.864755, -0.499267, 0.0541389)).length() < 1e-6);
        assert_eq!(
            21,
            r.dir_to_bin(Vector3D::new(0.864755, -0.499267, 0.0541389))
        );
        assert!((r.bin_dir(22) - Vector3D::new(0.807831, -0.586923, 0.0541389)).length() < 1e-6);
        assert_eq!(
            22,
            r.dir_to_bin(Vector3D::new(0.807831, -0.586923, 0.0541389))
        );
        assert!((r.bin_dir(23) - Vector3D::new(0.742055, -0.668149, 0.0541389)).length() < 1e-6);
        assert_eq!(
            23,
            r.dir_to_bin(Vector3D::new(0.742055, -0.668149, 0.0541389))
        );
        assert!((r.bin_dir(24) - Vector3D::new(0.668149, -0.742055, 0.0541389)).length() < 1e-6);
        assert_eq!(
            24,
            r.dir_to_bin(Vector3D::new(0.668149, -0.742055, 0.0541389))
        );
        assert!((r.bin_dir(25) - Vector3D::new(0.586923, -0.807831, 0.0541389)).length() < 1e-6);
        assert_eq!(
            25,
            r.dir_to_bin(Vector3D::new(0.586923, -0.807831, 0.0541389))
        );
        assert!((r.bin_dir(26) - Vector3D::new(0.499267, -0.864755, 0.0541389)).length() < 1e-6);
        assert_eq!(
            26,
            r.dir_to_bin(Vector3D::new(0.499267, -0.864755, 0.0541389))
        );
        assert!((r.bin_dir(27) - Vector3D::new(0.40614, -0.912206, 0.0541389)).length() < 1e-6);
        assert_eq!(
            27,
            r.dir_to_bin(Vector3D::new(0.40614, -0.912206, 0.0541389))
        );
        assert!((r.bin_dir(28) - Vector3D::new(0.308564, -0.949662, 0.0541389)).length() < 1e-6);
        assert_eq!(
            28,
            r.dir_to_bin(Vector3D::new(0.308564, -0.949662, 0.0541389))
        );
        assert!((r.bin_dir(29) - Vector3D::new(0.207607, -0.976713, 0.0541389)).length() < 1e-6);
        assert_eq!(
            29,
            r.dir_to_bin(Vector3D::new(0.207607, -0.976713, 0.0541389))
        );
        assert!((r.bin_dir(30) - Vector3D::new(0.104375, -0.993063, 0.0541389)).length() < 1e-6);
        assert_eq!(
            30,
            r.dir_to_bin(Vector3D::new(0.104375, -0.993063, 0.0541389))
        );
        assert!((r.bin_dir(31) - Vector3D::new(1.22285e-16, -0.998533, 0.0541389)).length() < 1e-6);
        assert_eq!(
            31,
            r.dir_to_bin(Vector3D::new(1.22285e-16, -0.998533, 0.0541389))
        );
        assert!((r.bin_dir(32) - Vector3D::new(-0.104375, -0.993063, 0.0541389)).length() < 1e-6);
        assert_eq!(
            32,
            r.dir_to_bin(Vector3D::new(-0.104375, -0.993063, 0.0541389))
        );
        assert!((r.bin_dir(33) - Vector3D::new(-0.207607, -0.976713, 0.0541389)).length() < 1e-6);
        assert_eq!(
            33,
            r.dir_to_bin(Vector3D::new(-0.207607, -0.976713, 0.0541389))
        );
        assert!((r.bin_dir(34) - Vector3D::new(-0.308564, -0.949662, 0.0541389)).length() < 1e-6);
        assert_eq!(
            34,
            r.dir_to_bin(Vector3D::new(-0.308564, -0.949662, 0.0541389))
        );
        assert!((r.bin_dir(35) - Vector3D::new(-0.40614, -0.912206, 0.0541389)).length() < 1e-6);
        assert_eq!(
            35,
            r.dir_to_bin(Vector3D::new(-0.40614, -0.912206, 0.0541389))
        );
        assert!((r.bin_dir(36) - Vector3D::new(-0.499267, -0.864755, 0.0541389)).length() < 1e-6);
        assert_eq!(
            36,
            r.dir_to_bin(Vector3D::new(-0.499267, -0.864755, 0.0541389))
        );
        assert!((r.bin_dir(37) - Vector3D::new(-0.586923, -0.807831, 0.0541389)).length() < 1e-6);
        assert_eq!(
            37,
            r.dir_to_bin(Vector3D::new(-0.586923, -0.807831, 0.0541389))
        );
        assert!((r.bin_dir(38) - Vector3D::new(-0.668149, -0.742055, 0.0541389)).length() < 1e-6);
        assert_eq!(
            38,
            r.dir_to_bin(Vector3D::new(-0.668149, -0.742055, 0.0541389))
        );
        assert!((r.bin_dir(39) - Vector3D::new(-0.742055, -0.668149, 0.0541389)).length() < 1e-6);
        assert_eq!(
            39,
            r.dir_to_bin(Vector3D::new(-0.742055, -0.668149, 0.0541389))
        );
        assert!((r.bin_dir(40) - Vector3D::new(-0.807831, -0.586923, 0.0541389)).length() < 1e-6);
        assert_eq!(
            40,
            r.dir_to_bin(Vector3D::new(-0.807831, -0.586923, 0.0541389))
        );
        assert!((r.bin_dir(41) - Vector3D::new(-0.864755, -0.499267, 0.0541389)).length() < 1e-6);
        assert_eq!(
            41,
            r.dir_to_bin(Vector3D::new(-0.864755, -0.499267, 0.0541389))
        );
        assert!((r.bin_dir(42) - Vector3D::new(-0.912206, -0.40614, 0.0541389)).length() < 1e-6);
        assert_eq!(
            42,
            r.dir_to_bin(Vector3D::new(-0.912206, -0.40614, 0.0541389))
        );
        assert!((r.bin_dir(43) - Vector3D::new(-0.949662, -0.308564, 0.0541389)).length() < 1e-6);
        assert_eq!(
            43,
            r.dir_to_bin(Vector3D::new(-0.949662, -0.308564, 0.0541389))
        );
        assert!((r.bin_dir(44) - Vector3D::new(-0.976713, -0.207607, 0.0541389)).length() < 1e-6);
        assert_eq!(
            44,
            r.dir_to_bin(Vector3D::new(-0.976713, -0.207607, 0.0541389))
        );
        assert!((r.bin_dir(45) - Vector3D::new(-0.993063, -0.104375, 0.0541389)).length() < 1e-6);
        assert_eq!(
            45,
            r.dir_to_bin(Vector3D::new(-0.993063, -0.104375, 0.0541389))
        );
        assert!(
            (r.bin_dir(46) - Vector3D::new(-0.998533, -1.83428e-16, 0.0541389)).length() < 1e-6
        );
        assert_eq!(
            46,
            r.dir_to_bin(Vector3D::new(-0.998533, -1.83428e-16, 0.0541389))
        );
        assert!((r.bin_dir(47) - Vector3D::new(-0.993063, 0.104375, 0.0541389)).length() < 1e-6);
        assert_eq!(
            47,
            r.dir_to_bin(Vector3D::new(-0.993063, 0.104375, 0.0541389))
        );
        assert!((r.bin_dir(48) - Vector3D::new(-0.976713, 0.207607, 0.0541389)).length() < 1e-6);
        assert_eq!(
            48,
            r.dir_to_bin(Vector3D::new(-0.976713, 0.207607, 0.0541389))
        );
        assert!((r.bin_dir(49) - Vector3D::new(-0.949662, 0.308564, 0.0541389)).length() < 1e-6);
        assert_eq!(
            49,
            r.dir_to_bin(Vector3D::new(-0.949662, 0.308564, 0.0541389))
        );
        assert!((r.bin_dir(50) - Vector3D::new(-0.912206, 0.40614, 0.0541389)).length() < 1e-6);
        assert_eq!(
            50,
            r.dir_to_bin(Vector3D::new(-0.912206, 0.40614, 0.0541389))
        );
        assert!((r.bin_dir(51) - Vector3D::new(-0.864755, 0.499267, 0.0541389)).length() < 1e-6);
        assert_eq!(
            51,
            r.dir_to_bin(Vector3D::new(-0.864755, 0.499267, 0.0541389))
        );
        assert!((r.bin_dir(52) - Vector3D::new(-0.807831, 0.586923, 0.0541389)).length() < 1e-6);
        assert_eq!(
            52,
            r.dir_to_bin(Vector3D::new(-0.807831, 0.586923, 0.0541389))
        );
        assert!((r.bin_dir(53) - Vector3D::new(-0.742055, 0.668149, 0.0541389)).length() < 1e-6);
        assert_eq!(
            53,
            r.dir_to_bin(Vector3D::new(-0.742055, 0.668149, 0.0541389))
        );
        assert!((r.bin_dir(54) - Vector3D::new(-0.668149, 0.742055, 0.0541389)).length() < 1e-6);
        assert_eq!(
            54,
            r.dir_to_bin(Vector3D::new(-0.668149, 0.742055, 0.0541389))
        );
        assert!((r.bin_dir(55) - Vector3D::new(-0.586923, 0.807831, 0.0541389)).length() < 1e-6);
        assert_eq!(
            55,
            r.dir_to_bin(Vector3D::new(-0.586923, 0.807831, 0.0541389))
        );
        assert!((r.bin_dir(56) - Vector3D::new(-0.499267, 0.864755, 0.0541389)).length() < 1e-6);
        assert_eq!(
            56,
            r.dir_to_bin(Vector3D::new(-0.499267, 0.864755, 0.0541389))
        );
        assert!((r.bin_dir(57) - Vector3D::new(-0.40614, 0.912206, 0.0541389)).length() < 1e-6);
        assert_eq!(
            57,
            r.dir_to_bin(Vector3D::new(-0.40614, 0.912206, 0.0541389))
        );
        assert!((r.bin_dir(58) - Vector3D::new(-0.308564, 0.949662, 0.0541389)).length() < 1e-6);
        assert_eq!(
            58,
            r.dir_to_bin(Vector3D::new(-0.308564, 0.949662, 0.0541389))
        );
        assert!((r.bin_dir(59) - Vector3D::new(-0.207607, 0.976713, 0.0541389)).length() < 1e-6);
        assert_eq!(
            59,
            r.dir_to_bin(Vector3D::new(-0.207607, 0.976713, 0.0541389))
        );
        assert!((r.bin_dir(60) - Vector3D::new(-0.104375, 0.993063, 0.0541389)).length() < 1e-6);
        assert_eq!(
            60,
            r.dir_to_bin(Vector3D::new(-0.104375, 0.993063, 0.0541389))
        );
        assert!((r.bin_dir(61) - Vector3D::new(0., 0.986827, 0.161782)).length() < 1e-6);
        assert_eq!(61, r.dir_to_bin(Vector3D::new(0., 0.986827, 0.161782)));
        assert!((r.bin_dir(62) - Vector3D::new(0.103151, 0.981421, 0.161782)).length() < 1e-6);
        assert_eq!(
            62,
            r.dir_to_bin(Vector3D::new(0.103151, 0.981421, 0.161782))
        );
        assert!((r.bin_dir(63) - Vector3D::new(0.205173, 0.965262, 0.161782)).length() < 1e-6);
        assert_eq!(
            63,
            r.dir_to_bin(Vector3D::new(0.205173, 0.965262, 0.161782))
        );
        assert!((r.bin_dir(64) - Vector3D::new(0.304946, 0.938528, 0.161782)).length() < 1e-6);
        assert_eq!(
            64,
            r.dir_to_bin(Vector3D::new(0.304946, 0.938528, 0.161782))
        );
        assert!((r.bin_dir(65) - Vector3D::new(0.401379, 0.901511, 0.161782)).length() < 1e-6);
        assert_eq!(
            65,
            r.dir_to_bin(Vector3D::new(0.401379, 0.901511, 0.161782))
        );
        assert!((r.bin_dir(66) - Vector3D::new(0.493413, 0.854617, 0.161782)).length() < 1e-6);
        assert_eq!(
            66,
            r.dir_to_bin(Vector3D::new(0.493413, 0.854617, 0.161782))
        );
        assert!((r.bin_dir(67) - Vector3D::new(0.580042, 0.798359, 0.161782)).length() < 1e-6);
        assert_eq!(
            67,
            r.dir_to_bin(Vector3D::new(0.580042, 0.798359, 0.161782))
        );
        assert!((r.bin_dir(68) - Vector3D::new(0.660316, 0.733355, 0.161782)).length() < 1e-6);
        assert_eq!(
            68,
            r.dir_to_bin(Vector3D::new(0.660316, 0.733355, 0.161782))
        );
        assert!((r.bin_dir(69) - Vector3D::new(0.733355, 0.660316, 0.161782)).length() < 1e-6);
        assert_eq!(
            69,
            r.dir_to_bin(Vector3D::new(0.733355, 0.660316, 0.161782))
        );
        assert!((r.bin_dir(70) - Vector3D::new(0.798359, 0.580042, 0.161782)).length() < 1e-6);
        assert_eq!(
            70,
            r.dir_to_bin(Vector3D::new(0.798359, 0.580042, 0.161782))
        );
        assert!((r.bin_dir(71) - Vector3D::new(0.854617, 0.493413, 0.161782)).length() < 1e-6);
        assert_eq!(
            71,
            r.dir_to_bin(Vector3D::new(0.854617, 0.493413, 0.161782))
        );
        assert!((r.bin_dir(72) - Vector3D::new(0.901511, 0.401379, 0.161782)).length() < 1e-6);
        assert_eq!(
            72,
            r.dir_to_bin(Vector3D::new(0.901511, 0.401379, 0.161782))
        );
        assert!((r.bin_dir(73) - Vector3D::new(0.938528, 0.304946, 0.161782)).length() < 1e-6);
        assert_eq!(
            73,
            r.dir_to_bin(Vector3D::new(0.938528, 0.304946, 0.161782))
        );
        assert!((r.bin_dir(74) - Vector3D::new(0.965262, 0.205173, 0.161782)).length() < 1e-6);
        assert_eq!(
            74,
            r.dir_to_bin(Vector3D::new(0.965262, 0.205173, 0.161782))
        );
        assert!((r.bin_dir(75) - Vector3D::new(0.981421, 0.103151, 0.161782)).length() < 1e-6);
        assert_eq!(
            75,
            r.dir_to_bin(Vector3D::new(0.981421, 0.103151, 0.161782))
        );
        assert!((r.bin_dir(76) - Vector3D::new(0.986827, 6.04257e-17, 0.161782)).length() < 1e-6);
        assert_eq!(
            76,
            r.dir_to_bin(Vector3D::new(0.986827, 6.04257e-17, 0.161782))
        );
        assert!((r.bin_dir(77) - Vector3D::new(0.981421, -0.103151, 0.161782)).length() < 1e-6);
        assert_eq!(
            77,
            r.dir_to_bin(Vector3D::new(0.981421, -0.103151, 0.161782))
        );
        assert!((r.bin_dir(78) - Vector3D::new(0.965262, -0.205173, 0.161782)).length() < 1e-6);
        assert_eq!(
            78,
            r.dir_to_bin(Vector3D::new(0.965262, -0.205173, 0.161782))
        );
        assert!((r.bin_dir(79) - Vector3D::new(0.938528, -0.304946, 0.161782)).length() < 1e-6);
        assert_eq!(
            79,
            r.dir_to_bin(Vector3D::new(0.938528, -0.304946, 0.161782))
        );
        assert!((r.bin_dir(80) - Vector3D::new(0.901511, -0.401379, 0.161782)).length() < 1e-6);
        assert_eq!(
            80,
            r.dir_to_bin(Vector3D::new(0.901511, -0.401379, 0.161782))
        );
        assert!((r.bin_dir(81) - Vector3D::new(0.854617, -0.493413, 0.161782)).length() < 1e-6);
        assert_eq!(
            81,
            r.dir_to_bin(Vector3D::new(0.854617, -0.493413, 0.161782))
        );
        assert!((r.bin_dir(82) - Vector3D::new(0.798359, -0.580042, 0.161782)).length() < 1e-6);
        assert_eq!(
            82,
            r.dir_to_bin(Vector3D::new(0.798359, -0.580042, 0.161782))
        );
        assert!((r.bin_dir(83) - Vector3D::new(0.733355, -0.660316, 0.161782)).length() < 1e-6);
        assert_eq!(
            83,
            r.dir_to_bin(Vector3D::new(0.733355, -0.660316, 0.161782))
        );
        assert!((r.bin_dir(84) - Vector3D::new(0.660316, -0.733355, 0.161782)).length() < 1e-6);
        assert_eq!(
            84,
            r.dir_to_bin(Vector3D::new(0.660316, -0.733355, 0.161782))
        );
        assert!((r.bin_dir(85) - Vector3D::new(0.580042, -0.798359, 0.161782)).length() < 1e-6);
        assert_eq!(
            85,
            r.dir_to_bin(Vector3D::new(0.580042, -0.798359, 0.161782))
        );
        assert!((r.bin_dir(86) - Vector3D::new(0.493413, -0.854617, 0.161782)).length() < 1e-6);
        assert_eq!(
            86,
            r.dir_to_bin(Vector3D::new(0.493413, -0.854617, 0.161782))
        );
        assert!((r.bin_dir(87) - Vector3D::new(0.401379, -0.901511, 0.161782)).length() < 1e-6);
        assert_eq!(
            87,
            r.dir_to_bin(Vector3D::new(0.401379, -0.901511, 0.161782))
        );
        assert!((r.bin_dir(88) - Vector3D::new(0.304946, -0.938528, 0.161782)).length() < 1e-6);
        assert_eq!(
            88,
            r.dir_to_bin(Vector3D::new(0.304946, -0.938528, 0.161782))
        );
        assert!((r.bin_dir(89) - Vector3D::new(0.205173, -0.965262, 0.161782)).length() < 1e-6);
        assert_eq!(
            89,
            r.dir_to_bin(Vector3D::new(0.205173, -0.965262, 0.161782))
        );
        assert!((r.bin_dir(90) - Vector3D::new(0.103151, -0.981421, 0.161782)).length() < 1e-6);
        assert_eq!(
            90,
            r.dir_to_bin(Vector3D::new(0.103151, -0.981421, 0.161782))
        );
        assert!((r.bin_dir(91) - Vector3D::new(1.20851e-16, -0.986827, 0.161782)).length() < 1e-6);
        assert_eq!(
            91,
            r.dir_to_bin(Vector3D::new(1.20851e-16, -0.986827, 0.161782))
        );
        assert!((r.bin_dir(92) - Vector3D::new(-0.103151, -0.981421, 0.161782)).length() < 1e-6);
        assert_eq!(
            92,
            r.dir_to_bin(Vector3D::new(-0.103151, -0.981421, 0.161782))
        );
        assert!((r.bin_dir(93) - Vector3D::new(-0.205173, -0.965262, 0.161782)).length() < 1e-6);
        assert_eq!(
            93,
            r.dir_to_bin(Vector3D::new(-0.205173, -0.965262, 0.161782))
        );
        assert!((r.bin_dir(94) - Vector3D::new(-0.304946, -0.938528, 0.161782)).length() < 1e-6);
        assert_eq!(
            94,
            r.dir_to_bin(Vector3D::new(-0.304946, -0.938528, 0.161782))
        );
        assert!((r.bin_dir(95) - Vector3D::new(-0.401379, -0.901511, 0.161782)).length() < 1e-6);
        assert_eq!(
            95,
            r.dir_to_bin(Vector3D::new(-0.401379, -0.901511, 0.161782))
        );
        assert!((r.bin_dir(96) - Vector3D::new(-0.493413, -0.854617, 0.161782)).length() < 1e-6);
        assert_eq!(
            96,
            r.dir_to_bin(Vector3D::new(-0.493413, -0.854617, 0.161782))
        );
        assert!((r.bin_dir(97) - Vector3D::new(-0.580042, -0.798359, 0.161782)).length() < 1e-6);
        assert_eq!(
            97,
            r.dir_to_bin(Vector3D::new(-0.580042, -0.798359, 0.161782))
        );
        assert!((r.bin_dir(98) - Vector3D::new(-0.660316, -0.733355, 0.161782)).length() < 1e-6);
        assert_eq!(
            98,
            r.dir_to_bin(Vector3D::new(-0.660316, -0.733355, 0.161782))
        );
        assert!((r.bin_dir(99) - Vector3D::new(-0.733355, -0.660316, 0.161782)).length() < 1e-6);
        assert_eq!(
            99,
            r.dir_to_bin(Vector3D::new(-0.733355, -0.660316, 0.161782))
        );
        assert!((r.bin_dir(100) - Vector3D::new(-0.798359, -0.580042, 0.161782)).length() < 1e-6);
        assert_eq!(
            100,
            r.dir_to_bin(Vector3D::new(-0.798359, -0.580042, 0.161782))
        );
        assert!((r.bin_dir(101) - Vector3D::new(-0.854617, -0.493413, 0.161782)).length() < 1e-6);
        assert_eq!(
            101,
            r.dir_to_bin(Vector3D::new(-0.854617, -0.493413, 0.161782))
        );
        assert!((r.bin_dir(102) - Vector3D::new(-0.901511, -0.401379, 0.161782)).length() < 1e-6);
        assert_eq!(
            102,
            r.dir_to_bin(Vector3D::new(-0.901511, -0.401379, 0.161782))
        );
        assert!((r.bin_dir(103) - Vector3D::new(-0.938528, -0.304946, 0.161782)).length() < 1e-6);
        assert_eq!(
            103,
            r.dir_to_bin(Vector3D::new(-0.938528, -0.304946, 0.161782))
        );
        assert!((r.bin_dir(104) - Vector3D::new(-0.965262, -0.205173, 0.161782)).length() < 1e-6);
        assert_eq!(
            104,
            r.dir_to_bin(Vector3D::new(-0.965262, -0.205173, 0.161782))
        );
        assert!((r.bin_dir(105) - Vector3D::new(-0.981421, -0.103151, 0.161782)).length() < 1e-6);
        assert_eq!(
            105,
            r.dir_to_bin(Vector3D::new(-0.981421, -0.103151, 0.161782))
        );
        assert!(
            (r.bin_dir(106) - Vector3D::new(-0.986827, -1.81277e-16, 0.161782)).length() < 1e-6
        );
        assert_eq!(
            106,
            r.dir_to_bin(Vector3D::new(-0.986827, -1.81277e-16, 0.161782))
        );
        assert!((r.bin_dir(107) - Vector3D::new(-0.981421, 0.103151, 0.161782)).length() < 1e-6);
        assert_eq!(
            107,
            r.dir_to_bin(Vector3D::new(-0.981421, 0.103151, 0.161782))
        );
        assert!((r.bin_dir(108) - Vector3D::new(-0.965262, 0.205173, 0.161782)).length() < 1e-6);
        assert_eq!(
            108,
            r.dir_to_bin(Vector3D::new(-0.965262, 0.205173, 0.161782))
        );
        assert!((r.bin_dir(109) - Vector3D::new(-0.938528, 0.304946, 0.161782)).length() < 1e-6);
        assert_eq!(
            109,
            r.dir_to_bin(Vector3D::new(-0.938528, 0.304946, 0.161782))
        );
        assert!((r.bin_dir(110) - Vector3D::new(-0.901511, 0.401379, 0.161782)).length() < 1e-6);
        assert_eq!(
            110,
            r.dir_to_bin(Vector3D::new(-0.901511, 0.401379, 0.161782))
        );
        assert!((r.bin_dir(111) - Vector3D::new(-0.854617, 0.493413, 0.161782)).length() < 1e-6);
        assert_eq!(
            111,
            r.dir_to_bin(Vector3D::new(-0.854617, 0.493413, 0.161782))
        );
        assert!((r.bin_dir(112) - Vector3D::new(-0.798359, 0.580042, 0.161782)).length() < 1e-6);
        assert_eq!(
            112,
            r.dir_to_bin(Vector3D::new(-0.798359, 0.580042, 0.161782))
        );
        assert!((r.bin_dir(113) - Vector3D::new(-0.733355, 0.660316, 0.161782)).length() < 1e-6);
        assert_eq!(
            113,
            r.dir_to_bin(Vector3D::new(-0.733355, 0.660316, 0.161782))
        );
        assert!((r.bin_dir(114) - Vector3D::new(-0.660316, 0.733355, 0.161782)).length() < 1e-6);
        assert_eq!(
            114,
            r.dir_to_bin(Vector3D::new(-0.660316, 0.733355, 0.161782))
        );
        assert!((r.bin_dir(115) - Vector3D::new(-0.580042, 0.798359, 0.161782)).length() < 1e-6);
        assert_eq!(
            115,
            r.dir_to_bin(Vector3D::new(-0.580042, 0.798359, 0.161782))
        );
        assert!((r.bin_dir(116) - Vector3D::new(-0.493413, 0.854617, 0.161782)).length() < 1e-6);
        assert_eq!(
            116,
            r.dir_to_bin(Vector3D::new(-0.493413, 0.854617, 0.161782))
        );
        assert!((r.bin_dir(117) - Vector3D::new(-0.401379, 0.901511, 0.161782)).length() < 1e-6);
        assert_eq!(
            117,
            r.dir_to_bin(Vector3D::new(-0.401379, 0.901511, 0.161782))
        );
        assert!((r.bin_dir(118) - Vector3D::new(-0.304946, 0.938528, 0.161782)).length() < 1e-6);
        assert_eq!(
            118,
            r.dir_to_bin(Vector3D::new(-0.304946, 0.938528, 0.161782))
        );
        assert!((r.bin_dir(119) - Vector3D::new(-0.205173, 0.965262, 0.161782)).length() < 1e-6);
        assert_eq!(
            119,
            r.dir_to_bin(Vector3D::new(-0.205173, 0.965262, 0.161782))
        );
        assert!((r.bin_dir(120) - Vector3D::new(-0.103151, 0.981421, 0.161782)).length() < 1e-6);
        assert_eq!(
            120,
            r.dir_to_bin(Vector3D::new(-0.103151, 0.981421, 0.161782))
        );
        assert!((r.bin_dir(121) - Vector3D::new(0., 0.96355, 0.267528)).length() < 1e-6);
        assert_eq!(121, r.dir_to_bin(Vector3D::new(0., 0.96355, 0.267528)));
        assert!((r.bin_dir(122) - Vector3D::new(0.100718, 0.958272, 0.267528)).length() < 1e-6);
        assert_eq!(
            122,
            r.dir_to_bin(Vector3D::new(0.100718, 0.958272, 0.267528))
        );
        assert!((r.bin_dir(123) - Vector3D::new(0.200333, 0.942494, 0.267528)).length() < 1e-6);
        assert_eq!(
            123,
            r.dir_to_bin(Vector3D::new(0.200333, 0.942494, 0.267528))
        );
        assert!((r.bin_dir(124) - Vector3D::new(0.297753, 0.91639, 0.267528)).length() < 1e-6);
        assert_eq!(
            124,
            r.dir_to_bin(Vector3D::new(0.297753, 0.91639, 0.267528))
        );
        assert!((r.bin_dir(125) - Vector3D::new(0.391911, 0.880247, 0.267528)).length() < 1e-6);
        assert_eq!(
            125,
            r.dir_to_bin(Vector3D::new(0.391911, 0.880247, 0.267528))
        );
        assert!((r.bin_dir(126) - Vector3D::new(0.481775, 0.834459, 0.267528)).length() < 1e-6);
        assert_eq!(
            126,
            r.dir_to_bin(Vector3D::new(0.481775, 0.834459, 0.267528))
        );
        assert!((r.bin_dir(127) - Vector3D::new(0.56636, 0.779528, 0.267528)).length() < 1e-6);
        assert_eq!(
            127,
            r.dir_to_bin(Vector3D::new(0.56636, 0.779528, 0.267528))
        );
        assert!((r.bin_dir(128) - Vector3D::new(0.644741, 0.716057, 0.267528)).length() < 1e-6);
        assert_eq!(
            128,
            r.dir_to_bin(Vector3D::new(0.644741, 0.716057, 0.267528))
        );
        assert!((r.bin_dir(129) - Vector3D::new(0.716057, 0.644741, 0.267528)).length() < 1e-6);
        assert_eq!(
            129,
            r.dir_to_bin(Vector3D::new(0.716057, 0.644741, 0.267528))
        );
        assert!((r.bin_dir(130) - Vector3D::new(0.779528, 0.56636, 0.267528)).length() < 1e-6);
        assert_eq!(
            130,
            r.dir_to_bin(Vector3D::new(0.779528, 0.56636, 0.267528))
        );
        assert!((r.bin_dir(131) - Vector3D::new(0.834459, 0.481775, 0.267528)).length() < 1e-6);
        assert_eq!(
            131,
            r.dir_to_bin(Vector3D::new(0.834459, 0.481775, 0.267528))
        );
        assert!((r.bin_dir(132) - Vector3D::new(0.880247, 0.391911, 0.267528)).length() < 1e-6);
        assert_eq!(
            132,
            r.dir_to_bin(Vector3D::new(0.880247, 0.391911, 0.267528))
        );
        assert!((r.bin_dir(133) - Vector3D::new(0.91639, 0.297753, 0.267528)).length() < 1e-6);
        assert_eq!(
            133,
            r.dir_to_bin(Vector3D::new(0.91639, 0.297753, 0.267528))
        );
        assert!((r.bin_dir(134) - Vector3D::new(0.942494, 0.200333, 0.267528)).length() < 1e-6);
        assert_eq!(
            134,
            r.dir_to_bin(Vector3D::new(0.942494, 0.200333, 0.267528))
        );
        assert!((r.bin_dir(135) - Vector3D::new(0.958272, 0.100718, 0.267528)).length() < 1e-6);
        assert_eq!(
            135,
            r.dir_to_bin(Vector3D::new(0.958272, 0.100718, 0.267528))
        );
        assert!((r.bin_dir(136) - Vector3D::new(0.96355, 5.90004e-17, 0.267528)).length() < 1e-6);
        assert_eq!(
            136,
            r.dir_to_bin(Vector3D::new(0.96355, 5.90004e-17, 0.267528))
        );
        assert!((r.bin_dir(137) - Vector3D::new(0.958272, -0.100718, 0.267528)).length() < 1e-6);
        assert_eq!(
            137,
            r.dir_to_bin(Vector3D::new(0.958272, -0.100718, 0.267528))
        );
        assert!((r.bin_dir(138) - Vector3D::new(0.942494, -0.200333, 0.267528)).length() < 1e-6);
        assert_eq!(
            138,
            r.dir_to_bin(Vector3D::new(0.942494, -0.200333, 0.267528))
        );
        assert!((r.bin_dir(139) - Vector3D::new(0.91639, -0.297753, 0.267528)).length() < 1e-6);
        assert_eq!(
            139,
            r.dir_to_bin(Vector3D::new(0.91639, -0.297753, 0.267528))
        );
        assert!((r.bin_dir(140) - Vector3D::new(0.880247, -0.391911, 0.267528)).length() < 1e-6);
        assert_eq!(
            140,
            r.dir_to_bin(Vector3D::new(0.880247, -0.391911, 0.267528))
        );
        assert!((r.bin_dir(141) - Vector3D::new(0.834459, -0.481775, 0.267528)).length() < 1e-6);
        assert_eq!(
            141,
            r.dir_to_bin(Vector3D::new(0.834459, -0.481775, 0.267528))
        );
        assert!((r.bin_dir(142) - Vector3D::new(0.779528, -0.56636, 0.267528)).length() < 1e-6);
        assert_eq!(
            142,
            r.dir_to_bin(Vector3D::new(0.779528, -0.56636, 0.267528))
        );
        assert!((r.bin_dir(143) - Vector3D::new(0.716057, -0.644741, 0.267528)).length() < 1e-6);
        assert_eq!(
            143,
            r.dir_to_bin(Vector3D::new(0.716057, -0.644741, 0.267528))
        );
        assert!((r.bin_dir(144) - Vector3D::new(0.644741, -0.716057, 0.267528)).length() < 1e-6);
        assert_eq!(
            144,
            r.dir_to_bin(Vector3D::new(0.644741, -0.716057, 0.267528))
        );
        assert!((r.bin_dir(145) - Vector3D::new(0.56636, -0.779528, 0.267528)).length() < 1e-6);
        assert_eq!(
            145,
            r.dir_to_bin(Vector3D::new(0.56636, -0.779528, 0.267528))
        );
        assert!((r.bin_dir(146) - Vector3D::new(0.481775, -0.834459, 0.267528)).length() < 1e-6);
        assert_eq!(
            146,
            r.dir_to_bin(Vector3D::new(0.481775, -0.834459, 0.267528))
        );
        assert!((r.bin_dir(147) - Vector3D::new(0.391911, -0.880247, 0.267528)).length() < 1e-6);
        assert_eq!(
            147,
            r.dir_to_bin(Vector3D::new(0.391911, -0.880247, 0.267528))
        );
        assert!((r.bin_dir(148) - Vector3D::new(0.297753, -0.91639, 0.267528)).length() < 1e-6);
        assert_eq!(
            148,
            r.dir_to_bin(Vector3D::new(0.297753, -0.91639, 0.267528))
        );
        assert!((r.bin_dir(149) - Vector3D::new(0.200333, -0.942494, 0.267528)).length() < 1e-6);
        assert_eq!(
            149,
            r.dir_to_bin(Vector3D::new(0.200333, -0.942494, 0.267528))
        );
        assert!((r.bin_dir(150) - Vector3D::new(0.100718, -0.958272, 0.267528)).length() < 1e-6);
        assert_eq!(
            150,
            r.dir_to_bin(Vector3D::new(0.100718, -0.958272, 0.267528))
        );
        assert!((r.bin_dir(151) - Vector3D::new(1.18001e-16, -0.96355, 0.267528)).length() < 1e-6);
        assert_eq!(
            151,
            r.dir_to_bin(Vector3D::new(1.18001e-16, -0.96355, 0.267528))
        );
        assert!((r.bin_dir(152) - Vector3D::new(-0.100718, -0.958272, 0.267528)).length() < 1e-6);
        assert_eq!(
            152,
            r.dir_to_bin(Vector3D::new(-0.100718, -0.958272, 0.267528))
        );
        assert!((r.bin_dir(153) - Vector3D::new(-0.200333, -0.942494, 0.267528)).length() < 1e-6);
        assert_eq!(
            153,
            r.dir_to_bin(Vector3D::new(-0.200333, -0.942494, 0.267528))
        );
        assert!((r.bin_dir(154) - Vector3D::new(-0.297753, -0.91639, 0.267528)).length() < 1e-6);
        assert_eq!(
            154,
            r.dir_to_bin(Vector3D::new(-0.297753, -0.91639, 0.267528))
        );
        assert!((r.bin_dir(155) - Vector3D::new(-0.391911, -0.880247, 0.267528)).length() < 1e-6);
        assert_eq!(
            155,
            r.dir_to_bin(Vector3D::new(-0.391911, -0.880247, 0.267528))
        );
        assert!((r.bin_dir(156) - Vector3D::new(-0.481775, -0.834459, 0.267528)).length() < 1e-6);
        assert_eq!(
            156,
            r.dir_to_bin(Vector3D::new(-0.481775, -0.834459, 0.267528))
        );
        assert!((r.bin_dir(157) - Vector3D::new(-0.56636, -0.779528, 0.267528)).length() < 1e-6);
        assert_eq!(
            157,
            r.dir_to_bin(Vector3D::new(-0.56636, -0.779528, 0.267528))
        );
        assert!((r.bin_dir(158) - Vector3D::new(-0.644741, -0.716057, 0.267528)).length() < 1e-6);
        assert_eq!(
            158,
            r.dir_to_bin(Vector3D::new(-0.644741, -0.716057, 0.267528))
        );
        assert!((r.bin_dir(159) - Vector3D::new(-0.716057, -0.644741, 0.267528)).length() < 1e-6);
        assert_eq!(
            159,
            r.dir_to_bin(Vector3D::new(-0.716057, -0.644741, 0.267528))
        );
        assert!((r.bin_dir(160) - Vector3D::new(-0.779528, -0.56636, 0.267528)).length() < 1e-6);
        assert_eq!(
            160,
            r.dir_to_bin(Vector3D::new(-0.779528, -0.56636, 0.267528))
        );
        assert!((r.bin_dir(161) - Vector3D::new(-0.834459, -0.481775, 0.267528)).length() < 1e-6);
        assert_eq!(
            161,
            r.dir_to_bin(Vector3D::new(-0.834459, -0.481775, 0.267528))
        );
        assert!((r.bin_dir(162) - Vector3D::new(-0.880247, -0.391911, 0.267528)).length() < 1e-6);
        assert_eq!(
            162,
            r.dir_to_bin(Vector3D::new(-0.880247, -0.391911, 0.267528))
        );
        assert!((r.bin_dir(163) - Vector3D::new(-0.91639, -0.297753, 0.267528)).length() < 1e-6);
        assert_eq!(
            163,
            r.dir_to_bin(Vector3D::new(-0.91639, -0.297753, 0.267528))
        );
        assert!((r.bin_dir(164) - Vector3D::new(-0.942494, -0.200333, 0.267528)).length() < 1e-6);
        assert_eq!(
            164,
            r.dir_to_bin(Vector3D::new(-0.942494, -0.200333, 0.267528))
        );
        assert!((r.bin_dir(165) - Vector3D::new(-0.958272, -0.100718, 0.267528)).length() < 1e-6);
        assert_eq!(
            165,
            r.dir_to_bin(Vector3D::new(-0.958272, -0.100718, 0.267528))
        );
        assert!((r.bin_dir(166) - Vector3D::new(-0.96355, -1.77001e-16, 0.267528)).length() < 1e-6);
        assert_eq!(
            166,
            r.dir_to_bin(Vector3D::new(-0.96355, -1.77001e-16, 0.267528))
        );
        assert!((r.bin_dir(167) - Vector3D::new(-0.958272, 0.100718, 0.267528)).length() < 1e-6);
        assert_eq!(
            167,
            r.dir_to_bin(Vector3D::new(-0.958272, 0.100718, 0.267528))
        );
        assert!((r.bin_dir(168) - Vector3D::new(-0.942494, 0.200333, 0.267528)).length() < 1e-6);
        assert_eq!(
            168,
            r.dir_to_bin(Vector3D::new(-0.942494, 0.200333, 0.267528))
        );
        assert!((r.bin_dir(169) - Vector3D::new(-0.91639, 0.297753, 0.267528)).length() < 1e-6);
        assert_eq!(
            169,
            r.dir_to_bin(Vector3D::new(-0.91639, 0.297753, 0.267528))
        );
        assert!((r.bin_dir(170) - Vector3D::new(-0.880247, 0.391911, 0.267528)).length() < 1e-6);
        assert_eq!(
            170,
            r.dir_to_bin(Vector3D::new(-0.880247, 0.391911, 0.267528))
        );
        assert!((r.bin_dir(171) - Vector3D::new(-0.834459, 0.481775, 0.267528)).length() < 1e-6);
        assert_eq!(
            171,
            r.dir_to_bin(Vector3D::new(-0.834459, 0.481775, 0.267528))
        );
        assert!((r.bin_dir(172) - Vector3D::new(-0.779528, 0.56636, 0.267528)).length() < 1e-6);
        assert_eq!(
            172,
            r.dir_to_bin(Vector3D::new(-0.779528, 0.56636, 0.267528))
        );
        assert!((r.bin_dir(173) - Vector3D::new(-0.716057, 0.644741, 0.267528)).length() < 1e-6);
        assert_eq!(
            173,
            r.dir_to_bin(Vector3D::new(-0.716057, 0.644741, 0.267528))
        );
        assert!((r.bin_dir(174) - Vector3D::new(-0.644741, 0.716057, 0.267528)).length() < 1e-6);
        assert_eq!(
            174,
            r.dir_to_bin(Vector3D::new(-0.644741, 0.716057, 0.267528))
        );
        assert!((r.bin_dir(175) - Vector3D::new(-0.56636, 0.779528, 0.267528)).length() < 1e-6);
        assert_eq!(
            175,
            r.dir_to_bin(Vector3D::new(-0.56636, 0.779528, 0.267528))
        );
        assert!((r.bin_dir(176) - Vector3D::new(-0.481775, 0.834459, 0.267528)).length() < 1e-6);
        assert_eq!(
            176,
            r.dir_to_bin(Vector3D::new(-0.481775, 0.834459, 0.267528))
        );
        assert!((r.bin_dir(177) - Vector3D::new(-0.391911, 0.880247, 0.267528)).length() < 1e-6);
        assert_eq!(
            177,
            r.dir_to_bin(Vector3D::new(-0.391911, 0.880247, 0.267528))
        );
        assert!((r.bin_dir(178) - Vector3D::new(-0.297753, 0.91639, 0.267528)).length() < 1e-6);
        assert_eq!(
            178,
            r.dir_to_bin(Vector3D::new(-0.297753, 0.91639, 0.267528))
        );
        assert!((r.bin_dir(179) - Vector3D::new(-0.200333, 0.942494, 0.267528)).length() < 1e-6);
        assert_eq!(
            179,
            r.dir_to_bin(Vector3D::new(-0.200333, 0.942494, 0.267528))
        );
        assert!((r.bin_dir(180) - Vector3D::new(-0.100718, 0.958272, 0.267528)).length() < 1e-6);
        assert_eq!(
            180,
            r.dir_to_bin(Vector3D::new(-0.100718, 0.958272, 0.267528))
        );
        assert!((r.bin_dir(181) - Vector3D::new(0., 0.928977, 0.370138)).length() < 1e-6);
        assert_eq!(181, r.dir_to_bin(Vector3D::new(0., 0.928977, 0.370138)));
        assert!((r.bin_dir(182) - Vector3D::new(0.0971045, 0.923888, 0.370138)).length() < 1e-6);
        assert_eq!(
            182,
            r.dir_to_bin(Vector3D::new(0.0971045, 0.923888, 0.370138))
        );
        assert!((r.bin_dir(183) - Vector3D::new(0.193145, 0.908676, 0.370138)).length() < 1e-6);
        assert_eq!(
            183,
            r.dir_to_bin(Vector3D::new(0.193145, 0.908676, 0.370138))
        );
        assert!((r.bin_dir(184) - Vector3D::new(0.28707, 0.883509, 0.370138)).length() < 1e-6);
        assert_eq!(
            184,
            r.dir_to_bin(Vector3D::new(0.28707, 0.883509, 0.370138))
        );
        assert!((r.bin_dir(185) - Vector3D::new(0.377849, 0.848662, 0.370138)).length() < 1e-6);
        assert_eq!(
            185,
            r.dir_to_bin(Vector3D::new(0.377849, 0.848662, 0.370138))
        );
        assert!((r.bin_dir(186) - Vector3D::new(0.464488, 0.804517, 0.370138)).length() < 1e-6);
        assert_eq!(
            186,
            r.dir_to_bin(Vector3D::new(0.464488, 0.804517, 0.370138))
        );
        assert!((r.bin_dir(187) - Vector3D::new(0.546039, 0.751558, 0.370138)).length() < 1e-6);
        assert_eq!(
            187,
            r.dir_to_bin(Vector3D::new(0.546039, 0.751558, 0.370138))
        );
        assert!((r.bin_dir(188) - Vector3D::new(0.621607, 0.690364, 0.370138)).length() < 1e-6);
        assert_eq!(
            188,
            r.dir_to_bin(Vector3D::new(0.621607, 0.690364, 0.370138))
        );
        assert!((r.bin_dir(189) - Vector3D::new(0.690364, 0.621607, 0.370138)).length() < 1e-6);
        assert_eq!(
            189,
            r.dir_to_bin(Vector3D::new(0.690364, 0.621607, 0.370138))
        );
        assert!((r.bin_dir(190) - Vector3D::new(0.751558, 0.546039, 0.370138)).length() < 1e-6);
        assert_eq!(
            190,
            r.dir_to_bin(Vector3D::new(0.751558, 0.546039, 0.370138))
        );
        assert!((r.bin_dir(191) - Vector3D::new(0.804517, 0.464488, 0.370138)).length() < 1e-6);
        assert_eq!(
            191,
            r.dir_to_bin(Vector3D::new(0.804517, 0.464488, 0.370138))
        );
        assert!((r.bin_dir(192) - Vector3D::new(0.848662, 0.377849, 0.370138)).length() < 1e-6);
        assert_eq!(
            192,
            r.dir_to_bin(Vector3D::new(0.848662, 0.377849, 0.370138))
        );
        assert!((r.bin_dir(193) - Vector3D::new(0.883509, 0.28707, 0.370138)).length() < 1e-6);
        assert_eq!(
            193,
            r.dir_to_bin(Vector3D::new(0.883509, 0.28707, 0.370138))
        );
        assert!((r.bin_dir(194) - Vector3D::new(0.908676, 0.193145, 0.370138)).length() < 1e-6);
        assert_eq!(
            194,
            r.dir_to_bin(Vector3D::new(0.908676, 0.193145, 0.370138))
        );
        assert!((r.bin_dir(195) - Vector3D::new(0.923888, 0.0971045, 0.370138)).length() < 1e-6);
        assert_eq!(
            195,
            r.dir_to_bin(Vector3D::new(0.923888, 0.0971045, 0.370138))
        );
        assert!((r.bin_dir(196) - Vector3D::new(0.928977, 5.68834e-17, 0.370138)).length() < 1e-6);
        assert_eq!(
            196,
            r.dir_to_bin(Vector3D::new(0.928977, 5.68834e-17, 0.370138))
        );
        assert!((r.bin_dir(197) - Vector3D::new(0.923888, -0.0971045, 0.370138)).length() < 1e-6);
        assert_eq!(
            197,
            r.dir_to_bin(Vector3D::new(0.923888, -0.0971045, 0.370138))
        );
        assert!((r.bin_dir(198) - Vector3D::new(0.908676, -0.193145, 0.370138)).length() < 1e-6);
        assert_eq!(
            198,
            r.dir_to_bin(Vector3D::new(0.908676, -0.193145, 0.370138))
        );
        assert!((r.bin_dir(199) - Vector3D::new(0.883509, -0.28707, 0.370138)).length() < 1e-6);
        assert_eq!(
            199,
            r.dir_to_bin(Vector3D::new(0.883509, -0.28707, 0.370138))
        );
        assert!((r.bin_dir(200) - Vector3D::new(0.848662, -0.377849, 0.370138)).length() < 1e-6);
        assert_eq!(
            200,
            r.dir_to_bin(Vector3D::new(0.848662, -0.377849, 0.370138))
        );
        assert!((r.bin_dir(201) - Vector3D::new(0.804517, -0.464488, 0.370138)).length() < 1e-6);
        assert_eq!(
            201,
            r.dir_to_bin(Vector3D::new(0.804517, -0.464488, 0.370138))
        );
        assert!((r.bin_dir(202) - Vector3D::new(0.751558, -0.546039, 0.370138)).length() < 1e-6);
        assert_eq!(
            202,
            r.dir_to_bin(Vector3D::new(0.751558, -0.546039, 0.370138))
        );
        assert!((r.bin_dir(203) - Vector3D::new(0.690364, -0.621607, 0.370138)).length() < 1e-6);
        assert_eq!(
            203,
            r.dir_to_bin(Vector3D::new(0.690364, -0.621607, 0.370138))
        );
        assert!((r.bin_dir(204) - Vector3D::new(0.621607, -0.690364, 0.370138)).length() < 1e-6);
        assert_eq!(
            204,
            r.dir_to_bin(Vector3D::new(0.621607, -0.690364, 0.370138))
        );
        assert!((r.bin_dir(205) - Vector3D::new(0.546039, -0.751558, 0.370138)).length() < 1e-6);
        assert_eq!(
            205,
            r.dir_to_bin(Vector3D::new(0.546039, -0.751558, 0.370138))
        );
        assert!((r.bin_dir(206) - Vector3D::new(0.464488, -0.804517, 0.370138)).length() < 1e-6);
        assert_eq!(
            206,
            r.dir_to_bin(Vector3D::new(0.464488, -0.804517, 0.370138))
        );
        assert!((r.bin_dir(207) - Vector3D::new(0.377849, -0.848662, 0.370138)).length() < 1e-6);
        assert_eq!(
            207,
            r.dir_to_bin(Vector3D::new(0.377849, -0.848662, 0.370138))
        );
        assert!((r.bin_dir(208) - Vector3D::new(0.28707, -0.883509, 0.370138)).length() < 1e-6);
        assert_eq!(
            208,
            r.dir_to_bin(Vector3D::new(0.28707, -0.883509, 0.370138))
        );
        assert!((r.bin_dir(209) - Vector3D::new(0.193145, -0.908676, 0.370138)).length() < 1e-6);
        assert_eq!(
            209,
            r.dir_to_bin(Vector3D::new(0.193145, -0.908676, 0.370138))
        );
        assert!((r.bin_dir(210) - Vector3D::new(0.0971045, -0.923888, 0.370138)).length() < 1e-6);
        assert_eq!(
            210,
            r.dir_to_bin(Vector3D::new(0.0971045, -0.923888, 0.370138))
        );
        assert!((r.bin_dir(211) - Vector3D::new(1.13767e-16, -0.928977, 0.370138)).length() < 1e-6);
        assert_eq!(
            211,
            r.dir_to_bin(Vector3D::new(1.13767e-16, -0.928977, 0.370138))
        );
        assert!((r.bin_dir(212) - Vector3D::new(-0.0971045, -0.923888, 0.370138)).length() < 1e-6);
        assert_eq!(
            212,
            r.dir_to_bin(Vector3D::new(-0.0971045, -0.923888, 0.370138))
        );
        assert!((r.bin_dir(213) - Vector3D::new(-0.193145, -0.908676, 0.370138)).length() < 1e-6);
        assert_eq!(
            213,
            r.dir_to_bin(Vector3D::new(-0.193145, -0.908676, 0.370138))
        );
        assert!((r.bin_dir(214) - Vector3D::new(-0.28707, -0.883509, 0.370138)).length() < 1e-6);
        assert_eq!(
            214,
            r.dir_to_bin(Vector3D::new(-0.28707, -0.883509, 0.370138))
        );
        assert!((r.bin_dir(215) - Vector3D::new(-0.377849, -0.848662, 0.370138)).length() < 1e-6);
        assert_eq!(
            215,
            r.dir_to_bin(Vector3D::new(-0.377849, -0.848662, 0.370138))
        );
        assert!((r.bin_dir(216) - Vector3D::new(-0.464488, -0.804517, 0.370138)).length() < 1e-6);
        assert_eq!(
            216,
            r.dir_to_bin(Vector3D::new(-0.464488, -0.804517, 0.370138))
        );
        assert!((r.bin_dir(217) - Vector3D::new(-0.546039, -0.751558, 0.370138)).length() < 1e-6);
        assert_eq!(
            217,
            r.dir_to_bin(Vector3D::new(-0.546039, -0.751558, 0.370138))
        );
        assert!((r.bin_dir(218) - Vector3D::new(-0.621607, -0.690364, 0.370138)).length() < 1e-6);
        assert_eq!(
            218,
            r.dir_to_bin(Vector3D::new(-0.621607, -0.690364, 0.370138))
        );
        assert!((r.bin_dir(219) - Vector3D::new(-0.690364, -0.621607, 0.370138)).length() < 1e-6);
        assert_eq!(
            219,
            r.dir_to_bin(Vector3D::new(-0.690364, -0.621607, 0.370138))
        );
        assert!((r.bin_dir(220) - Vector3D::new(-0.751558, -0.546039, 0.370138)).length() < 1e-6);
        assert_eq!(
            220,
            r.dir_to_bin(Vector3D::new(-0.751558, -0.546039, 0.370138))
        );
        assert!((r.bin_dir(221) - Vector3D::new(-0.804517, -0.464488, 0.370138)).length() < 1e-6);
        assert_eq!(
            221,
            r.dir_to_bin(Vector3D::new(-0.804517, -0.464488, 0.370138))
        );
        assert!((r.bin_dir(222) - Vector3D::new(-0.848662, -0.377849, 0.370138)).length() < 1e-6);
        assert_eq!(
            222,
            r.dir_to_bin(Vector3D::new(-0.848662, -0.377849, 0.370138))
        );
        assert!((r.bin_dir(223) - Vector3D::new(-0.883509, -0.28707, 0.370138)).length() < 1e-6);
        assert_eq!(
            223,
            r.dir_to_bin(Vector3D::new(-0.883509, -0.28707, 0.370138))
        );
        assert!((r.bin_dir(224) - Vector3D::new(-0.908676, -0.193145, 0.370138)).length() < 1e-6);
        assert_eq!(
            224,
            r.dir_to_bin(Vector3D::new(-0.908676, -0.193145, 0.370138))
        );
        assert!((r.bin_dir(225) - Vector3D::new(-0.923888, -0.0971045, 0.370138)).length() < 1e-6);
        assert_eq!(
            225,
            r.dir_to_bin(Vector3D::new(-0.923888, -0.0971045, 0.370138))
        );
        assert!((r.bin_dir(226) - Vector3D::new(-0.928977, -1.7065e-16, 0.370138)).length() < 1e-6);
        assert_eq!(
            226,
            r.dir_to_bin(Vector3D::new(-0.928977, -1.7065e-16, 0.370138))
        );
        assert!((r.bin_dir(227) - Vector3D::new(-0.923888, 0.0971045, 0.370138)).length() < 1e-6);
        assert_eq!(
            227,
            r.dir_to_bin(Vector3D::new(-0.923888, 0.0971045, 0.370138))
        );
        assert!((r.bin_dir(228) - Vector3D::new(-0.908676, 0.193145, 0.370138)).length() < 1e-6);
        assert_eq!(
            228,
            r.dir_to_bin(Vector3D::new(-0.908676, 0.193145, 0.370138))
        );
        assert!((r.bin_dir(229) - Vector3D::new(-0.883509, 0.28707, 0.370138)).length() < 1e-6);
        assert_eq!(
            229,
            r.dir_to_bin(Vector3D::new(-0.883509, 0.28707, 0.370138))
        );
        assert!((r.bin_dir(230) - Vector3D::new(-0.848662, 0.377849, 0.370138)).length() < 1e-6);
        assert_eq!(
            230,
            r.dir_to_bin(Vector3D::new(-0.848662, 0.377849, 0.370138))
        );
        assert!((r.bin_dir(231) - Vector3D::new(-0.804517, 0.464488, 0.370138)).length() < 1e-6);
        assert_eq!(
            231,
            r.dir_to_bin(Vector3D::new(-0.804517, 0.464488, 0.370138))
        );
        assert!((r.bin_dir(232) - Vector3D::new(-0.751558, 0.546039, 0.370138)).length() < 1e-6);
        assert_eq!(
            232,
            r.dir_to_bin(Vector3D::new(-0.751558, 0.546039, 0.370138))
        );
        assert!((r.bin_dir(233) - Vector3D::new(-0.690364, 0.621607, 0.370138)).length() < 1e-6);
        assert_eq!(
            233,
            r.dir_to_bin(Vector3D::new(-0.690364, 0.621607, 0.370138))
        );
        assert!((r.bin_dir(234) - Vector3D::new(-0.621607, 0.690364, 0.370138)).length() < 1e-6);
        assert_eq!(
            234,
            r.dir_to_bin(Vector3D::new(-0.621607, 0.690364, 0.370138))
        );
        assert!((r.bin_dir(235) - Vector3D::new(-0.546039, 0.751558, 0.370138)).length() < 1e-6);
        assert_eq!(
            235,
            r.dir_to_bin(Vector3D::new(-0.546039, 0.751558, 0.370138))
        );
        assert!((r.bin_dir(236) - Vector3D::new(-0.464488, 0.804517, 0.370138)).length() < 1e-6);
        assert_eq!(
            236,
            r.dir_to_bin(Vector3D::new(-0.464488, 0.804517, 0.370138))
        );
        assert!((r.bin_dir(237) - Vector3D::new(-0.377849, 0.848662, 0.370138)).length() < 1e-6);
        assert_eq!(
            237,
            r.dir_to_bin(Vector3D::new(-0.377849, 0.848662, 0.370138))
        );
        assert!((r.bin_dir(238) - Vector3D::new(-0.28707, 0.883509, 0.370138)).length() < 1e-6);
        assert_eq!(
            238,
            r.dir_to_bin(Vector3D::new(-0.28707, 0.883509, 0.370138))
        );
        assert!((r.bin_dir(239) - Vector3D::new(-0.193145, 0.908676, 0.370138)).length() < 1e-6);
        assert_eq!(
            239,
            r.dir_to_bin(Vector3D::new(-0.193145, 0.908676, 0.370138))
        );
        assert!((r.bin_dir(240) - Vector3D::new(-0.0971045, 0.923888, 0.370138)).length() < 1e-6);
        assert_eq!(
            240,
            r.dir_to_bin(Vector3D::new(-0.0971045, 0.923888, 0.370138))
        );
        assert!((r.bin_dir(241) - Vector3D::new(0., 0.883512, 0.468408)).length() < 1e-6);
        assert_eq!(241, r.dir_to_bin(Vector3D::new(0., 0.883512, 0.468408)));
        assert!((r.bin_dir(242) - Vector3D::new(0.115321, 0.875953, 0.468408)).length() < 1e-6);
        assert_eq!(
            242,
            r.dir_to_bin(Vector3D::new(0.115321, 0.875953, 0.468408))
        );
        assert!((r.bin_dir(243) - Vector3D::new(0.22867, 0.853407, 0.468408)).length() < 1e-6);
        assert_eq!(
            243,
            r.dir_to_bin(Vector3D::new(0.22867, 0.853407, 0.468408))
        );
        assert!((r.bin_dir(244) - Vector3D::new(0.338105, 0.816259, 0.468408)).length() < 1e-6);
        assert_eq!(
            244,
            r.dir_to_bin(Vector3D::new(0.338105, 0.816259, 0.468408))
        );
        assert!((r.bin_dir(245) - Vector3D::new(0.441756, 0.765144, 0.468408)).length() < 1e-6);
        assert_eq!(
            245,
            r.dir_to_bin(Vector3D::new(0.441756, 0.765144, 0.468408))
        );
        assert!((r.bin_dir(246) - Vector3D::new(0.537848, 0.700937, 0.468408)).length() < 1e-6);
        assert_eq!(
            246,
            r.dir_to_bin(Vector3D::new(0.537848, 0.700937, 0.468408))
        );
        assert!((r.bin_dir(247) - Vector3D::new(0.624737, 0.624737, 0.468408)).length() < 1e-6);
        assert_eq!(
            247,
            r.dir_to_bin(Vector3D::new(0.624737, 0.624737, 0.468408))
        );
        assert!((r.bin_dir(248) - Vector3D::new(0.700937, 0.537848, 0.468408)).length() < 1e-6);
        assert_eq!(
            248,
            r.dir_to_bin(Vector3D::new(0.700937, 0.537848, 0.468408))
        );
        assert!((r.bin_dir(249) - Vector3D::new(0.765144, 0.441756, 0.468408)).length() < 1e-6);
        assert_eq!(
            249,
            r.dir_to_bin(Vector3D::new(0.765144, 0.441756, 0.468408))
        );
        assert!((r.bin_dir(250) - Vector3D::new(0.816259, 0.338105, 0.468408)).length() < 1e-6);
        assert_eq!(
            250,
            r.dir_to_bin(Vector3D::new(0.816259, 0.338105, 0.468408))
        );
        assert!((r.bin_dir(251) - Vector3D::new(0.853407, 0.22867, 0.468408)).length() < 1e-6);
        assert_eq!(
            251,
            r.dir_to_bin(Vector3D::new(0.853407, 0.22867, 0.468408))
        );
        assert!((r.bin_dir(252) - Vector3D::new(0.875953, 0.115321, 0.468408)).length() < 1e-6);
        assert_eq!(
            252,
            r.dir_to_bin(Vector3D::new(0.875953, 0.115321, 0.468408))
        );
        assert!((r.bin_dir(253) - Vector3D::new(0.883512, 5.40995e-17, 0.468408)).length() < 1e-6);
        assert_eq!(
            253,
            r.dir_to_bin(Vector3D::new(0.883512, 5.40995e-17, 0.468408))
        );
        assert!((r.bin_dir(254) - Vector3D::new(0.875953, -0.115321, 0.468408)).length() < 1e-6);
        assert_eq!(
            254,
            r.dir_to_bin(Vector3D::new(0.875953, -0.115321, 0.468408))
        );
        assert!((r.bin_dir(255) - Vector3D::new(0.853407, -0.22867, 0.468408)).length() < 1e-6);
        assert_eq!(
            255,
            r.dir_to_bin(Vector3D::new(0.853407, -0.22867, 0.468408))
        );
        assert!((r.bin_dir(256) - Vector3D::new(0.816259, -0.338105, 0.468408)).length() < 1e-6);
        assert_eq!(
            256,
            r.dir_to_bin(Vector3D::new(0.816259, -0.338105, 0.468408))
        );
        assert!((r.bin_dir(257) - Vector3D::new(0.765144, -0.441756, 0.468408)).length() < 1e-6);
        assert_eq!(
            257,
            r.dir_to_bin(Vector3D::new(0.765144, -0.441756, 0.468408))
        );
        assert!((r.bin_dir(258) - Vector3D::new(0.700937, -0.537848, 0.468408)).length() < 1e-6);
        assert_eq!(
            258,
            r.dir_to_bin(Vector3D::new(0.700937, -0.537848, 0.468408))
        );
        assert!((r.bin_dir(259) - Vector3D::new(0.624737, -0.624737, 0.468408)).length() < 1e-6);
        assert_eq!(
            259,
            r.dir_to_bin(Vector3D::new(0.624737, -0.624737, 0.468408))
        );
        assert!((r.bin_dir(260) - Vector3D::new(0.537848, -0.700937, 0.468408)).length() < 1e-6);
        assert_eq!(
            260,
            r.dir_to_bin(Vector3D::new(0.537848, -0.700937, 0.468408))
        );
        assert!((r.bin_dir(261) - Vector3D::new(0.441756, -0.765144, 0.468408)).length() < 1e-6);
        assert_eq!(
            261,
            r.dir_to_bin(Vector3D::new(0.441756, -0.765144, 0.468408))
        );
        assert!((r.bin_dir(262) - Vector3D::new(0.338105, -0.816259, 0.468408)).length() < 1e-6);
        assert_eq!(
            262,
            r.dir_to_bin(Vector3D::new(0.338105, -0.816259, 0.468408))
        );
        assert!((r.bin_dir(263) - Vector3D::new(0.22867, -0.853407, 0.468408)).length() < 1e-6);
        assert_eq!(
            263,
            r.dir_to_bin(Vector3D::new(0.22867, -0.853407, 0.468408))
        );
        assert!((r.bin_dir(264) - Vector3D::new(0.115321, -0.875953, 0.468408)).length() < 1e-6);
        assert_eq!(
            264,
            r.dir_to_bin(Vector3D::new(0.115321, -0.875953, 0.468408))
        );
        assert!((r.bin_dir(265) - Vector3D::new(1.08199e-16, -0.883512, 0.468408)).length() < 1e-6);
        assert_eq!(
            265,
            r.dir_to_bin(Vector3D::new(1.08199e-16, -0.883512, 0.468408))
        );
        assert!((r.bin_dir(266) - Vector3D::new(-0.115321, -0.875953, 0.468408)).length() < 1e-6);
        assert_eq!(
            266,
            r.dir_to_bin(Vector3D::new(-0.115321, -0.875953, 0.468408))
        );
        assert!((r.bin_dir(267) - Vector3D::new(-0.22867, -0.853407, 0.468408)).length() < 1e-6);
        assert_eq!(
            267,
            r.dir_to_bin(Vector3D::new(-0.22867, -0.853407, 0.468408))
        );
        assert!((r.bin_dir(268) - Vector3D::new(-0.338105, -0.816259, 0.468408)).length() < 1e-6);
        assert_eq!(
            268,
            r.dir_to_bin(Vector3D::new(-0.338105, -0.816259, 0.468408))
        );
        assert!((r.bin_dir(269) - Vector3D::new(-0.441756, -0.765144, 0.468408)).length() < 1e-6);
        assert_eq!(
            269,
            r.dir_to_bin(Vector3D::new(-0.441756, -0.765144, 0.468408))
        );
        assert!((r.bin_dir(270) - Vector3D::new(-0.537848, -0.700937, 0.468408)).length() < 1e-6);
        assert_eq!(
            270,
            r.dir_to_bin(Vector3D::new(-0.537848, -0.700937, 0.468408))
        );
        assert!((r.bin_dir(271) - Vector3D::new(-0.624737, -0.624737, 0.468408)).length() < 1e-6);
        assert_eq!(
            271,
            r.dir_to_bin(Vector3D::new(-0.624737, -0.624737, 0.468408))
        );
        assert!((r.bin_dir(272) - Vector3D::new(-0.700937, -0.537848, 0.468408)).length() < 1e-6);
        assert_eq!(
            272,
            r.dir_to_bin(Vector3D::new(-0.700937, -0.537848, 0.468408))
        );
        assert!((r.bin_dir(273) - Vector3D::new(-0.765144, -0.441756, 0.468408)).length() < 1e-6);
        assert_eq!(
            273,
            r.dir_to_bin(Vector3D::new(-0.765144, -0.441756, 0.468408))
        );
        assert!((r.bin_dir(274) - Vector3D::new(-0.816259, -0.338105, 0.468408)).length() < 1e-6);
        assert_eq!(
            274,
            r.dir_to_bin(Vector3D::new(-0.816259, -0.338105, 0.468408))
        );
        assert!((r.bin_dir(275) - Vector3D::new(-0.853407, -0.22867, 0.468408)).length() < 1e-6);
        assert_eq!(
            275,
            r.dir_to_bin(Vector3D::new(-0.853407, -0.22867, 0.468408))
        );
        assert!((r.bin_dir(276) - Vector3D::new(-0.875953, -0.115321, 0.468408)).length() < 1e-6);
        assert_eq!(
            276,
            r.dir_to_bin(Vector3D::new(-0.875953, -0.115321, 0.468408))
        );
        assert!(
            (r.bin_dir(277) - Vector3D::new(-0.883512, -1.62299e-16, 0.468408)).length() < 1e-6
        );
        assert_eq!(
            277,
            r.dir_to_bin(Vector3D::new(-0.883512, -1.62299e-16, 0.468408))
        );
        assert!((r.bin_dir(278) - Vector3D::new(-0.875953, 0.115321, 0.468408)).length() < 1e-6);
        assert_eq!(
            278,
            r.dir_to_bin(Vector3D::new(-0.875953, 0.115321, 0.468408))
        );
        assert!((r.bin_dir(279) - Vector3D::new(-0.853407, 0.22867, 0.468408)).length() < 1e-6);
        assert_eq!(
            279,
            r.dir_to_bin(Vector3D::new(-0.853407, 0.22867, 0.468408))
        );
        assert!((r.bin_dir(280) - Vector3D::new(-0.816259, 0.338105, 0.468408)).length() < 1e-6);
        assert_eq!(
            280,
            r.dir_to_bin(Vector3D::new(-0.816259, 0.338105, 0.468408))
        );
        assert!((r.bin_dir(281) - Vector3D::new(-0.765144, 0.441756, 0.468408)).length() < 1e-6);
        assert_eq!(
            281,
            r.dir_to_bin(Vector3D::new(-0.765144, 0.441756, 0.468408))
        );
        assert!((r.bin_dir(282) - Vector3D::new(-0.700937, 0.537848, 0.468408)).length() < 1e-6);
        assert_eq!(
            282,
            r.dir_to_bin(Vector3D::new(-0.700937, 0.537848, 0.468408))
        );
        assert!((r.bin_dir(283) - Vector3D::new(-0.624737, 0.624737, 0.468408)).length() < 1e-6);
        assert_eq!(
            283,
            r.dir_to_bin(Vector3D::new(-0.624737, 0.624737, 0.468408))
        );
        assert!((r.bin_dir(284) - Vector3D::new(-0.537848, 0.700937, 0.468408)).length() < 1e-6);
        assert_eq!(
            284,
            r.dir_to_bin(Vector3D::new(-0.537848, 0.700937, 0.468408))
        );
        assert!((r.bin_dir(285) - Vector3D::new(-0.441756, 0.765144, 0.468408)).length() < 1e-6);
        assert_eq!(
            285,
            r.dir_to_bin(Vector3D::new(-0.441756, 0.765144, 0.468408))
        );
        assert!((r.bin_dir(286) - Vector3D::new(-0.338105, 0.816259, 0.468408)).length() < 1e-6);
        assert_eq!(
            286,
            r.dir_to_bin(Vector3D::new(-0.338105, 0.816259, 0.468408))
        );
        assert!((r.bin_dir(287) - Vector3D::new(-0.22867, 0.853407, 0.468408)).length() < 1e-6);
        assert_eq!(
            287,
            r.dir_to_bin(Vector3D::new(-0.22867, 0.853407, 0.468408))
        );
        assert!((r.bin_dir(288) - Vector3D::new(-0.115321, 0.875953, 0.468408)).length() < 1e-6);
        assert_eq!(
            288,
            r.dir_to_bin(Vector3D::new(-0.115321, 0.875953, 0.468408))
        );
        assert!((r.bin_dir(289) - Vector3D::new(0., 0.827689, 0.561187)).length() < 1e-6);
        assert_eq!(289, r.dir_to_bin(Vector3D::new(0., 0.827689, 0.561187)));
        assert!((r.bin_dir(290) - Vector3D::new(0.108035, 0.820608, 0.561187)).length() < 1e-6);
        assert_eq!(
            290,
            r.dir_to_bin(Vector3D::new(0.108035, 0.820608, 0.561187))
        );
        assert!((r.bin_dir(291) - Vector3D::new(0.214222, 0.799486, 0.561187)).length() < 1e-6);
        assert_eq!(
            291,
            r.dir_to_bin(Vector3D::new(0.214222, 0.799486, 0.561187))
        );
        assert!((r.bin_dir(292) - Vector3D::new(0.316743, 0.764685, 0.561187)).length() < 1e-6);
        assert_eq!(
            292,
            r.dir_to_bin(Vector3D::new(0.316743, 0.764685, 0.561187))
        );
        assert!((r.bin_dir(293) - Vector3D::new(0.413844, 0.7168, 0.561187)).length() < 1e-6);
        assert_eq!(293, r.dir_to_bin(Vector3D::new(0.413844, 0.7168, 0.561187)));
        assert!((r.bin_dir(294) - Vector3D::new(0.503865, 0.65665, 0.561187)).length() < 1e-6);
        assert_eq!(
            294,
            r.dir_to_bin(Vector3D::new(0.503865, 0.65665, 0.561187))
        );
        assert!((r.bin_dir(295) - Vector3D::new(0.585265, 0.585265, 0.561187)).length() < 1e-6);
        assert_eq!(
            295,
            r.dir_to_bin(Vector3D::new(0.585265, 0.585265, 0.561187))
        );
        assert!((r.bin_dir(296) - Vector3D::new(0.65665, 0.503865, 0.561187)).length() < 1e-6);
        assert_eq!(
            296,
            r.dir_to_bin(Vector3D::new(0.65665, 0.503865, 0.561187))
        );
        assert!((r.bin_dir(297) - Vector3D::new(0.7168, 0.413844, 0.561187)).length() < 1e-6);
        assert_eq!(297, r.dir_to_bin(Vector3D::new(0.7168, 0.413844, 0.561187)));
        assert!((r.bin_dir(298) - Vector3D::new(0.764685, 0.316743, 0.561187)).length() < 1e-6);
        assert_eq!(
            298,
            r.dir_to_bin(Vector3D::new(0.764685, 0.316743, 0.561187))
        );
        assert!((r.bin_dir(299) - Vector3D::new(0.799486, 0.214222, 0.561187)).length() < 1e-6);
        assert_eq!(
            299,
            r.dir_to_bin(Vector3D::new(0.799486, 0.214222, 0.561187))
        );
        assert!((r.bin_dir(300) - Vector3D::new(0.820608, 0.108035, 0.561187)).length() < 1e-6);
        assert_eq!(
            300,
            r.dir_to_bin(Vector3D::new(0.820608, 0.108035, 0.561187))
        );
        assert!((r.bin_dir(301) - Vector3D::new(0.827689, 5.06813e-17, 0.561187)).length() < 1e-6);
        assert_eq!(
            301,
            r.dir_to_bin(Vector3D::new(0.827689, 5.06813e-17, 0.561187))
        );
        assert!((r.bin_dir(302) - Vector3D::new(0.820608, -0.108035, 0.561187)).length() < 1e-6);
        assert_eq!(
            302,
            r.dir_to_bin(Vector3D::new(0.820608, -0.108035, 0.561187))
        );
        assert!((r.bin_dir(303) - Vector3D::new(0.799486, -0.214222, 0.561187)).length() < 1e-6);
        assert_eq!(
            303,
            r.dir_to_bin(Vector3D::new(0.799486, -0.214222, 0.561187))
        );
        assert!((r.bin_dir(304) - Vector3D::new(0.764685, -0.316743, 0.561187)).length() < 1e-6);
        assert_eq!(
            304,
            r.dir_to_bin(Vector3D::new(0.764685, -0.316743, 0.561187))
        );
        assert!((r.bin_dir(305) - Vector3D::new(0.7168, -0.413844, 0.561187)).length() < 1e-6);
        assert_eq!(
            305,
            r.dir_to_bin(Vector3D::new(0.7168, -0.413844, 0.561187))
        );
        assert!((r.bin_dir(306) - Vector3D::new(0.65665, -0.503865, 0.561187)).length() < 1e-6);
        assert_eq!(
            306,
            r.dir_to_bin(Vector3D::new(0.65665, -0.503865, 0.561187))
        );
        assert!((r.bin_dir(307) - Vector3D::new(0.585265, -0.585265, 0.561187)).length() < 1e-6);
        assert_eq!(
            307,
            r.dir_to_bin(Vector3D::new(0.585265, -0.585265, 0.561187))
        );
        assert!((r.bin_dir(308) - Vector3D::new(0.503865, -0.65665, 0.561187)).length() < 1e-6);
        assert_eq!(
            308,
            r.dir_to_bin(Vector3D::new(0.503865, -0.65665, 0.561187))
        );
        assert!((r.bin_dir(309) - Vector3D::new(0.413844, -0.7168, 0.561187)).length() < 1e-6);
        assert_eq!(
            309,
            r.dir_to_bin(Vector3D::new(0.413844, -0.7168, 0.561187))
        );
        assert!((r.bin_dir(310) - Vector3D::new(0.316743, -0.764685, 0.561187)).length() < 1e-6);
        assert_eq!(
            310,
            r.dir_to_bin(Vector3D::new(0.316743, -0.764685, 0.561187))
        );
        assert!((r.bin_dir(311) - Vector3D::new(0.214222, -0.799486, 0.561187)).length() < 1e-6);
        assert_eq!(
            311,
            r.dir_to_bin(Vector3D::new(0.214222, -0.799486, 0.561187))
        );
        assert!((r.bin_dir(312) - Vector3D::new(0.108035, -0.820608, 0.561187)).length() < 1e-6);
        assert_eq!(
            312,
            r.dir_to_bin(Vector3D::new(0.108035, -0.820608, 0.561187))
        );
        assert!((r.bin_dir(313) - Vector3D::new(1.01363e-16, -0.827689, 0.561187)).length() < 1e-6);
        assert_eq!(
            313,
            r.dir_to_bin(Vector3D::new(1.01363e-16, -0.827689, 0.561187))
        );
        assert!((r.bin_dir(314) - Vector3D::new(-0.108035, -0.820608, 0.561187)).length() < 1e-6);
        assert_eq!(
            314,
            r.dir_to_bin(Vector3D::new(-0.108035, -0.820608, 0.561187))
        );
        assert!((r.bin_dir(315) - Vector3D::new(-0.214222, -0.799486, 0.561187)).length() < 1e-6);
        assert_eq!(
            315,
            r.dir_to_bin(Vector3D::new(-0.214222, -0.799486, 0.561187))
        );
        assert!((r.bin_dir(316) - Vector3D::new(-0.316743, -0.764685, 0.561187)).length() < 1e-6);
        assert_eq!(
            316,
            r.dir_to_bin(Vector3D::new(-0.316743, -0.764685, 0.561187))
        );
        assert!((r.bin_dir(317) - Vector3D::new(-0.413844, -0.7168, 0.561187)).length() < 1e-6);
        assert_eq!(
            317,
            r.dir_to_bin(Vector3D::new(-0.413844, -0.7168, 0.561187))
        );
        assert!((r.bin_dir(318) - Vector3D::new(-0.503865, -0.65665, 0.561187)).length() < 1e-6);
        assert_eq!(
            318,
            r.dir_to_bin(Vector3D::new(-0.503865, -0.65665, 0.561187))
        );
        assert!((r.bin_dir(319) - Vector3D::new(-0.585265, -0.585265, 0.561187)).length() < 1e-6);
        assert_eq!(
            319,
            r.dir_to_bin(Vector3D::new(-0.585265, -0.585265, 0.561187))
        );
        assert!((r.bin_dir(320) - Vector3D::new(-0.65665, -0.503865, 0.561187)).length() < 1e-6);
        assert_eq!(
            320,
            r.dir_to_bin(Vector3D::new(-0.65665, -0.503865, 0.561187))
        );
        assert!((r.bin_dir(321) - Vector3D::new(-0.7168, -0.413844, 0.561187)).length() < 1e-6);
        assert_eq!(
            321,
            r.dir_to_bin(Vector3D::new(-0.7168, -0.413844, 0.561187))
        );
        assert!((r.bin_dir(322) - Vector3D::new(-0.764685, -0.316743, 0.561187)).length() < 1e-6);
        assert_eq!(
            322,
            r.dir_to_bin(Vector3D::new(-0.764685, -0.316743, 0.561187))
        );
        assert!((r.bin_dir(323) - Vector3D::new(-0.799486, -0.214222, 0.561187)).length() < 1e-6);
        assert_eq!(
            323,
            r.dir_to_bin(Vector3D::new(-0.799486, -0.214222, 0.561187))
        );
        assert!((r.bin_dir(324) - Vector3D::new(-0.820608, -0.108035, 0.561187)).length() < 1e-6);
        assert_eq!(
            324,
            r.dir_to_bin(Vector3D::new(-0.820608, -0.108035, 0.561187))
        );
        assert!(
            (r.bin_dir(325) - Vector3D::new(-0.827689, -1.52044e-16, 0.561187)).length() < 1e-6
        );
        assert_eq!(
            325,
            r.dir_to_bin(Vector3D::new(-0.827689, -1.52044e-16, 0.561187))
        );
        assert!((r.bin_dir(326) - Vector3D::new(-0.820608, 0.108035, 0.561187)).length() < 1e-6);
        assert_eq!(
            326,
            r.dir_to_bin(Vector3D::new(-0.820608, 0.108035, 0.561187))
        );
        assert!((r.bin_dir(327) - Vector3D::new(-0.799486, 0.214222, 0.561187)).length() < 1e-6);
        assert_eq!(
            327,
            r.dir_to_bin(Vector3D::new(-0.799486, 0.214222, 0.561187))
        );
        assert!((r.bin_dir(328) - Vector3D::new(-0.764685, 0.316743, 0.561187)).length() < 1e-6);
        assert_eq!(
            328,
            r.dir_to_bin(Vector3D::new(-0.764685, 0.316743, 0.561187))
        );
        assert!((r.bin_dir(329) - Vector3D::new(-0.7168, 0.413844, 0.561187)).length() < 1e-6);
        assert_eq!(
            329,
            r.dir_to_bin(Vector3D::new(-0.7168, 0.413844, 0.561187))
        );
        assert!((r.bin_dir(330) - Vector3D::new(-0.65665, 0.503865, 0.561187)).length() < 1e-6);
        assert_eq!(
            330,
            r.dir_to_bin(Vector3D::new(-0.65665, 0.503865, 0.561187))
        );
        assert!((r.bin_dir(331) - Vector3D::new(-0.585265, 0.585265, 0.561187)).length() < 1e-6);
        assert_eq!(
            331,
            r.dir_to_bin(Vector3D::new(-0.585265, 0.585265, 0.561187))
        );
        assert!((r.bin_dir(332) - Vector3D::new(-0.503865, 0.65665, 0.561187)).length() < 1e-6);
        assert_eq!(
            332,
            r.dir_to_bin(Vector3D::new(-0.503865, 0.65665, 0.561187))
        );
        assert!((r.bin_dir(333) - Vector3D::new(-0.413844, 0.7168, 0.561187)).length() < 1e-6);
        assert_eq!(
            333,
            r.dir_to_bin(Vector3D::new(-0.413844, 0.7168, 0.561187))
        );
        assert!((r.bin_dir(334) - Vector3D::new(-0.316743, 0.764685, 0.561187)).length() < 1e-6);
        assert_eq!(
            334,
            r.dir_to_bin(Vector3D::new(-0.316743, 0.764685, 0.561187))
        );
        assert!((r.bin_dir(335) - Vector3D::new(-0.214222, 0.799486, 0.561187)).length() < 1e-6);
        assert_eq!(
            335,
            r.dir_to_bin(Vector3D::new(-0.214222, 0.799486, 0.561187))
        );
        assert!((r.bin_dir(336) - Vector3D::new(-0.108035, 0.820608, 0.561187)).length() < 1e-6);
        assert_eq!(
            336,
            r.dir_to_bin(Vector3D::new(-0.108035, 0.820608, 0.561187))
        );
        assert!((r.bin_dir(337) - Vector3D::new(0., 0.762162, 0.647386)).length() < 1e-6);
        assert_eq!(337, r.dir_to_bin(Vector3D::new(0., 0.762162, 0.647386)));
        assert!((r.bin_dir(338) - Vector3D::new(0.0994821, 0.755642, 0.647386)).length() < 1e-6);
        assert_eq!(
            338,
            r.dir_to_bin(Vector3D::new(0.0994821, 0.755642, 0.647386))
        );
        assert!((r.bin_dir(339) - Vector3D::new(0.197262, 0.736192, 0.647386)).length() < 1e-6);
        assert_eq!(
            339,
            r.dir_to_bin(Vector3D::new(0.197262, 0.736192, 0.647386))
        );
        assert!((r.bin_dir(340) - Vector3D::new(0.291667, 0.704146, 0.647386)).length() < 1e-6);
        assert_eq!(
            340,
            r.dir_to_bin(Vector3D::new(0.291667, 0.704146, 0.647386))
        );
        assert!((r.bin_dir(341) - Vector3D::new(0.381081, 0.660052, 0.647386)).length() < 1e-6);
        assert_eq!(
            341,
            r.dir_to_bin(Vector3D::new(0.381081, 0.660052, 0.647386))
        );
        assert!((r.bin_dir(342) - Vector3D::new(0.463975, 0.604664, 0.647386)).length() < 1e-6);
        assert_eq!(
            342,
            r.dir_to_bin(Vector3D::new(0.463975, 0.604664, 0.647386))
        );
        assert!((r.bin_dir(343) - Vector3D::new(0.53893, 0.53893, 0.647386)).length() < 1e-6);
        assert_eq!(343, r.dir_to_bin(Vector3D::new(0.53893, 0.53893, 0.647386)));
        assert!((r.bin_dir(344) - Vector3D::new(0.604664, 0.463975, 0.647386)).length() < 1e-6);
        assert_eq!(
            344,
            r.dir_to_bin(Vector3D::new(0.604664, 0.463975, 0.647386))
        );
        assert!((r.bin_dir(345) - Vector3D::new(0.660052, 0.381081, 0.647386)).length() < 1e-6);
        assert_eq!(
            345,
            r.dir_to_bin(Vector3D::new(0.660052, 0.381081, 0.647386))
        );
        assert!((r.bin_dir(346) - Vector3D::new(0.704146, 0.291667, 0.647386)).length() < 1e-6);
        assert_eq!(
            346,
            r.dir_to_bin(Vector3D::new(0.704146, 0.291667, 0.647386))
        );
        assert!((r.bin_dir(347) - Vector3D::new(0.736192, 0.197262, 0.647386)).length() < 1e-6);
        assert_eq!(
            347,
            r.dir_to_bin(Vector3D::new(0.736192, 0.197262, 0.647386))
        );
        assert!((r.bin_dir(348) - Vector3D::new(0.755642, 0.0994821, 0.647386)).length() < 1e-6);
        assert_eq!(
            348,
            r.dir_to_bin(Vector3D::new(0.755642, 0.0994821, 0.647386))
        );
        assert!((r.bin_dir(349) - Vector3D::new(0.762162, 4.6669e-17, 0.647386)).length() < 1e-6);
        assert_eq!(
            349,
            r.dir_to_bin(Vector3D::new(0.762162, 4.6669e-17, 0.647386))
        );
        assert!((r.bin_dir(350) - Vector3D::new(0.755642, -0.0994821, 0.647386)).length() < 1e-6);
        assert_eq!(
            350,
            r.dir_to_bin(Vector3D::new(0.755642, -0.0994821, 0.647386))
        );
        assert!((r.bin_dir(351) - Vector3D::new(0.736192, -0.197262, 0.647386)).length() < 1e-6);
        assert_eq!(
            351,
            r.dir_to_bin(Vector3D::new(0.736192, -0.197262, 0.647386))
        );
        assert!((r.bin_dir(352) - Vector3D::new(0.704146, -0.291667, 0.647386)).length() < 1e-6);
        assert_eq!(
            352,
            r.dir_to_bin(Vector3D::new(0.704146, -0.291667, 0.647386))
        );
        assert!((r.bin_dir(353) - Vector3D::new(0.660052, -0.381081, 0.647386)).length() < 1e-6);
        assert_eq!(
            353,
            r.dir_to_bin(Vector3D::new(0.660052, -0.381081, 0.647386))
        );
        assert!((r.bin_dir(354) - Vector3D::new(0.604664, -0.463975, 0.647386)).length() < 1e-6);
        assert_eq!(
            354,
            r.dir_to_bin(Vector3D::new(0.604664, -0.463975, 0.647386))
        );
        assert!((r.bin_dir(355) - Vector3D::new(0.53893, -0.53893, 0.647386)).length() < 1e-6);
        assert_eq!(
            355,
            r.dir_to_bin(Vector3D::new(0.53893, -0.53893, 0.647386))
        );
        assert!((r.bin_dir(356) - Vector3D::new(0.463975, -0.604664, 0.647386)).length() < 1e-6);
        assert_eq!(
            356,
            r.dir_to_bin(Vector3D::new(0.463975, -0.604664, 0.647386))
        );
        assert!((r.bin_dir(357) - Vector3D::new(0.381081, -0.660052, 0.647386)).length() < 1e-6);
        assert_eq!(
            357,
            r.dir_to_bin(Vector3D::new(0.381081, -0.660052, 0.647386))
        );
        assert!((r.bin_dir(358) - Vector3D::new(0.291667, -0.704146, 0.647386)).length() < 1e-6);
        assert_eq!(
            358,
            r.dir_to_bin(Vector3D::new(0.291667, -0.704146, 0.647386))
        );
        assert!((r.bin_dir(359) - Vector3D::new(0.197262, -0.736192, 0.647386)).length() < 1e-6);
        assert_eq!(
            359,
            r.dir_to_bin(Vector3D::new(0.197262, -0.736192, 0.647386))
        );
        assert!((r.bin_dir(360) - Vector3D::new(0.0994821, -0.755642, 0.647386)).length() < 1e-6);
        assert_eq!(
            360,
            r.dir_to_bin(Vector3D::new(0.0994821, -0.755642, 0.647386))
        );
        assert!((r.bin_dir(361) - Vector3D::new(9.33379e-17, -0.762162, 0.647386)).length() < 1e-6);
        assert_eq!(
            361,
            r.dir_to_bin(Vector3D::new(9.33379e-17, -0.762162, 0.647386))
        );
        assert!((r.bin_dir(362) - Vector3D::new(-0.0994821, -0.755642, 0.647386)).length() < 1e-6);
        assert_eq!(
            362,
            r.dir_to_bin(Vector3D::new(-0.0994821, -0.755642, 0.647386))
        );
        assert!((r.bin_dir(363) - Vector3D::new(-0.197262, -0.736192, 0.647386)).length() < 1e-6);
        assert_eq!(
            363,
            r.dir_to_bin(Vector3D::new(-0.197262, -0.736192, 0.647386))
        );
        assert!((r.bin_dir(364) - Vector3D::new(-0.291667, -0.704146, 0.647386)).length() < 1e-6);
        assert_eq!(
            364,
            r.dir_to_bin(Vector3D::new(-0.291667, -0.704146, 0.647386))
        );
        assert!((r.bin_dir(365) - Vector3D::new(-0.381081, -0.660052, 0.647386)).length() < 1e-6);
        assert_eq!(
            365,
            r.dir_to_bin(Vector3D::new(-0.381081, -0.660052, 0.647386))
        );
        assert!((r.bin_dir(366) - Vector3D::new(-0.463975, -0.604664, 0.647386)).length() < 1e-6);
        assert_eq!(
            366,
            r.dir_to_bin(Vector3D::new(-0.463975, -0.604664, 0.647386))
        );
        assert!((r.bin_dir(367) - Vector3D::new(-0.53893, -0.53893, 0.647386)).length() < 1e-6);
        assert_eq!(
            367,
            r.dir_to_bin(Vector3D::new(-0.53893, -0.53893, 0.647386))
        );
        assert!((r.bin_dir(368) - Vector3D::new(-0.604664, -0.463975, 0.647386)).length() < 1e-6);
        assert_eq!(
            368,
            r.dir_to_bin(Vector3D::new(-0.604664, -0.463975, 0.647386))
        );
        assert!((r.bin_dir(369) - Vector3D::new(-0.660052, -0.381081, 0.647386)).length() < 1e-6);
        assert_eq!(
            369,
            r.dir_to_bin(Vector3D::new(-0.660052, -0.381081, 0.647386))
        );
        assert!((r.bin_dir(370) - Vector3D::new(-0.704146, -0.291667, 0.647386)).length() < 1e-6);
        assert_eq!(
            370,
            r.dir_to_bin(Vector3D::new(-0.704146, -0.291667, 0.647386))
        );
        assert!((r.bin_dir(371) - Vector3D::new(-0.736192, -0.197262, 0.647386)).length() < 1e-6);
        assert_eq!(
            371,
            r.dir_to_bin(Vector3D::new(-0.736192, -0.197262, 0.647386))
        );
        assert!((r.bin_dir(372) - Vector3D::new(-0.755642, -0.0994821, 0.647386)).length() < 1e-6);
        assert_eq!(
            372,
            r.dir_to_bin(Vector3D::new(-0.755642, -0.0994821, 0.647386))
        );
        assert!(
            (r.bin_dir(373) - Vector3D::new(-0.762162, -1.40007e-16, 0.647386)).length() < 1e-6
        );
        assert_eq!(
            373,
            r.dir_to_bin(Vector3D::new(-0.762162, -1.40007e-16, 0.647386))
        );
        assert!((r.bin_dir(374) - Vector3D::new(-0.755642, 0.0994821, 0.647386)).length() < 1e-6);
        assert_eq!(
            374,
            r.dir_to_bin(Vector3D::new(-0.755642, 0.0994821, 0.647386))
        );
        assert!((r.bin_dir(375) - Vector3D::new(-0.736192, 0.197262, 0.647386)).length() < 1e-6);
        assert_eq!(
            375,
            r.dir_to_bin(Vector3D::new(-0.736192, 0.197262, 0.647386))
        );
        assert!((r.bin_dir(376) - Vector3D::new(-0.704146, 0.291667, 0.647386)).length() < 1e-6);
        assert_eq!(
            376,
            r.dir_to_bin(Vector3D::new(-0.704146, 0.291667, 0.647386))
        );
        assert!((r.bin_dir(377) - Vector3D::new(-0.660052, 0.381081, 0.647386)).length() < 1e-6);
        assert_eq!(
            377,
            r.dir_to_bin(Vector3D::new(-0.660052, 0.381081, 0.647386))
        );
        assert!((r.bin_dir(378) - Vector3D::new(-0.604664, 0.463975, 0.647386)).length() < 1e-6);
        assert_eq!(
            378,
            r.dir_to_bin(Vector3D::new(-0.604664, 0.463975, 0.647386))
        );
        assert!((r.bin_dir(379) - Vector3D::new(-0.53893, 0.53893, 0.647386)).length() < 1e-6);
        assert_eq!(
            379,
            r.dir_to_bin(Vector3D::new(-0.53893, 0.53893, 0.647386))
        );
        assert!((r.bin_dir(380) - Vector3D::new(-0.463975, 0.604664, 0.647386)).length() < 1e-6);
        assert_eq!(
            380,
            r.dir_to_bin(Vector3D::new(-0.463975, 0.604664, 0.647386))
        );
        assert!((r.bin_dir(381) - Vector3D::new(-0.381081, 0.660052, 0.647386)).length() < 1e-6);
        assert_eq!(
            381,
            r.dir_to_bin(Vector3D::new(-0.381081, 0.660052, 0.647386))
        );
        assert!((r.bin_dir(382) - Vector3D::new(-0.291667, 0.704146, 0.647386)).length() < 1e-6);
        assert_eq!(
            382,
            r.dir_to_bin(Vector3D::new(-0.291667, 0.704146, 0.647386))
        );
        assert!((r.bin_dir(383) - Vector3D::new(-0.197262, 0.736192, 0.647386)).length() < 1e-6);
        assert_eq!(
            383,
            r.dir_to_bin(Vector3D::new(-0.197262, 0.736192, 0.647386))
        );
        assert!((r.bin_dir(384) - Vector3D::new(-0.0994821, 0.755642, 0.647386)).length() < 1e-6);
        assert_eq!(
            384,
            r.dir_to_bin(Vector3D::new(-0.0994821, 0.755642, 0.647386))
        );
        assert!((r.bin_dir(385) - Vector3D::new(0., 0.687699, 0.725995)).length() < 1e-6);
        assert_eq!(385, r.dir_to_bin(Vector3D::new(0., 0.687699, 0.725995)));
        assert!((r.bin_dir(386) - Vector3D::new(0.0897628, 0.681816, 0.725995)).length() < 1e-6);
        assert_eq!(
            386,
            r.dir_to_bin(Vector3D::new(0.0897628, 0.681816, 0.725995))
        );
        assert!((r.bin_dir(387) - Vector3D::new(0.17799, 0.664267, 0.725995)).length() < 1e-6);
        assert_eq!(
            387,
            r.dir_to_bin(Vector3D::new(0.17799, 0.664267, 0.725995))
        );
        assert!((r.bin_dir(388) - Vector3D::new(0.263171, 0.635351, 0.725995)).length() < 1e-6);
        assert_eq!(
            388,
            r.dir_to_bin(Vector3D::new(0.263171, 0.635351, 0.725995))
        );
        assert!((r.bin_dir(389) - Vector3D::new(0.34385, 0.595565, 0.725995)).length() < 1e-6);
        assert_eq!(
            389,
            r.dir_to_bin(Vector3D::new(0.34385, 0.595565, 0.725995))
        );
        assert!((r.bin_dir(390) - Vector3D::new(0.418645, 0.545589, 0.725995)).length() < 1e-6);
        assert_eq!(
            390,
            r.dir_to_bin(Vector3D::new(0.418645, 0.545589, 0.725995))
        );
        assert!((r.bin_dir(391) - Vector3D::new(0.486277, 0.486277, 0.725995)).length() < 1e-6);
        assert_eq!(
            391,
            r.dir_to_bin(Vector3D::new(0.486277, 0.486277, 0.725995))
        );
        assert!((r.bin_dir(392) - Vector3D::new(0.545589, 0.418645, 0.725995)).length() < 1e-6);
        assert_eq!(
            392,
            r.dir_to_bin(Vector3D::new(0.545589, 0.418645, 0.725995))
        );
        assert!((r.bin_dir(393) - Vector3D::new(0.595565, 0.34385, 0.725995)).length() < 1e-6);
        assert_eq!(
            393,
            r.dir_to_bin(Vector3D::new(0.595565, 0.34385, 0.725995))
        );
        assert!((r.bin_dir(394) - Vector3D::new(0.635351, 0.263171, 0.725995)).length() < 1e-6);
        assert_eq!(
            394,
            r.dir_to_bin(Vector3D::new(0.635351, 0.263171, 0.725995))
        );
        assert!((r.bin_dir(395) - Vector3D::new(0.664267, 0.17799, 0.725995)).length() < 1e-6);
        assert_eq!(
            395,
            r.dir_to_bin(Vector3D::new(0.664267, 0.17799, 0.725995))
        );
        assert!((r.bin_dir(396) - Vector3D::new(0.681816, 0.0897628, 0.725995)).length() < 1e-6);
        assert_eq!(
            396,
            r.dir_to_bin(Vector3D::new(0.681816, 0.0897628, 0.725995))
        );
        assert!((r.bin_dir(397) - Vector3D::new(0.687699, 4.21094e-17, 0.725995)).length() < 1e-6);
        assert_eq!(
            397,
            r.dir_to_bin(Vector3D::new(0.687699, 4.21094e-17, 0.725995))
        );
        assert!((r.bin_dir(398) - Vector3D::new(0.681816, -0.0897628, 0.725995)).length() < 1e-6);
        assert_eq!(
            398,
            r.dir_to_bin(Vector3D::new(0.681816, -0.0897628, 0.725995))
        );
        assert!((r.bin_dir(399) - Vector3D::new(0.664267, -0.17799, 0.725995)).length() < 1e-6);
        assert_eq!(
            399,
            r.dir_to_bin(Vector3D::new(0.664267, -0.17799, 0.725995))
        );
        assert!((r.bin_dir(400) - Vector3D::new(0.635351, -0.263171, 0.725995)).length() < 1e-6);
        assert_eq!(
            400,
            r.dir_to_bin(Vector3D::new(0.635351, -0.263171, 0.725995))
        );
        assert!((r.bin_dir(401) - Vector3D::new(0.595565, -0.34385, 0.725995)).length() < 1e-6);
        assert_eq!(
            401,
            r.dir_to_bin(Vector3D::new(0.595565, -0.34385, 0.725995))
        );
        assert!((r.bin_dir(402) - Vector3D::new(0.545589, -0.418645, 0.725995)).length() < 1e-6);
        assert_eq!(
            402,
            r.dir_to_bin(Vector3D::new(0.545589, -0.418645, 0.725995))
        );
        assert!((r.bin_dir(403) - Vector3D::new(0.486277, -0.486277, 0.725995)).length() < 1e-6);
        assert_eq!(
            403,
            r.dir_to_bin(Vector3D::new(0.486277, -0.486277, 0.725995))
        );
        assert!((r.bin_dir(404) - Vector3D::new(0.418645, -0.545589, 0.725995)).length() < 1e-6);
        assert_eq!(
            404,
            r.dir_to_bin(Vector3D::new(0.418645, -0.545589, 0.725995))
        );
        assert!((r.bin_dir(405) - Vector3D::new(0.34385, -0.595565, 0.725995)).length() < 1e-6);
        assert_eq!(
            405,
            r.dir_to_bin(Vector3D::new(0.34385, -0.595565, 0.725995))
        );
        assert!((r.bin_dir(406) - Vector3D::new(0.263171, -0.635351, 0.725995)).length() < 1e-6);
        assert_eq!(
            406,
            r.dir_to_bin(Vector3D::new(0.263171, -0.635351, 0.725995))
        );
        assert!((r.bin_dir(407) - Vector3D::new(0.17799, -0.664267, 0.725995)).length() < 1e-6);
        assert_eq!(
            407,
            r.dir_to_bin(Vector3D::new(0.17799, -0.664267, 0.725995))
        );
        assert!((r.bin_dir(408) - Vector3D::new(0.0897628, -0.681816, 0.725995)).length() < 1e-6);
        assert_eq!(
            408,
            r.dir_to_bin(Vector3D::new(0.0897628, -0.681816, 0.725995))
        );
        assert!((r.bin_dir(409) - Vector3D::new(8.42189e-17, -0.687699, 0.725995)).length() < 1e-6);
        assert_eq!(
            409,
            r.dir_to_bin(Vector3D::new(8.42189e-17, -0.687699, 0.725995))
        );
        assert!((r.bin_dir(410) - Vector3D::new(-0.0897628, -0.681816, 0.725995)).length() < 1e-6);
        assert_eq!(
            410,
            r.dir_to_bin(Vector3D::new(-0.0897628, -0.681816, 0.725995))
        );
        assert!((r.bin_dir(411) - Vector3D::new(-0.17799, -0.664267, 0.725995)).length() < 1e-6);
        assert_eq!(
            411,
            r.dir_to_bin(Vector3D::new(-0.17799, -0.664267, 0.725995))
        );
        assert!((r.bin_dir(412) - Vector3D::new(-0.263171, -0.635351, 0.725995)).length() < 1e-6);
        assert_eq!(
            412,
            r.dir_to_bin(Vector3D::new(-0.263171, -0.635351, 0.725995))
        );
        assert!((r.bin_dir(413) - Vector3D::new(-0.34385, -0.595565, 0.725995)).length() < 1e-6);
        assert_eq!(
            413,
            r.dir_to_bin(Vector3D::new(-0.34385, -0.595565, 0.725995))
        );
        assert!((r.bin_dir(414) - Vector3D::new(-0.418645, -0.545589, 0.725995)).length() < 1e-6);
        assert_eq!(
            414,
            r.dir_to_bin(Vector3D::new(-0.418645, -0.545589, 0.725995))
        );
        assert!((r.bin_dir(415) - Vector3D::new(-0.486277, -0.486277, 0.725995)).length() < 1e-6);
        assert_eq!(
            415,
            r.dir_to_bin(Vector3D::new(-0.486277, -0.486277, 0.725995))
        );
        assert!((r.bin_dir(416) - Vector3D::new(-0.545589, -0.418645, 0.725995)).length() < 1e-6);
        assert_eq!(
            416,
            r.dir_to_bin(Vector3D::new(-0.545589, -0.418645, 0.725995))
        );
        assert!((r.bin_dir(417) - Vector3D::new(-0.595565, -0.34385, 0.725995)).length() < 1e-6);
        assert_eq!(
            417,
            r.dir_to_bin(Vector3D::new(-0.595565, -0.34385, 0.725995))
        );
        assert!((r.bin_dir(418) - Vector3D::new(-0.635351, -0.263171, 0.725995)).length() < 1e-6);
        assert_eq!(
            418,
            r.dir_to_bin(Vector3D::new(-0.635351, -0.263171, 0.725995))
        );
        assert!((r.bin_dir(419) - Vector3D::new(-0.664267, -0.17799, 0.725995)).length() < 1e-6);
        assert_eq!(
            419,
            r.dir_to_bin(Vector3D::new(-0.664267, -0.17799, 0.725995))
        );
        assert!((r.bin_dir(420) - Vector3D::new(-0.681816, -0.0897628, 0.725995)).length() < 1e-6);
        assert_eq!(
            420,
            r.dir_to_bin(Vector3D::new(-0.681816, -0.0897628, 0.725995))
        );
        assert!(
            (r.bin_dir(421) - Vector3D::new(-0.687699, -1.26328e-16, 0.725995)).length() < 1e-6
        );
        assert_eq!(
            421,
            r.dir_to_bin(Vector3D::new(-0.687699, -1.26328e-16, 0.725995))
        );
        assert!((r.bin_dir(422) - Vector3D::new(-0.681816, 0.0897628, 0.725995)).length() < 1e-6);
        assert_eq!(
            422,
            r.dir_to_bin(Vector3D::new(-0.681816, 0.0897628, 0.725995))
        );
        assert!((r.bin_dir(423) - Vector3D::new(-0.664267, 0.17799, 0.725995)).length() < 1e-6);
        assert_eq!(
            423,
            r.dir_to_bin(Vector3D::new(-0.664267, 0.17799, 0.725995))
        );
        assert!((r.bin_dir(424) - Vector3D::new(-0.635351, 0.263171, 0.725995)).length() < 1e-6);
        assert_eq!(
            424,
            r.dir_to_bin(Vector3D::new(-0.635351, 0.263171, 0.725995))
        );
        assert!((r.bin_dir(425) - Vector3D::new(-0.595565, 0.34385, 0.725995)).length() < 1e-6);
        assert_eq!(
            425,
            r.dir_to_bin(Vector3D::new(-0.595565, 0.34385, 0.725995))
        );
        assert!((r.bin_dir(426) - Vector3D::new(-0.545589, 0.418645, 0.725995)).length() < 1e-6);
        assert_eq!(
            426,
            r.dir_to_bin(Vector3D::new(-0.545589, 0.418645, 0.725995))
        );
        assert!((r.bin_dir(427) - Vector3D::new(-0.486277, 0.486277, 0.725995)).length() < 1e-6);
        assert_eq!(
            427,
            r.dir_to_bin(Vector3D::new(-0.486277, 0.486277, 0.725995))
        );
        assert!((r.bin_dir(428) - Vector3D::new(-0.418645, 0.545589, 0.725995)).length() < 1e-6);
        assert_eq!(
            428,
            r.dir_to_bin(Vector3D::new(-0.418645, 0.545589, 0.725995))
        );
        assert!((r.bin_dir(429) - Vector3D::new(-0.34385, 0.595565, 0.725995)).length() < 1e-6);
        assert_eq!(
            429,
            r.dir_to_bin(Vector3D::new(-0.34385, 0.595565, 0.725995))
        );
        assert!((r.bin_dir(430) - Vector3D::new(-0.263171, 0.635351, 0.725995)).length() < 1e-6);
        assert_eq!(
            430,
            r.dir_to_bin(Vector3D::new(-0.263171, 0.635351, 0.725995))
        );
        assert!((r.bin_dir(431) - Vector3D::new(-0.17799, 0.664267, 0.725995)).length() < 1e-6);
        assert_eq!(
            431,
            r.dir_to_bin(Vector3D::new(-0.17799, 0.664267, 0.725995))
        );
        assert!((r.bin_dir(432) - Vector3D::new(-0.0897628, 0.681816, 0.725995)).length() < 1e-6);
        assert_eq!(
            432,
            r.dir_to_bin(Vector3D::new(-0.0897628, 0.681816, 0.725995))
        );
        assert!((r.bin_dir(433) - Vector3D::new(0., 0.605174, 0.796093)).length() < 1e-6);
        assert_eq!(433, r.dir_to_bin(Vector3D::new(0., 0.605174, 0.796093)));
        assert!((r.bin_dir(434) - Vector3D::new(0.105087, 0.59598, 0.796093)).length() < 1e-6);
        assert_eq!(
            434,
            r.dir_to_bin(Vector3D::new(0.105087, 0.59598, 0.796093))
        );
        assert!((r.bin_dir(435) - Vector3D::new(0.206982, 0.568678, 0.796093)).length() < 1e-6);
        assert_eq!(
            435,
            r.dir_to_bin(Vector3D::new(0.206982, 0.568678, 0.796093))
        );
        assert!((r.bin_dir(436) - Vector3D::new(0.302587, 0.524096, 0.796093)).length() < 1e-6);
        assert_eq!(
            436,
            r.dir_to_bin(Vector3D::new(0.302587, 0.524096, 0.796093))
        );
        assert!((r.bin_dir(437) - Vector3D::new(0.388998, 0.46359, 0.796093)).length() < 1e-6);
        assert_eq!(
            437,
            r.dir_to_bin(Vector3D::new(0.388998, 0.46359, 0.796093))
        );
        assert!((r.bin_dir(438) - Vector3D::new(0.46359, 0.388998, 0.796093)).length() < 1e-6);
        assert_eq!(
            438,
            r.dir_to_bin(Vector3D::new(0.46359, 0.388998, 0.796093))
        );
        assert!((r.bin_dir(439) - Vector3D::new(0.524096, 0.302587, 0.796093)).length() < 1e-6);
        assert_eq!(
            439,
            r.dir_to_bin(Vector3D::new(0.524096, 0.302587, 0.796093))
        );
        assert!((r.bin_dir(440) - Vector3D::new(0.568678, 0.206982, 0.796093)).length() < 1e-6);
        assert_eq!(
            440,
            r.dir_to_bin(Vector3D::new(0.568678, 0.206982, 0.796093))
        );
        assert!((r.bin_dir(441) - Vector3D::new(0.59598, 0.105087, 0.796093)).length() < 1e-6);
        assert_eq!(
            441,
            r.dir_to_bin(Vector3D::new(0.59598, 0.105087, 0.796093))
        );
        assert!((r.bin_dir(442) - Vector3D::new(0.605174, 3.70562e-17, 0.796093)).length() < 1e-6);
        assert_eq!(
            442,
            r.dir_to_bin(Vector3D::new(0.605174, 3.70562e-17, 0.796093))
        );
        assert!((r.bin_dir(443) - Vector3D::new(0.59598, -0.105087, 0.796093)).length() < 1e-6);
        assert_eq!(
            443,
            r.dir_to_bin(Vector3D::new(0.59598, -0.105087, 0.796093))
        );
        assert!((r.bin_dir(444) - Vector3D::new(0.568678, -0.206982, 0.796093)).length() < 1e-6);
        assert_eq!(
            444,
            r.dir_to_bin(Vector3D::new(0.568678, -0.206982, 0.796093))
        );
        assert!((r.bin_dir(445) - Vector3D::new(0.524096, -0.302587, 0.796093)).length() < 1e-6);
        assert_eq!(
            445,
            r.dir_to_bin(Vector3D::new(0.524096, -0.302587, 0.796093))
        );
        assert!((r.bin_dir(446) - Vector3D::new(0.46359, -0.388998, 0.796093)).length() < 1e-6);
        assert_eq!(
            446,
            r.dir_to_bin(Vector3D::new(0.46359, -0.388998, 0.796093))
        );
        assert!((r.bin_dir(447) - Vector3D::new(0.388998, -0.46359, 0.796093)).length() < 1e-6);
        assert_eq!(
            447,
            r.dir_to_bin(Vector3D::new(0.388998, -0.46359, 0.796093))
        );
        assert!((r.bin_dir(448) - Vector3D::new(0.302587, -0.524096, 0.796093)).length() < 1e-6);
        assert_eq!(
            448,
            r.dir_to_bin(Vector3D::new(0.302587, -0.524096, 0.796093))
        );
        assert!((r.bin_dir(449) - Vector3D::new(0.206982, -0.568678, 0.796093)).length() < 1e-6);
        assert_eq!(
            449,
            r.dir_to_bin(Vector3D::new(0.206982, -0.568678, 0.796093))
        );
        assert!((r.bin_dir(450) - Vector3D::new(0.105087, -0.59598, 0.796093)).length() < 1e-6);
        assert_eq!(
            450,
            r.dir_to_bin(Vector3D::new(0.105087, -0.59598, 0.796093))
        );
        assert!((r.bin_dir(451) - Vector3D::new(7.41125e-17, -0.605174, 0.796093)).length() < 1e-6);
        assert_eq!(
            451,
            r.dir_to_bin(Vector3D::new(7.41125e-17, -0.605174, 0.796093))
        );
        assert!((r.bin_dir(452) - Vector3D::new(-0.105087, -0.59598, 0.796093)).length() < 1e-6);
        assert_eq!(
            452,
            r.dir_to_bin(Vector3D::new(-0.105087, -0.59598, 0.796093))
        );
        assert!((r.bin_dir(453) - Vector3D::new(-0.206982, -0.568678, 0.796093)).length() < 1e-6);
        assert_eq!(
            453,
            r.dir_to_bin(Vector3D::new(-0.206982, -0.568678, 0.796093))
        );
        assert!((r.bin_dir(454) - Vector3D::new(-0.302587, -0.524096, 0.796093)).length() < 1e-6);
        assert_eq!(
            454,
            r.dir_to_bin(Vector3D::new(-0.302587, -0.524096, 0.796093))
        );
        assert!((r.bin_dir(455) - Vector3D::new(-0.388998, -0.46359, 0.796093)).length() < 1e-6);
        assert_eq!(
            455,
            r.dir_to_bin(Vector3D::new(-0.388998, -0.46359, 0.796093))
        );
        assert!((r.bin_dir(456) - Vector3D::new(-0.46359, -0.388998, 0.796093)).length() < 1e-6);
        assert_eq!(
            456,
            r.dir_to_bin(Vector3D::new(-0.46359, -0.388998, 0.796093))
        );
        assert!((r.bin_dir(457) - Vector3D::new(-0.524096, -0.302587, 0.796093)).length() < 1e-6);
        assert_eq!(
            457,
            r.dir_to_bin(Vector3D::new(-0.524096, -0.302587, 0.796093))
        );
        assert!((r.bin_dir(458) - Vector3D::new(-0.568678, -0.206982, 0.796093)).length() < 1e-6);
        assert_eq!(
            458,
            r.dir_to_bin(Vector3D::new(-0.568678, -0.206982, 0.796093))
        );
        assert!((r.bin_dir(459) - Vector3D::new(-0.59598, -0.105087, 0.796093)).length() < 1e-6);
        assert_eq!(
            459,
            r.dir_to_bin(Vector3D::new(-0.59598, -0.105087, 0.796093))
        );
        assert!(
            (r.bin_dir(460) - Vector3D::new(-0.605174, -1.11169e-16, 0.796093)).length() < 1e-6
        );
        assert_eq!(
            460,
            r.dir_to_bin(Vector3D::new(-0.605174, -1.11169e-16, 0.796093))
        );
        assert!((r.bin_dir(461) - Vector3D::new(-0.59598, 0.105087, 0.796093)).length() < 1e-6);
        assert_eq!(
            461,
            r.dir_to_bin(Vector3D::new(-0.59598, 0.105087, 0.796093))
        );
        assert!((r.bin_dir(462) - Vector3D::new(-0.568678, 0.206982, 0.796093)).length() < 1e-6);
        assert_eq!(
            462,
            r.dir_to_bin(Vector3D::new(-0.568678, 0.206982, 0.796093))
        );
        assert!((r.bin_dir(463) - Vector3D::new(-0.524096, 0.302587, 0.796093)).length() < 1e-6);
        assert_eq!(
            463,
            r.dir_to_bin(Vector3D::new(-0.524096, 0.302587, 0.796093))
        );
        assert!((r.bin_dir(464) - Vector3D::new(-0.46359, 0.388998, 0.796093)).length() < 1e-6);
        assert_eq!(
            464,
            r.dir_to_bin(Vector3D::new(-0.46359, 0.388998, 0.796093))
        );
        assert!((r.bin_dir(465) - Vector3D::new(-0.388998, 0.46359, 0.796093)).length() < 1e-6);
        assert_eq!(
            465,
            r.dir_to_bin(Vector3D::new(-0.388998, 0.46359, 0.796093))
        );
        assert!((r.bin_dir(466) - Vector3D::new(-0.302587, 0.524096, 0.796093)).length() < 1e-6);
        assert_eq!(
            466,
            r.dir_to_bin(Vector3D::new(-0.302587, 0.524096, 0.796093))
        );
        assert!((r.bin_dir(467) - Vector3D::new(-0.206982, 0.568678, 0.796093)).length() < 1e-6);
        assert_eq!(
            467,
            r.dir_to_bin(Vector3D::new(-0.206982, 0.568678, 0.796093))
        );
        assert!((r.bin_dir(468) - Vector3D::new(-0.105087, 0.59598, 0.796093)).length() < 1e-6);
        assert_eq!(
            468,
            r.dir_to_bin(Vector3D::new(-0.105087, 0.59598, 0.796093))
        );
        assert!((r.bin_dir(469) - Vector3D::new(0., 0.515554, 0.856857)).length() < 1e-6);
        assert_eq!(469, r.dir_to_bin(Vector3D::new(0., 0.515554, 0.856857)));
        assert!((r.bin_dir(470) - Vector3D::new(0.089525, 0.507721, 0.856857)).length() < 1e-6);
        assert_eq!(
            470,
            r.dir_to_bin(Vector3D::new(0.089525, 0.507721, 0.856857))
        );
        assert!((r.bin_dir(471) - Vector3D::new(0.17633, 0.484462, 0.856857)).length() < 1e-6);
        assert_eq!(
            471,
            r.dir_to_bin(Vector3D::new(0.17633, 0.484462, 0.856857))
        );
        assert!((r.bin_dir(472) - Vector3D::new(0.257777, 0.446483, 0.856857)).length() < 1e-6);
        assert_eq!(
            472,
            r.dir_to_bin(Vector3D::new(0.257777, 0.446483, 0.856857))
        );
        assert!((r.bin_dir(473) - Vector3D::new(0.331392, 0.394937, 0.856857)).length() < 1e-6);
        assert_eq!(
            473,
            r.dir_to_bin(Vector3D::new(0.331392, 0.394937, 0.856857))
        );
        assert!((r.bin_dir(474) - Vector3D::new(0.394937, 0.331392, 0.856857)).length() < 1e-6);
        assert_eq!(
            474,
            r.dir_to_bin(Vector3D::new(0.394937, 0.331392, 0.856857))
        );
        assert!((r.bin_dir(475) - Vector3D::new(0.446483, 0.257777, 0.856857)).length() < 1e-6);
        assert_eq!(
            475,
            r.dir_to_bin(Vector3D::new(0.446483, 0.257777, 0.856857))
        );
        assert!((r.bin_dir(476) - Vector3D::new(0.484462, 0.17633, 0.856857)).length() < 1e-6);
        assert_eq!(
            476,
            r.dir_to_bin(Vector3D::new(0.484462, 0.17633, 0.856857))
        );
        assert!((r.bin_dir(477) - Vector3D::new(0.507721, 0.089525, 0.856857)).length() < 1e-6);
        assert_eq!(
            477,
            r.dir_to_bin(Vector3D::new(0.507721, 0.089525, 0.856857))
        );
        assert!((r.bin_dir(478) - Vector3D::new(0.515554, 3.15686e-17, 0.856857)).length() < 1e-6);
        assert_eq!(
            478,
            r.dir_to_bin(Vector3D::new(0.515554, 3.15686e-17, 0.856857))
        );
        assert!((r.bin_dir(479) - Vector3D::new(0.507721, -0.089525, 0.856857)).length() < 1e-6);
        assert_eq!(
            479,
            r.dir_to_bin(Vector3D::new(0.507721, -0.089525, 0.856857))
        );
        assert!((r.bin_dir(480) - Vector3D::new(0.484462, -0.17633, 0.856857)).length() < 1e-6);
        assert_eq!(
            480,
            r.dir_to_bin(Vector3D::new(0.484462, -0.17633, 0.856857))
        );
        assert!((r.bin_dir(481) - Vector3D::new(0.446483, -0.257777, 0.856857)).length() < 1e-6);
        assert_eq!(
            481,
            r.dir_to_bin(Vector3D::new(0.446483, -0.257777, 0.856857))
        );
        assert!((r.bin_dir(482) - Vector3D::new(0.394937, -0.331392, 0.856857)).length() < 1e-6);
        assert_eq!(
            482,
            r.dir_to_bin(Vector3D::new(0.394937, -0.331392, 0.856857))
        );
        assert!((r.bin_dir(483) - Vector3D::new(0.331392, -0.394937, 0.856857)).length() < 1e-6);
        assert_eq!(
            483,
            r.dir_to_bin(Vector3D::new(0.331392, -0.394937, 0.856857))
        );
        assert!((r.bin_dir(484) - Vector3D::new(0.257777, -0.446483, 0.856857)).length() < 1e-6);
        assert_eq!(
            484,
            r.dir_to_bin(Vector3D::new(0.257777, -0.446483, 0.856857))
        );
        assert!((r.bin_dir(485) - Vector3D::new(0.17633, -0.484462, 0.856857)).length() < 1e-6);
        assert_eq!(
            485,
            r.dir_to_bin(Vector3D::new(0.17633, -0.484462, 0.856857))
        );
        assert!((r.bin_dir(486) - Vector3D::new(0.089525, -0.507721, 0.856857)).length() < 1e-6);
        assert_eq!(
            486,
            r.dir_to_bin(Vector3D::new(0.089525, -0.507721, 0.856857))
        );
        assert!((r.bin_dir(487) - Vector3D::new(6.31371e-17, -0.515554, 0.856857)).length() < 1e-6);
        assert_eq!(
            487,
            r.dir_to_bin(Vector3D::new(6.31371e-17, -0.515554, 0.856857))
        );
        assert!((r.bin_dir(488) - Vector3D::new(-0.089525, -0.507721, 0.856857)).length() < 1e-6);
        assert_eq!(
            488,
            r.dir_to_bin(Vector3D::new(-0.089525, -0.507721, 0.856857))
        );
        assert!((r.bin_dir(489) - Vector3D::new(-0.17633, -0.484462, 0.856857)).length() < 1e-6);
        assert_eq!(
            489,
            r.dir_to_bin(Vector3D::new(-0.17633, -0.484462, 0.856857))
        );
        assert!((r.bin_dir(490) - Vector3D::new(-0.257777, -0.446483, 0.856857)).length() < 1e-6);
        assert_eq!(
            490,
            r.dir_to_bin(Vector3D::new(-0.257777, -0.446483, 0.856857))
        );
        assert!((r.bin_dir(491) - Vector3D::new(-0.331392, -0.394937, 0.856857)).length() < 1e-6);
        assert_eq!(
            491,
            r.dir_to_bin(Vector3D::new(-0.331392, -0.394937, 0.856857))
        );
        assert!((r.bin_dir(492) - Vector3D::new(-0.394937, -0.331392, 0.856857)).length() < 1e-6);
        assert_eq!(
            492,
            r.dir_to_bin(Vector3D::new(-0.394937, -0.331392, 0.856857))
        );
        assert!((r.bin_dir(493) - Vector3D::new(-0.446483, -0.257777, 0.856857)).length() < 1e-6);
        assert_eq!(
            493,
            r.dir_to_bin(Vector3D::new(-0.446483, -0.257777, 0.856857))
        );
        assert!((r.bin_dir(494) - Vector3D::new(-0.484462, -0.17633, 0.856857)).length() < 1e-6);
        assert_eq!(
            494,
            r.dir_to_bin(Vector3D::new(-0.484462, -0.17633, 0.856857))
        );
        assert!((r.bin_dir(495) - Vector3D::new(-0.507721, -0.089525, 0.856857)).length() < 1e-6);
        assert_eq!(
            495,
            r.dir_to_bin(Vector3D::new(-0.507721, -0.089525, 0.856857))
        );
        assert!(
            (r.bin_dir(496) - Vector3D::new(-0.515554, -9.47057e-17, 0.856857)).length() < 1e-6
        );
        assert_eq!(
            496,
            r.dir_to_bin(Vector3D::new(-0.515554, -9.47057e-17, 0.856857))
        );
        assert!((r.bin_dir(497) - Vector3D::new(-0.507721, 0.089525, 0.856857)).length() < 1e-6);
        assert_eq!(
            497,
            r.dir_to_bin(Vector3D::new(-0.507721, 0.089525, 0.856857))
        );
        assert!((r.bin_dir(498) - Vector3D::new(-0.484462, 0.17633, 0.856857)).length() < 1e-6);
        assert_eq!(
            498,
            r.dir_to_bin(Vector3D::new(-0.484462, 0.17633, 0.856857))
        );
        assert!((r.bin_dir(499) - Vector3D::new(-0.446483, 0.257777, 0.856857)).length() < 1e-6);
        assert_eq!(
            499,
            r.dir_to_bin(Vector3D::new(-0.446483, 0.257777, 0.856857))
        );
        assert!((r.bin_dir(500) - Vector3D::new(-0.394937, 0.331392, 0.856857)).length() < 1e-6);
        assert_eq!(
            500,
            r.dir_to_bin(Vector3D::new(-0.394937, 0.331392, 0.856857))
        );
        assert!((r.bin_dir(501) - Vector3D::new(-0.331392, 0.394937, 0.856857)).length() < 1e-6);
        assert_eq!(
            501,
            r.dir_to_bin(Vector3D::new(-0.331392, 0.394937, 0.856857))
        );
        assert!((r.bin_dir(502) - Vector3D::new(-0.257777, 0.446483, 0.856857)).length() < 1e-6);
        assert_eq!(
            502,
            r.dir_to_bin(Vector3D::new(-0.257777, 0.446483, 0.856857))
        );
        assert!((r.bin_dir(503) - Vector3D::new(-0.17633, 0.484462, 0.856857)).length() < 1e-6);
        assert_eq!(
            503,
            r.dir_to_bin(Vector3D::new(-0.17633, 0.484462, 0.856857))
        );
        assert!((r.bin_dir(504) - Vector3D::new(-0.089525, 0.507721, 0.856857)).length() < 1e-6);
        assert_eq!(
            504,
            r.dir_to_bin(Vector3D::new(-0.089525, 0.507721, 0.856857))
        );
        assert!((r.bin_dir(505) - Vector3D::new(0., 0.419889, 0.907575)).length() < 1e-6);
        assert_eq!(505, r.dir_to_bin(Vector3D::new(0., 0.419889, 0.907575)));
        assert!((r.bin_dir(506) - Vector3D::new(0.108675, 0.405582, 0.907575)).length() < 1e-6);
        assert_eq!(
            506,
            r.dir_to_bin(Vector3D::new(0.108675, 0.405582, 0.907575))
        );
        assert!((r.bin_dir(507) - Vector3D::new(0.209945, 0.363635, 0.907575)).length() < 1e-6);
        assert_eq!(
            507,
            r.dir_to_bin(Vector3D::new(0.209945, 0.363635, 0.907575))
        );
        assert!((r.bin_dir(508) - Vector3D::new(0.296906, 0.296906, 0.907575)).length() < 1e-6);
        assert_eq!(
            508,
            r.dir_to_bin(Vector3D::new(0.296906, 0.296906, 0.907575))
        );
        assert!((r.bin_dir(509) - Vector3D::new(0.363635, 0.209945, 0.907575)).length() < 1e-6);
        assert_eq!(
            509,
            r.dir_to_bin(Vector3D::new(0.363635, 0.209945, 0.907575))
        );
        assert!((r.bin_dir(510) - Vector3D::new(0.405582, 0.108675, 0.907575)).length() < 1e-6);
        assert_eq!(
            510,
            r.dir_to_bin(Vector3D::new(0.405582, 0.108675, 0.907575))
        );
        assert!((r.bin_dir(511) - Vector3D::new(0.419889, 2.57108e-17, 0.907575)).length() < 1e-6);
        assert_eq!(
            511,
            r.dir_to_bin(Vector3D::new(0.419889, 2.57108e-17, 0.907575))
        );
        assert!((r.bin_dir(512) - Vector3D::new(0.405582, -0.108675, 0.907575)).length() < 1e-6);
        assert_eq!(
            512,
            r.dir_to_bin(Vector3D::new(0.405582, -0.108675, 0.907575))
        );
        assert!((r.bin_dir(513) - Vector3D::new(0.363635, -0.209945, 0.907575)).length() < 1e-6);
        assert_eq!(
            513,
            r.dir_to_bin(Vector3D::new(0.363635, -0.209945, 0.907575))
        );
        assert!((r.bin_dir(514) - Vector3D::new(0.296906, -0.296906, 0.907575)).length() < 1e-6);
        assert_eq!(
            514,
            r.dir_to_bin(Vector3D::new(0.296906, -0.296906, 0.907575))
        );
        assert!((r.bin_dir(515) - Vector3D::new(0.209945, -0.363635, 0.907575)).length() < 1e-6);
        assert_eq!(
            515,
            r.dir_to_bin(Vector3D::new(0.209945, -0.363635, 0.907575))
        );
        assert!((r.bin_dir(516) - Vector3D::new(0.108675, -0.405582, 0.907575)).length() < 1e-6);
        assert_eq!(
            516,
            r.dir_to_bin(Vector3D::new(0.108675, -0.405582, 0.907575))
        );
        assert!((r.bin_dir(517) - Vector3D::new(5.14216e-17, -0.419889, 0.907575)).length() < 1e-6);
        assert_eq!(
            517,
            r.dir_to_bin(Vector3D::new(5.14216e-17, -0.419889, 0.907575))
        );
        assert!((r.bin_dir(518) - Vector3D::new(-0.108675, -0.405582, 0.907575)).length() < 1e-6);
        assert_eq!(
            518,
            r.dir_to_bin(Vector3D::new(-0.108675, -0.405582, 0.907575))
        );
        assert!((r.bin_dir(519) - Vector3D::new(-0.209945, -0.363635, 0.907575)).length() < 1e-6);
        assert_eq!(
            519,
            r.dir_to_bin(Vector3D::new(-0.209945, -0.363635, 0.907575))
        );
        assert!((r.bin_dir(520) - Vector3D::new(-0.296906, -0.296906, 0.907575)).length() < 1e-6);
        assert_eq!(
            520,
            r.dir_to_bin(Vector3D::new(-0.296906, -0.296906, 0.907575))
        );
        assert!((r.bin_dir(521) - Vector3D::new(-0.363635, -0.209945, 0.907575)).length() < 1e-6);
        assert_eq!(
            521,
            r.dir_to_bin(Vector3D::new(-0.363635, -0.209945, 0.907575))
        );
        assert!((r.bin_dir(522) - Vector3D::new(-0.405582, -0.108675, 0.907575)).length() < 1e-6);
        assert_eq!(
            522,
            r.dir_to_bin(Vector3D::new(-0.405582, -0.108675, 0.907575))
        );
        assert!(
            (r.bin_dir(523) - Vector3D::new(-0.419889, -7.71324e-17, 0.907575)).length() < 1e-6
        );
        assert_eq!(
            523,
            r.dir_to_bin(Vector3D::new(-0.419889, -7.71324e-17, 0.907575))
        );
        assert!((r.bin_dir(524) - Vector3D::new(-0.405582, 0.108675, 0.907575)).length() < 1e-6);
        assert_eq!(
            524,
            r.dir_to_bin(Vector3D::new(-0.405582, 0.108675, 0.907575))
        );
        assert!((r.bin_dir(525) - Vector3D::new(-0.363635, 0.209945, 0.907575)).length() < 1e-6);
        assert_eq!(
            525,
            r.dir_to_bin(Vector3D::new(-0.363635, 0.209945, 0.907575))
        );
        assert!((r.bin_dir(526) - Vector3D::new(-0.296906, 0.296906, 0.907575)).length() < 1e-6);
        assert_eq!(
            526,
            r.dir_to_bin(Vector3D::new(-0.296906, 0.296906, 0.907575))
        );
        assert!((r.bin_dir(527) - Vector3D::new(-0.209945, 0.363635, 0.907575)).length() < 1e-6);
        assert_eq!(
            527,
            r.dir_to_bin(Vector3D::new(-0.209945, 0.363635, 0.907575))
        );
        assert!((r.bin_dir(528) - Vector3D::new(-0.108675, 0.405582, 0.907575)).length() < 1e-6);
        assert_eq!(
            528,
            r.dir_to_bin(Vector3D::new(-0.108675, 0.405582, 0.907575))
        );
        assert!((r.bin_dir(529) - Vector3D::new(0., 0.319302, 0.947653)).length() < 1e-6);
        assert_eq!(529, r.dir_to_bin(Vector3D::new(0., 0.319302, 0.947653)));
        assert!((r.bin_dir(530) - Vector3D::new(0.0826413, 0.308422, 0.947653)).length() < 1e-6);
        assert_eq!(
            530,
            r.dir_to_bin(Vector3D::new(0.0826413, 0.308422, 0.947653))
        );
        assert!((r.bin_dir(531) - Vector3D::new(0.159651, 0.276523, 0.947653)).length() < 1e-6);
        assert_eq!(
            531,
            r.dir_to_bin(Vector3D::new(0.159651, 0.276523, 0.947653))
        );
        assert!((r.bin_dir(532) - Vector3D::new(0.22578, 0.22578, 0.947653)).length() < 1e-6);
        assert_eq!(532, r.dir_to_bin(Vector3D::new(0.22578, 0.22578, 0.947653)));
        assert!((r.bin_dir(533) - Vector3D::new(0.276523, 0.159651, 0.947653)).length() < 1e-6);
        assert_eq!(
            533,
            r.dir_to_bin(Vector3D::new(0.276523, 0.159651, 0.947653))
        );
        assert!((r.bin_dir(534) - Vector3D::new(0.308422, 0.0826413, 0.947653)).length() < 1e-6);
        assert_eq!(
            534,
            r.dir_to_bin(Vector3D::new(0.308422, 0.0826413, 0.947653))
        );
        assert!((r.bin_dir(535) - Vector3D::new(0.319302, 1.95516e-17, 0.947653)).length() < 1e-6);
        assert_eq!(
            535,
            r.dir_to_bin(Vector3D::new(0.319302, 1.95516e-17, 0.947653))
        );
        assert!((r.bin_dir(536) - Vector3D::new(0.308422, -0.0826413, 0.947653)).length() < 1e-6);
        assert_eq!(
            536,
            r.dir_to_bin(Vector3D::new(0.308422, -0.0826413, 0.947653))
        );
        assert!((r.bin_dir(537) - Vector3D::new(0.276523, -0.159651, 0.947653)).length() < 1e-6);
        assert_eq!(
            537,
            r.dir_to_bin(Vector3D::new(0.276523, -0.159651, 0.947653))
        );
        assert!((r.bin_dir(538) - Vector3D::new(0.22578, -0.22578, 0.947653)).length() < 1e-6);
        assert_eq!(
            538,
            r.dir_to_bin(Vector3D::new(0.22578, -0.22578, 0.947653))
        );
        assert!((r.bin_dir(539) - Vector3D::new(0.159651, -0.276523, 0.947653)).length() < 1e-6);
        assert_eq!(
            539,
            r.dir_to_bin(Vector3D::new(0.159651, -0.276523, 0.947653))
        );
        assert!((r.bin_dir(540) - Vector3D::new(0.0826413, -0.308422, 0.947653)).length() < 1e-6);
        assert_eq!(
            540,
            r.dir_to_bin(Vector3D::new(0.0826413, -0.308422, 0.947653))
        );
        assert!((r.bin_dir(541) - Vector3D::new(3.91032e-17, -0.319302, 0.947653)).length() < 1e-6);
        assert_eq!(
            541,
            r.dir_to_bin(Vector3D::new(3.91032e-17, -0.319302, 0.947653))
        );
        assert!((r.bin_dir(542) - Vector3D::new(-0.0826413, -0.308422, 0.947653)).length() < 1e-6);
        assert_eq!(
            542,
            r.dir_to_bin(Vector3D::new(-0.0826413, -0.308422, 0.947653))
        );
        assert!((r.bin_dir(543) - Vector3D::new(-0.159651, -0.276523, 0.947653)).length() < 1e-6);
        assert_eq!(
            543,
            r.dir_to_bin(Vector3D::new(-0.159651, -0.276523, 0.947653))
        );
        assert!((r.bin_dir(544) - Vector3D::new(-0.22578, -0.22578, 0.947653)).length() < 1e-6);
        assert_eq!(
            544,
            r.dir_to_bin(Vector3D::new(-0.22578, -0.22578, 0.947653))
        );
        assert!((r.bin_dir(545) - Vector3D::new(-0.276523, -0.159651, 0.947653)).length() < 1e-6);
        assert_eq!(
            545,
            r.dir_to_bin(Vector3D::new(-0.276523, -0.159651, 0.947653))
        );
        assert!((r.bin_dir(546) - Vector3D::new(-0.308422, -0.0826413, 0.947653)).length() < 1e-6);
        assert_eq!(
            546,
            r.dir_to_bin(Vector3D::new(-0.308422, -0.0826413, 0.947653))
        );
        assert!(
            (r.bin_dir(547) - Vector3D::new(-0.319302, -5.86547e-17, 0.947653)).length() < 1e-6
        );
        assert_eq!(
            547,
            r.dir_to_bin(Vector3D::new(-0.319302, -5.86547e-17, 0.947653))
        );
        assert!((r.bin_dir(548) - Vector3D::new(-0.308422, 0.0826413, 0.947653)).length() < 1e-6);
        assert_eq!(
            548,
            r.dir_to_bin(Vector3D::new(-0.308422, 0.0826413, 0.947653))
        );
        assert!((r.bin_dir(549) - Vector3D::new(-0.276523, 0.159651, 0.947653)).length() < 1e-6);
        assert_eq!(
            549,
            r.dir_to_bin(Vector3D::new(-0.276523, 0.159651, 0.947653))
        );
        assert!((r.bin_dir(550) - Vector3D::new(-0.22578, 0.22578, 0.947653)).length() < 1e-6);
        assert_eq!(
            550,
            r.dir_to_bin(Vector3D::new(-0.22578, 0.22578, 0.947653))
        );
        assert!((r.bin_dir(551) - Vector3D::new(-0.159651, 0.276523, 0.947653)).length() < 1e-6);
        assert_eq!(
            551,
            r.dir_to_bin(Vector3D::new(-0.159651, 0.276523, 0.947653))
        );
        assert!((r.bin_dir(552) - Vector3D::new(-0.0826413, 0.308422, 0.947653)).length() < 1e-6);
        assert_eq!(
            552,
            r.dir_to_bin(Vector3D::new(-0.0826413, 0.308422, 0.947653))
        );
        assert!((r.bin_dir(553) - Vector3D::new(0., 0.21497, 0.976621)).length() < 1e-6);
        assert_eq!(553, r.dir_to_bin(Vector3D::new(0., 0.21497, 0.976621)));
        assert!((r.bin_dir(554) - Vector3D::new(0.107485, 0.18617, 0.976621)).length() < 1e-6);
        assert_eq!(
            554,
            r.dir_to_bin(Vector3D::new(0.107485, 0.18617, 0.976621))
        );
        assert!((r.bin_dir(555) - Vector3D::new(0.18617, 0.107485, 0.976621)).length() < 1e-6);
        assert_eq!(
            555,
            r.dir_to_bin(Vector3D::new(0.18617, 0.107485, 0.976621))
        );
        assert!((r.bin_dir(556) - Vector3D::new(0.21497, 1.31631e-17, 0.976621)).length() < 1e-6);
        assert_eq!(
            556,
            r.dir_to_bin(Vector3D::new(0.21497, 1.31631e-17, 0.976621))
        );
        assert!((r.bin_dir(557) - Vector3D::new(0.18617, -0.107485, 0.976621)).length() < 1e-6);
        assert_eq!(
            557,
            r.dir_to_bin(Vector3D::new(0.18617, -0.107485, 0.976621))
        );
        assert!((r.bin_dir(558) - Vector3D::new(0.107485, -0.18617, 0.976621)).length() < 1e-6);
        assert_eq!(
            558,
            r.dir_to_bin(Vector3D::new(0.107485, -0.18617, 0.976621))
        );
        assert!((r.bin_dir(559) - Vector3D::new(2.63263e-17, -0.21497, 0.976621)).length() < 1e-6);
        assert_eq!(
            559,
            r.dir_to_bin(Vector3D::new(2.63263e-17, -0.21497, 0.976621))
        );
        assert!((r.bin_dir(560) - Vector3D::new(-0.107485, -0.18617, 0.976621)).length() < 1e-6);
        assert_eq!(
            560,
            r.dir_to_bin(Vector3D::new(-0.107485, -0.18617, 0.976621))
        );
        assert!((r.bin_dir(561) - Vector3D::new(-0.18617, -0.107485, 0.976621)).length() < 1e-6);
        assert_eq!(
            561,
            r.dir_to_bin(Vector3D::new(-0.18617, -0.107485, 0.976621))
        );
        assert!((r.bin_dir(562) - Vector3D::new(-0.21497, -3.94894e-17, 0.976621)).length() < 1e-6);
        assert_eq!(
            562,
            r.dir_to_bin(Vector3D::new(-0.21497, -3.94894e-17, 0.976621))
        );
        assert!((r.bin_dir(563) - Vector3D::new(-0.18617, 0.107485, 0.976621)).length() < 1e-6);
        assert_eq!(
            563,
            r.dir_to_bin(Vector3D::new(-0.18617, 0.107485, 0.976621))
        );
        assert!((r.bin_dir(564) - Vector3D::new(-0.107485, 0.18617, 0.976621)).length() < 1e-6);
        assert_eq!(
            564,
            r.dir_to_bin(Vector3D::new(-0.107485, 0.18617, 0.976621))
        );
        assert!((r.bin_dir(565) - Vector3D::new(0., 0.108119, 0.994138)).length() < 1e-6);
        assert_eq!(565, r.dir_to_bin(Vector3D::new(0., 0.108119, 0.994138)));
        assert!((r.bin_dir(566) - Vector3D::new(0.0540595, 0.0936338, 0.994138)).length() < 1e-6);
        assert_eq!(
            566,
            r.dir_to_bin(Vector3D::new(0.0540595, 0.0936338, 0.994138))
        );
        assert!((r.bin_dir(567) - Vector3D::new(0.0936338, 0.0540595, 0.994138)).length() < 1e-6);
        assert_eq!(
            567,
            r.dir_to_bin(Vector3D::new(0.0936338, 0.0540595, 0.994138))
        );
        assert!((r.bin_dir(568) - Vector3D::new(0.108119, 6.62038e-18, 0.994138)).length() < 1e-6);
        assert_eq!(
            568,
            r.dir_to_bin(Vector3D::new(0.108119, 6.62038e-18, 0.994138))
        );
        assert!((r.bin_dir(569) - Vector3D::new(0.0936338, -0.0540595, 0.994138)).length() < 1e-6);
        assert_eq!(
            569,
            r.dir_to_bin(Vector3D::new(0.0936338, -0.0540595, 0.994138))
        );
        assert!((r.bin_dir(570) - Vector3D::new(0.0540595, -0.0936338, 0.994138)).length() < 1e-6);
        assert_eq!(
            570,
            r.dir_to_bin(Vector3D::new(0.0540595, -0.0936338, 0.994138))
        );
        assert!((r.bin_dir(571) - Vector3D::new(1.32408e-17, -0.108119, 0.994138)).length() < 1e-6);
        assert_eq!(
            571,
            r.dir_to_bin(Vector3D::new(1.32408e-17, -0.108119, 0.994138))
        );
        assert!((r.bin_dir(572) - Vector3D::new(-0.0540595, -0.0936338, 0.994138)).length() < 1e-6);
        assert_eq!(
            572,
            r.dir_to_bin(Vector3D::new(-0.0540595, -0.0936338, 0.994138))
        );
        assert!((r.bin_dir(573) - Vector3D::new(-0.0936338, -0.0540595, 0.994138)).length() < 1e-6);
        assert_eq!(
            573,
            r.dir_to_bin(Vector3D::new(-0.0936338, -0.0540595, 0.994138))
        );
        assert!(
            (r.bin_dir(574) - Vector3D::new(-0.108119, -1.98611e-17, 0.994138)).length() < 1e-6
        );
        assert_eq!(
            574,
            r.dir_to_bin(Vector3D::new(-0.108119, -1.98611e-17, 0.994138))
        );
        assert!((r.bin_dir(575) - Vector3D::new(-0.0936338, 0.0540595, 0.994138)).length() < 1e-6);
        assert_eq!(
            575,
            r.dir_to_bin(Vector3D::new(-0.0936338, 0.0540595, 0.994138))
        );
        assert!((r.bin_dir(576) - Vector3D::new(-0.0540595, 0.0936338, 0.994138)).length() < 1e-6);
        assert_eq!(
            576,
            r.dir_to_bin(Vector3D::new(-0.0540595, 0.0936338, 0.994138))
        );
        assert!((r.bin_dir(577) - Vector3D::new(0., 6.12323e-17, 1.)).length() < 1e-6);
        assert_eq!(577, r.dir_to_bin(Vector3D::new(0., 6.12323e-17, 1.)));
    }
}
