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

use crate::intersection::{IntersectionInfo, SurfaceSide};
use crate::RefCount;

use crate::Float;
use crate::{BBox3D, Point3D, Ray3D, Segment3D, Transform, Vector3D};

/// Intersects a [`Ray3D`] in local coordinates with Triangle described by the
/// [`Point3D`] `a`, `b`, and `c`. We use the [Möller–Trumbore intersection algorithm](https://en.wikipedia.org/wiki/Möller–Trumbore_intersection_algorithm).
///
/// Returns the [`Point3D`] of intersection and the values `u` and `v`, indicating
/// the parametric coordinates of the point of intersection
///
pub fn intersect_triangle(
    ray: &Ray3D,
    vertex0: Point3D,
    vertex1: Point3D,
    vertex2: Point3D,
) -> Option<(Point3D, Float, Float)> {
    let edge1 = vertex1 - vertex0;
    let edge2 = vertex2 - vertex0;
    let h = ray.direction.cross(edge2);
    let a = edge1 * h;
    const TINY: Float = 100. * Float::EPSILON;
    if a > -TINY && a < TINY {
        return None;
    }
    let f = 1. / a;
    let s = ray.origin - vertex0;
    let u = f * (s * h);
    if !(0.0..=1.).contains(&u) {
        return None;
    }
    let q = s.cross(edge1);
    let v = f * (ray.direction * q);

    if !(0.0..=1.0).contains(&v) {
        return None;
    }
    let t = f * (edge2 * q);
    if t > TINY {
        return Some((ray.project(t), u, v));
    }
    None
}

/// A simple 3-dimentional Triangle.
#[derive(Clone, Copy, Debug)]
pub struct Triangle3D {
    /// First vertex
    a: Point3D,
    /// Second vertex
    b: Point3D,
    /// Third vertex
    c: Point3D,
    /// The normal, following right-hand convention
    normal: Vector3D,
    /// The area of the triangle
    area: Float,
}

/// Represents the position of a [`Point3D`] in a [`Triangle3D`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum PointInTriangle {
    /// The point is in vertex A
    VertexA,
    /// The point is in vertex B
    VertexB,
    /// The point is in vertex C
    VertexC,
    /// The point is in the edge between A and B
    EdgeAB,
    /// The point is in the edge between B and C
    EdgeBC,
    /// The point is in the edge between A and C
    EdgeAC,
    /// The point is inside of the [`Triangle3D`]
    Inside,
    /// The point is outside of the [`Triangle3D`]
    Outside,
}

impl PointInTriangle {
    /// Checks whether a [`PointInTriangle`] lies in a vertex
    pub fn is_vertex(&self) -> bool {
        matches!(self, Self::VertexA | Self::VertexB | Self::VertexC)
    }

    /// Checks whether a [`PointInTriangle`] lies in an edge
    pub fn is_edge(&self) -> bool {
        matches!(self, Self::EdgeAB | Self::EdgeAC | Self::EdgeBC)
    }
}

impl std::fmt::Display for Triangle3D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Triangle3D[\n\t{}\n\t{}\n\t{}\n]\n",
            self.a, self.b, self.c
        )
    }
}

impl Triangle3D {
    /// Creates a new [`Triangle3D`]
    pub fn new(vertex_a: Point3D, vertex_b: Point3D, vertex_c: Point3D) -> Result<Self, String> {
        // check that points are not the same
        if vertex_a.compare(vertex_b) || vertex_a.compare(vertex_c) || vertex_b.compare(vertex_c) {
            let msg = "Trying to build a Triangle3D with two equal points".to_string();
            return Err(msg);
        }

        // Check that they are not collinear
        if vertex_a.is_collinear(vertex_b, vertex_c)? {
            let msg = "Trying to build a Triangle3D from collinear points".to_string();
            return Err(msg);
        }

        // Build the triangle
        let mut t = Triangle3D {
            a: vertex_a,
            b: vertex_b,
            c: vertex_c,

            area: -1.,
            normal: Vector3D::new(0., 0., 0.),
        };

        t.set_area();
        t.set_normal();

        Ok(t)
    }

    /* Normal */

    /// Calculates the normal of the Triangle
    fn set_normal(&mut self) {
        let b_a = self.b - self.a;
        let c_a = self.c - self.b;
        self.normal = b_a.cross(c_a);
        self.normal.normalize();
    }

