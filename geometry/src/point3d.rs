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

use serde::Deserialize;
use serde::Serialize;

use crate::Float;

use crate::Vector3D;

/// A very simple implementation of a 3D-Point
#[derive(Default, Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Point3D {
    /// The X component
    pub x: Float,
    /// The Y component
    pub y: Float,
    /// The Z component
    pub z: Float,
}

impl std::fmt::Display for Point3D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Point3D({:.5},{:.5},{:.5})", self.x, self.y, self.z)
    }
}

impl std::convert::From<Vector3D> for Point3D {
    fn from(v: Vector3D) -> Self {
        Point3D::new(v.x, v.y, v.z)
    }
}

impl Point3D {
    /// Creates a new [`Point3D`]
    /// ```
    /// # use geometry::Point3D;
    ///
    /// let pt = Point3D::new(0., 1., 2.);
    /// ```
    pub fn new(x: Float, y: Float, z: Float) -> Point3D {
        Point3D { x, y, z }
    }

    /// Checks whether the [`Point3D`] is located at the origin
    /// ```
    /// # use geometry::Point3D;
    ///
    /// let pt = Point3D::new(0., 0., 0.);
    /// assert!(pt.is_zero());
    /// ```
    pub fn is_zero(&self) -> bool {
        const TINY: Float = 100. * Float::EPSILON;
        self.x.abs() < TINY && self.y.abs() < TINY && self.z.abs() < TINY
    }

    /// Calculates the square of the distance between two [`Point3D`]
    /// This is faster than calculating the `distance()`
    /// ```
    /// # use geometry::Point3D;
    ///
    /// let a = Point3D::new(0., 0., 0.);
    /// let b = Point3D::new(2., 0., 0.);
    /// assert_eq!(a.squared_distance(b), 4.);
    /// ```
    pub fn squared_distance(&self, point: Point3D) -> Float {
        let dx = (self.x - point.x) * (self.x - point.x);
        let dy = (self.y - point.y) * (self.y - point.y);
        let dz = (self.z - point.z) * (self.z - point.z);
        dx + dy + dz
    }

    /// Calculates the distance between two [`Point3D`]
    /// ```
    /// # use geometry::Point3D;
    ///
    /// let a = Point3D::new(0., 0., 0.);
    /// let b = Point3D::new(2., 0., 0.);
    /// assert_eq!(a.distance(b), 2.);
    /// ```
    pub fn distance(&self, point: Point3D) -> Float {
        let d2 = self.squared_distance(point);
        d2.sqrt()
    }

    /// Checks if two [`Point3D`] are sifnificantly close,
    /// as defined by the given `eps` distance.
    /// ```
    /// # use geometry::Point3D;
    ///
    /// let a = Point3D::new(0., 0., 0.);
    /// let b = Point3D::new(0.1, 0., 0.);
    /// assert!(!a.compare_by(b, 0.0));
    /// assert!(a.compare_by(a, 0.0));
    /// assert!(a.compare_by(b, 0.12));
    /// ```
    pub fn compare_by(&self, p: Point3D, eps: Float) -> bool {
        (*self - p).length_squared() <= eps * eps
    }

    /// Checks if two [`Point3D`] are sifnificantly close,
    /// using a maximum distance of 1e-3
    /// ```
    /// # use geometry::Point3D;
    ///
    /// let a = Point3D::new(0., 0., 0.);
    /// let b = Point3D::new(0.1, 0., 0.);
    /// assert!(!a.compare(b));
    /// assert!(a.compare(a));
    /// ```
    pub fn compare(&self, p: Point3D) -> bool {
        self.compare_by(p, 1e-3)
    }

    /// Checks if a certain [`Point3D`] is collinear with two
    /// other [`Point3D`]. Will return an error if the two other
    /// points are the same
    pub fn is_collinear(self, b: Point3D, c: Point3D) -> Result<bool, String> {
        // check that they are not ALL the same
        if self.compare(b) && self.compare(c) {
            let msg = "Trying to test collinearity with three equal Point3D".to_string();
            return Err(msg);
        }

        // Check if two of them are the same
        if self.compare(b) || self.compare(c) || b.compare(c) {
            return Ok(true);
        }

        let ab = b - self;
        let bc = c - b;
        let cross = ab.cross(bc).length();

        Ok(cross < 1e-5)
    }

    /// Calculates the ditance of from a [`Point3D`] to a plane, defined based on
    /// a [`Point3D`] `p` and a `normal` [`Vector3D`].
    ///
    /// The distance can be negative, if the normal is not pointing into `self`
    /// direction.
    pub fn distance_to_plane(self, p: Self, normal: Vector3D) -> Float {
        let aux = self - p;

        aux * normal
    }

