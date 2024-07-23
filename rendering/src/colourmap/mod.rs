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

pub mod inferno;
pub mod magma;
pub mod plasma;
pub mod radiance;
pub mod viridis;

use crate::Float;

/// The options of Colourmap to choose from when falsecolouring an image.
pub enum Colourmap {
    /// Emulates the default Radiance's Colourmap.
    Radiance,
    /// One of the colourmaps that is "analytically be perfectly
    /// perceptually-uniform, both in regular form and also when
    /// converted to black-and-white" described in [here](https://bids.github.io/colormap/)
    Inferno,
    /// One of the colourmaps that is "analytically be perfectly
    /// perceptually-uniform, both in regular form and also when
    /// converted to black-and-white" described in [here](https://bids.github.io/colormap/)
    Magma,
    /// One of the colourmaps that is "analytically be perfectly
    /// perceptually-uniform, both in regular form and also when
    /// converted to black-and-white" described in [here](https://bids.github.io/colormap/)
    Plasma,
    /// The deafult Colourmap in this library.
    /// Also, one of the colourmaps that is "analytically be perfectly
    /// perceptually-uniform, both in regular form and also when
    /// converted to black-and-white" described in [here](https://bids.github.io/colormap/).
    Viridis,
}

impl Colourmap {
    fn as_slice(&self) -> &[[Float; 3]] {
        match self {
            Colourmap::Inferno => crate::colourmap::inferno::INFERNO_COLOURMAP.as_slice(),
            Colourmap::Magma => crate::colourmap::magma::MAGMA_COLOURMAP.as_slice(),
            Colourmap::Plasma => crate::colourmap::plasma::PLASMA_COLOURMAP.as_slice(),
            Colourmap::Radiance => crate::colourmap::radiance::RADIANCE_COLOURMAP.as_slice(),
            Colourmap::Viridis => crate::colourmap::viridis::VIRIDIS_COLOURMAP.as_slice(),
        }
    }

    /// Maps a color, linearly
    pub fn map_linear(&self, x: Float, min: Float, max: Float) -> [Float; 3] {
        let slice = self.as_slice();
        map_linear_colour(x, min, max, slice)
    }
}

/// Maps a linear RGB colour
fn map_linear_colour(
    x: Float,
    min: Float,
    max: Float,
    map: &[[crate::Float; 3]],
) -> [crate::Float; 3] {
    if x <= min {
        return map[0];
    } else if x >= max {
        return *map.last().expect("Given an empty colour map");
    }

    let delta = (max - min) / (map.len() - 1) as Float;
    for i in 1..map.len() {
        let bin_start = min + (i - 1) as Float * delta;
        let bin_end = bin_start + delta;
        if x <= bin_end {
            let lam = (x - bin_start) / delta;

            let r = map[i - 1][0] + (map[i][0] - map[i - 1][0]) * lam;
            let g = map[i - 1][1] + (map[i][1] - map[i - 1][1]) * lam;
            let b = map[i - 1][2] + (map[i][2] - map[i - 1][2]) * lam;
            return [r, g, b];
        }
    }
    unreachable!()
}

#[cfg(test)]
mod tests {
    use super::Colourmap;

    #[test]
    fn test_map_linear() {
        let min = 0.01;
        let max = 8.;
        let x = 5.;

        let map = Colourmap::Viridis;
        let slice = map.as_slice();

        let y = map.map_linear(x, min, max);
        println!("{:?}", y);

        let y = map.map_linear(min - 0.01, min, max);
        println!("{:?} | {:?}", y, slice[0]);
        let y = map.map_linear(max + 0.01, min, max);
        println!("{:?} | {:?}", y, slice[slice.len() - 1]);
    }
}