    /// Gets the normal of the [`Triangle3D`]. This value is
    /// cached, so this should be fast.
    pub fn normal(&self) -> Vector3D {
        self.normal
    }

    /// Sets the area of the [`Triangle3D`]. This is used when
    /// constructing the [`Triangle3D`]
    fn set_area(&mut self) {
        let ab = self.ab().length();
        let bc = self.bc().length();
        let ca = self.ca().length();

        self.area = ((ca + bc + ab)
            * ((ca + bc + ab) / 2. - ab)
            * ((ca + bc + ab) / 2. - bc)
            * ((ca + bc + ab) / 2. - ca)
            / 2.)
            .sqrt();
    }

    /// Gets the Area of the [`Triangle3D`]... it is cached when creater
    pub fn area(&self) -> Float {
        self.area
    }

    /// Gets the Circumradus of the [`Triangle3D`]. This value
    /// is not cached, so it might be slow to rely on this
    /// too often.
    pub fn circumradius(&self) -> Float {
        let a = self.ab().length();
        let b = self.bc().length();
        let c = self.ca().length();
        let s = (a + b + c) * (b + c - a) * (c + a - b) * (a + b - c);

        #[cfg(not(feature = "quick_inv_sqrt"))]
        return a * b * c / s.sqrt();

        #[cfg(feature = "quick_inv_sqrt")]
        return a * b * c * crate::quick_inverse_sqrt::quick_inv_sqrt(s);
    }

    /// Gets the Aspect Ratio of the [`Triangle3D`]. This is useful
    /// for triangulating polygons.
    pub fn aspect_ratio(&self) -> Float {
        let mut min_segment = 1E19;
        for i in 0..3 {
            let s = self.segment(i).unwrap();
            if s.length() < min_segment {
                min_segment = s.length();
            }
        }
        // return
        self.circumradius() / min_segment
    }

    /// retrieves the Circumcenter
    pub fn circumcenter(&self) -> Point3D {
        let ab = self.b - self.a;
        let ac = self.c - self.a;

        let ab_cross_ac = ab.cross(ac);

        let ac_sq_length = ac.length() * ac.length();
        let ab_sq_length = ab.length() * ab.length();
        let ab_x_ac_sq_length = ab_cross_ac.length() * ab_cross_ac.length();

        let a_center = ((ab_cross_ac.cross(ab)) * ac_sq_length
            + ac.cross(ab_cross_ac) * ab_sq_length)
            / (2. * ab_x_ac_sq_length);

        self.a + a_center
    }

    /// Gets the centroid of the [`Triangle3D`]. This value
    /// is not cached, so it might be slow to rely on this
    /// too often.
    pub fn centroid(&self) -> Point3D {
        let dx = self.a.x + self.b.x + self.c.x;
        let dy = self.a.y + self.b.y + self.c.y;
        let dz = self.a.z + self.b.z + self.c.z;

        Point3D::new(dx / 3., dy / 3., dz / 3.)
    }

    /// Gets  a specific vertex. Returns an error if the number
    /// of the vertex is greater than 2 (i.e., valid vertices are 0, 1, 2)
    pub fn vertex(&self, i: usize) -> Result<Point3D, String> {
        match i {
            0 => Ok(self.a),
            1 => Ok(self.b),
            2 => Ok(self.c),
            _ => Err(format!(
                "Trying to get vertex with index '{}'... it is out of bounds",
                i
            )),
        }
    }

    /// Retrieves the first vertex of an A,B,C [`Triangle3D`]
    pub fn a(&self) -> Point3D {
        self.a
    }

    /// Retrieves the second vertex of an A,B,C [`Triangle3D`]
    pub fn b(&self) -> Point3D {
        self.b
    }

    /// Retrieves the third vertex of an A,B,C [`Triangle3D`]
    pub fn c(&self) -> Point3D {
        self.c
    }

    /// Sets the first vertex of an A,B,C [`Triangle3D`],
    /// updating the triangle's normal and area.
    pub fn set_a(&mut self, p: Point3D) {
        self.a = p;
        self.set_area();
        self.set_normal();
    }

    /// Sets the first vertex of an A,B,C [`Triangle3D`],
    /// updating the triangle's normal and area.
    pub fn set_b(&mut self, p: Point3D) {
        self.b = p;
        self.set_area();
        self.set_normal();
    }

