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

/*
 PART OF THIS FILE contains code to write four byte rgbe file format
 developed by Greg Ward. It handles the conversions between rgbe and
 pixels consisting of floats.

 This code was translated into Rust... The original work is available
 at http://www.graphics.cornell.edu/~bjw/
 written by Bruce Walter  (bjw@graphics.cornell.edu)  5/26/95
 based on code written by Greg Ward
*/
use crate::colour::Spectrum;
use crate::colourmap::Colourmap;
use crate::Float;
use jpeg_encoder::{ColorType, Encoder};
use std::io::Write;
use std::path::Path;

/// Equivalent to C's `frexp` function
fn rusty_frexp(s: Float) -> (Float, i32) {
    if 0.0 == s {
        (s, 0)
    } else {
        let lg = s.abs().log2();
        let x = (lg - lg.floor() - 1.0).exp2();
        let exp = lg.floor() + 1.0;
        (s.signum() * x, exp as i32)
    }
}

/// Equivalent to C's `ldexp` function
fn rusty_ldexp(x: Float, n: i32) -> Float {
    x * (2. as Float).powi(n)
}

fn colour_to_rgbe(red: Float, green: Float, blue: Float) -> [u8; 4] {
    let mut v = red;
    if green > v {
        v = green;
    }
    if blue > v {
        v = blue;
    }
    if v < 1e-19 {
        [0, 0, 0, 0]
    } else {
        let (mut mantissa, e) = rusty_frexp(v);
        mantissa *= 256.0 / v;
        let r = (red * mantissa).floor() as u8;
        let g = (green * mantissa).floor() as u8;
        let b = (blue * mantissa).floor() as u8;

        debug_assert!(e + 128 >= 0);
        debug_assert!(e + 128 <= u8::MAX as i32);

        let e = (e + 128) as u8;

        [r, g, b, e]
    }
}

fn rgbe_to_colour(r: u8, g: u8, b: u8, e: u8) -> Spectrum {
    if e == 0 {
        return Spectrum::BLACK;
    }

    let n = e as i32 - (128 + 8) as i32;
    let f = rusty_ldexp(1., n);
    let red = r as Float * f;
    let green = g as Float * f;
    let blue = b as Float * f;

    Spectrum([red, green, blue])
}

/// A buffer with all the physical values in the image
/// (i.e., Radiance, Irradiance or whatever being calculated)
///
pub struct ImageBuffer {
    /// Number of columns
    pub width: usize,
    /// Number of rows
    pub height: usize,
    /// All the pixels, iterating from top
    /// to bottom, left to right
    pub pixels: Vec<Spectrum>,
}

impl std::ops::IndexMut<(usize, usize)> for ImageBuffer {
    fn index_mut(&mut self, pixel: (usize, usize)) -> &mut Self::Output {
        let (x, y) = pixel;
        let i = y * self.width + x;
        &mut self.pixels[i]
    }
}

impl std::ops::Index<(usize, usize)> for ImageBuffer {
    type Output = Spectrum;

    fn index(&self, pixel: (usize, usize)) -> &Self::Output {
        let (x, y) = pixel;
        let i = y * self.width + x;
        &self.pixels[i]
    }
}