    /// Calculates the squared distance from `self` to the 3D line between `a` and `b`
    pub fn squared_distance_to_line(self, a: Self, b: Self) -> Float {
        assert!(
            !a.compare_by(b, 1e-5),
            "Line cannot be defined by a single point (i.e., a == b) "
        );
        let dir = (b - a).get_normalized();
        let a_self = self - a;

        // prpjection of a_self into dir
        let proj = dir * (a_self * dir);
        // a_self = proj + perp ---> perp = a_self - proj
        let perp = a_self - proj;
        perp.length_squared()
    }

    /// Calculates the distance from `self` to the 3D line between `a` and `b`
    pub fn distance_to_line(self, a: Self, b: Self) -> Float {
        self.squared_distance_to_line(a, b).sqrt()
    }

    /// Projects a point into a plane, defined by an `anchor` [`Point3D`] and a `normal` [`Vector3D`]
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::{Vector3D, Point3D};
    ///
    /// let origin = Point3D::new(0., 0., 0.);
    /// let up = Vector3D::new(0., 0., 1.);
    /// let right = Vector3D::new(1., 0., 0.);
    ///
    /// let p = Point3D::new(0., 1., 1.);    
    /// let proj = p.project_into_plane(origin, up);
    /// assert!(proj.compare(Point3D::new(0., 1., 0.)));
    ///
    /// let p = Point3D::new(0., 1., -1.);    
    /// let proj = p.project_into_plane(origin, up);
    /// assert!(proj.compare(Point3D::new(0., 1., 0.)));
    ///
    /// let p = Point3D::new(2., 1., 1.);    
    /// let proj = p.project_into_plane(origin, right);
    /// assert!(proj.compare(Point3D::new(0., 1., 1.)));
    ///
    /// let p = Point3D::new(-2., 1., -1.);    
    /// let proj = p.project_into_plane(origin, right);
    /// assert!(proj.compare(Point3D::new(0., 1., -1.)));
    ///
    /// ```
    pub fn project_into_plane(self, anchor: Self, normal: Vector3D) -> Self {
        let distance = self.distance_to_plane(anchor, normal);
        self - normal * distance
    }
}

impl std::ops::Add<Vector3D> for Point3D {
    type Output = Point3D;