    /// Sets the first vertex of an A,B,C [`Triangle3D`],
    /// updating the triangle's normal and area.
    pub fn set_c(&mut self, p: Point3D) {
        self.c = p;
        self.set_area();
        self.set_normal();
    }

    /// Retrieves a segment of an A,B,C [`Triangle3D`]
    pub fn segment(&self, i: usize) -> Result<Segment3D, String> {
        match i {
            0 => Ok(self.ab()),
            1 => Ok(self.bc()),
            2 => Ok(self.ca()),
            _ => Err(format!(
                "Trying to get Segment with index '{}'... it is out of bounds",
                i
            )),
        }
    }

    /// Retrieves the first segment of an A,B,C [`Triangle3D`]
    pub fn ab(&self) -> Segment3D {
        Segment3D::new(self.a, self.b)
    }

    /// Retrieves the second segment of an A,B,C [`Triangle3D`]
    pub fn bc(&self) -> Segment3D {
        Segment3D::new(self.b, self.c)
    }

    /// Retrieves the third segment of an A,B,C [`Triangle3D`]
    pub fn ca(&self) -> Segment3D {
        Segment3D::new(self.c, self.a)
    }

    /// Tests whether a point—which is assumed to be within the plane
    /// of the triangle—is inside the triangle.
    ///
    /// # Explanation
    /// This is not something I made up. I found it [here](http://blackpawn.com/texts/pointinpoly/)
    /// and also in other places. (the explanation is here not as novelty)
    ///  
    /// If a triangle in 3 dimentions as vertices $`\vec{A}`$, $`\vec{B}`$, and $`\vec{C}`$, then any
    /// point in the plane of the triangle can be described by the following equation:
    ///
    /// ```math
    /// \vec{P} = \vec{A} + \alpha \left(\vec{B}-\vec{A}\right)+\beta \left(\vec{C}-\vec{A}\right)
    /// ```
    /// Or, after stating that $`\hat{e}_1=\left(\vec{B}-\vec{A}\right)`$ and $`\hat{e}_2 = \left(\vec{C}-\vec{A}\right)`$
    /// and rearranging, we can express the same equation as:
    ///     
    /// ```math
    /// \vec{P} - \vec{A}=  \alpha \hat{e}_1+\beta \hat{e}_2
    /// ```
    ///
    /// Since this is a 3D equation,  it is actually three equations and there are only two incognita
    /// ($`\alpha`$ and $`\beta`$). So, we can multiply it by $`\hat{e}_1`$ and $`\hat{e}_2`$ and produce
    /// a system of equations that we can solve. After doing this, the equation above becomes:
    ///
    /// ```math
    /// \hat{e}_1\left(\vec{P} - \vec{A}\right)=  \alpha \hat{e}_1\hat{e}_1+\beta \hat{e}_1\hat{e}_2
    /// ```
    /// and
    /// ```math
    /// \hat{e}_2\left(\vec{P} - \vec{A}\right)=  \alpha \hat{e}_1\hat{e}_2+\beta \hat{e}_2\hat{e}_2
    /// ```
    ///
    /// Which can be represented in matrix form as follows:
    ///
    /// ```math
    /// \begin{Bmatrix}\hat{e}_1\left(\vec{P} - \vec{A}\right)\\\hat{e}_2\left(\vec{P} - \vec{A}\right)\end{Bmatrix}=\begin{pmatrix}\hat{e}_1\hat{e}_1&\hat{e}_1\hat{e}_2\\\hat{e}_1\hat{e}_2&\hat{e}_2\hat{e}_2\end{pmatrix}\begin{Bmatrix}\alpha\\\beta\end{Bmatrix}
    /// ```
    ///
    /// What is left from here is to interpret the values of $`\alpha`$ and $`\beta`$ from the equations,
    /// which allows us to position the point within the triangle.
    pub fn test_point(&self, p: Point3D) -> PointInTriangle {
        // get vertices
        let vertex_a = self.a;
        let vertex_b = self.b;
        let vertex_c = self.c;

        // Compute vectors
        let e1 = vertex_b - vertex_a;
        let e2 = vertex_c - vertex_a;
        let p_minus_a = p - vertex_a;

        // Compute dot products
        let e2e2 = e2 * e2;
        let e1e2 = e2 * e1;
        let e1e1 = e1 * e1;

        let left1 = e1 * p_minus_a;
        let left2 = e2 * p_minus_a;

        // Compute barycentric coordinates
        let det = e1e1 * e2e2 - e1e2 * e1e2;
        let alpha = (e2e2 * left1 - e1e2 * left2) / det;
        let beta = (-e1e2 * left1 + e1e1 * left2) / det;
        let w = 1. - alpha - beta;

        // Check if point is in triangle
        const TINY: Float = 100. * Float::EPSILON;
        if alpha >= -TINY && beta >= -TINY && w >= -TINY {
            // Somewhere in the triangle
            if alpha <= TINY && beta <= TINY {
                // w is 1... Vertex A
                PointInTriangle::VertexA
            } else if alpha <= TINY && w <= TINY {
                // beta is 1; Vertex C
                PointInTriangle::VertexC
            } else if beta <= TINY && w <= TINY {
                // Alpha is 1; Vertex B
                PointInTriangle::VertexB
            } else if alpha <= TINY {
                PointInTriangle::EdgeAC
            } else if w <= TINY {
                PointInTriangle::EdgeBC
            } else if beta <= TINY {
                PointInTriangle::EdgeAB
            } else {
                PointInTriangle::Inside
            }
        } else {
            PointInTriangle::Outside
        }
    }

