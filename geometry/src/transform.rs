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

use crate::gamma;
use crate::Float;
use crate::{BBox3D, Point3D, Ray3D, Vector3D};

macro_rules! elem {
    ( $row : expr, $col:expr ) => {{
        assert!($row < 4);
        // assert!($row >= 0);
        assert!($col < 4);
        // assert!($col >= 0);
        4 * $row + $col
    }};
}

/// Represents a transformation matrix
#[derive(Debug, Clone)]
pub struct Transform {
    elements: [Float; 16],
    inv_elements: [Float; 16],
}

impl Default for Transform {
    fn default() -> Self {
        Self::new()
    }
}

fn mul4x4point(matrix: &[Float; 16], pt: Point3D) -> Point3D {
    let (x, y, z) = (pt.x, pt.y, pt.z);
    let new_x = matrix[elem!(0, 0)] * x
        + matrix[elem!(0, 1)] * y
        + matrix[elem!(0, 2)] * z
        + matrix[elem!(0, 3)];
    let new_y = matrix[elem!(1, 0)] * x
        + matrix[elem!(1, 1)] * y
        + matrix[elem!(1, 2)] * z
        + matrix[elem!(1, 3)];
    let new_z = matrix[elem!(2, 0)] * x
        + matrix[elem!(2, 1)] * y
        + matrix[elem!(2, 2)] * z
        + matrix[elem!(2, 3)];
    let w = matrix[elem!(3, 0)] * x
        + matrix[elem!(3, 1)] * y
        + matrix[elem!(3, 2)] * z
        + matrix[elem!(3, 3)];

    debug_assert!((1.0 - w.abs()) < 2. * Float::EPSILON);

    Point3D::new(new_x, new_y, new_z) / w
}

fn mul4x4vec(matrix: &[Float; 16], vec: Vector3D) -> Vector3D {
    let (x, y, z) = (vec.x, vec.y, vec.z);
    let new_x = matrix[elem!(0, 0)] * x + matrix[elem!(0, 1)] * y + matrix[elem!(0, 2)] * z;
    let new_y = matrix[elem!(1, 0)] * x + matrix[elem!(1, 1)] * y + matrix[elem!(1, 2)] * z;
    let new_z = matrix[elem!(2, 0)] * x + matrix[elem!(2, 1)] * y + matrix[elem!(2, 2)] * z;

    Vector3D::new(new_x, new_y, new_z)
}

fn mul4x4_abs(matrix: &[Float; 16], x: Float, y: Float, z: Float) -> Point3D {
    let err_x = (matrix[elem!(0, 0)] * x).abs()
        + (matrix[elem!(0, 1)] * y).abs()
        + (matrix[elem!(0, 2)] * z).abs()
        + matrix[elem!(0, 3)].abs();
    let err_y = (matrix[elem!(1, 0)] * x).abs()
        + (matrix[elem!(1, 1)] * y).abs()
        + (matrix[elem!(1, 2)] * z).abs()
        + matrix[elem!(1, 3)].abs();

    let err_z = (matrix[elem!(2, 0)] * x).abs()
        + (matrix[elem!(2, 1)] * y).abs()
        + (matrix[elem!(2, 2)] * z).abs()
        + matrix[elem!(2, 3)].abs();

    Point3D::new(err_x, err_y, err_z)
}

/// Multiplies two 4x4 matrices, represented as `[Float; 16]`
pub fn mul4x4(m1: &[Float; 16], m2: &[Float; 16]) -> [Float; 16] {
    let mut ret = [0.; 16];

    // This was for SIMD... but did not help.
    // let m1_row0 = simd_row4x4(0, m1);
    // let m1_row1 = simd_row4x4(1, m1);
    // let m1_row2 = simd_row4x4(2, m1);
    // let m1_row3 = simd_row4x4(3, m1);

    // let m2_col0 = simd_col4x4(0, m2);
    // let m2_col1 = simd_col4x4(1, m2);
    // let m2_col2 = simd_col4x4(2, m2);
    // let m2_col3 = simd_col4x4(3, m2);

    // ret[elem!(0,0)] = (m1_row0*m2_col0).reduce_sum();
    // ret[elem!(1,0)] = (m1_row1*m2_col0).reduce_sum();
    // ret[elem!(2,0)] = (m1_row2*m2_col0).reduce_sum();
    // ret[elem!(3,0)] = (m1_row3*m2_col0).reduce_sum();

    // ret[elem!(0,1)] = (m1_row0*m2_col1).reduce_sum();
    // ret[elem!(1,1)] = (m1_row1*m2_col1).reduce_sum();
    // ret[elem!(2,1)] = (m1_row2*m2_col1).reduce_sum();
    // ret[elem!(3,1)] = (m1_row3*m2_col1).reduce_sum();

    // ret[elem!(0,2)] = (m1_row0*m2_col2).reduce_sum();
    // ret[elem!(1,2)] = (m1_row1*m2_col2).reduce_sum();
    // ret[elem!(2,2)] = (m1_row2*m2_col2).reduce_sum();
    // ret[elem!(3,2)] = (m1_row3*m2_col2).reduce_sum();

    // ret[elem!(0,3)] = (m1_row0*m2_col3).reduce_sum();
    // ret[elem!(1,3)] = (m1_row1*m2_col3).reduce_sum();
    // ret[elem!(2,3)] = (m1_row2*m2_col3).reduce_sum();
    // ret[elem!(3,3)] = (m1_row3*m2_col3).reduce_sum();

    let mut calc_elem = |row: usize, col: usize| {
        ret[elem!(row, col)] = m1[elem!(row, 0)] * m2[elem!(0, col)]
            + m1[elem!(row, 1)] * m2[elem!(1, col)]
            + m1[elem!(row, 2)] * m2[elem!(2, col)]
            + m1[elem!(row, 3)] * m2[elem!(3, col)];
    };

    calc_elem(0, 0);
    calc_elem(0, 1);
    calc_elem(0, 2);
    calc_elem(0, 3);

    calc_elem(1, 0);
    calc_elem(1, 1);
    calc_elem(1, 2);
    calc_elem(1, 3);

    calc_elem(2, 0);
    calc_elem(2, 1);
    calc_elem(2, 2);
    calc_elem(2, 3);

    calc_elem(3, 0);
    calc_elem(3, 1);
    calc_elem(3, 2);
    calc_elem(3, 3);

    ret
}

