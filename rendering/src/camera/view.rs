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
use geometry::{Point3D, Vector3D};

#[derive(Debug, Clone, Copy)]
pub struct View {
    /// The point of view
    pub view_point: Point3D,
    /// The view direction
    pub view_direction: Vector3D,
    /// The "up", according to the camera
    pub view_up: Vector3D,
    /// Horizontal angle of the Field of View (i.e., frustum) in degrees
    pub field_of_view: Float,
}

impl Default for View {
    fn default() -> Self {
        Self {
            view_point: Point3D::new(0., 0., 0.),
            view_direction: Vector3D::new(0., 1., 0.),
            view_up: Vector3D::new(0., 0., 1.),
            field_of_view: 60.,
        }
    }
}

impl View {
    /// Checks that the view direction, view up, and view right are
    /// unit vectors and orthogonal to each other, and that
    /// `view_right = view_direction.cross(view_up)`
    pub fn normalize(&mut self) {
        self.view_direction.normalize();
        self.view_up.normalize();
    }

    pub fn view_right(&mut self) -> Vector3D {
        let view_right = self.view_direction.cross(self.view_up);
        self.view_up = view_right.cross(self.view_direction);
        self.normalize();
        view_right.get_normalized()
    }
}