    /// Gets the edge number corresponding to a [`Segment3D`] in the
    /// Triangle.
    pub fn get_edge_index_from_segment(&self, s: &Segment3D) -> Option<usize> {
        if s.compare(&self.ab()) {
            Some(0)
        } else if s.compare(&self.bc()) {
            Some(1)
        } else if s.compare(&self.ca()) {
            Some(2)
        } else {
            None
        }
    }

    /// Retreives a certain edge based on two vertices
    pub fn get_edge_index_from_points(&self, a: Point3D, b: Point3D) -> Option<usize> {
        let segment = Segment3D::new(a, b);
        self.get_edge_index_from_segment(&segment)
    }

    /// Checks if a [`Triangle3D`] contains a certain Vertex in
    /// the same position as a [`Point3D`] `p`
    pub fn has_vertex(&self, p: Point3D) -> bool {
        self.a.compare(p) || self.b.compare(p) || self.c.compare(p)
    }

    /// Checks whether three [`Triangle3D`] objects are made
    /// of he same vertices.
    ///
    /// Vertices do not need to be in the same order,
    /// necessarily... So, check if the three vertices
    /// are in the other triangle.
    pub fn compare(&self, t: &Triangle3D) -> bool {
        // is this faster than iterate?
        if !t.has_vertex(self.a) {
            return false;
        }
        if !t.has_vertex(self.b) {
            return false;
        }
        if !t.has_vertex(self.c) {
            return false;
        }
        true
    }

    /// Intersects a [`Ray3D`] in local coordinates with the [`Triangle3D`]. Returns the
    /// a [`Point3D`] of intersection and the values `u` and `v`, indicating
    /// the parametric coordinates of the point of intersection
    pub fn basic_intersection(
        &self,
        ray: &Ray3D,
        _o_error: Point3D,
        _d_error: Point3D,
    ) -> Option<(Point3D, Float, Float)> {
        intersect_triangle(ray, self.a(), self.b(), self.c())
    }