impl std::ops::MulAssign<Self> for Transform {
    fn mul_assign(&mut self, other: Self) {
        self.elements = mul4x4(&self.elements, &other.elements);
        self.inv_elements = mul4x4(&self.inv_elements, &other.inv_elements);
    }
}

impl Transform {
    /// Crates a new Identity [`Transform`] (i.e., does nothing to
    /// the points being passed)
    pub fn new() -> Self {
        let mut elements = [0.0; 16];
        for i in 0..4 {
            elements[elem!(i, i)] = 1.0;
        }
        let inv_elements = elements;

        Self {
            elements,
            inv_elements,
        }
    }

    /// Creates a new Translation [`Transform`]
    pub fn translate(x: Float, y: Float, z: Float) -> Self {
        // Get an identity
        let mut ret = Self::new();

        // set translation components
        ret.elements[elem!(0, 3)] = x;
        ret.elements[elem!(1, 3)] = y;
        ret.elements[elem!(2, 3)] = z;

        // set inverse translation components
        ret.inv_elements[elem!(0, 3)] = -x;
        ret.inv_elements[elem!(1, 3)] = -y;
        ret.inv_elements[elem!(2, 3)] = -z;

        ret
    }

    /// Creates a new Scale [`Transform`]
    pub fn scale(x: Float, y: Float, z: Float) -> Self {
        let mut elements = [0.0; 16];
        elements[elem!(0, 0)] = x;
        elements[elem!(1, 1)] = y;
        elements[elem!(2, 2)] = z;
        elements[elem!(3, 3)] = 1.0;

        let mut inv_elements = [0.0; 16];
        inv_elements[elem!(0, 0)] = 1.0 / x;
        inv_elements[elem!(1, 1)] = 1.0 / y;
        inv_elements[elem!(2, 2)] = 1.0 / z;
        inv_elements[elem!(3, 3)] = 1.0;

        Self {
            elements,
            inv_elements,
        }
    }

    /// Creates a new [`Transform`] that rotates counterclockwise on the X axis.
    /// The input is in degrees
    pub fn rotate_x(degrees: Float) -> Self {
        let rad = degrees.to_radians();
        let mut ret = Self::new();

        ret.elements[elem!(1, 1)] = rad.cos();
        ret.inv_elements[elem!(1, 1)] = rad.cos();

        ret.elements[elem!(1, 2)] = -rad.sin();
        ret.inv_elements[elem!(2, 1)] = -rad.sin();

        ret.elements[elem!(2, 1)] = rad.sin();
        ret.inv_elements[elem!(1, 2)] = rad.sin();

        ret.elements[elem!(2, 2)] = rad.cos();
        ret.inv_elements[elem!(2, 2)] = rad.cos();

        ret
    }

    /// Creates a new [`Transform`] that rotates counterclockwise on the Y axis.
    /// The input is in degrees
    pub fn rotate_y(degrees: Float) -> Self {
        let rad = degrees.to_radians();
        let mut ret = Self::new();

        ret.elements[elem!(0, 0)] = rad.cos();
        ret.inv_elements[elem!(0, 0)] = rad.cos();

        ret.elements[elem!(0, 2)] = rad.sin();
        ret.inv_elements[elem!(2, 0)] = rad.sin();

        ret.elements[elem!(2, 0)] = -rad.sin();
        ret.inv_elements[elem!(0, 2)] = -rad.sin();

        ret.elements[elem!(2, 2)] = rad.cos();
        ret.inv_elements[elem!(2, 2)] = rad.cos();

        ret
    }

    /// Creates a new [`Transform`] that rotates counterclockwise on the Z axis.
    /// The input is in degrees
    pub fn rotate_z(degrees: Float) -> Self {
        let rad = degrees.to_radians();
        let mut ret = Self::new();

        ret.elements[elem!(0, 0)] = rad.cos();
        ret.inv_elements[elem!(0, 0)] = rad.cos();

        ret.elements[elem!(0, 1)] = -rad.sin();
        ret.inv_elements[elem!(1, 0)] = -rad.sin();

        ret.elements[elem!(1, 0)] = rad.sin();
        ret.inv_elements[elem!(0, 1)] = rad.sin();

        ret.elements[elem!(1, 1)] = rad.cos();
        ret.inv_elements[elem!(1, 1)] = rad.cos();

        ret
    }