impl ImageBuffer {
    /// Creates a new empty [`ImageBuffer`]
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![Spectrum::BLACK; width * height],
        }
    }

    /// Gets the absolute difference between two `ImageBuffer`
    pub fn diff(&self, other: &Self) -> Result<Self, String> {
        if (self.width, self.height) != (other.width, other.height) {
            return Err(format!(
                "Size mismatch: comparing image of size '{}x{}' with another one of size '{}x{}'",
                self.height, self.width, other.height, other.width
            ));
        }

        let pixels = self
            .pixels
            .iter()
            .zip(other.pixels.iter())
            .map(|(a, b)| Spectrum::gray((a.radiance() - b.radiance()).abs()))
            .collect();

        Ok(Self::from_pixels(self.width, self.height, pixels))
    }

    /// Creates a new empty [`ImageBuffer`]
    pub fn from_pixels(width: usize, height: usize, pixels: Vec<Spectrum>) -> Self {
        if pixels.len() != width * height {
            panic!("Width ({}) and Height ({}) does not match the number of pixels (n_pixels is {}... expecting width*height={})", width, height, pixels.len(), width*height)
        }
        Self {
            width,
            height,
            pixels,
        }
    }

    /// Saves the image in HDRE format
    pub fn save_hdre(&self, filename: &Path) {
        // Create the file
        let mut file = std::fs::File::create(filename).unwrap();
        // Write header
        // let gamma = 1.0;
        // let exposure = 1.0;
        file.write_all(b"#?RGBE\n").unwrap();
        // file.write_all(format!("GAMMA={}\n", gamma).as_bytes()).unwrap();
        // file.write_all(format!("EXPOSURE={}\n", exposure).as_bytes()).unwrap();
        file.write_all(b"FORMAT=32-bit_rle_rgbe\n\n").unwrap();
        file.write_all(format!("-Y {} +X {}\n", self.height, self.width).as_bytes())
            .unwrap();

        for pixel in self.pixels.iter() {
            file.write_all(&colour_to_rgbe(pixel.0[0], pixel.0[1], pixel.0[2]))
                .unwrap();
        }
    }

    /// Creates a new empty [`ImageBuffer`] from a File
    pub fn from_file(filename: &Path) -> Result<Self, String> {
        let content = match std::fs::read(filename) {
            Ok(v) => v,
            Err(_) => {
                return Err(format!(
                    "Could not read image file '{}'",
                    filename.to_str().unwrap()
                ))
            }
        };
        let mut content = content.as_slice();
        let filename = filename.to_str().unwrap();

        // READ HEADER
        let height: Option<usize>;
        let width: Option<usize>;
        loop {
            let nl = match &content.iter().position(|u| *u as char == '\n') {
                None => return Err("Apparentyly incorrectly formatted file".to_string()),
                Some(i) => *i,
            };

            let line = &content[0..nl];
            content = &content[nl + 1..];

            if line.starts_with(b"-Y") {
                let errmsg = {
                    let l = std::str::from_utf8(line).unwrap();
                    Err(format!("When reading file '{}' : Expecting SIZE line to be in the format '-Y number +X number'... found '{}'",filename, l))
                };

                // Size
                let tuple: Vec<&[u8]> = line
                    .split(|c| c.is_ascii_whitespace())
                    .into_iter()
                    .collect();
                if tuple.len() != 4 || tuple[2].ne(b"+X") {
                    return errmsg;
                }
                let l = std::str::from_utf8(tuple[1]).unwrap();
                height = match l.parse::<usize>() {
                    Ok(v) => Some(v),
                    Err(_) => {
                        return errmsg;
                    }
                };
                let l = std::str::from_utf8(tuple[3]).unwrap();
                width = match l.parse::<usize>() {
                    Ok(v) => Some(v),
                    Err(_) => {
                        return errmsg;
                    }
                };
                break; // Done with header
            }

            if line.starts_with(b"FORMAT") {
                // Format
                let tuple: Vec<&[u8]> = line.split(|c| *c == b'=').into_iter().collect();
                if tuple.len() != 2 {
                    let l = std::str::from_utf8(line).unwrap();
                    return Err(format!(
                        "Expecting FORMAT line to be in the format 'FORMAT=number'... found '{}'",
                        l
                    ));
                }
                let exp_format = b"32-bit_rle_rgbe";
                if tuple[1].ne(exp_format) {
                    let exp_format = std::str::from_utf8(exp_format).unwrap();
                    let found_format = std::str::from_utf8(tuple[1]).unwrap();
                    return Err(format!(
                        "Expecting FORMAT to be '{}'... found '{}'",
                        exp_format, found_format
                    ));
                };
                continue;
            }
        }
        // Setup
        let width = width.unwrap();
        let height = height.unwrap();

        // Read body
        let pixels = content
            .chunks_exact(4)
            .map(|x| {
                let (r, g, b, e) = (x[0], x[1], x[2], x[3]);
                rgbe_to_colour(r, g, b, e)
            })
            .collect();

        // return
        Ok(Self {
            width,
            height,
            pixels,
        })
    } // end of from_file()

    /// Creates a new version of an image, but in (log10) falsecolour
    pub fn save_log_falsecolour(
        &self,
        min: Option<Float>,
        max: Option<Float>,
        scale: Colourmap,
        outfile: &Path,
    ) {
        let log_luminance: Vec<Float> = self.pixels.iter().map(|x| x.luminance().log10()).collect();
        const MIN_MIN: Float = 0.001;
        // Get min and max
        let log_min = match min {
            Some(mut v) => {
                v = v.max(MIN_MIN);
                v.log10()
            }
            None => MIN_MIN.log10(),
        };
        let log_max = match max {
            Some(v) => v.log10(),
            None => {
                let mut m = MIN_MIN;
                log_luminance.iter().for_each(|v| {
                    if *v > m {
                        m = *v
                    }
                });
                if m <= MIN_MIN {
                    m = 2. * MIN_MIN;
                }
                m
            }
        };

        let scale = match scale {
            Colourmap::Inferno => crate::colourmap::inferno::INFERNO_COLOURMAP.as_slice(),
            Colourmap::Magma => crate::colourmap::magma::MAGMA_COLOURMAP.as_slice(),
            Colourmap::Plasma => crate::colourmap::plasma::PLASMA_COLOURMAP.as_slice(),
            Colourmap::Radiance => crate::colourmap::radiance::RADIANCE_COLOURMAP.as_slice(),
            Colourmap::Viridis => crate::colourmap::viridis::VIRIDIS_COLOURMAP.as_slice(),
        };

        let mut data: Vec<u8> = Vec::with_capacity(self.width * self.height * 3);
        log_luminance.iter().for_each(|x| {
            let s = crate::colourmap::map_linear_colour(*x, log_min, log_max, scale);
            data.push((s[0] * 256.).round() as u8);
            data.push((s[1] * 256.).round() as u8);
            data.push((s[2] * 256.).round() as u8);
        });

        let encoder = Encoder::new_file(outfile, 100).unwrap();
        encoder
            .encode(&data, self.width as u16, self.height as u16, ColorType::Rgb)
            .unwrap();
    }

    /// Creates a new version of an image, but in (linear) falsecolour
    pub fn save_falsecolour(
        &self,
        min: Option<Float>,
        max: Option<Float>,
        scale: Colourmap,
        outfile: &Path,
    ) {
        let luminance: Vec<Float> = self.pixels.iter().map(|x| x.luminance()).collect();

        const MIN_MIN: Float = 0.000;
        // Get min and max
        let min = match min {
            Some(mut v) => {
                v = v.max(MIN_MIN);
                v
            }
            None => MIN_MIN,
        };
        let max = match max {
            Some(v) => v,
            None => {
                /* // From RADIANCE's Falsecolor.pl
                 # Find a good scale for auto mode.
                if ($scale =~ m/[aA].* /) {
                    my @histo = split(/\s/, `phisto $picture`);
                    # e.g. $ phisto tests/richmond.hdr| tail -2
                    # 3.91267	14
                    # 3.94282	6
                    my $LogLmax = $histo[-4];
                    $scale = $mult / 179 * 10**$LogLmax;
                }
                */
                let mut m = MIN_MIN;
                luminance.iter().for_each(|v| {
                    if *v > m {
                        m = *v
                    }
                });
                if m <= MIN_MIN {
                    m = 2. * MIN_MIN;
                }
                m
            }
        };

        let scale = match scale {
            Colourmap::Inferno => crate::colourmap::inferno::INFERNO_COLOURMAP.as_slice(),
            Colourmap::Magma => crate::colourmap::magma::MAGMA_COLOURMAP.as_slice(),
            Colourmap::Plasma => crate::colourmap::plasma::PLASMA_COLOURMAP.as_slice(),
            Colourmap::Radiance => crate::colourmap::radiance::RADIANCE_COLOURMAP.as_slice(),
            Colourmap::Viridis => crate::colourmap::viridis::VIRIDIS_COLOURMAP.as_slice(),
        };

        let mut data: Vec<u8> = Vec::with_capacity(self.width * self.height * 3);
        luminance.iter().for_each(|x| {
            let s = crate::colourmap::map_linear_colour(*x, min, max, scale);
            data.push((s[0] * 256.).round() as u8);
            data.push((s[1] * 256.).round() as u8);
            data.push((s[2] * 256.).round() as u8);
        });

        let encoder = Encoder::new_file(outfile, 100).unwrap();
        encoder
            .encode(&data, self.width as u16, self.height as u16, ColorType::Rgb)
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PI;
    use std::os::raw::c_double;
    use std::os::raw::c_int;

    extern "C" {
        fn frexp(x: c_double, exp: *mut c_int) -> c_double;
        fn ldexp(x: c_double, ex: c_int) -> c_double;
    }

    fn c_frexp(x: Float) -> (Float, i32) {
        let mut exp: c_int = 0;
        let res = unsafe { frexp(x.into(), &mut exp) } as Float;
        (res, exp as i32)
    }

    fn c_ldexp(x: Float, n: i32) -> Float {
        (unsafe { ldexp(x.into(), n) }) as Float
    }

    #[test]
    fn test_frexp() {
        let xs: Vec<Float> = vec![1e6, 2., PI.into(), 123987., 0., 99., 2.3123, 1024., 0.1];
        for x in xs.iter() {
            let (c_mantissa, c_exp) = c_frexp(*x);
            let (mantissa, exp) = rusty_frexp(*x);
            println!(
                "... x={} | c_mant: {}, c_exp: {}; mant: {}, exp: {}",
                x, c_mantissa, c_exp, mantissa, exp
            );
            assert_eq!(exp, c_exp);
            assert!((mantissa - c_mantissa).abs() < 0.001);
        }
    }

    #[test]
    fn test_ldexp() {
        let is: Vec<i32> = vec![1, 2, 3, 4, 5, 6, -1, -2, -3, -4];
        let xs: Vec<Float> = vec![1e6, 2., PI, 123987., 0., 99., 2.3123, 1024., 0.1];
        for x in xs.iter() {
            for i in is.iter() {
                let c = c_ldexp(*x, *i);
                let r = rusty_ldexp(*x, *i);
                println!("{}*2^{} = {} in C and {} in Rust", x, i, c, r);
                assert!(
                    (c - r).abs() < 1e-5,
                    "c = {}, r = {} | diff is {}",
                    c,
                    r,
                    (c - r).abs()
                );
            }
        }
    }

    #[test]
    fn test_colour_to_rgbe() {
        // Produced automatically
        assert_eq!(colour_to_rgbe(807., 249., 73.), [201, 62, 18, 138]);
        assert_eq!(
            colour_to_rgbe(984943658., 1144108930., 470211272.),
            [117, 136, 56, 159]
        );
        assert_eq!(
            colour_to_rgbe(101027544., 1457850878., 1458777923.),
            [12, 173, 173, 159]
        );
        assert_eq!(
            colour_to_rgbe(2007237709., 823564440., 1115438165.),
            [239, 98, 132, 159]
        );
        assert_eq!(
            colour_to_rgbe(1784484492., 74243042., 114807987.),
            [212, 8, 13, 159]
        );
        assert_eq!(
            colour_to_rgbe(1137522503., 1441282327., 16531729.),
            [135, 171, 1, 159]
        );
        assert_eq!(
            colour_to_rgbe(823378840., 143542612., 896544303.),
            [196, 34, 213, 158]
        );
        assert_eq!(
            colour_to_rgbe(1474833169., 1264817709., 1998097157.),
            [175, 150, 238, 159]
        );
        assert_eq!(
            colour_to_rgbe(1817129560., 1131570933., 197493099.),
            [216, 134, 23, 159]
        );
        assert_eq!(
            colour_to_rgbe(1404280278., 893351816., 1505795335.),
            [167, 106, 179, 159]
        );
    }

    #[test]
    fn test_rgbe_to_colour() {
        let check = |a: Spectrum, b: Spectrum| -> Result<(), String> {
            let a_r = a.radiance();
            let b_r = b.radiance();
            let percent_error = (b_r - a_r).abs() / b_r;
            if percent_error > 0.015 {
                return Err(format!("Radiance Error {} is to high ", percent_error));
            }
            let ared_ratio = a.0[0] / a.0[2];
            let bred_ratio = b.0[0] / b.0[2];
            let percent_error = (bred_ratio - ared_ratio).abs() / bred_ratio;
            if percent_error > 0.05 {
                return Err(format!("Red Ratio Error {} is to high ", percent_error));
            }

            let agreen_ratio = a.0[0] / a.0[2];
            let bgreen_ratio = b.0[0] / b.0[2];
            let percent_error = (agreen_ratio - bgreen_ratio).abs() / bgreen_ratio;
            if percent_error > 0.05 {
                return Err(format!("Green Ratio Error {} is to high ", percent_error));
            }

            Ok(())
        };

        // Produced automatically
        check(
            rgbe_to_colour(201, 62, 18, 138),
            Spectrum([807., 249., 73.]),
        )
        .unwrap();
        check(
            rgbe_to_colour(117, 136, 56, 159),
            Spectrum([984943658., 1144108930., 470211272.]),
        )
        .unwrap();
        check(
            rgbe_to_colour(12, 173, 173, 159),
            Spectrum([101027544., 1457850878., 1458777923.]),
        )
        .unwrap();
        check(
            rgbe_to_colour(239, 98, 132, 159),
            Spectrum([2007237709., 823564440., 1115438165.]),
        )
        .unwrap();
        check(
            rgbe_to_colour(212, 8, 13, 159),
            Spectrum([1784484492., 74243042., 114807987.]),
        )
        .unwrap();

        check(
            rgbe_to_colour(196, 34, 213, 158),
            Spectrum([823378840., 143542612., 896544303.]),
        )
        .unwrap();
        check(
            rgbe_to_colour(175, 150, 238, 159),
            Spectrum([1474833169., 1264817709., 1998097157.]),
        )
        .unwrap();
        check(
            rgbe_to_colour(216, 134, 23, 159),
            Spectrum([1817129560., 1131570933., 197493099.]),
        )
        .unwrap();
        check(
            rgbe_to_colour(167, 106, 179, 159),
            Spectrum([1404280278., 893351816., 1505795335.]),
        )
        .unwrap();
    }

    // #[test]
    // #[ignore]
    // fn test_from_file() {
    //     let buffer =
    //         ImageBuffer::from_file(Path::new("./tests/scenes/images/rad_cornell.hdr")).unwrap();
    //     assert_eq!(buffer.width, 512);
    //     assert_eq!(buffer.height, 367);
    //     // assert_eq!(buffer.pixels.len(), 1024*768);
    //     buffer.save_hdre(Path::new("./tests/scenes/images/cornell_COPIED.hdr"))
    // }

    // #[test]
    // #[ignore]
    // fn test_falsecolor() {
    //     let buffer =
    //         ImageBuffer::from_file(Path::new("./tests/scenes/images/rad_cornell.hdr")).unwrap();
    //     // assert_eq!(buffer.width, 512);
    //     // assert_eq!(buffer.height, 367);
    //     // assert_eq!(buffer.pixels.len(), 1024*768);
    //     buffer.save_falsecolour(
    //         None,
    //         Some(100.),
    //         Colourmap::Radiance,
    //         Path::new("./tests/scenes/images/cornell_fc.jpeg"),
    //     );
    //     // buffer.save_hdre(Path::new("./tests/scenes/images/cornell_fc.hdr"))
    // }
}
