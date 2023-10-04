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

use crate::Point3D;
use std::fmt;

/// A 3-dimensional Vector
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Vector3D {
    /// the X component
    pub x: Float,
    /// the Y component    
    pub y: Float,
    /// the Z component
    pub z: Float,
}

impl fmt::Display for Vector3D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Vector3D({:.5},{:.5},{:.5})", self.x, self.y, self.z)
    }
}

impl std::convert::From<Point3D> for Vector3D {
    fn from(v: Point3D) -> Vector3D {
        Vector3D::new(v.x, v.y, v.z)
    }
}

impl Vector3D {
    /// Creates a new vector.
    pub fn new(x: Float, y: Float, z: Float) -> Vector3D {
        Vector3D { x, y, z }
    }

    /// Returns a new vector, containing the absolute values of each component of `self`
    pub fn abs(&self) -> Self {
        Self {
            x: self.x.abs(),
            y: self.y.abs(),
            z: self.z.abs(),
        }
    }

    /// Transforms a [`Vector3D`] into a [`Point3D`]
    pub fn as_point3d(&self) -> Point3D {
        Point3D {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }

    /// Gets a normalized [`Vector3D`] that is perpendicular
    /// to `self`
    #[cfg(not(feature = "quick_inv_sqrt"))]
    pub fn get_perpendicular(&self) -> Result<Self, String> {
        const TINY: Float = 100. * Float::EPSILON;
        let ax: Float;
        let ay: Float;
        let az: Float;
        if self.x.abs() > TINY {
            let vx2 = self.x * self.x;
            let vy2 = self.y * self.y;

            ay = self.x / (vx2 + vy2).sqrt();
            az = 0.;
            ax = -self.y * ay / self.x;
        } else if self.y.abs() > TINY {
            let vx2 = self.x * self.x;
            let vy2 = self.y * self.y;
            ax = self.y / (vx2 + vy2).sqrt();
            az = 0.;
            ay = -self.x * ax / self.y;
        } else if self.z.abs() > TINY {
            let vx2 = self.x * self.x;
            let vz2 = self.z * self.z;

            ax = self.z / (vz2 + vx2).sqrt();
            ay = 0.0;
            az = -self.x * ax / self.z;
        } else {
            return Err(format!(
                "Trying to get a Vector3D perpendicular to a Zero Vector (self = {})",
                self
            ));
        }
        let ret = Self::new(ax, ay, az);
        // ret.normalize();
        Ok(ret)
    }

    /// Gets a normalized [`Vector3D`] that is perpendicular
    /// to `self`
    #[cfg(feature = "quick_inv_sqrt")]
    pub fn get_perpendicular(&self) -> Result<Self, String> {
        const TINY: Float = 100. * Float::EPSILON;
        let ax: Float;
        let ay: Float;
        let az: Float;
        if self.x.abs() > TINY {
            let vx2 = self.x * self.x;
            let vy2 = self.y * self.y;

            ay = self.x * crate::quick_inverse_sqrt::quick_inv_sqrt(vx2 + vy2);
            az = 0.;
            ax = -self.y * ay / self.x;
        } else if self.y.abs() > TINY {
            let vx2 = self.x * self.x;
            let vy2 = self.y * self.y;
            ax = self.y * crate::quick_inverse_sqrt::quick_inv_sqrt(vx2 + vy2);
            az = 0.;
            ay = -self.x * ax / self.y;
        } else if self.z.abs() > TINY {
            let vx2 = self.x * self.x;
            let vz2 = self.z * self.z;

            ax = self.z * crate::quick_inverse_sqrt::quick_inv_sqrt(vz2 + vx2);
            ay = 0.0;
            az = -self.x * ax / self.z;
        } else {
            return Err(format!(
                "Trying to get a Vector3D perpendicular to a Zero Vector (self = {})",
                self
            ));
        }
        let ret = Self::new(ax, ay, az);
        // ret.normalize();
        Ok(ret)
    }

    /// Checks whether a vector is equal (or close) to another vector
    pub fn compare(&self, p: Vector3D) -> bool {
        const TINY: Float = 1e-5;
        (self.x - p.x).abs() < TINY && (self.y - p.y).abs() < TINY && (self.z - p.z).abs() < TINY
    }

    /// Cross product between two vectors
    pub fn cross(&self, v: Vector3D) -> Vector3D {
        let dx = self.y * v.z - self.z * v.y;
        let dy = self.z * v.x - self.x * v.z;
        let dz = self.x * v.y - self.y * v.x;

        Vector3D::new(dx, dy, dz)
    }

    /// Calculates the length of a vector
    pub fn length(&self) -> Float {
        self.length_squared().sqrt()
    }

    /// Calculates the length of a vector squared.
    ///
    /// Often using the squared of the length is the same as using the length of a vector
    /// (e.g., when comparing distances, the squared lengths of vectors are sorted in the same order
    /// as the lengths themselves). In such cases, using `length_squared()` is preferred because
    /// it is faster
    pub fn length_squared(&self) -> Float {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// Normalizes a Vector, mutating it.
    pub fn normalize(&mut self) {

        let l2 = self.length_squared();
        assert!(l2 > 1e-20, "Trying to normalize a vector with zero length");

        #[cfg(not(feature = "quick_inv_sqrt"))]
        let l = 1. / l2.sqrt();
        #[cfg(feature = "quick_inv_sqrt")]
        let l = crate::quick_inverse_sqrt::quick_inv_sqrt(l2);

        self.x *= l;
        self.y *= l;
        self.z *= l;
    }

    /// Returns a normalized version of a vector
    pub fn get_normalized(&self) -> Vector3D {
        let mut ret = *self;
        ret.normalize();
        ret
    }

    /// Checks whether a vector is Zero
    pub fn is_zero(&self) -> bool {
        const TINY: Float = 100. * Float::EPSILON;
        self.x.abs() < TINY && self.y.abs() < TINY && self.z.abs() < TINY
    }

    /// Checks if two vectors are parallel
    pub fn is_parallel(&self, v: Vector3D) -> bool {
        // If any of them is zero, false
        if v.is_zero() || self.is_zero() {
            return false;
        }

        let ab_squared = self.length_squared() * v.length_squared();
        let dot: Float = *self * v;
        let r = ((dot * dot) - ab_squared).abs();
        r < 1e-5
    } // end of is_parallel

    /// Checks if two vectors are parallel and follow the same direction.
    pub fn is_same_direction(&self, v: Vector3D) -> bool {
        if !self.is_parallel(v) {
            // If they are not parallel, don't bother
            false
        } else {
            // Dot product must be positive for the
            // vectors to point in the same general
            // direction
            *self * v > 0.
        }
    }
}

impl std::ops::Mul<Vector3D> for Vector3D {
    type Output = Float;

    fn mul(self, other: Vector3D) -> Float {
        {
            self.x * other.x + self.y * other.y + self.z * other.z
        }
    }
}

impl std::ops::Mul<Float> for Vector3D {
    type Output = Vector3D;

    fn mul(self, s: Float) -> Vector3D {
        Vector3D::new(self.x * s, self.y * s, self.z * s)
    }
}

impl std::ops::Mul<Point3D> for Vector3D {
    type Output = Float;

    fn mul(self, other: Point3D) -> Float {
        {
            self.x * other.x + self.y * other.y + self.z * other.z
        }
    }
}

impl std::ops::MulAssign<Float> for Vector3D {
    fn mul_assign(&mut self, s: Float) {
        self.x *= s;
        self.y *= s;
        self.z *= s;
    }
}

impl std::ops::Div<Float> for Vector3D {
    type Output = Vector3D;

    fn div(self, s: Float) -> Vector3D {
        Vector3D::new(self.x / s, self.y / s, self.z / s)
    }
}

impl std::ops::DivAssign<Float> for Vector3D {
    fn div_assign(&mut self, s: Float) {
        self.x /= s;
        self.y /= s;
        self.z /= s;
    }
}

impl std::ops::Add<Vector3D> for Vector3D {
    type Output = Vector3D;

    fn add(self, other: Vector3D) -> Vector3D {
        Vector3D::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl std::ops::AddAssign<Vector3D> for Vector3D {
    fn add_assign(&mut self, other: Vector3D) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

impl std::ops::SubAssign<Vector3D> for Vector3D {
    fn sub_assign(&mut self, other: Vector3D) {
        self.x -= other.x;
        self.y -= other.y;
        self.z -= other.z;
    }
}

impl std::ops::Neg for Vector3D {
    type Output = Vector3D;

    fn neg(self) -> Vector3D {
        Vector3D::new(-self.x, -self.y, -self.z)
    }
}

impl std::ops::Sub<Vector3D> for Vector3D {
    type Output = Vector3D;

    fn sub(self, other: Vector3D) -> Vector3D {
        Vector3D::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

/***********/
/* TESTING */
/***********/

#[cfg(test)]
mod testing {
    use super::*;

    #[test]
    fn test_get_perpendicular() -> Result<(), String> {
        fn check(v: Vector3D) -> Result<(), String> {
            let perp = v.get_perpendicular()?;
            if (v * perp).abs() > 100. * Float::EPSILON {
                return Err(format!(
                    "Vector {} is not perpendicular with vector {} (dot prod is {})",
                    v,
                    perp,
                    (v * perp).abs()
                ));
            }
            if (1. - perp.length()).abs() > 0.00001 {
                return Err(format!(
                    "Perpendicular Vector {} is not normalized... length is {}",
                    perp,
                    perp.length()
                ));
            }
            Ok(())
        }

        check(Vector3D::new(0., 0., 1.))?;
        check(Vector3D::new(0., 1., 1.))?;
        check(Vector3D::new(0., 1., 0.))?;
        check(Vector3D::new(1., 1., 0.))?;
        check(Vector3D::new(1., 0., 0.))?;

        check(Vector3D::new(3., 0., 0.))?;
        check(Vector3D::new(3., 123.1, 0.))?;
        check(Vector3D::new(-3., -123.1, 0.))?;
        check(Vector3D::new(-3., -123.1, 10.))?;
        check(Vector3D::new(-3., 0., 10.))?;
        check(Vector3D::new(0., -123.1, 10.))?;
        check(Vector3D::new(0., 0., 50.))?;

        assert!(check(Vector3D::new(0., 0., 0.)).is_err());
        Ok(())
    }

    #[test]
    fn test_new() {
        let x = 1.0;
        let y = 2.0;
        let z = 3.0;

        let pt = Vector3D::new(x, y, z);
        assert_eq!(x, pt.x);
        assert_eq!(y, pt.y);
        assert_eq!(z, pt.z);

        assert_eq!(x, pt.x);
        assert_eq!(y, pt.y);
        assert_eq!(z, pt.z);
    }

    #[test]
    fn test_dot() {
        let x = Vector3D::new(1.0, 0.0, 0.0);
        let y = Vector3D::new(0.0, 1.0, 0.0);

        assert_eq!(x * y, 0.0);
        assert_eq!(y * x, 0.0);
        assert_eq!(x * x, 1.0);
        assert_eq!(y * y, 1.0);

        let x = Vector3D::new(1.0, 3.0, -10.0);
        let y = Vector3D::new(0.1, 2.0, 10.0);

        assert_eq!(x * y, 1. * 0.1 + 3. * 2. - 10. * 10.);
        assert_eq!(y * x, 1. * 0.1 + 3. * 2. - 10. * 10.);
        assert_eq!(x * x, 1. + 3. * 3. + 10. * 10.);
        assert_eq!(y * y, 0.1 * 0.1 + 2.0 * 2.0 + 10. * 10.);

        // Against a point
        let x = Vector3D::new(1.0, 3.0, -10.0);
        let y = Point3D::new(0.1, 2.0, 10.0);

        assert_eq!(x * y, 1. * 0.1 + 3. * 2. - 10. * 10.);
        assert_eq!(y * x, 1. * 0.1 + 3. * 2. - 10. * 10.);
        assert_eq!(x * x, 1. + 3. * 3. + 10. * 10.);
        assert_eq!(y * y, 0.1 * 0.1 + 2.0 * 2.0 + 10. * 10.);
    }

    #[test]
    fn test_scale() {
        let x = 1.2;
        let y = 5.22;
        let z = 9.123;
        let s = 4.123;

        let v = Vector3D::new(x, y, z);
        let new_v = v * s;

        assert_eq!(new_v.x, x * s);
        assert_eq!(new_v.y, y * s);
        assert_eq!(new_v.z, z * s);

        let v = Vector3D::new(x, y, z);
        let new_v = v / s;

        assert_eq!(new_v.x, x / s);
        assert_eq!(new_v.y, y / s);
        assert_eq!(new_v.z, z / s);

        let x = 1.2;
        let y = 5.22;
        let z = 9.123;
        let s = 4.123;

        let mut v = Vector3D::new(x, y, z);
        v *= s;

        assert_eq!(v.x, x * s);
        assert_eq!(v.y, y * s);
        assert_eq!(v.z, z * s);

        let mut v = Vector3D::new(x, y, z);
        v /= s;

        assert_eq!(v.x, x / s);
        assert_eq!(v.y, y / s);
        assert_eq!(v.z, z / s);
    }

    #[test]
    fn test_add() {
        let x = 1.2;
        let y = 5.22;
        let z = 9.123;
        let s = 1.212312;

        let mut v = Vector3D::new(x, y, z);
        let d = Vector3D::new(s * x, s * y, s * z);

        let v2 = v + d;

        assert!(v2.x - x * (s + 1.0) < 1E-10);
        assert!(v2.y - y * (s + 1.0) < 1E-10);
        assert!(v2.z - z * (s + 1.0) < 1E-10);

        v += d;
        assert!(v.x - x * (s + 1.0) < 1E-10);
        assert!(v.y - y * (s + 1.0) < 1E-10);
        assert!(v.z - z * (s + 1.0) < 1E-10);
    }

    #[test]
    fn test_sub() {
        let x = 1.2;
        let y = 5.22;
        let z = 9.123;
        let s = 1.212312;

        let mut v = Vector3D::new(x, y, z);
        let d = Vector3D::new(-s * x, -s * y, -s * z);

        let v2 = v - d;

        assert!(v2.x - x * (s + 1.0) < 1E-10);
        assert!(v2.y - y * (s + 1.0) < 1E-10);
        assert!(v2.z - z * (s + 1.0) < 1E-10);

        v -= d;
        assert!(v.x - x * (s + 1.0) < 1E-10);
        assert!(v.y - y * (s + 1.0) < 1E-10);
        assert!(v.z - z * (s + 1.0) < 1E-10);
    }

    #[test]
    fn test_compare() {
        let x = 123.1;
        let y = 543.1;
        let z = 9123.2;
        let d = 0.1;

        let p = Vector3D::new(x, y, z);
        let p2 = Vector3D::new(x, y, z);
        assert!(p.compare(p2));

        let p2 = Vector3D::new(x + d, y, z);
        assert!(!p.compare(p2));

        let p2 = Vector3D::new(x, y, z - d);
        assert!(!p.compare(p2));

        let p2 = Vector3D::new(x, -y, z);
        assert!(!p.compare(p2));
    }

    #[test]
    fn test_cross() {
        let zero = Vector3D::new(0.0, 0.0, 0.0);
        let x = Vector3D::new(1.0, 0.0, 0.0);
        let y = Vector3D::new(0.0, 1.0, 0.0);
        let z = Vector3D::new(0.0, 0.0, 1.0);

        assert!(x.cross(x).compare(zero));
        assert!(x.cross(y).compare(z));
        assert!(y.cross(z).compare(x));

        let a = Vector3D::new(2., 3., 4.);
        let b = Vector3D::new(5., 6., 7.);
        let c = a.cross(b);

        assert_eq!(c.x, -3.);
        assert_eq!(c.y, 6.);
        assert_eq!(c.z, -3.);
    }

    #[test]
    fn test_length() {
        let a = 5.0;
        let x = Vector3D::new(a, a, a);

        let l = x.length();
        let delta = l - (3.0 as Float).sqrt() * a;
        assert!(delta.abs() < 1E-5);
    }

    #[test]
    fn test_normalize() {
        let mut v = Vector3D::new(12.1, 54.1, 765.1);
        let l = v.length();

        let v_norm_ref = v / l;

        let v_norm = v.get_normalized();
        assert!(v_norm.compare(v_norm_ref));

        v.normalize();
        assert!(v.compare(v_norm_ref));
    }

    #[test]
    fn test_is_zero() {
        let zero = Vector3D::new(0.0, 0.0, 0.0);

        assert!(zero.is_zero());

        let nearly_zero = Vector3D::new(1E-4, 0.0, 0.0);
        assert!(!nearly_zero.is_zero());
    }

    #[test]
    fn test_is_parallel() {
        // Some Any of them is zero vector.
        let v1 = Vector3D::new(123.1, 5.6, 99.12);
        let v2 = Vector3D::new(0., 0., 0.);
        assert!(!v1.is_parallel(v2));
        assert!(!v2.is_parallel(v1));
        assert!(!v2.is_parallel(v2));

        let v1 = Vector3D::new(123.1, 5.6, 99.12);
        let mut v2 = v1 * 3.14;

        // V2 is v1 scaled... same dir and
        // parallel
        assert!(v1.is_parallel(v2));
        assert!(v1.is_same_direction(v2));

        // V2 goes in the opposite dir...
        // Paralel, but no in same dir
        v2 /= -1.0;
        assert!(v1.is_parallel(v2));
        assert!(!v1.is_same_direction(v2));

        // Are not parallel... none of them
        //are true
        let d = Vector3D::new(0.1, 0.0, 0.0);
        v2 += d;
        assert!(!v1.is_parallel(v2));
        assert!(!v1.is_same_direction(v2));
    }

    #[test]
    fn test_negation() {
        let x = 1.2;
        let y = 5.22;
        let z = 9.123;
        let v1 = Vector3D::new(x, y, z);
        let v2 = Vector3D::new(-x, -y, -z);

        assert_eq!((-v1).x, -x);
        assert_eq!((-v1).y, -y);
        assert_eq!((-v1).z, -z);

        assert!(v1.compare(-v2));
    }
}