    /// Checks whether a [`Transform`] modifies the hand-ness of the coordinate system
    pub fn changes_hands(&self) -> bool {
        let m = &self.elements;
        let determinant = m[elem!(0, 0)]
            * (m[elem!(1, 1)] * m[elem!(2, 2)] - m[elem!(1, 2)] * m[elem!(2, 1)])
            - m[elem!(0, 1)] * (m[elem!(1, 0)] * m[elem!(2, 2)] - m[elem!(1, 2)] * m[elem!(2, 0)])
            + m[elem!(0, 2)] * (m[elem!(1, 0)] * m[elem!(2, 1)] - m[elem!(1, 1)] * m[elem!(2, 0)]);

        // return
        determinant < 0.
    }

    /* OVER POINTS */

    /// Transforms a [`Point3D`] into another [`Point3D`]
    pub fn transform_pt(&self, pt: Point3D) -> Point3D {
        mul4x4point(&self.elements, pt)
    }

    /// Inverse Transforms a [`Point3D`] into another [`Point3D`]
    pub fn inv_transform_pt(&self, pt: Point3D) -> Point3D {
        mul4x4point(&self.inv_elements, pt)
    }

    /// Transforms a [`Point3D`] into another [`Point3D`], propagating
    /// an error value
    pub fn transform_pt_with_error(&self, pt: Point3D) -> (Point3D, Point3D) {
        let ret = self.transform_pt(pt);
        let (x, y, z) = (pt.x, pt.y, pt.z);
        let err_ret = mul4x4_abs(&self.elements, x, y, z) * gamma!(3);
        (ret, err_ret)
    }

    /// Inverse Transforms a [`Point3D`] into another [`Point3D`], propagating
    /// an error value
    pub fn inv_transform_pt_with_error(&self, pt: Point3D) -> (Point3D, Point3D) {
        let ret = self.inv_transform_pt(pt);
        let (x, y, z) = (pt.x, pt.y, pt.z);
        let err_ret = mul4x4_abs(&self.inv_elements, x, y, z) * gamma!(3);
        (ret, err_ret)
    }

    /// Transforms a [`Point3D`] into another [`Point3D`], calculating
    /// an error value
    pub fn transform_pt_propagate_error(&self, pt: Point3D, error: Point3D) -> (Point3D, Point3D) {
        let (ret, err2) = self.transform_pt_with_error(pt);

        // propagate error
        let (x, y, z) = (error.x, error.y, error.z);
        let err1 = mul4x4_abs(&self.elements, x, y, z) * (1. + gamma!(3));

        (ret, err1 + err2)
    }

    /// Inverse Transforms a [`Point3D`] into another [`Point3D`], calculating
    /// an error value
    pub fn inv_transform_pt_propagate_error(
        &self,
        pt: Point3D,
        error: Point3D,
    ) -> (Point3D, Point3D) {
        let (ret, err2) = self.inv_transform_pt_with_error(pt);

        // propagate error
        let (x, y, z) = (error.x, error.y, error.z);
        let err1 = mul4x4_abs(&self.inv_elements, x, y, z) * (1. + gamma!(3));

        (ret, err1 + err2)
    }

    /* OVER VECTORS */

    /// Transforms a [`Vector3D`] into another [`Vector3D`]
    pub fn transform_vec(&self, vec: Vector3D) -> Vector3D {
        mul4x4vec(&self.elements, vec)
    }

    /// Inverse Transforms a [`Vector3D`] into another [`Vector3D`]
    pub fn inv_transform_vec(&self, vec: Vector3D) -> Vector3D {
        mul4x4vec(&self.inv_elements, vec)
    }

    /// Transforms a [`Vector3D`] into another [`Vector3D`], calculating
    /// an error value
    pub fn transform_vec_with_error(&self, vec: Vector3D) -> (Vector3D, Point3D) {
        let ret = self.transform_vec(vec);

        // propagate error
        let (x, y, z) = (vec.x, vec.y, vec.z);
        let err_ret = mul4x4_abs(&self.elements, x, y, z) * gamma!(3);

        (ret, err_ret)
    }

    /// Inverse Transforms a [`Vector3D`] into another [`Vector3D`], calculating
    /// an error value
    pub fn inv_transform_vec_with_error(&self, vec: Vector3D) -> (Vector3D, Point3D) {
        let ret = self.inv_transform_vec(vec);

        // propagate error
        let (x, y, z) = (vec.x, vec.y, vec.z);
        let err_ret = mul4x4_abs(&self.inv_elements, x, y, z) * gamma!(3);

        (ret, err_ret)
    }