    /// The name of the figure. Useful for debugging.
    pub fn id(&self) -> &'static str {
        "triangle"
    }

    /// Gets a `BBox3D` bounding the object, in local coordinates
    pub fn bounds(&self) -> BBox3D {
        let bbox = BBox3D::from_point(self.a);
        let bbox = BBox3D::from_union_point(&bbox, self.b);
        BBox3D::from_union_point(&bbox, self.c)
    }

    /// Borrows the [`Transform`]
    pub fn transform(&self) -> &Option<RefCount<Transform>> {
        &None
    }

    /// Intersects an object with a [`Ray3D]` (IN LOCAL COORDINATES) traveling forward, returning the distance
    /// `t` and the normal [`Vector3D`] at that point. If the distance
    /// is negative (i.e., the object is behind the plane), it should return
    /// [`None`].    
    ///
    ///  The steps should be:
    /// * Calculate the point of intersection
    /// * Calculate normal and side of intersection
    /// * Calculate (u,v)
    /// * Calculate first derivative (i.e., dp_du, dp_dv)
    /// * Calculate second derivative (i.e., d2p_duu, d2p_dvv, d2p_duv)
    /// * Call `IntersectionInfo::new(ray,p, u, v, dpdu, dpdv, d2p_duu, d2p_dvv, d2p_duv)`
    pub fn intersect_local_ray(
        &self,
        ray: &Ray3D,
        o_error: Point3D,
        d_error: Point3D,
    ) -> Option<IntersectionInfo> {
        let (phit, _u, _v) = self.basic_intersection(ray, o_error, d_error)?;
        // eprintln!("Ray is {:?}", ray);
        // eprintln!("Triangle is {}", self);
        // eprintln!("phit = {} | u = {} | v = {}", phit, u, v);
        let dpdu = self.b() - self.a();
        let dpdv = self.c() - self.a();
        // eprintln!("dpdu = {} | dpdv = {}", dpdu, dpdv);
        let normal = dpdu.cross(dpdv).get_normalized();
        // eprintln!("normal = {}", normal);
        let (normal, side) = SurfaceSide::get_side(normal, ray.direction);

        Some(IntersectionInfo {
            p: phit,
            dpdu,
            dpdv,
            normal,
            side,
            #[cfg(feature = "textures")]
            u: _u,
            #[cfg(feature = "textures")]
            v: _u,
            #[cfg(feature = "textures")]
            dndu: Vector3D::new(0., 0., 0.),
            #[cfg(feature = "textures")]
            dndv: Vector3D::new(0., 0., 0.),
        })
    }

    /// Like `intersect_local_ray` but simplified because there is not need
    /// for calcuating the paramtrisized elements
    pub fn simple_intersect_local_ray(
        &self,
        ray: &Ray3D,
        o_error: Point3D,
        d_error: Point3D,
    ) -> Option<Point3D> {
        let (p, _u, _v) = self.basic_intersection(ray, o_error, d_error)?;
        Some(p)
    }

    /// Intersects an object with a [`Ray3D]` (IN WORLD COORDINATES) traveling forward,
    /// returning a detailed [`IntersectionInfo`] about the intersaction .
    pub fn intersect(&self, ray: &Ray3D) -> Option<IntersectionInfo> {
        // Transform ray into object space, if needed
        let (local_ray, o_error, d_error) = if let Some(t) = self.transform() {
            t.inv_transform_ray(ray)
        } else {
            (*ray, Point3D::new(0., 0., 0.), Point3D::new(0., 0., 0.))
        };

        // Intersect, and return transformed back... if needed
        match self.intersect_local_ray(&local_ray, o_error, d_error) {
            None => None,
            Some(info) => {
                if let Some(t) = self.transform() {
                    Some(info.transform(t))
                } else {
                    Some(info)
                }
            }
        }
    }

    /// Intersects an object with a [`Ray3D]` (IN WORLD COORDINATES) traveling forward,
    /// returning the point of intersection, if any.
    pub fn simple_intersect(&self, ray: &Ray3D) -> Option<Point3D> {
        // Transform ray into object space, if needed
        let (local_ray, o_error, d_error) = if let Some(t) = self.transform() {
            t.inv_transform_ray(ray)
        } else {
            let t = Transform::new();
            t.inv_transform_ray(ray)
        };

        // Intersect, and return transformed back... if needed
        match self.simple_intersect_local_ray(&local_ray, o_error, d_error) {
            None => None,
            Some(phit) => {
                if let Some(t) = self.transform() {
                    Some(t.transform_pt(phit))
                } else {
                    Some(phit)
                }
            }
        }
    }

    /// Gets a `BBox3D` bounding the object, in world's coordinates.
    pub fn world_bounds(&self) -> BBox3D {
        let local_b = self.bounds();
        match self.transform() {
            Some(t) => t.transform_bbox(local_b),
            None => local_b,
        }
    }
} // end of impl Triangle3D

/***********/
/* TESTING */
/***********/

#[cfg(test)]
mod testing {
    use super::*;

