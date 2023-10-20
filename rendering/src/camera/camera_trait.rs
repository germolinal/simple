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

use crate::camera::{CameraSample, View};
use crate::rand::*;
use crate::ray::Ray;
use crate::Float;
use geometry::Ray3D;

pub trait Camera: Sync {
    fn pixel_from_ray(&self, ray: &Ray3D) -> ((usize, usize), Float);

    /// Generates a ray that will go through the View Point and a
    /// certain `CameraSample`
    fn gen_ray(&self, sample: &CameraSample) -> (Ray, Float);

    /// Generates a random CameraSample
    fn gen_random_sample(&self, rng: &mut RandGen) -> CameraSample;

    /// Gets the film resolution (width,height) in pixels
    fn film_resolution(&self) -> (usize, usize);

    /// Borrows the view
    fn view(&self) -> &View;

    fn pixel_index(&self, pxl: (usize, usize)) -> usize {
        let (x, y) = pxl;
        let (width, _height) = self.film_resolution();
        width * y + x
    }
}