    /// Transforms a [`Vector3D`] into another [`Vector3D`], propagating
    /// an error value
    pub fn transform_vec_propagate_error(
        &self,
        vec: Vector3D,
        error: Point3D,
    ) -> (Vector3D, Point3D) {
        let (ret, err2) = self.transform_vec_with_error(vec);

        // propagate error
        let (x, y, z) = (error.x, error.y, error.z);
        let err1 = mul4x4_abs(&self.elements, x, y, z) * (1. + gamma!(3));

        (ret, err1 + err2)
    }

    /// Inverse  Transforms a [`Vector3D`] into another [`Vector3D`], propagating
    /// an error value
    pub fn inv_transform_vec_propagate_error(
        &self,
        vec: Vector3D,
        error: Point3D,
    ) -> (Vector3D, Point3D) {
        let (ret, err2) = self.inv_transform_vec_with_error(vec);

        // propagate error
        let (x, y, z) = (error.x, error.y, error.z);
        let err1 = mul4x4_abs(&self.inv_elements, x, y, z) * (1. + gamma!(3));

        (ret, err1 + err2)
    }

    /* OVER NORMAL */

    /// Transforms a [`Vector3D`] into another [`Point3D`]
    pub fn transform_normal(&self, vec: Vector3D) -> Vector3D {
        let (x, y, z) = (vec.x, vec.y, vec.z);
        let new_x = self.inv_elements[elem!(0, 0)] * x
            + self.inv_elements[elem!(1, 0)] * y
            + self.inv_elements[elem!(2, 0)] * z;
        let new_y = self.inv_elements[elem!(0, 1)] * x
            + self.inv_elements[elem!(1, 1)] * y
            + self.inv_elements[elem!(2, 1)] * z;
        let new_z = self.inv_elements[elem!(0, 2)] * x
            + self.inv_elements[elem!(1, 2)] * y
            + self.inv_elements[elem!(2, 2)] * z;

        Vector3D::new(new_x, new_y, new_z)
    }

    /// Inv transforms a [`Vector3D`] into another [`Point3D`]
    pub fn inv_transform_normal(&self, vec: Vector3D) -> Vector3D {
        let (x, y, z) = (vec.x, vec.y, vec.z);
        let new_x = self.elements[elem!(0, 0)] * x
            + self.elements[elem!(1, 0)] * y
            + self.elements[elem!(2, 0)] * z;
        let new_y = self.elements[elem!(0, 1)] * x
            + self.elements[elem!(1, 1)] * y
            + self.elements[elem!(2, 1)] * z;
        let new_z = self.elements[elem!(0, 2)] * x
            + self.elements[elem!(1, 2)] * y
            + self.elements[elem!(2, 2)] * z;

        Vector3D::new(new_x, new_y, new_z)
    }

    /* OVER RAYS */

    /// Transforms a [`Ray3D`]. returns a new [`Ray3D`] and also the
    /// errors associated to the `origin` and the `direction` in a tuple
    pub fn transform_ray(&self, ray: &Ray3D) -> (Ray3D, Point3D, Point3D) {
        let (mut origin, o_error) = self.transform_pt_with_error(ray.origin);
        let (direction, d_error) = self.transform_vec_with_error(ray.direction);
        let l2 = direction.length_squared();
        if l2 > 0. {
            // move origin to account for the error
            let dt = (direction.abs() * o_error) / l2;
            origin += direction * dt;
        }

        let r = Ray3D { origin, direction };
        (r, o_error, d_error)
    }

    /// Inverse Transforms a [`Ray3D`]. returns a new [`Ray3D`] and also the
    /// errors associated to the `origin` and the `direction` in a tuple
    pub fn inv_transform_ray(&self, ray: &Ray3D) -> (Ray3D, Point3D, Point3D) {
        let (mut origin, o_error) = self.inv_transform_pt_with_error(ray.origin);
        let (direction, d_error) = self.inv_transform_vec_with_error(ray.direction);
        let l2 = direction.length_squared();
        if l2 > 0. {
            // move origin to account for the error
            let dt = (direction.abs() * o_error) / l2;
            origin += direction * dt;
        }

        let r = Ray3D { origin, direction };
        (r, o_error, d_error)
    }

    /// Transforms a [`Ray3D`] and propagates the error. returns a new [`Ray3D`] and also the
    /// errors associated to the `origin` and the `direction` in a tuple
    pub fn transform_ray_propagate_error(
        &self,
        ray: &Ray3D,
        o_error_in: Point3D,
        d_error_in: Point3D,
    ) -> (Ray3D, Point3D, Point3D) {
        let (mut origin, o_error) = self.transform_pt_propagate_error(ray.origin, o_error_in);
        let (direction, d_error) = self.transform_vec_propagate_error(ray.direction, d_error_in);

        let l2 = direction.length_squared();
        if l2 > 0. {
            // move origin to account for the error
            let dt = (direction.abs() * o_error) / l2;
            origin += direction * dt;
        }

        let r = Ray3D { origin, direction };
        (r, o_error, d_error)
    }