    #[test]
    fn test_new() -> Result<(), String> {
        // Buid triangle
        // This is a right triangle, so it fits
        // within a circle, and has a circumradius equals
        // half of its hypotenuse
        let a = Point3D::new(0., 0., 0.);
        let b = Point3D::new(6., 0., 0.);
        let c = Point3D::new(0., 8., 0.);

        let t = Triangle3D::new(a, b, c)?;

        // Test vertices
        assert!(t.a.compare(a));
        assert!(t.b.compare(b));
        assert!(t.c.compare(c));

        let a = t.vertex(0)?;
        assert!(a.compare(a));
        let b = t.vertex(1)?;
        assert!(b.compare(b));
        let c = t.vertex(2)?;
        assert!(c.compare(c));

        // test segments
        let ab = t.segment(0)?;
        let ab_ref = Segment3D::new(a, b);
        assert!(ab.compare(&ab_ref));

        let bc = t.segment(1)?;
        let bc_ref = Segment3D::new(b, c);
        assert!(bc.compare(&bc_ref));

        let ca = t.segment(2)?;
        let ca_ref = Segment3D::new(c, a);
        assert!(ca.compare(&ca_ref));

        // Check that getting out of bounds is error
        assert!(t.vertex(3).is_err());

        // Check that the area is OK
        assert!((t.area() - 6. * 8. / 2.).abs() < 1E-10);

        // Check circumradius
        assert!((t.circumradius() - 5.).abs() < 0.001);

        // Check aspect ratio; circumradius/smallest side
        assert!((t.aspect_ratio() - 5. / 6.).abs() < 0.001);

        // Check circumcenter
        assert!(t.circumcenter().compare(Point3D::new(3., 4., 0.)));

        // check centroid
        assert!(t.centroid().compare(Point3D::new(6. / 3., 8. / 3., 0.)));
        Ok(())
    }

    #[test]
    fn test_triangle_intersect() -> Result<(), String> {
        let a = Point3D::new(0., 0., 0.);
        let b = Point3D::new(1., 0., 0.);
        let c = Point3D::new(0., 1., 0.);

        let triangle = Triangle3D::new(a, b, c)?;

        let test_hit = |pt: Point3D, offset: Vector3D, expect_hit: bool| -> Result<(), String> {
            let offset = offset.get_normalized();
            let ray = Ray3D {
                origin: pt + offset,
                direction: offset * -1.,
            };

            if let Some(phit) = triangle.simple_intersect(&ray) {
                if !expect_hit {
                    return Err(format!(
                        "Was NOT expecting hit: pt = {}, offset = {}",
                        pt, offset
                    ));
                }
                if !phit.compare(pt) {
                    return Err(format!(
                        "Hit in incorrect point...: pt = {}, offset = {}, phit = {}",
                        pt, offset, phit
                    ));
                }
            } else {
                if expect_hit {
                    return Err(format!(
                        "WAS expecting hit: pt = {}, offset = {}",
                        pt, offset
                    ));
                }
            }

            Ok(())
        }; // end of closure

        let offset = Vector3D::new(0., 0., -1.);
        // Vertex A.
        test_hit(triangle.a(), offset, true)?;

        // Vertex B.
        test_hit(triangle.b(), offset, true)?;

        // Vertex C.
        test_hit(triangle.c(), offset, true)?;

        // Segment AB.
        let p = Point3D::new(0.5, 0., 0.);
        test_hit(p, offset, true)?;

        // Segment AC.
        let p = Point3D::new(0., 0.5, 0.);
        test_hit(p, offset, true)?;

        // Segment BC.
        let p = Point3D::new(0.5, 0.5, 0.);
        test_hit(p, offset, true)?;

        // Point outside
        let p = Point3D::new(0., -1., 0.);
        test_hit(p, offset, false)?;

        // Point inside
        let p = Point3D::new(0.1, 0.1, 0.);
        test_hit(p, offset, true)?;

        Ok(())
    }

    #[test]
    fn test_test_point() -> Result<(), String> {
        let a = Point3D::new(0., 0., 0.);
        let b = Point3D::new(1., 0., 0.);
        let c = Point3D::new(0., 1., 0.);

        let triangle = Triangle3D::new(a, b, c)?;

        // Vertex A.
        assert_eq!(PointInTriangle::VertexA, triangle.test_point(a));

        // Vertex B.
        assert_eq!(PointInTriangle::VertexB, triangle.test_point(b));

        // Vertex C.
        assert_eq!(PointInTriangle::VertexC, triangle.test_point(c));

        // Segment AB.
        let p = Point3D::new(0.5, 0., 0.);
        assert_eq!(PointInTriangle::EdgeAB, triangle.test_point(p));

        // Segment AC.
        let p = Point3D::new(0., 0.5, 0.);
        assert_eq!(PointInTriangle::EdgeAC, triangle.test_point(p));

        // Segment BC.
        let p = Point3D::new(0.5, 0.5, 0.);
        assert_eq!(PointInTriangle::EdgeBC, triangle.test_point(p));

        // Point outside
        let p = Point3D::new(0., -1., 0.);
        assert_eq!(PointInTriangle::Outside, triangle.test_point(p));

        // Point inside
        let p = Point3D::new(0.1, 0.1, 0.);
        assert_eq!(PointInTriangle::Inside, triangle.test_point(p));

        // CASE 2
        let a = Point3D::new(2., 2., 0.);
        let b = Point3D::new(0., 0., 0.);
        let c = Point3D::new(4., 0., 0.);
        let triangle = Triangle3D::new(a, b, c)?;
        let p = Point3D::new(2., 0., 0.);

        assert_eq!(PointInTriangle::EdgeBC, triangle.test_point(p));
        Ok(())
    }

