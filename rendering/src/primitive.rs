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

use crate::rand::RandGen;
use crate::Float;
use geometry::{
    BBox3D, Cylinder3D, DistantSource3D, Point3D, Ray3D, Sphere3D, Triangle3D, Vector3D,
};

use crate::primitive_samplers::*;
use crate::samplers::uniform_sample_disc;
use geometry::intersection::IntersectionInfo;

#[derive(Clone, Debug)]
pub enum Primitive {
    Sphere(Sphere3D),
    Triangle(Triangle3D),
    Cylinder(Cylinder3D),
    Source(DistantSource3D),
}

impl std::default::Default for Primitive {
    fn default() -> Self {
        Self::Sphere(Sphere3D::new(1.0, Point3D::new(0., 0., 0.)))
    }
}

impl Primitive {
    /// The name of the `Primitive`. Useful for debugging.
    pub fn id(&self) -> &'static str {
        match self {
            Self::Sphere(s) => s.id(),
            Self::Triangle(s) => s.id(),
            Self::Cylinder(s) => s.id(),
            Self::Source(s) => s.id(),
        }
    }

    /// Gets a `BBox3D` bounding the primitive, in world's coordinates.
    pub fn world_bounds(&self) -> BBox3D {
        match self {
            Self::Sphere(s) => s.world_bounds(),
            Self::Triangle(s) => s.world_bounds(),
            Self::Cylinder(s) => s.world_bounds(),
            Self::Source(_) => panic!("Trying to get the bounds of a DistantSource3D"),
        }
    }

    /// Intersects an object with a [`Ray3D]` (IN WORLD COORDINATES) traveling forward, returning the distance
    /// `t` and the normal [`Vector3D`] at that point. If the distance
    /// is negative (i.e., the object is behind the plane), it should return
    /// [`None`]. Returns detailed [`IntersectionInfo`] about the intersaction .    
    pub fn intersect(&self, ray: &Ray3D) -> Option<IntersectionInfo> {
        match self {
            Self::Sphere(s) => s.intersect(ray),
            Self::Triangle(s) => s.intersect(ray),
            Self::Cylinder(s) => s.intersect(ray),
            Self::Source(s) => s.intersect(ray),
        }
    }

    /// Intersects an object with a [`Ray3D]` (IN WORLD COORDINATES) traveling forward, returning the distance
    /// `t` and the normal [`Vector3D`] at that point. If the distance
    /// is negative (i.e., the object is behind the plane), it should return
    /// [`None`]. Returns only the point of intersection.
    pub fn simple_intersect(&self, ray: &Ray3D) -> Option<Point3D> {
        match self {
            Self::Sphere(s) => s.simple_intersect(ray),
            Self::Triangle(s) => s.simple_intersect(ray),
            Self::Cylinder(s) => s.simple_intersect(ray),
            Self::Source(s) => s.simple_intersect(ray),
        }
    }

    pub fn solid_angle_pdf(&self, info: &IntersectionInfo, ray: &Ray3D) -> Float {
        match self {
            Self::Sphere(s) => sphere_solid_angle_pdf(s, info, ray),
            Self::Triangle(s) => triangle_solid_angle_pdf(s, info, ray),
            Self::Cylinder(_s) => unimplemented!(),
            Self::Source(s) => source_solid_angle_pdf(s, info, ray),
        }
    }

    pub fn direction(&self, point: Point3D) -> (Float, Vector3D) {
        match self {
            Self::Sphere(s) => sphere_direction(s, point),
            Self::Triangle(s) => triangle_direction(s, point),
            Self::Cylinder(_s) => unimplemented!(),
            Self::Source(s) => source_direction(s, point),
        }
    }

    pub fn sample_direction(&self, rng: &mut RandGen, point: Point3D) -> Vector3D {
        let surface_point = match self {
            Self::Sphere(s) => sample_sphere_surface(s, rng),
            Self::Triangle(s) => sample_triangle_surface(s, rng),
            Self::Cylinder(_s) => unimplemented!(),
            Self::Source(s) => {
                let radius = (s.angle / 2.0).tan();
                let normal = s.direction.get_normalized();
                uniform_sample_disc(rng, radius, point + normal, normal)
            }
        };
        let direction = surface_point - point;
        direction.get_normalized()
    }
}