    /// Inverse Transforms a [`Ray3D`] and propagates the error. returns a new [`Ray3D`] and also the
    /// errors associated to the `origin` and the `direction` in a tuple
    pub fn inv_transform_ray_propagate_error(
        &self,
        ray: &Ray3D,
        o_error_in: Point3D,
        d_error_in: Point3D,
    ) -> (Ray3D, Point3D, Point3D) {
        let (mut origin, o_error) = self.inv_transform_pt_propagate_error(ray.origin, o_error_in);
        let (direction, d_error) =
            self.inv_transform_vec_propagate_error(ray.direction, d_error_in);

        let l2 = direction.length_squared();
        if l2 > 0. {
            // move origin to account for the error
            let dt = (direction.abs() * o_error) / l2;
            origin += direction * dt;
        }

        let r = Ray3D { origin, direction };
        (r, o_error, d_error)
    }

    /// Transforms a [`BBox3D`] object
    pub fn transform_bbox(&self, bbox: BBox3D) -> BBox3D {
        let p0 = self.transform_pt(Point3D::new(bbox.min.x, bbox.min.y, bbox.min.z));
        let mut ret = BBox3D::from_point(p0);
        ret = BBox3D::from_union_point(
            &ret,
            self.transform_pt(Point3D::new(bbox.max.x, bbox.min.y, bbox.min.z)),
        );
        ret = BBox3D::from_union_point(
            &ret,
            self.transform_pt(Point3D::new(bbox.min.x, bbox.max.y, bbox.min.z)),
        );
        ret = BBox3D::from_union_point(
            &ret,
            self.transform_pt(Point3D::new(bbox.min.x, bbox.min.y, bbox.max.z)),
        );
        ret = BBox3D::from_union_point(
            &ret,
            self.transform_pt(Point3D::new(bbox.min.x, bbox.max.y, bbox.max.z)),
        );
        ret = BBox3D::from_union_point(
            &ret,
            self.transform_pt(Point3D::new(bbox.max.x, bbox.max.y, bbox.min.z)),
        );
        ret = BBox3D::from_union_point(
            &ret,
            self.transform_pt(Point3D::new(bbox.max.x, bbox.min.y, bbox.max.z)),
        );
        ret = BBox3D::from_union_point(
            &ret,
            self.transform_pt(Point3D::new(bbox.max.x, bbox.max.y, bbox.max.z)),
        );
        ret
    }

    /// Applies the inverse of a transform to a [`BBox3D`] object
    pub fn inv_transform_bbox(&self, bbox: BBox3D) -> BBox3D {
        let p0 = self.inv_transform_pt(Point3D::new(bbox.min.x, bbox.min.y, bbox.min.z));
        let mut ret = BBox3D::from_point(p0);
        ret = BBox3D::from_union_point(
            &ret,
            self.inv_transform_pt(Point3D::new(bbox.max.x, bbox.min.y, bbox.min.z)),
        );
        ret = BBox3D::from_union_point(
            &ret,
            self.inv_transform_pt(Point3D::new(bbox.min.x, bbox.max.y, bbox.min.z)),
        );
        ret = BBox3D::from_union_point(
            &ret,
            self.inv_transform_pt(Point3D::new(bbox.min.x, bbox.min.y, bbox.max.z)),
        );
        ret = BBox3D::from_union_point(
            &ret,
            self.inv_transform_pt(Point3D::new(bbox.min.x, bbox.max.y, bbox.max.z)),
        );
        ret = BBox3D::from_union_point(
            &ret,
            self.inv_transform_pt(Point3D::new(bbox.max.x, bbox.max.y, bbox.min.z)),
        );
        ret = BBox3D::from_union_point(
            &ret,
            self.inv_transform_pt(Point3D::new(bbox.max.x, bbox.min.y, bbox.max.z)),
        );
        ret = BBox3D::from_union_point(
            &ret,
            self.inv_transform_pt(Point3D::new(bbox.max.x, bbox.max.y, bbox.max.z)),
        );
        ret
    }
} // end of Impl Transform

#[cfg(test)]
mod testing {
    use super::*;

    #[test]
    fn test_macro_i() {
        assert_eq!(elem!(0, 0), 0);
        assert_eq!(elem!(0, 1), 1);
        assert_eq!(elem!(0, 2), 2);
        assert_eq!(elem!(0, 3), 3);

        assert_eq!(elem!(1, 0), 4);
        assert_eq!(elem!(1, 1), 5);
        assert_eq!(elem!(1, 2), 6);
        assert_eq!(elem!(1, 3), 7);

        assert_eq!(elem!(2, 0), 8);
        assert_eq!(elem!(2, 1), 9);
        assert_eq!(elem!(2, 2), 10);
        assert_eq!(elem!(2, 3), 11);

        assert_eq!(elem!(3, 0), 12);
        assert_eq!(elem!(3, 1), 13);
        assert_eq!(elem!(3, 2), 14);
        assert_eq!(elem!(3, 3), 15);
    }

    #[test]
    #[should_panic]
    fn test_macro_panic_out_of_bounds() {
        assert_eq!(elem!(4, 0), 0);
    }
    #[test]
    #[should_panic]
    fn test_macro_panic_out_of_bounds_2() {
        assert_eq!(elem!(0, 4), 0);
    }
    #[test]
    #[should_panic]
    fn test_macro_panic_negative() {
        assert_eq!(elem!(-1, 0), 0);
    }
    #[test]
    #[should_panic]
    fn test_macro_panic_negative_2() {
        assert_eq!(elem!(0, -1), 0);
    }