    #[test]
    fn test_circumcenter() -> Result<(), String> {
        // TEST 1... centerd in origin
        let mut l = 3.;
        let a = Point3D::new(-l, 0., 0.);
        let b = Point3D::new(l, 0., 0.);
        let c = Point3D::new(0., l, 0.);

        let t = Triangle3D::new(a, b, c)?;

        let center1 = t.circumcenter();
        assert!(center1.compare(Point3D::new(0., 0., 0.)));

        // Test 2: Centered somewhere else
        l = 3.;
        let cx = 23.;
        let cy = 21.;
        let cz = 3.12314;
        let a2 = Point3D::new(-l + cx, 0. + cy, cz);
        let b2 = Point3D::new(l + cx, 0. + cy, cz);
        let c2 = Point3D::new(0. + cx, l + cy, cz);

        let t2 = Triangle3D::new(a2, b2, c2)?;

        let center2 = t2.circumcenter();

        assert!(center2.compare(Point3D::new(cx, cy, cz)));

        // Test 3: From internet solved problem
        let a3 = Point3D::new(3., 2., 0.);
        let b3 = Point3D::new(1., 4., 0.);
        let c3 = Point3D::new(5., 4., 0.);

        let t3 = Triangle3D::new(a3, b3, c3)?;
        let center3 = t3.circumcenter();
        assert!(center3.compare(Point3D::new(3., 4., 0.)));

        Ok(())
    }

    #[test]
    fn test_get_edge_index_from_points() -> Result<(), String> {
        let a = Point3D::new(0., 0., 0.);
        let b = Point3D::new(6., 0., 0.);
        let c = Point3D::new(0., 8., 0.);

        let t = Triangle3D::new(a, b, c)?;

        assert_eq!(
            t.get_edge_index_from_segment(&Segment3D::new(a, b)),
            Some(0)
        );
        assert_eq!(
            t.get_edge_index_from_segment(&Segment3D::new(b, a)),
            Some(0)
        );
        assert_eq!(
            t.get_edge_index_from_segment(&Segment3D::new(b, c)),
            Some(1)
        );
        assert_eq!(
            t.get_edge_index_from_segment(&Segment3D::new(c, b)),
            Some(1)
        );
        assert_eq!(
            t.get_edge_index_from_segment(&Segment3D::new(c, a)),
            Some(2)
        );
        assert_eq!(
            t.get_edge_index_from_segment(&Segment3D::new(a, c)),
            Some(2)
        );

        // test segments
        let ab = t.segment(0)?;
        let ab_ref = Segment3D::new(a, b);
        assert!(ab.compare(&ab_ref));

        let bc = t.segment(1)?;
        let bc_ref = Segment3D::new(b, c);
        assert!(bc.compare(&bc_ref));

        let ca = t.segment(2)?;
        let ca_ref = Segment3D::new(c, a);
        assert!(ca.compare(&ca_ref));
        Ok(())
    }

    #[test]
    fn test_has_vertex() -> Result<(), String> {
        let a = Point3D::new(0., 0., 0.);
        let b = Point3D::new(6., 0., 0.);
        let c = Point3D::new(0., 8., 0.);

        let not_there = Point3D::new(12., 1., 99.);

        let t = Triangle3D::new(a, b, c)?;

        assert!(t.has_vertex(a));
        assert!(t.has_vertex(b));
        assert!(t.has_vertex(c));

        assert!(!t.has_vertex(not_there));

        let t2 = Triangle3D::new(a, c, b)?;
        assert!(t.compare(&t2));

        let t3 = Triangle3D::new(a, c, not_there)?;
        assert!(!t.compare(&t3));

        Ok(())
    }
}