    fn add(self, other: Vector3D) -> Point3D {
        Point3D {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}
impl std::ops::Add for Point3D {
    type Output = Point3D;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl std::ops::AddAssign for Point3D {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

impl std::ops::AddAssign<Vector3D> for Point3D {
    fn add_assign(&mut self, other: Vector3D) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

impl std::ops::SubAssign<Vector3D> for Point3D {
    fn sub_assign(&mut self, other: Vector3D) {
        self.x -= other.x;
        self.y -= other.y;
        self.z -= other.z;
    }
}

impl std::ops::Sub<Point3D> for Point3D {
    type Output = Vector3D;

    fn sub(self, other: Point3D) -> Vector3D {
        Vector3D::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl std::ops::Sub<Vector3D> for Point3D {
    type Output = Point3D;

    fn sub(self, other: Vector3D) -> Point3D {
        Point3D {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl std::ops::Mul<Vector3D> for Point3D {
    type Output = Float;

    fn mul(self, other: Vector3D) -> Float {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
}

impl std::ops::Mul<Point3D> for Point3D {
    type Output = Float;

    fn mul(self, other: Point3D) -> Float {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
}

impl std::ops::Mul<Float> for Point3D {
    type Output = Self;

    fn mul(self, other: Float) -> Self {
        Self {
            x: self.x * other,
            y: self.y * other,
            z: self.z * other,
        }
    }
}

impl std::ops::MulAssign<Float> for Point3D {
    fn mul_assign(&mut self, other: Float) {
        self.x *= other;
        self.y *= other;
        self.z *= other;
    }
}

impl std::ops::Div<Float> for Point3D {
    type Output = Self;

    fn div(self, other: Float) -> Self {
        Self {
            x: self.x / other,
            y: self.y / other,
            z: self.z / other,
        }
    }
}

impl std::ops::DivAssign<Float> for Point3D {
    fn div_assign(&mut self, other: Float) {
        self.x /= other;
        self.y /= other;
        self.z /= other;
    }
}

/*********/
/* TESTS */
/*********/

#[cfg(test)]
mod testing {
    use super::*;

    #[test]
    fn test_new() {
        let x = 1.0;
        let y = 2.0;
        let z = 3.0;

        let pt = Point3D::new(x, y, z);
        assert_eq!(x, pt.x);
        assert_eq!(y, pt.y);
        assert_eq!(z, pt.z);

        assert_eq!(x, pt.x);
        assert_eq!(y, pt.y);
        assert_eq!(z, pt.z);
    }

    #[test]
    fn test_squared_distance() {
        // Difference in Z
        let a = Point3D::new(0.0, 0.0, 0.0);
        let b = Point3D::new(0.0, 0.0, 2.0);
        assert_eq!(4.0, a.squared_distance(b));

        let a = Point3D::new(0.0, 0.0, 2.0);
        let b = Point3D::new(0.0, 0.0, 0.0);
        assert_eq!(4.0, a.squared_distance(b));

        // Difference in Y
        let a = Point3D::new(0.0, 2.0, 0.0);
        let b = Point3D::new(0.0, 0.0, 0.0);
        assert_eq!(4.0, a.squared_distance(b));

        let a = Point3D::new(0.0, 0.0, 0.0);
        let b = Point3D::new(0.0, 2.0, 0.0);
        assert_eq!(4.0, a.squared_distance(b));

        // Difference in X
        let a = Point3D::new(0.0, 0.0, 0.0);
        let b = Point3D::new(2.0, 0.0, 0.0);
        assert_eq!(4.0, a.squared_distance(b));

        let a = Point3D::new(2.0, 0.0, 0.0);
        let b = Point3D::new(0.0, 0.0, 0.0);
        assert_eq!(4.0, a.squared_distance(b));

        // Difference in X and Z
        let a = Point3D::new(0.0, 0.0, 0.0);
        let b = Point3D::new(3.0, 0.0, 4.0);
        assert_eq!(25.0, a.squared_distance(b));

        let a = Point3D::new(3.0, 0.0, 4.0);
        let b = Point3D::new(0.0, 0.0, 0.0);
        assert_eq!(25.0, a.squared_distance(b));
    }

    #[test]
    fn test_mul() {
        let x = 23.;
        let y = 59.;
        let z = -0.23;

        let pt = Point3D::new(x, y, z);
        let other_pt = Point3D::new(z, x, y);
        let other_vec = Vector3D::new(z, x, y);

        assert_eq!(pt * other_pt, x * z + y * x + z * y);
        assert_eq!(pt * other_vec, x * z + y * x + z * y);
    }

    #[test]
    fn test_distance() {
        // Difference in Z
        let a = Point3D::new(0.0, 0.0, 0.0);
        let b = Point3D::new(0.0, 0.0, 2.0);
        assert_eq!(2.0, a.distance(b));

        let a = Point3D::new(0.0, 0.0, 2.0);
        let b = Point3D::new(0.0, 0.0, 0.0);
        assert_eq!(2.0, a.distance(b));

        // Difference in Y
        let a = Point3D::new(0.0, 2.0, 0.0);
        let b = Point3D::new(0.0, 0.0, 0.0);
        assert_eq!(2.0, a.distance(b));

        let a = Point3D::new(0.0, 0.0, 0.0);
        let b = Point3D::new(0.0, 2.0, 0.0);
        assert_eq!(2.0, a.distance(b));

        // Difference in X
        let a = Point3D::new(0.0, 0.0, 0.0);
        let b = Point3D::new(2.0, 0.0, 0.0);
        assert_eq!(2.0, a.distance(b));

        let a = Point3D::new(2.0, 0.0, 0.0);
        let b = Point3D::new(0.0, 0.0, 0.0);
        assert_eq!(2.0, a.distance(b));

        // Difference in X and Z
        let a = Point3D::new(0.0, 0.0, 0.0);
        let b = Point3D::new(3.0, 0.0, 4.0);
        assert_eq!(5.0, a.distance(b));

        let a = Point3D::new(3.0, 0.0, 4.0);
        let b = Point3D::new(0.0, 0.0, 0.0);
        assert_eq!(5.0, a.distance(b));
    }

    #[test]
    fn test_add() {
        let x = 1.2;
        let y = 5.22;
        let z = 9.123;
        let ini = Point3D::new(0.0, 0.0, 0.0);
        let v = Vector3D::new(x, y, z);
        let fin = ini + v;
        assert_eq!(x, fin.x);
        assert_eq!(y, fin.y);
        assert_eq!(z, fin.z);

        let fin = fin + v;
        assert_eq!(2.0 * x, fin.x);
        assert_eq!(2.0 * y, fin.y);
        assert_eq!(2.0 * z, fin.z);

        // Go backwards now
        let v = Vector3D::new(-x, -y, -z);
        let fin = fin + v;
        assert_eq!(x, fin.x);
        assert_eq!(y, fin.y);
        assert_eq!(z, fin.z);

        let fin = fin + v;
        assert_eq!(0.0, fin.x);
        assert_eq!(0.0, fin.y);
        assert_eq!(0.0, fin.z);
    }

    #[test]
    fn test_sub_vec() {
        let x = 1.2;
        let y = 5.22;
        let z = 9.123;
        let ini = Point3D::new(2.0 * x, 2.0 * y, 2.0 * z);
        let v = Vector3D::new(x, y, z);
        let fin = ini - v;

        assert_eq!(x, fin.x);
        assert_eq!(y, fin.y);
        assert_eq!(z, fin.z);

        let fin = fin - v;
        assert_eq!(0.0 * x, fin.x);
        assert_eq!(0.0 * y, fin.y);
        assert_eq!(0.0 * z, fin.z);

        let v = Vector3D::new(-x, -y, -z);
        let fin = fin - v;
        assert_eq!(x, fin.x);
        assert_eq!(y, fin.y);
        assert_eq!(z, fin.z);

        let fin = fin - v;
        assert_eq!(2.0 * x, fin.x);
        assert_eq!(2.0 * y, fin.y);
        assert_eq!(2.0 * z, fin.z);
    }

    #[test]
    fn test_sub_point() {
        let x = 1.2;
        let y = 5.22;
        let z = 9.123;
        let ini = Point3D::new(x, y, z);
        let fin = Point3D::new(2.0 * x, 2.0 * y, 2.0 * z);
        let delta = fin - ini;
        assert_eq!(delta.x, x);
        assert_eq!(delta.y, y);
        assert_eq!(delta.z, z);

        let ini = Point3D::new(x, y, z);
        let fin = Point3D::new(0.0 * x, 0.0 * y, 0.0 * z);
        let delta = fin - ini;
        assert_eq!(delta.x, -x);
        assert_eq!(delta.y, -y);
        assert_eq!(delta.z, -z);

        let ini = Point3D::new(0.0, 0.0, 0.0);
        let fin = Point3D::new(2.0 * x, 2.0 * y, 2.0 * z);
        let delta = fin - ini;
        assert_eq!(delta.x, 2.0 * x);
        assert_eq!(delta.y, 2.0 * y);
        assert_eq!(delta.z, 2.0 * z);
    }

    #[test]
    fn test_as_vector3d() {
        let x = 123.1;
        let y = 543.1;
        let z = 9123.2;

        let p = Point3D::new(x, y, z);
        let v: Vector3D = p.into();

        assert_eq!(x, v.x);
        assert_eq!(y, v.y);
        assert_eq!(z, v.z);
    }

    #[test]
    fn test_compare() {
        let x = 123.1;
        let y = 543.1;
        let z = 9123.2;
        let d = 0.1;

        let p = Point3D::new(x, y, z);
        let p2 = Point3D::new(x, y, z);
        assert!(p.compare(p2));

        let p2 = Point3D::new(x + d, y, z);
        assert!(!p.compare(p2));

        let p2 = Point3D::new(x, y, z - d);
        assert!(!p.compare(p2));

        let p2 = Point3D::new(x, -y, z);
        assert!(!p.compare(p2));
    }

    #[test]
    fn test_is_collinear() -> Result<(), String> {
        let a = Point3D::new(0., 0., 0.);
        let b = Point3D::new(1., 0., 0.);
        let c = Point3D::new(3., 0., 0.);

        let d = Point3D::new(1., 2., 4.);

        assert!(a.is_collinear(b, c)?);
        assert!(a.is_collinear(a, c)?);
        assert!(b.is_collinear(a, c)?);
        assert!(b.is_collinear(b, c)?);
        assert!(c.is_collinear(b, a)?);
        assert!(c.is_collinear(c, a)?);

        assert!(!a.is_collinear(b, d)?);

        assert!(c.is_collinear(c, c).is_err());
        assert!(a.is_collinear(a, a).is_err());
        assert!(b.is_collinear(b, b).is_err());
        Ok(())
    }

    #[test]
    fn test_distance_to_plane() {
        let point = Point3D {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };
        let plane_point = Point3D {
            x: 4.0,
            y: 5.0,
            z: 6.0,
        };
        let plane_normal = Vector3D {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        };

        let distance = point.distance_to_plane(plane_point, plane_normal);
        println!("Distance to the plane: {}", distance);
    }

    #[test]
    fn test_distance_to_line() {
        let pt = Point3D::new(0., 0., 0.);
        let a = Point3D::new(0., 0., 1.);
        let b = Point3D::new(1., 0., 1.);
        let exp = 1.0;
        assert!(
            (exp - pt.distance_to_line(a, b)).abs() < 1e-9,
            "expecting {}... found {}",
            exp,
            pt.distance_to_line(a, b)
        );

        let pt = Point3D::new(0., 0., 0.);
        let a = Point3D::new(-1., 0., 0.);
        let b = Point3D::new(-1., 0., 1.);
        let exp = 1.0;
        assert!(
            (exp - pt.distance_to_line(a, b)).abs() < 1e-9,
            "expecting {}... found {}",
            exp,
            pt.distance_to_line(a, b)
        );
    }
} // end of Testing module