    #[test]
    fn test_default_transform() {
        let t = Transform::default();
        for row in 0..4 {
            for col in 0..4 {
                if row == col {
                    assert_eq!(t.elements[elem!(row, col)], 1.0);
                    assert_eq!(t.inv_elements[elem!(row, col)], 1.0);
                } else {
                    assert_eq!(t.elements[elem!(row, col)], 0.0);
                    assert_eq!(t.inv_elements[elem!(row, col)], 0.0);
                }
            }
        }

        fn check(x: Float, y: Float, z: Float) -> Result<(), String> {
            let t = Transform::new();
            let pt = Point3D::new(x, y, z);
            let vec = Vector3D::new(x, y, z);
            if pt != t.transform_pt(pt) {
                return Err(format!(
                    "point {} is not the same as transformed {}",
                    pt,
                    t.transform_pt(pt)
                ));
            }
            if vec != t.transform_vec(vec) {
                return Err(format!(
                    "vec {} is not the same as transformed {}",
                    vec,
                    t.transform_vec(vec)
                ));
            }
            Ok(())
        }

        check(0., 0., 0.).unwrap();
        check(0., 0., 1.).unwrap();
        check(0., 1., 0.).unwrap();
        check(1., 0., 0.).unwrap();
        check(1., 1., 0.).unwrap();
        check(1., 1., 1.).unwrap();
        check(-1., -1., 1.).unwrap();
    }

    #[test]
    fn test_translate_transform() {
        let (x, y, z) = (2.1, -4.2, 19.2);
        let t = Transform::translate(x, y, z);
        for row in 0..4 {
            for col in 0..4 {
                if row == col {
                    assert_eq!(t.elements[elem!(row, col)], 1.0);
                    assert_eq!(t.inv_elements[elem!(row, col)], 1.0);
                } else if col == 3 {
                    if row == 0 {
                        assert_eq!(t.elements[elem!(row, col)], x);
                        assert_eq!(t.inv_elements[elem!(row, col)], -x);
                    } else if row == 1 {
                        assert_eq!(t.elements[elem!(row, col)], y);
                        assert_eq!(t.inv_elements[elem!(row, col)], -y);
                    } else if row == 2 {
                        assert_eq!(t.elements[elem!(row, col)], z);
                        assert_eq!(t.inv_elements[elem!(row, col)], -z);
                    }
                } else {
                    assert_eq!(t.elements[elem!(row, col)], 0.0);
                    assert_eq!(t.inv_elements[elem!(row, col)], 0.0)
                }
            }
        }

        fn check(x: Float, y: Float, z: Float) -> Result<(), String> {
            let t = Transform::translate(x, y, z);

            let pt = Point3D::new(3. + 2. * x, -1. * y + 9., z);
            let vec = Vector3D::new(x, 21. * y, -33. * z);

            let pprime = pt + Vector3D::new(x, y, z);
            let vecprime = vec;

            if pprime != t.transform_pt(pt) {
                return Err(format!(
                    "point {} is not the same as transformed {}",
                    t.transform_pt(pt),
                    pprime
                ));
            }
            if vecprime != t.transform_vec(vec) {
                return Err(format!(
                    "vector {} is not the same as transformed {}",
                    t.transform_vec(vec),
                    vecprime
                ));
            }
            Ok(())
        }

        check(0., 0., 0.).unwrap();
        check(0., 0., 1.).unwrap();
        check(0., 1., 0.).unwrap();
        check(1., 0., 0.).unwrap();
        check(1., 1., 0.).unwrap();
        check(1., 1., 1.).unwrap();
        check(-1., -1., 1.).unwrap();
    }

    #[test]
    fn test_scale_transform() {
        let (x, y, z) = (2.1, -4.2, 19.2);
        let t = Transform::scale(x, y, z);
        for row in 0..4 {
            for col in 0..4 {
                if row == col {
                    if row == 0 {
                        assert_eq!(t.elements[elem!(row, col)], x);
                        assert_eq!(t.inv_elements[elem!(row, col)], 1. / x);
                    } else if row == 1 {
                        assert_eq!(t.elements[elem!(row, col)], y);
                        assert_eq!(t.inv_elements[elem!(row, col)], 1. / y);
                    } else if row == 2 {
                        assert_eq!(t.elements[elem!(row, col)], z);
                        assert_eq!(t.inv_elements[elem!(row, col)], 1. / z);
                    } else {
                        assert_eq!(t.elements[elem!(row, col)], 1.);
                        assert_eq!(t.inv_elements[elem!(row, col)], 1.);
                    }
                } else {
                    assert_eq!(t.elements[elem!(row, col)], 0.0);
                    assert_eq!(t.inv_elements[elem!(row, col)], 0.0)
                }
            }
        }

        fn check(x: Float, y: Float, z: Float) -> Result<(), String> {
            let t = Transform::scale(x, y, z);

            // shufle them a bit
            let pt = Point3D::new(2. * x, y, z);
            let vec = Vector3D::new(x, -1. * y, 3. * z);

            let pprime = Point3D::new(pt.x * x, pt.y * y, pt.z * z);
            let vecprime = Vector3D::new(vec.x * x, vec.y * y, vec.z * z);

            if pprime != t.transform_pt(pt) {
                return Err(format!(
                    "point {} is not the same as transformed {}",
                    pt, pprime
                ));
            }
            if vecprime != t.transform_vec(vec) {
                return Err(format!(
                    "vector {} is not the same as transformed {}",
                    vec, vecprime
                ));
            }
            Ok(())
        }

        check(0., 0., 0.).unwrap();
        check(0., 0., 1.).unwrap();
        check(0., 1., 0.).unwrap();
        check(1., 0., 0.).unwrap();
        check(1., 1., 0.).unwrap();
        check(1., 1., 1.).unwrap();
        check(-1., -1., 1.).unwrap();
    }

