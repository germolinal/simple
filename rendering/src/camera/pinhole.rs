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

use crate::camera::{Camera, CameraSample, Film, View};
use crate::rand::*;
use crate::ray::Ray;
use crate::Float;
use geometry::{Ray3D, Vector3D};

#[derive(Debug, Clone)]
pub struct Pinhole {
    view: View,
    film: Film,
    film_distance: Float,

    /// A [`Vector3D`] which is the result of view_direction.cross(view_up)
    u: Vector3D,
}

impl Pinhole {
    pub fn new(view: View, film: Film) -> Self {
        let film_distance = 1. / (view.field_of_view.to_radians() / 2.0).tan();
        let u = view.view_direction.cross(view.view_up);
        Pinhole {
            view,
            film,
            film_distance,
            u,
        }
    }
}

impl Camera for Pinhole {
    /// Generates a random CameraSample
    fn gen_random_sample(&self, rng: &mut RandGen) -> CameraSample {
        let (width, height) = self.film.resolution;
        let (x, y): (usize, usize) = rng.gen();
        CameraSample {
            p_film: (x % width, y % height),
        }
    }

    fn pixel_from_ray(&self, ray: &Ray3D) -> ((usize, usize), Float) {
        if (ray.origin - self.view.view_point).length_squared() > 1e-24 {
            panic!("Trying to get a pixel of a camera through a ray that does not start at its view point... ViewPoint = {}, ray.origin = {} | distance = {}", self.view.view_point, ray.origin, (self.view.view_point-ray.origin).length());
        }

        // Let's do a Ray/Plane intersection... the plane is Normal to the camera's view directio
        let direction = ray.direction;
        let normal = self.view.view_direction;
        let cos = normal * direction;
        if cos.abs() < 1e-12 {
            // The ray does not intersect at all.... its weight is Zero
            return ((0, 0), 0.);
        }

        // Calculate a point in the plane
        let s = self.view.view_point + normal * self.film_distance;
        let ray_length = (s - ray.origin) * normal / cos;
        // Let's get the intersection point centered at the origin
        let intersection_pt = direction * ray_length;
        // Transform that point to be aligned with u and up and normal
        let x = intersection_pt * self.u;
        #[cfg(debug_assertions)]
        {
            let y = intersection_pt * normal;
            assert!(
                (y - self.film_distance).abs() < 1e-12,
                "y = {} | film distance = {}",
                y,
                self.film_distance
            );
        }
        let z = intersection_pt * self.view.view_up;

        // If it is out of the FOV, return None.
        if z.abs() > 1. || x.abs() > 1. {
            return ((0, 0), 0.);
        }

        let (width, height) = self.film.resolution;
        let xlim = 2.;
        let aspect_ratio = height as Float / width as Float;
        let ylim = aspect_ratio * xlim;
        let dx = xlim / width as Float;
        let dy = ylim / height as Float;

        // Else, calculate:
        let x = (x + 1.) / dx;
        let y = (1. - z) / dy;

        // return
        ((x.floor() as usize, y.floor() as usize), 1.)
    }

    /// Generates a ray that will go through the View Point and a
    /// certain `CameraSample`
    fn gen_ray(&self, sample: &CameraSample) -> (Ray, Float) {
        let (width, height) = self.film.resolution;
        let aspect_ratio = height as Float / width as Float;
        let xlim = 2.;
        let ylim = aspect_ratio * xlim;

        let (x_pixel, y_pixel) = sample.p_film;
        let dx = xlim / width as Float;
        let dy = ylim / height as Float;

        let x = dx / 2. + x_pixel as Float * dx - xlim / 2.;
        let y = dy / 2. + y_pixel as Float * dy - ylim / 2.;

        let direction =
            self.view.view_direction * self.film_distance + self.u * x - self.view.view_up * y;

        let ray = Ray {
            geometry: Ray3D {
                direction: direction.get_normalized(),
                origin: self.view.view_point,
            },
            ..Ray::default()
        };

        // return
        (ray, 1.)
    }

    fn film_resolution(&self) -> (usize, usize) {
        self.film.resolution
    }

    fn view(&self) -> &View {
        &self.view
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geometry::Point3D;

    #[test]
    fn test_ray_pixel() {
        // Create film
        let film = Film {
            resolution: (512, 512),
        };

        // Create view
        let view = View {
            view_direction: Vector3D::new(0., 1., 0.).get_normalized(),
            view_point: Point3D::new(2., 1., 1.),
            ..View::default()
        };
        // Create camera
        let camera = Pinhole::new(view, film);

        let sample = CameraSample { p_film: (10, 20) };
        // Let's assume this is right
        let (ray, _weight) = camera.gen_ray(&sample);
        let (found_pixel, _weight) = camera.pixel_from_ray(&ray.geometry);
        assert_eq!(sample.p_film, found_pixel);
    }
}
