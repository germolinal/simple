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

/// This is the Mathemagically famous Fast Inverse Square Root algorithm
/// Comments come from Wikipedia... https://en.wikipedia.org/wiki/Fast_inverse_square_root
pub fn quick_inv_sqrt(number: Float) -> Float {
    let number = number as f32;
    let mut i: i32;
    let x2: f32;
    let mut y: f32;
    const THREE_HALFS: f32 = 1.5;

    x2 = number * 0.5;
    y = number;
    i = unsafe {
        // evil floating point bit level hacking
        std::mem::transmute::<f32, i32>(y)
    };
    i = 0x5f3759df - (i >> 1); // what the fuck?
    y = unsafe { std::mem::transmute::<i32, f32>(i) };
    y = y * (THREE_HALFS - (x2 * y * y)); // 1st iteration
    y = y * (THREE_HALFS - (x2 * y * y)); // 2nd iteration, this can be removed

    // return
    y as Float
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_quick_sqrt() {
        let mut max_error = (0.0, 0.0);
        let mut max_error_percent = (0.0, 0.0);
        for i in 1..500 {
            let x = i as Float;
            let exp = 1. / x.sqrt();
            let found = quick_inv_sqrt(x);
            let err = (exp - found).abs();
            if err > max_error.1 {
                max_error = (x, err)
            }
            let err_percent = err / x;
            assert!(err_percent * 100. < 0.001);
            if err_percent > max_error_percent.1 {
                max_error_percent = (x, err_percent)
            }
        }
        println!(
            "Max error for {} = {} | found = {}, should have been {}",
            max_error.0,
            max_error.1,
            quick_inv_sqrt(max_error.0),
            max_error.0.sqrt()
        );
        println!(
            "Max error percent for {} = {}% | found = {}, should have been {}",
            max_error_percent.0,
            max_error_percent.1 * 100.,
            quick_inv_sqrt(max_error_percent.0),
            max_error_percent.0.sqrt()
        );
    }
}