    fn compare_pts(pt1: Point3D, pt2: Point3D) -> Result<(), String> {
        if (pt1 - pt2).length() > 1e-6 {
            return Err(format!("Points {} and {} are not equal", pt1, pt2));
        }
        Ok(())
    }
    fn compare_vecs(vec1: Vector3D, vec2: Vector3D) -> Result<(), String> {
        if (vec1 - vec2).length() > 1e-6 {
            return Err(format!("Vectors {} and {} are not equal", vec1, vec2));
        }
        Ok(())
    }
    #[test]
    fn test_rotate_x_transform() {
        /* POINTS */
        let ex = Point3D::new(1., 0., 0.);
        let ey = Point3D::new(0., 1., 0.);
        let ez = Point3D::new(0., 0., 1.);

        let t = Transform::rotate_x(90.);
        compare_pts(t.transform_pt(ex), ex).unwrap();
        compare_pts(t.transform_pt(ey), ez).unwrap();
        compare_pts(t.transform_pt(ez), ey * -1.).unwrap();

        let t = Transform::rotate_x(-90.);
        compare_pts(t.transform_pt(ex), ex).unwrap();
        compare_pts(t.transform_pt(ey), ez * -1.).unwrap();
        compare_pts(t.transform_pt(ez), ey).unwrap();

        let t = Transform::rotate_x(180.);
        compare_pts(t.transform_pt(ex), ex).unwrap();
        compare_pts(t.transform_pt(ey), ey * -1.).unwrap();
        compare_pts(t.transform_pt(ez), ez * -1.).unwrap();

        let t = Transform::rotate_x(45.);
        compare_pts(t.transform_pt(ex), ex).unwrap();
        let sq2 = 1. / (2 as Float).sqrt();
        compare_pts(t.transform_pt(ey), Point3D::new(0., sq2, sq2)).unwrap();
        compare_pts(t.transform_pt(ez), Point3D::new(0., -sq2, sq2)).unwrap();

        /* VECTORS */
        let ex = Vector3D::new(1., 0., 0.);
        let ey = Vector3D::new(0., 1., 0.);
        let ez = Vector3D::new(0., 0., 1.);

        let t = Transform::rotate_x(90.);
        compare_vecs(t.transform_vec(ex), ex).unwrap();
        compare_vecs(t.transform_vec(ey), ez).unwrap();
        compare_vecs(t.transform_vec(ez), ey * -1.).unwrap();

        let t = Transform::rotate_x(-90.);
        compare_vecs(t.transform_vec(ex), ex).unwrap();
        compare_vecs(t.transform_vec(ey), ez * -1.).unwrap();
        compare_vecs(t.transform_vec(ez), ey).unwrap();

        let t = Transform::rotate_x(180.);
        compare_vecs(t.transform_vec(ex), ex).unwrap();
        compare_vecs(t.transform_vec(ey), ey * -1.).unwrap();
        compare_vecs(t.transform_vec(ez), ez * -1.).unwrap();

        let t = Transform::rotate_x(45.);
        compare_vecs(t.transform_vec(ex), ex).unwrap();
        let sq2 = 1. / (2 as Float).sqrt();
        compare_vecs(t.transform_vec(ey), Vector3D::new(0., sq2, sq2)).unwrap();
        compare_vecs(t.transform_vec(ez), Vector3D::new(0., -sq2, sq2)).unwrap();
    }

