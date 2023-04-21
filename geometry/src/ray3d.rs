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

use crate::{Point3D, Vector3D};

use crate::Float;

/// A simple Ray structure, consiting of an origin (i.e., [`Point3D`])
/// and a direction (i.e., a [`Vector3D`])
#[derive(Debug, Copy, Clone)]
pub struct Ray3D {
    /// The origin
    pub origin: Point3D,
    /// The direction
    pub direction: Vector3D,
}

impl Ray3D {
    /// Returns the point that the [Ray3D] would be in after
    /// advancing `t`
    pub fn project(&self, t: Float) -> Point3D {
        self.origin + self.direction * t
    }

    /// Translates the [Ray3D] `t` units of its length
    /// into the future.
    pub fn advance(&mut self, t: Float) {
        self.origin += self.direction * t
    }
}