    #[test]
    fn test_rotate_y_transform() {
        /* POINTS */
        let ex = Point3D::new(1., 0., 0.);
        let ey = Point3D::new(0., 1., 0.);
        let ez = Point3D::new(0., 0., 1.);

        let t = Transform::rotate_y(90.);
        compare_pts(t.transform_pt(ex), ez * -1.).unwrap();
        compare_pts(t.transform_pt(ey), ey).unwrap();
        compare_pts(t.transform_pt(ez), ex).unwrap();

        let t = Transform::rotate_y(-90.);
        compare_pts(t.transform_pt(ex), ez).unwrap();
        compare_pts(t.transform_pt(ey), ey).unwrap();
        compare_pts(t.transform_pt(ez), ex * -1.).unwrap();

        let t = Transform::rotate_y(180.);
        compare_pts(t.transform_pt(ex), ex * -1.).unwrap();
        compare_pts(t.transform_pt(ey), ey).unwrap();
        compare_pts(t.transform_pt(ez), ez * -1.).unwrap();

        let t = Transform::rotate_y(45.);
        let sq2 = 1. / (2 as Float).sqrt();
        compare_pts(t.transform_pt(ex), Point3D::new(sq2, 0., -sq2)).unwrap();
        compare_pts(t.transform_pt(ey), ey).unwrap();
        compare_pts(t.transform_pt(ez), Point3D::new(sq2, 0., sq2)).unwrap();

        /* VECTORS */

        let ex = Vector3D::new(1., 0., 0.);
        let ey = Vector3D::new(0., 1., 0.);
        let ez = Vector3D::new(0., 0., 1.);

        let t = Transform::rotate_y(90.);
        compare_vecs(t.transform_vec(ex), ez * -1.).unwrap();
        compare_vecs(t.transform_vec(ey), ey).unwrap();
        compare_vecs(t.transform_vec(ez), ex).unwrap();

        let t = Transform::rotate_y(-90.);
        compare_vecs(t.transform_vec(ex), ez).unwrap();
        compare_vecs(t.transform_vec(ey), ey).unwrap();
        compare_vecs(t.transform_vec(ez), ex * -1.).unwrap();

        let t = Transform::rotate_y(180.);
        compare_vecs(t.transform_vec(ex), ex * -1.).unwrap();
        compare_vecs(t.transform_vec(ey), ey).unwrap();
        compare_vecs(t.transform_vec(ez), ez * -1.).unwrap();

        let t = Transform::rotate_y(45.);
        let sq2 = 1. / (2 as Float).sqrt();
        compare_vecs(t.transform_vec(ex), Vector3D::new(sq2, 0., -sq2)).unwrap();
        compare_vecs(t.transform_vec(ey), ey).unwrap();
        compare_vecs(t.transform_vec(ez), Vector3D::new(sq2, 0., sq2)).unwrap();
    }

    #[test]
    fn test_rotate_z_transform() {
        /* POINTS */
        let ex = Point3D::new(1., 0., 0.);
        let ey = Point3D::new(0., 1., 0.);
        let ez = Point3D::new(0., 0., 1.);

        let t = Transform::rotate_z(90.);
        compare_pts(t.transform_pt(ex), ey).unwrap();
        compare_pts(t.transform_pt(ey), ex * -1.).unwrap();
        compare_pts(t.transform_pt(ez), ez).unwrap();

        let t = Transform::rotate_z(-90.);
        compare_pts(t.transform_pt(ex), ey * -1.).unwrap();
        compare_pts(t.transform_pt(ey), ex).unwrap();
        compare_pts(t.transform_pt(ez), ez).unwrap();

        let t = Transform::rotate_z(180.);
        compare_pts(t.transform_pt(ex), ex * -1.).unwrap();
        compare_pts(t.transform_pt(ey), ey * -1.).unwrap();
        compare_pts(t.transform_pt(ez), ez).unwrap();

        let t = Transform::rotate_z(45.);
        let sq2 = 1. / (2 as Float).sqrt();
        compare_pts(t.transform_pt(ex), Point3D::new(sq2, sq2, 0.)).unwrap();
        compare_pts(t.transform_pt(ey), Point3D::new(-sq2, sq2, 0.)).unwrap();
        compare_pts(t.transform_pt(ez), ez).unwrap();

        /* VECTORS */
        let ex = Vector3D::new(1., 0., 0.);
        let ey = Vector3D::new(0., 1., 0.);
        let ez = Vector3D::new(0., 0., 1.);

        let t = Transform::rotate_z(90.);
        compare_vecs(t.transform_vec(ex), ey).unwrap();
        compare_vecs(t.transform_vec(ey), ex * -1.).unwrap();
        compare_vecs(t.transform_vec(ez), ez).unwrap();

        let t = Transform::rotate_z(-90.);
        compare_vecs(t.transform_vec(ex), ey * -1.).unwrap();
        compare_vecs(t.transform_vec(ey), ex).unwrap();
        compare_vecs(t.transform_vec(ez), ez).unwrap();

        let t = Transform::rotate_z(180.);
        compare_vecs(t.transform_vec(ex), ex * -1.).unwrap();
        compare_vecs(t.transform_vec(ey), ey * -1.).unwrap();
        compare_vecs(t.transform_vec(ez), ez).unwrap();

        let t = Transform::rotate_z(45.);
        let sq2 = 1. / (2 as Float).sqrt();
        compare_vecs(t.transform_vec(ex), Vector3D::new(sq2, sq2, 0.)).unwrap();
        compare_vecs(t.transform_vec(ey), Vector3D::new(-sq2, sq2, 0.)).unwrap();
        compare_vecs(t.transform_vec(ez), ez).unwrap();
    }

    #[test]
    fn test_mul_4x4() {
        // this is an example from the internet
        let a = [
            5., 7., 9., 10., 2., 3., 3., 8., 8., 10., 2., 3., 3., 3., 4., 8.,
        ];

        let b = [
            3., 10., 12., 18., 12., 1., 4., 9., 9., 10., 12., 2., 3., 12., 4., 10.,
        ];

        let exp = [
            210., 267., 236., 271., 93., 149., 104., 149., 171., 146., 172., 268., 105., 169.,
            128., 169.,
        ];
        let c = mul4x4(&a, &b);
        assert_eq!(c, exp)
    }
}
