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

use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::{BBox3D, Float};
use crate::{Point3D, Segment3D, Vector3D};

/// A set of [`Point3D`] in sequence, forming a closed loop.
/// It has some particularities.
///
/// ```
/// use geometry::{Loop3D, Point3D};
/// let mut the_loop = Loop3D::new();
/// assert!(the_loop.is_empty());
/// let l = 0.5;
///
/// the_loop.push(Point3D::new(-l, -l, 0.)).unwrap();
/// the_loop.push(Point3D::new(-l, l, 0.)).unwrap();
/// the_loop.push(Point3D::new(l, l, 0.)).unwrap();
/// the_loop.push(Point3D::new(l, -l, 0.)).unwrap();
///
/// assert!(the_loop.area().is_err());
/// the_loop.close().unwrap();
///
/// let a = the_loop.area().unwrap();
/// assert!((4. * l * l - a).abs() < 0.0001);
/// ```
/// # Note:
/// It has some peculiarities. For instance, it attempts
/// to reduce the number of points on a [`Loop3D`]. This is done by
/// identifying when a colinear [`Point3D`] is to be added and, instead
/// of extending the [`Loop3D`], replacing the last element.
///
/// ```
/// use geometry::{Loop3D, Point3D};
/// let mut the_loop = Loop3D::new();
///
/// the_loop.push(Point3D::new(0., 0., 0.)).unwrap();
/// the_loop.push(Point3D::new(1., 1., 0.)).unwrap();
/// assert_eq!(2, the_loop.len());
///
/// // Adding a collinear point will not extend.
/// let collinear = Point3D::new(2., 2., 0.);
/// the_loop.push(collinear).unwrap();
/// assert_eq!(2, the_loop.len());
/// assert_eq!(the_loop[1], collinear);
/// ```
///
#[derive(Debug, Clone)]
pub struct Loop3D {
    /// The points of the [`Loop3D`]
    vertices: Vec<Point3D>,

    /// The normal, following a right-hand-side convention
    normal: Vector3D,

    /// A flag indicating whether the [`Loop3D`] is considered finished or not.
    closed: bool,

    /// The area of the [`Loop3D`], only calculated when closing it
    area: Float,

    /// The perimeter of the loop
    perimeter: Float,
}

impl std::iter::IntoIterator for Loop3D {
    type Item = Point3D;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.vertices.into_iter()
    }
}

impl Serialize for Loop3D {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(3 * self.vertices.len()))?;
        for Point3D { x, y, z } in self.vertices.iter() {
            seq.serialize_element(x)?;
            seq.serialize_element(y)?;
            seq.serialize_element(z)?;
        }

        seq.end()
    }
}

impl<'de> Deserialize<'de> for Loop3D {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let data: Value = Deserialize::deserialize(deserializer)?;
        let mut ret = Self::new();

        if let Value::Array(a) = data {
            let mut it = a.iter();

            while let Some(x) = it.next() {
                let x = match x {
                    Value::Number(x) => x.as_f64().unwrap() as Float,
                    _ => panic!("Expecting Polygon3D to be an array of numbers"),
                };
                let y = it.next();
                let y = match y {
                    Some(Value::Number(y)) => y.as_f64().unwrap() as Float,
                    _ => panic!("Expecting Polygon3D to be an array of numbers"),
                };
                let z = it.next();
                let z = match z {
                    Some(Value::Number(z)) => z.as_f64().unwrap() as Float,
                    _ => panic!("Expecting Polygon3D to be an array of numbers"),
                };
                ret.push(Point3D { x, y, z }).unwrap();
            }
        }

        ret.close().unwrap();

        Ok(ret)
    }
}

impl Default for Loop3D {
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::Index<usize> for Loop3D {
    type Output = Point3D;

    fn index(&self, index: usize) -> &Self::Output {
        if index >= self.vertices.len() {
            panic!(
                "Trying to get a vertex out of bounds... {} available, index was {}",
                self.vertices.len(),
                index
            );
        }
        &self.vertices[index]
    }
}

impl Loop3D {
    /// Creates a new and empty [`Loop3D`]. It has a Zero Normal and an
    /// area of -1. These attributes are filled automatically when pushing
    /// vertices into the Loop.
    pub fn new() -> Loop3D {
        Loop3D {
            vertices: Vec::new(),
            normal: Vector3D::new(0., 0., 0.),
            closed: false,
            area: -1.0,
            perimeter: -1.0,
        }
    }

    /// Reverses the order of the vertices in the [`Loop3D`]
    /// and also the normal.
    ///
    /// # Example
    /// ```
    /// use geometry::{Loop3D, Point3D};
    ///
    /// let mut l = Loop3D::with_capacity(4);
    /// let a = Point3D::new(0., 0., 0.);
    /// let b = Point3D::new(1., 1., 0.);
    /// let c = Point3D::new(0., 1., 0.);
    ///
    /// l.push(a).unwrap();
    /// l.push(b).unwrap();
    /// l.push(c).unwrap();
    ///
    /// let normal = l.normal();
    ///
    /// // reverse
    /// l.reverse();
    ///
    /// let v = l.vertices();
    /// assert!((normal * -1.0).compare(l.normal()));
    /// assert!(a.compare(v[2]));
    /// assert!(b.compare(v[1]));
    /// assert!(c.compare(v[0]));
    ///
    /// ```
    pub fn reverse(&mut self) {
        self.normal *= -1.0;
        self.vertices = self.vertices.iter().rev().copied().collect();
    }

    /// Returns a clone of the [`Loop3D`] but reversed (vertices in the
    /// opposite order, and the normal [`Vector3D`] pointing on the
    /// opposite direction)      
    ///
    /// # Example
    /// ```
    /// use geometry::{Loop3D, Point3D};
    ///
    /// let mut l = Loop3D::with_capacity(4);
    /// let a = Point3D::new(0., 0., 0.);
    /// let b = Point3D::new(1., 1., 0.);
    /// let c = Point3D::new(0., 1., 0.);
    ///
    /// l.push(a).unwrap();
    /// l.push(b).unwrap();
    /// l.push(c).unwrap();
    ///    
    /// // reverse
    /// let rev_l = l.get_reversed();
    ///
    /// assert_eq!(rev_l.len(), l.len());
    /// assert!((l.normal()*-1.0).compare(rev_l.normal()));
    /// let v = l.vertices();
    /// let rev_v = rev_l.vertices();
    /// let n = v.len();
    ///
    /// for i in 0..n {
    ///     let p = v[i];
    ///     let rev_p = rev_v[n-1-i];
    ///     assert!(p.compare(rev_p));
    /// }
    ///
    /// ```   
    pub fn get_reversed(&self) -> Self {
        let mut ret = self.clone();
        ret.reverse();
        ret
    }

    /// Creates a new and empty [`Loop3D`] with a specific `capacity`. It has a Zero Normal and an
    /// area of -1. These attributes are filled automatically when pushing
    /// vertices into the Loop.
    pub fn with_capacity(capacity: usize) -> Loop3D {
        Loop3D {
            vertices: Vec::with_capacity(capacity),
            normal: Vector3D::new(0., 0., 0.),
            closed: false,
            area: -1.0,
            perimeter: -1.0,
        }
    }

    /// is closed?
    pub fn closed(&self) -> bool {
        self.closed
    }

    /// Gets a [`BBox3D`] containing the `Loop3D`
    pub fn bbox(&self) -> Result<BBox3D, String> {
        if self.vertices.is_empty() {
            return Err("Trying to get a BBox3D of an empty Loop3D".to_string());
        }
        let first_point = self.vertices[0]; // safe because we know this exists
        let mut ret = BBox3D::from_point(first_point);

        for v in self.vertices.iter().skip(1) {
            ret = BBox3D::from_union_point(&ret, *v);
        }

        Ok(ret)
    }

    /// Creates a clone of `self`, removing the
    /// collinear points.
    ///
    /// The returned [`Loop3D`] will be closed if `self`
    /// is closed and it has more than 3 vertices (it might not
    /// happen, as loops can be modified after closed... not very
    /// safe, but possible)
    ///
    /// # Example
    ///
    /// ```
    /// use geometry::{Loop3D, Point3D};
    ///
    /// let mut l = Loop3D::with_capacity(4);
    /// // Add a triangle
    /// l.push(Point3D::new(0., 0., 0.)).unwrap();
    /// l.push(Point3D::new(1., 1., 0.)).unwrap();
    /// l.push(Point3D::new(0., 1., 0.)).unwrap();
    /// l.push(Point3D::new(0., 0.5, 0.)).unwrap();
    ///
    /// l = l.sanitize().unwrap();    
    /// ```
    pub fn sanitize(self) -> Result<Self, String> {
        let mut new = Self::with_capacity(self.len());
        for v in self.vertices.iter() {
            new.push(*v)?;
        }
        if self.closed && new.vertices.len() >= 3 {
            new.close()?
        }
        Ok(new)
    }

    /// Checks if the [`Loop3D`] has Zero vertices
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }

    /// Borrows the vertices
    pub fn vertices(&self) -> &[Point3D] {
        &self.vertices
    }

    /// Borrows a mutable reference to the vertices
    pub fn mut_vertices(&mut self) -> &mut Vec<Point3D> {
        &mut self.vertices
    }

    /// Removes a vertex
    pub fn remove(&mut self, i: usize) {
        self.vertices.remove(i);
    }

    /// Checks whether a [`Point3D`] can be added to a [`Loop3D`] while keeping
    /// it valid
    ///
    /// It checks that:
    /// * That the [`Loop3D`] has not been closed yet
    /// * that the new [`Point3D`] is coplanar witht he rest of the [`Point3D`]    
    /// * Adding the [`Point3D`] will not make the [`Loop3D`] intersect with itself
    fn valid_to_add(&self, point: Point3D) -> Result<(), String> {
        //check if it is closed
        if self.closed {
            return Err("Trying to add a point to a closed Loop3D".to_string());
        }

        // Check if we can already validate coplanarity
        if !self.normal.is_zero() {
            // Normal should be there.
            if !self.is_coplanar(point)? {
                return Err("Trying to add a non-coplanar point to Loop3D".to_string());
            }
        }

        // Check if the new vertex would make the loop intersect with itself
        let n = self.vertices.len();
        if n >= 3 {
            let last_v = self.vertices[n - 1];
            let new_edge = Segment3D::new(last_v, point);
            let mut intersect = Point3D::new(0., 0., 0.); // dummy variable
            for i in 0..n - 2 {
                let v = self.vertices[i];
                let v_p1 = self.vertices[i + 1];
                let this_s = Segment3D::new(v, v_p1);
                // Check the point of intersection.
                if new_edge.intersect(&this_s, &mut intersect) {
                    return Err(
                        "Trying to push a point that would make the Loop3D intersect with itself"
                            .to_string(),
                    );
                }
            }
        }
        Ok(())
    }

    /// Pushes a new [`Point3D`] into the [`Loop3D`].
    ///
    /// If the [`Point3D`] being
    /// pushed is collinear with the previous two, then instead of pushing a new
    /// [`Point3D`] it will update the last one (i.e., because the shape and area)
    /// of the [`Loop3D`] will still be the same.
    ///
    /// Returns an error if the point being added would make the [`Loop3D`]
    /// intersect itself, or if the new [`Point3D`] is not coplanar with the
    /// [`Loop3D`], or if the [`Loop3D`] is closed.
    pub fn push(&mut self, point: Point3D) -> Result<(), String> {
        // Check the point
        self.valid_to_add(point)?;

        if let Some(last) = self.vertices.last() {
            if last.compare(point) {
                return Ok(());
            }
        }

        let n = self.vertices.len();

        // If there are previous points, Check the points before the new addition
        if n >= 2 {
            let a = self.vertices[n - 2];
            let b = self.vertices[n - 1];

            if a.is_collinear(b, point)? {
                // if it is collinear, update last point instead of
                // adding a new one
                self.vertices[n - 1] = point;
            } else {
                self.vertices.push(point);
            }
        } else {
            self.vertices.push(point);
        }

        // Calcualte the normal if possible
        if self.vertices.len() == 3 {
            self.set_normal()?;
        }
        Ok(())
    }

    /// Counts the vertices in the [`Loop3D`]    
    pub fn len(&self) -> usize {
        self.vertices.len()
    }

    /// Checks if a [`Segment3D`] intersects any of the other [`Segment3D`]
    /// in the [`Loop3D`] and if its midpoint is inside of it.
    ///
    /// Note that the idea is to call it by using segments that go from
    /// one vertex to another. I mean, this function takes *any* segment,
    /// meaning that a small [`Segment3D`] "floating" inside of a big [`Loop3D`]
    /// will be considered a diagonal... be careful with this
    pub fn is_diagonal(&self, s: Segment3D) -> Result<bool, String> {
        if s.length < 1e-5 {
            // Very small segment cannot be diagonal
            return Ok(false);
        }
        let mut inter = Point3D::new(0., 0., 0.);
        let n = self.len();
        // It cannot intercect any
        for i in 0..=n {
            let a = self.vertices[i % n];
            let b = self.vertices[(i + 1) % n];

            let poly_s = Segment3D::new(a, b);
            let intersects = s.intersect(&poly_s, &mut inter);
            // If they are contained and are the same length, then they are the same segment

            const TINY: Float = 1e-7;

            let different_length = (s.length() - poly_s.length()).abs() > TINY;
            let contains = s.contains(&poly_s)? && different_length;
            if intersects || contains {
                return Ok(false);
            }
        }
        // And the midpoint must be in the loop.
        if !self.test_point(s.midpoint()).unwrap() {
            return Ok(false);
        }

        Ok(true)
    }

    /// Opens a [`Loop3D`]
    pub fn open(&mut self) {
        self.closed = false
    }

    /// Closes a [`Loop3D`], calculating its area and checking the connection
    /// between the first and last vertex. If the first and the last
    pub fn close(&mut self) -> Result<(), String> {
        // Check if we can try to close now...
        if self.vertices.len() < 3 {
            return Err("Loops need at least 3 vertices".to_string());
        }

        // Check the last vertex for collinearity
        let n = self.vertices.len();
        let a = self.vertices[n - 2];
        let b = self.vertices[n - 1];
        let c = self.vertices[0];

        if a.is_collinear(b, c)? {
            // collinear. Remove the last vertex
            self.vertices.pop();
        }

        // Check if closing would intercept
        self.valid_to_add(self.vertices[0])?;

        // Check the first vertex for collinearity
        let n = self.vertices.len();
        let a = self.vertices[n - 1];
        let b = self.vertices[0];
        let c = self.vertices[1];

        if a.is_collinear(b, c)? {
            // collinear. Remove the last vertex
            self.vertices.remove(0);
        }

        // Close
        self.closed = true;
        self.set_area()?;
        self.set_perimeter()?;
        Ok(())
    }

    /// Sets the normal [`Vector3D`] for a [`Loop3D`]
    fn set_normal(&mut self) -> Result<(), String> {
        if self.vertices.len() < 3 {
            return Err(
                "Trying to set the normal of a Polygon3D with less than three Point3D".to_string(),
            );
        }

        let a = self.vertices[0];
        let b = self.vertices[1];
        let c = self.vertices[2];

        let ab = b - a;
        let bc = c - b;

        self.normal = ab.cross(bc);
        self.normal.normalize();

        Ok(())
    }

    /// Retrieves the normal of the vector.
    ///
    /// # Note
    /// If the [`Loop3D`] has less than 3 vertices, then
    /// the Normal will be `Vector3D(0., 0., 0.)`, which is the default.
    pub fn normal(&self) -> Vector3D {
        self.normal
    }

    /// Checks whether a [`Point3D`] is coplanar with the rest of the
    /// points.
    pub fn is_coplanar(&self, p: Point3D) -> Result<bool, String> {
        // This should not happen, but you never know..
        if self.vertices.is_empty() {
            let msg = "Trying to check whether point is coplanar in a Loop3D without any vertices"
                .to_string();
            return Err(msg);
        }

        if self.normal.is_zero() {
            let msg =
                "Trying to check whether point is coplanar in a Loop3D without normal".to_string();
            return Err(msg);
        }

        let first_point = self.vertices[0];
        let d = first_point - p;

        let aux = (self.normal * d).abs();
        Ok(aux < 1e-7)
    }

    /// Tests whether a [`Point3D`] dwells inside of the [`Loop3D`].
    pub fn test_point(&self, point: Point3D) -> Result<bool, String> {
        // Check if the loop is done
        if !self.closed {
            return Err("Trying to test_point in an open Loop3D".to_string());
        }

        // Check if coplanar
        if !self.is_coplanar(point)? {
            return Ok(false);
        }

        // Ray cast
        let d = (point - (self.vertices[0] + self.vertices[1]) * 0.5) * 1000.; // Should be enough...?
        let ray = Segment3D::new(point, point + d);

        let mut n_cross = 0;
        let n = self.vertices.len();
        for i in 0..n {
            let vertex_a = self.vertices[i];
            let vertex_b = self.vertices[(i + 1) % n];
            let segment_ab = Segment3D::new(vertex_a, vertex_b);

            // Check if the point is in the segment.
            if segment_ab.contains_point(point)? {
                return Ok(true);
            }

            // Check if the ray and the segment touch. We only consider
            // touching at the start (e.g., t_a between [0 and 1) ) in
            // order not to count vertices twice.
            if let Some((t_a, t_b)) = segment_ab.get_intersection_pt(&ray) {
                // If the ray intersects
                if (0. ..=1.).contains(&t_b) && (0. ..=1.).contains(&t_a) {
                    if t_a < Float::EPSILON {
                        // if the intersection is at the start of the segment
                        let side_normal = d.cross(segment_ab.as_vector3d());
                        if side_normal.is_same_direction(self.normal) {
                            n_cross += 1
                        }
                    } else if t_a < 1. {
                        // intersection is within the segment (not including the end)
                        n_cross += 1;
                    } else {
                        // if the intersection is at the end of the segment
                        let side_normal = d.cross(segment_ab.as_reversed_vector3d());
                        if side_normal.is_same_direction(self.normal) {
                            n_cross += 1
                        }
                    }
                }
            }
        }
        // If did not touch OR touched an odd number of
        // times, then it was outside
        Ok(n_cross != 0 && n_cross % 2 != 0)
    } // end of test_point

    /// Calculates and caches the perimeter of the [`Loop3D`]
    fn set_perimeter(&mut self) -> Result<Float, String> {
        if !self.closed {
            let msg =
                "Trying to calculate the perimeter of a Loop3D that is not closed".to_string();
            return Err(msg);
        }

        if self.normal.is_zero() {
            let msg = "Trying to calculate the perimeter of a Loop3D with Zero normal".to_string();
            return Err(msg);
        }

        let n = self.vertices.len();
        if n < 3 {
            let msg =
                "Trying to calculate the perimeter of a Loop3D with less than three valid vertices"
                    .to_string();
            return Err(msg);
        }

        let mut per = 0.0;
        for i in 0..n {
            per += (self.vertices[i % n] - self.vertices[(i + 1) % n]).length();
        }

        self.perimeter = per;
        Ok(self.perimeter)
    }

    /// Calculates and caches the area of the [`Loop3D`]
    fn set_area(&mut self) -> Result<Float, String> {
        if !self.closed {
            let msg = "Trying to calculate the area of a Loop3D that is not closed".to_string();
            return Err(msg);
        }

        if self.normal.is_zero() {
            let msg = "Trying to calculate the area of a Loop3D with Zero normal".to_string();
            return Err(msg);
        }

        // Set the right hand side vector
        let mut rhs = Vector3D::new(0.0, 0.0, 0.0);

        let n = self.vertices.len();
        if n < 3 {
            let msg =
                "Trying to calculate the area of a Loop3D with less than three valid vertices"
                    .to_string();
            return Err(msg);
        }

        // We need to go around from 0 to N vertices,
        // ... so, n+1 valid vertices.
        let mut v: Vector3D = self.vertices[0].into();
        let mut v_p1: Vector3D = self.vertices[1].into();
        for i in 2..n + 2 {
            rhs += v.cross(v_p1);
            v = v_p1;
            v_p1 = self.vertices[i % n].into();
        }

        let area = self.normal * rhs / 2.0;
        if area < 0. {
            self.normal *= -1.;
        }
        self.area = area.abs();
        Ok(self.area)
    }

    /// Returns the area of the [`Loop3D`]
    pub fn area(&self) -> Result<Float, String> {
        if !self.is_closed() {
            Err("Trying to get the area of an open Loop3D".to_string())
        } else {
            Ok(self.area)
        }
    }

    /// Returns the perimeter of the [`Loop3D`]
    pub fn perimeter(&self) -> Result<Float, String> {
        if !self.is_closed() {
            Err("Trying to get the perimeter of an open Loop3D".to_string())
        } else {
            Ok(self.perimeter)
        }
    }

    /// Returns the centroid; i.e., the average of all vertices.
    pub fn centroid(&self) -> Result<Point3D, String> {
        if !self.is_closed() {
            Err("Trying to get the centroid of an open Loop3D".to_string())
        } else {
            let n = self.vertices.len() as Float;
            let (mut x, mut y, mut z) = (0., 0., 0.);
            for v in &self.vertices {
                x += v.x;
                y += v.y;
                z += v.z;
            }

            Ok(Point3D::new(x / n, y / n, z / n))
        }
    }

    /// Indicates whether the [`Loop3D`] has been closed already
    pub fn is_closed(&self) -> bool {
        self.closed
    }

    /// Checks whether the [`Loop3D`] contains a [`Segment3D`] `s`
    pub fn contains_segment(&self, s: &Segment3D) -> bool {
        let n = self.vertices.len();
        for (i, v) in self.vertices.iter().enumerate() {
            let next_v = self.vertices[(i + 1) % n];
            let segment = Segment3D::new(*v, next_v);
            if segment.compare(s) {
                return true;
            }
        }
        false
    }

    /// Projects one [`Loop3D`] into the plane of another [`Loop3D`].
    ///
    /// This means translating all points of `self` to the plane of `other`.
    pub fn project_into_plane(&mut self, other: &Self) -> Result<(), String> {
        if self.is_empty() {
            return Err("when projecting Loop3D: self is empty".to_string());
        }
        if other.is_empty() {
            return Err("when projecting Loop3D: other is empty".to_string());
        }
        if !self.is_closed() {
            return Err("when projecting Loop3D: self is not closed".to_string());
        }
        if !other.is_closed() {
            return Err("when projecting Loop3D: other is not closed".to_string());
        }

        let anchor = other.vertices.get(0).ok_or("no vertices")?;
        let normal = other.normal();

        for p in self.vertices.iter_mut() {
            *p = p.project_into_plane(*anchor, normal);
        }
        Ok(())
    }

    /// Calculates the distance between two [`Loop3D`] in their
    /// normal direction.
    ///
    /// Returns an error if their normal is not the same.
    pub fn normal_distance(&self, other: &Self) -> Result<Float, String> {
        if self.is_empty() {
            return Err(
                "when calculating normal distance between Loop3D: self is empty".to_string(),
            );
        }
        if other.is_empty() {
            return Err(
                "when calculating normal distance between Loop3D: other is empty".to_string(),
            );
        }
        if !self.is_closed() {
            return Err(
                "when calculating normal distance between Loop3D: self is not closed".to_string(),
            );
        }
        if !other.is_closed() {
            return Err(
                "when calculating normal distance between Loop3D: other is not closed".to_string(),
            );
        }
        let normal = self.normal;

        if !normal.is_parallel(other.normal) {
            return Err(
                "trying to calculate the distance of two loops that do not share a normal"
                    .to_string(),
            );
        }
        let anchor = other.vertices.get(0).ok_or("no vertices in other")?;
        let p = self.vertices.get(0).ok_or("no vertices in self")?;
        let d = p.distance_to_plane(*anchor, normal);

        Ok(d)
    }

    /// Goes through all the the [`Point3D`]s in `self.vertices` and in `points`
    /// (this is a slow algorithm) and—when a point in `self` is "very close" to
    /// one in `points`—it modifies the formet to match the latter.
    ///
    /// The criteria for `very_close` is modulated by the `snap_distance` value.
    ///
    /// Returns `None` if no point was snapped; and `Some(i)` when `i` points were snapped.
    pub fn snap(&mut self, points: &[Point3D], snap_distance: Float) -> Option<usize> {
        let d2 = snap_distance.powi(2);
        let mut count = 0;

        for p in self.vertices.iter_mut() {
            for other_p in points {
                if p.squared_distance(*other_p) <= d2 {
                    *p = *other_p;
                    count += 1;
                }
            }
        }

        if count == 0 {
            None
        } else {
            Some(count)
        }
    }

    /// Intersects a [`Loop3D`] with a [`Segment3D`], returning
    /// a vector with all the [`Point3D`] where intersection
    /// happens.
    pub fn intersect(&self, s: &Segment3D) -> Vec<(usize, Point3D)> {
        let mut ret = Vec::new();

        let n = self.vertices.len();
        let mut inter = Point3D::default();
        for i in 0..n {
            let current_segment = Segment3D::new(self.vertices[i], self.vertices[(i + 1) % n]);

            if current_segment.intersect(s, &mut inter) {
                ret.push((i, inter))
            }
        }

        ret
    }

    /// Clips a [`Loop3D`] as determined by another [`Loop3D`], returning a `Vec` of new [`Loop3D`]
    /// created by the clipping.
    ///
    /// Uses the Sutherland-Hodgman algorithm. The algorithm proceeds by iteratively clipping
    /// the polygon against each edge of the cutting segment.
    pub fn clip(&self, cutter: &Loop3D) -> Result<Loop3D, String> {
        let edge_count = self.len();

        // let mut input_polygon = self.clone();
        let mut output_polygon: Loop3D = Loop3D::new();

        for i in 0..edge_count {
            let start = self.vertices[i];
            let end = self.vertices[(i + 1) % edge_count];

            let inside_start = cutter.test_point(start)?;
            let inside_end = cutter.test_point(end)?;

            let current_segment = Segment3D::new(start, end);

            if inside_start && inside_end {
                // Both start and end points are inside the loop, add the end point to the output polygon
                output_polygon.push(start)?;
            } else if inside_start && !inside_end {
                // Start point is inside, end point is outside,
                // add the `start` point and the intersection
                output_polygon.push(start)?;

                let intersections = cutter.intersect(&current_segment);
                assert!(intersections.len() <= 1, "Expecting None or One intersection points between segment and Loop3D when clipping.");
                if intersections.len() == 1 {
                    let (.., inter_pt) = intersections[0];
                    output_polygon.push(inter_pt)?;
                } else {
                    // we had established that one was inside and the other outside...
                    unreachable!()
                }
            } else if !inside_start && inside_end {
                // Start point is outside, end point is inside, add the intersection point to the output polygon
                let intersections = cutter.intersect(&current_segment);
                assert!(intersections.len() <= 1, "Expecting None or One intersection points between segment and Loop3D when clipping.");
                if intersections.len() == 1 {
                    output_polygon.push(intersections[0].1)?;
                } else {
                    unreachable!()
                }
            } else {
                // Both outside... but they can go THROUGH
                let intersections = cutter.intersect(&current_segment);
                if !intersections.is_empty() {
                    todo!()
                }
            }
        }

        output_polygon.close()?;

        Ok(output_polygon)
    }

    /// Splits a [`Loop3D`] into two pieces according to a
    /// [`Segment3D`] that goes through it
    pub fn split(&self, seg: &Segment3D) -> Result<Vec<Self>, String> {
        const TINY: Float = 0.001;
        let mut seg = *seg;

        // Snap Segment to Loop, if they are very close.
        if self.closed {
            let normal = self.normal;
            let anchor = self.vertices[0];

            let d_start = seg.start.distance_to_plane(anchor, normal).abs();
            let d_end = seg.end.distance_to_plane(anchor, normal).abs();
            if d_start < TINY && d_end < TINY {
                seg.start = seg.start.project_into_plane(anchor, normal);
                seg.end = seg.end.project_into_plane(anchor, normal);
            }
        } else {
            return Err("trying to split an open Loop3D".to_string());
        }

        if self.contains_segment(&seg) || seg.length() < 1e-6 {
            return Ok(vec![self.clone()]);
        }

        let intersections = self.intersect(&seg);

        if intersections.is_empty() || intersections.len() == 1 {
            // No intersections, or no cutting... just return original
            return Ok(vec![self.clone()]);
        }

        if intersections.len() != 2 {
            // this might as well happen... we will postpone this for now
            todo!()
        }

        // We intersect twice... meaning that we branch (first) and merge (after)
        let (mut branch_index, mut branch_pt) = intersections[0];
        let (mut merge_index, mut merge_pt) = intersections[1];

        // Same point? (e.g., corner), just return
        if branch_pt.compare(merge_pt) {
            return Ok(vec![self.clone()]);
        }

        // Ensure that we branch befor emerging
        assert_ne!(
            branch_index, merge_index,
            "Intersecting twice the same segment???"
        ); // they cannot be equal
        if branch_index > merge_index {
            std::mem::swap(&mut branch_index, &mut merge_index);
            std::mem::swap(&mut branch_pt, &mut merge_pt);
        }

        let mut left = Loop3D::new();
        let mut right = Loop3D::new();

        let n = self.len();
        for i in 0..=n {
            let current_point = self.vertices[i % n];
            if i <= branch_index {
                left.push(current_point)?;
            } else if i == branch_index + 1 {
                left.push(branch_pt)?;
                right.push(branch_pt)?;
                right.push(current_point)?;
            } else if i < merge_index + 1 {
                right.push(current_point)?;
            } else if i == merge_index + 1 {
                right.push(merge_pt)?;
                left.push(merge_pt)?;
                left.push(current_point)?;
            } else {
                left.push(current_point)?;
            }
        }

        let mut ret = Vec::with_capacity(2);
        if left.len() > 2 {
            left.close()?;
            ret.push(left)
        }
        if right.len() > 2 {
            right.close()?;
            ret.push(right);
        }
        Ok(ret)
    }

    /// Recursively splits a [`Loop3D`] based on the [`Segment3D`] in `segments`
    ///
    /// This can be a slow algorithm.
    pub fn split_from_slice(&self, segments: &[Segment3D]) -> Result<Vec<Self>, String> {
        // initialize.
        let mut ret = vec![self.clone()];

        // cut each loop with the cutter.
        for cutter in segments.iter() {
            let mut new_ret = Vec::new();

            for the_loop in ret.iter() {
                let new_loops = the_loop.split(cutter)?;

                new_ret.extend_from_slice(&new_loops);
            }

            ret = new_ret;
        }

        Ok(ret)
    }

    /// Retrieves a vector with all the segments in this [`Loop3D`]
    pub fn segments(&self) -> Vec<Segment3D> {
        let n = self.len();
        let mut ret = Vec::with_capacity(n);

        for i in 0..n {
            let start = self.vertices[i];
            let end = self.vertices[(i + 1) % n];
            ret.push(Segment3D::new(start, end));
        }

        ret
    }
}

/***********/
/* TESTING */
/***********/

#[cfg(test)]
mod testing {

    use std::convert::TryInto;

    use crate::{Polygon3D, Triangulation3D};

    use super::*;

    #[test]
    fn serde_ok() -> Result<(), String> {
        let a = "[
            0.0,0,0,  
            1.0,1,1,  
            2,3,-1
        ]";

        let p: Loop3D = serde_json::from_str(a).map_err(|e| e.to_string())?;
        assert!(p.closed);
        assert_eq!(p.vertices.len(), 3);

        assert_eq!(p.vertices[0].x, 0.);
        assert_eq!(p.vertices[0].y, 0.);
        assert_eq!(p.vertices[0].z, 0.);

        assert_eq!(p.vertices[1].x, 1.);
        assert_eq!(p.vertices[1].y, 1.);
        assert_eq!(p.vertices[1].z, 1.);

        assert_eq!(p.vertices[2].x, 2.);
        assert_eq!(p.vertices[2].y, 3.);
        assert_eq!(p.vertices[2].z, -1.);

        println!("{}", serde_json::to_string(&p).map_err(|e| e.to_string())?);
        Ok(())
    }

    #[test]
    fn test_new() {
        let l = Loop3D::new();
        assert_eq!(l.vertices.len(), 0);

        let l = Loop3D::default();
        assert_eq!(l.vertices.len(), 0);

        let v = Point3D::new(0., 1., 0.);
        let v_cp = v;
        let v_clone = v.clone();
        assert_eq!(v, v_cp);
        assert_eq!(v, v_clone);
    }

    #[test]
    fn test_push() -> Result<(), String> {
        let mut l = Loop3D::new();
        assert_eq!(l.vertices.len(), 0);

        l.push(Point3D::new(1., 2., 3.))?;

        assert_eq!(l.vertices.len(), 1);
        assert_eq!(l[0], Point3D::new(1., 2., 3.));

        // n_vertices
        assert_eq!(l.len(), 1);

        l.push(Point3D::new(4., 5., 6.))?;
        assert_eq!(l.len(), 2);
        assert_eq!(l[1], Point3D::new(4., 5., 6.));

        // Collinear point in the middle
        let mut l = Loop3D::new();
        assert_eq!(0, l.vertices.len());
        l.push(Point3D::new(-2., -2., 0.))?; // 0
        assert_eq!(1, l.vertices.len());
        l.push(Point3D::new(0., -2., 0.))?; // 1 -- collinear point
        assert_eq!(2, l.vertices.len());
        l.push(Point3D::new(2., -2., 0.))?; // 2
        assert_eq!(2, l.vertices.len());
        l.push(Point3D::new(2., 2., 0.))?; // 3
        assert_eq!(l.len(), 3);
        l.push(Point3D::new(-2., 2., 0.))?; // 4
        assert_eq!(l.len(), 4);
        l.close()?;
        assert_eq!(l.len(), 4);
        assert!((l.area - 16.).abs() < 1e-4);

        assert_eq!(l[0], Point3D::new(-2., -2., 0.));
        assert_eq!(l[1], Point3D::new(2., -2., 0.));
        assert_eq!(l[2], Point3D::new(2., 2., 0.));
        assert_eq!(l[3], Point3D::new(-2., 2., 0.));

        // Collinear point in the end
        let mut l = Loop3D::new();
        assert_eq!(0, l.vertices.len());
        l.push(Point3D::new(-2., -2., 0.))?; // 0
        assert_eq!(1, l.vertices.len());
        l.push(Point3D::new(2., -2., 0.))?; // 1
        assert_eq!(2, l.vertices.len());
        l.push(Point3D::new(2., 2., 0.))?; // 2
        assert_eq!(l.len(), 3);
        l.push(Point3D::new(-2., 2., 0.))?; // 3
        assert_eq!(l.len(), 4);
        l.push(Point3D::new(-2., 0., 0.))?; // 4 -- collinear point... will be removed when closing
        assert_eq!(5, l.vertices.len());
        l.close()?;
        assert_eq!(l.len(), 4);
        assert!((l.area - 16.).abs() < 1e-4);
        assert_eq!(l[0], Point3D::new(-2., -2., 0.));
        assert_eq!(l[1], Point3D::new(2., -2., 0.));
        assert_eq!(l[2], Point3D::new(2., 2., 0.));
        assert_eq!(l[3], Point3D::new(-2., 2., 0.));

        // Collinear point in the beginning
        let mut l = Loop3D::new();
        assert_eq!(0, l.vertices.len());
        l.push(Point3D::new(0., -2., 0.))?; // 0  -- collinear point... will be removed when closing
        assert_eq!(1, l.vertices.len());
        l.push(Point3D::new(2., -2., 0.))?; // 1
        assert_eq!(2, l.vertices.len());
        l.push(Point3D::new(2., 2., 0.))?; // 2
        assert_eq!(l.len(), 3);
        l.push(Point3D::new(-2., 2., 0.))?; // 3
        assert_eq!(l.len(), 4);
        l.push(Point3D::new(-2., -2., 0.))?; // 4
        assert_eq!(5, l.vertices.len());
        l.close()?;
        assert_eq!(l.len(), 4);
        assert!((l.area - 16.).abs() < 1e-4);
        assert_eq!(l[0], Point3D::new(2., -2., 0.));
        assert_eq!(l[1], Point3D::new(2., 2., 0.));
        assert_eq!(l[2], Point3D::new(-2., 2., 0.));
        assert_eq!(l[3], Point3D::new(-2., -2., 0.));

        // INTERSECT WITH ITSELF.

        let mut l = Loop3D::new();
        assert!(l.push(Point3D::new(-2., -2., 0.)).is_ok()); // 0 collinear
        assert!(l.push(Point3D::new(2., 2., 0.)).is_ok()); // 1
        assert!(l.push(Point3D::new(-2., 2., 0.)).is_ok()); // 2

        // This should fail.
        assert!(l.push(Point3D::new(2., -2., 0.)).is_err()); // 3
        Ok(())
    } // end of test_push

    #[test]
    fn test_is_coplanar() -> Result<(), String> {
        let mut l = Loop3D::new();
        l.push(Point3D::new(-2., -2., 0.))?; // 0
        l.push(Point3D::new(2., -2., 0.))?; // 1
        l.push(Point3D::new(2., 2., 0.))?; // 2
        l.push(Point3D::new(-2., 2., 0.))?; // 3
        l.close()?;

        let r = l.is_coplanar(Point3D::new(0., 0., 0.));
        match r {
            Ok(b) => {
                assert!(b)
            }
            Err(e) => panic!("{}", e),
        }

        let r = l.is_coplanar(Point3D::new(0., 0., 10.1));
        match r {
            Ok(b) => {
                assert!(!b);
            }
            Err(e) => return Err(format!("{}", e)),
        }
        Ok(())
    }

    #[test]
    fn test_point_weird() -> Result<(), String> {
        let mut the_loop = Loop3D::new();
        the_loop.push(Point3D::new(10., -1.2246467991473533E-15, 0.))?;

        the_loop.push(Point3D::new(-10., 1.2246467991473533E-15, 0.))?;

        the_loop.push(Point3D::new(-10., 1.2246467991473533E-15, 3.))?;

        the_loop.push(Point3D::new(10., -1.2246467991473533E-15, 3.))?;

        the_loop.close()?;

        let a = Point3D::new(-10., 1.2246467991473533E-15, 0.);
        let b = Point3D::new(10., -1.2246467991473533E-15, 3.);
        let mid = (a + b) / 2.;
        let r = the_loop.test_point(mid)?;
        assert!(r);
        Ok(())
    }

    #[test]
    fn test_point_convex_loop_interior() -> Result<(), String> {
        //let normal = Vector3D::new(0., 0., 1.);
        let mut the_loop = Loop3D::new();
        let l = 0.5;
        the_loop.push(Point3D::new(-l, -l, 0.))?;
        the_loop.push(Point3D::new(l, -l, 0.))?;
        the_loop.push(Point3D::new(l, l, 0.))?;
        the_loop.push(Point3D::new(-l, l, 0.))?;
        the_loop.close()?;

        let r = the_loop.test_point(Point3D::new(0., 0., 0.))?;
        assert!(r);
        Ok(())
    }

    #[test]
    fn test_point_convex_loop_exterior() -> Result<(), String> {
        //Vector3D normal = Vector3D(0, 0, 1);
        let mut the_loop = Loop3D::new();
        let l = 1. / (2 as Float).sqrt();
        the_loop.push(Point3D::new(-l, -l, 0.))?;
        the_loop.push(Point3D::new(l, -l, 0.))?;
        the_loop.push(Point3D::new(l, l, 0.))?;
        the_loop.push(Point3D::new(-l, l, 0.))?;
        the_loop.close()?;

        let r = the_loop.test_point(Point3D::new(-10., 0., 0.))?;
        assert!(!r);
        Ok(())
    }

    #[test]
    fn test_point_concave_loop_exterior1() -> Result<(), String> {
        //Vector3D normal = Vector3D(0, 0, 1);
        let mut the_loop = Loop3D::new();

        let l = 1.0 / (2 as Float).sqrt();
        let bigl = 2. / (2 as Float).sqrt();

        the_loop.push(Point3D::new(-bigl, -bigl, 0.))?;
        the_loop.push(Point3D::new(0.0, -bigl, 0.))?; // collinear point, moving in X
        the_loop.push(Point3D::new(bigl, -bigl, 0.))?;
        the_loop.push(Point3D::new(bigl, 0., 0.))?; // collinear point, moving in Y
        the_loop.push(Point3D::new(bigl, bigl, 0.))?;
        the_loop.push(Point3D::new(-bigl, bigl, 0.))?;
        the_loop.push(Point3D::new(-bigl, -bigl, 0.))?;

        the_loop.push(Point3D::new(-l, -l, 0.))?;
        the_loop.push(Point3D::new(-l, l, 0.))?;
        the_loop.push(Point3D::new(l, l, 0.))?;
        the_loop.push(Point3D::new(l, -l, 0.))?;
        the_loop.push(Point3D::new(-l, -l, 0.))?;

        the_loop.close()?;

        let r = the_loop.test_point(Point3D::new(0., 0., 0.))?;
        assert!(!r);
        Ok(())
    }

    #[test]
    fn test_point_concave_loop_exterior2() -> Result<(), String> {
        //Vector3D normal = Vector3D(0, 0, 1);
        let mut the_loop = Loop3D::new();

        let l = 1. / (2 as Float).sqrt();
        let bigl = 2. / (2 as Float).sqrt();

        the_loop.push(Point3D::new(-bigl, -bigl, 0.))?;
        the_loop.push(Point3D::new(0.0, -bigl, 0.))?; // collinear point, moving in X
        the_loop.push(Point3D::new(bigl, -bigl, 0.))?;
        the_loop.push(Point3D::new(bigl, 0., 0.))?; // collinear point, moving in Y
        the_loop.push(Point3D::new(bigl, bigl, 0.))?;
        the_loop.push(Point3D::new(-bigl, bigl, 0.))?;
        the_loop.push(Point3D::new(-bigl, -bigl, 0.))?;

        the_loop.push(Point3D::new(-l, -l, 0.))?;
        the_loop.push(Point3D::new(-l, l, 0.))?;
        the_loop.push(Point3D::new(l, l, 0.))?;
        the_loop.push(Point3D::new(l, -l, 0.))?;
        the_loop.push(Point3D::new(-l, -l, 0.))?;

        the_loop.close()?;

        let r = the_loop.test_point(Point3D::new(-10., 0., 0.))?;
        assert!(!r);
        Ok(())
    }

    #[test]
    fn test_point_concave_loop_interior() -> Result<(), String> {
        //Vector3D normal = Vector3D(0, 0, 1);
        let mut the_loop = Loop3D::new();
        let l = 0.5;
        let bigl = 1.;

        the_loop.push(Point3D::new(-bigl, -bigl, 0.))?;
        the_loop.push(Point3D::new(0., -bigl, 0.))?; // collinear point, moving in X
        the_loop.push(Point3D::new(bigl, -bigl, 0.))?;
        the_loop.push(Point3D::new(bigl, 0., 0.))?; // collinear point, moving in Y
        the_loop.push(Point3D::new(bigl, bigl, 0.))?;
        the_loop.push(Point3D::new(-bigl, bigl, 0.))?;
        the_loop.push(Point3D::new(-bigl, -bigl, 0.))?;

        the_loop.push(Point3D::new(-l, -l, 0.))?;
        the_loop.push(Point3D::new(-l, l, 0.))?;
        the_loop.push(Point3D::new(l, l, 0.))?;
        the_loop.push(Point3D::new(l, -l, 0.))?;
        the_loop.push(Point3D::new(-l, -l, 0.))?;

        the_loop.close()?;

        let r = the_loop.test_point(Point3D::new(-(bigl + l) / 2., -(bigl + l) / 2., 0.))?;
        assert!(r);
        Ok(())
    }

    #[test]
    fn test_point_through_vertex() -> Result<(), String> {
        //Vector3D normal = Vector3D(0, 0, 1);
        let mut the_loop = Loop3D::new();
        let l = 1.;
        let bigl = 3.;

        the_loop.push(Point3D::new(0., 0., 0.))?;
        the_loop.push(Point3D::new(bigl, 0., 0.))?;
        the_loop.push(Point3D::new(bigl, bigl, 0.))?;
        the_loop.push(Point3D::new(0., bigl, 0.))?;
        the_loop.push(Point3D::new(0., 0., 0.))?;

        the_loop.push(Point3D::new(l, l, 0.))?;
        the_loop.push(Point3D::new(l, 2. * l, 0.))?;
        the_loop.push(Point3D::new(2. * l, 2. * l, 0.))?;
        the_loop.push(Point3D::new(2. * l, l, 0.))?;
        the_loop.push(Point3D::new(l, l, 0.))?;

        the_loop.close()?;

        let r = the_loop.test_point(Point3D::new(l / 2., l, 0.))?;
        assert!(r);
        Ok(())
    }

    #[test]
    fn test_point_concave_loop_interior_with_clean() -> Result<(), String> {
        //Vector3D normal = Vector3D(0, 0, 1);

        let mut the_loop = Loop3D::new();
        let l = 1. / (2 as Float).sqrt();
        let bigl = 2. / (2 as Float).sqrt();

        the_loop.push(Point3D::new(-bigl, -bigl, 0.))?;
        the_loop.push(Point3D::new(0., -bigl, 0.))?; // collinear point, moving in X
        the_loop.push(Point3D::new(bigl, -bigl, 0.))?;
        the_loop.push(Point3D::new(bigl, 0., 0.))?; // collinear point, moving in Y
        the_loop.push(Point3D::new(bigl, bigl, 0.))?;
        the_loop.push(Point3D::new(-bigl, bigl, 0.))?;
        the_loop.push(Point3D::new(-bigl, -bigl, 0.))?;

        the_loop.push(Point3D::new(-l, -l, 0.))?;
        the_loop.push(Point3D::new(-l, l, 0.))?;
        the_loop.push(Point3D::new(l, l, 0.))?;
        the_loop.push(Point3D::new(l, -l, 0.))?;
        the_loop.push(Point3D::new(-l, -l, 0.))?;

        the_loop.close()?;

        let r = the_loop.test_point(Point3D::new(
            -1.5 / (2 as Float).sqrt(),
            -1.5 / (2 as Float).sqrt(),
            0.,
        ))?;
        assert!(r);
        Ok(())
    }

    #[test]
    fn test_point_non_coplanar() -> Result<(), String> {
        let mut the_loop = Loop3D::new();
        let l = 1. / (2 as Float).sqrt();

        the_loop.push(Point3D::new(-l, -l, 0.))?;
        the_loop.push(Point3D::new(-l, l, 0.))?;
        the_loop.push(Point3D::new(l, l, 0.))?;
        the_loop.push(Point3D::new(l, -l, 0.))?;
        the_loop.close()?;

        let r = the_loop.test_point(Point3D::new(
            -1.5 / (2 as Float).sqrt(),
            -1.5 / (2 as Float).sqrt(),
            1.,
        ))?;
        assert!(!r);
        Ok(())
    }

    #[test]
    fn test_area_1() -> Result<(), String> {
        //Vector3D normal = Vector3D(0, 0, 1);
        let mut the_loop = Loop3D::new();
        let l = 0.5;

        the_loop.push(Point3D::new(-l, -l, 0.))?;
        the_loop.push(Point3D::new(-l, l, 0.))?;
        the_loop.push(Point3D::new(l, l, 0.))?;
        the_loop.push(Point3D::new(l, -l, 0.))?;

        assert!(the_loop.area().is_err());
        the_loop.close()?;

        let a = the_loop.area()?;
        assert!((4. * l * l - a).abs() < 0.0001);
        Ok(())
    }

    #[test]
    fn test_area_2() -> Result<(), String> {
        let l = 1.;

        // Counterclock wise
        let mut the_loop = Loop3D::new();
        the_loop.push(Point3D::new(l, 0., 0.))?; //1
        the_loop.push(Point3D::new(l, l, 0.))?; //2
        the_loop.push(Point3D::new(2.0 * l, l, 0.))?; //3
        the_loop.push(Point3D::new(2.0 * l, 2.0 * l, 0.))?; //4
        the_loop.push(Point3D::new(0.0, 2.0 * l, 0.))?; //5
        the_loop.push(Point3D::new(0., 0., 0.))?; //0

        assert!(the_loop.area().is_err());

        the_loop.close()?;

        let a = the_loop.area()?;
        assert!((3.0 - a).abs() < 1e-4);
        assert!(
            the_loop.normal().compare(Vector3D::new(0., 0., 1.)),
            "normal = {}",
            the_loop.normal()
        );

        // Clockwise
        let l = 1.;

        // Counterclock wise
        let mut the_loop = Loop3D::new();
        the_loop.push(Point3D::new(l, 0., 0.))?; //1
        the_loop.push(Point3D::new(0., 0., 0.))?; //0
        the_loop.push(Point3D::new(0.0, 2.0 * l, 0.))?; //5
        the_loop.push(Point3D::new(2.0 * l, 2.0 * l, 0.))?; //4
        the_loop.push(Point3D::new(2.0 * l, l, 0.))?; //3
        the_loop.push(Point3D::new(l, l, 0.))?; //2

        the_loop.close()?;

        let a = the_loop.area()?;
        assert!((3.0 - a).abs() < 1e-4);
        assert!(
            the_loop.normal().compare(Vector3D::new(0., 0., -1.)),
            "normal = {}",
            the_loop.normal()
        );
        Ok(())
    }

    #[test]
    fn test_close() -> Result<(), String> {
        let l = 1.;

        // Two elements... cannot close
        let mut the_loop = Loop3D::new();
        the_loop.push(Point3D::new(0., 0., 0.))?;
        the_loop.push(Point3D::new(l, 0., 0.))?;
        assert!(the_loop.close().is_err());

        // Three elements... can close
        let mut the_loop = Loop3D::new();
        the_loop.push(Point3D::new(0., 0., 0.))?;
        the_loop.push(Point3D::new(l, 0., 0.))?;
        the_loop.push(Point3D::new(0., l, 0.))?;
        assert!(the_loop.close().is_ok());

        // Three elements, but in the same line... cannot close
        let mut the_loop = Loop3D::new();
        the_loop.push(Point3D::new(0., 0., 0.))?;
        the_loop.push(Point3D::new(l, 0., 0.))?;
        the_loop.push(Point3D::new(2.0 * l, 0., 0.))?;
        assert!(!the_loop.close().is_ok());

        // four elements, two in the same line... can close
        let mut the_loop = Loop3D::new();
        the_loop.push(Point3D::new(0., 0., 0.))?;
        the_loop.push(Point3D::new(l, 0., 0.))?;
        the_loop.push(Point3D::new(2.0 * l, 0., 0.))?;
        the_loop.push(Point3D::new(0., l, 0.))?;
        assert!(the_loop.close().is_ok());
        Ok(())
    }

    #[test]
    fn test_is_diagonal() -> Result<(), String> {
        let mut l = Loop3D::new();
        l.push(Point3D::new(-2., -2., 0.))?; // 0
        l.push(Point3D::new(2., -2., 0.))?; // 1
        l.push(Point3D::new(2., 2., 0.))?; // 2
        l.push(Point3D::new(-2., 2., 0.))?; // 3
        l.close()?;

        // Intersects a segment
        assert!(!l.is_diagonal(Segment3D::new(
            Point3D::new(-4., 0., 0.),
            Point3D::new(4., 0., 0.),
        ))?);

        // Doesn't intersect, but is outside
        assert!(!l.is_diagonal(Segment3D::new(
            Point3D::new(4., 0., 0.),
            Point3D::new(5., 0., 0.),
        ))?);

        // Doesn't intersect, is inside == is_diagonal
        assert!(l.is_diagonal(Segment3D::new(
            Point3D::new(-2., -2., 0.),
            Point3D::new(2., 2., 0.),
        ))?);

        Ok(())
    }

    #[test]
    fn test_contains_segment() -> Result<(), String> {
        let mut l = Loop3D::new();
        let p0 = Point3D::new(-2., -2., 0.);
        let p1 = Point3D::new(2., -2., 0.);
        let p2 = Point3D::new(2., 2., 0.);
        let p3 = Point3D::new(-2., 2., 0.);
        l.push(p0)?; // 0
        l.push(p1)?; // 1
        l.push(p2)?; // 2
        l.push(p3)?; // 3

        // Existing segments, in both directions
        assert!(l.contains_segment(&Segment3D::new(p0, p1)));
        assert!(l.contains_segment(&Segment3D::new(p1, p2)));
        assert!(l.contains_segment(&Segment3D::new(p2, p3)));
        assert!(l.contains_segment(&Segment3D::new(p3, p0)));
        assert!(l.contains_segment(&Segment3D::new(p3, p2)));
        assert!(l.contains_segment(&Segment3D::new(p2, p1)));
        assert!(l.contains_segment(&Segment3D::new(p1, p0)));
        assert!(l.contains_segment(&Segment3D::new(p0, p3)));

        // Diagonals
        assert!(!l.contains_segment(&Segment3D::new(p1, p3)));
        assert!(!l.contains_segment(&Segment3D::new(p3, p1)));
        assert!(!l.contains_segment(&Segment3D::new(p0, p2)));
        assert!(!l.contains_segment(&Segment3D::new(p2, p0)));

        // Segment inside
        assert!(!l.contains_segment(&Segment3D::new(
            Point3D::new(-0.5, -0.5, 0.),
            Point3D::new(0.5, 0.5, 0.),
        )));

        // Segment that crosses from in to out
        assert!(!l.contains_segment(&Segment3D::new(
            Point3D::new(-0.5, -0.5, 0.),
            Point3D::new(10.5, 10.5, 0.),
        )));

        // Segment contained in another segment
        assert!(!l.contains_segment(&Segment3D::new(
            Point3D::new(-1., -2., 0.),
            Point3D::new(1., -2., 0.),
        )));
        Ok(())
    }

    #[test]
    fn test_valid_to_add() -> Result<(), String> {
        let mut outer = Loop3D::new();

        let p0 = Point3D::new(0., 0., 0.);
        assert!(outer.valid_to_add(p0).is_ok());
        outer.push(p0)?;

        let p1 = Point3D::new(0., 3., 0.);
        assert!(outer.valid_to_add(p1).is_ok());
        outer.push(p1)?;

        let p2 = Point3D::new(3., 3., 0.);
        assert!(outer.valid_to_add(p2).is_ok());
        outer.push(p2)?;

        let p3 = Point3D::new(5., 5., 0.);
        assert!(outer.valid_to_add(p3).is_ok());
        outer.push(p3)?;

        let p4 = Point3D::new(3., 6., 0.);
        assert!(outer.valid_to_add(p4).is_ok());
        outer.push(p4)?;

        let p5 = Point3D::new(0., 5., 0.);
        assert!(outer.valid_to_add(p5).is_ok());

        Ok(())
    }

    #[test]
    fn test_perimeter_centroid() -> Result<(), String> {
        // A square with the center at the origin.
        /*****/
        let mut outer_loop = Loop3D::new();
        let l = 2. as Float;
        outer_loop.push(Point3D::new(-l, -l, 0.))?;
        outer_loop.push(Point3D::new(l, -l, 0.))?;
        outer_loop.push(Point3D::new(l, l, 0.))?;
        outer_loop.push(Point3D::new(-l, l, 0.))?;
        outer_loop.close()?;

        assert_eq!(outer_loop.perimeter, 8. * l);
        assert_eq!(outer_loop.perimeter()?, 8. * l);

        let c = outer_loop.centroid()?;
        assert!(c.x.abs() < 1e-8);
        assert!(c.y.abs() < 1e-8);
        assert!(c.z.abs() < 1e-8);

        Ok(())
    }

    #[test]
    fn test_sanitize() -> Result<(), String> {
        let mut l = Loop3D::with_capacity(4);
        // Add a triangle
        l.vertices.push(Point3D::new(0., 0., 0.));
        l.vertices.push(Point3D::new(1., 1., 0.));
        l.vertices.push(Point3D::new(0., 1., 0.));
        // And then two aligned points
        l.vertices.push(Point3D::new(0., 0.5, 0.)); // <- Collinear point
        l.vertices.push(Point3D::new(0., 0.2, 0.));

        // start with 5 points
        assert_eq!(l.len(), 5);

        // Sanitizing removes the collinear point,
        // withouth closing.
        l = l.sanitize()?;
        assert_eq!(l.len(), 4);
        assert!(!l.closed());

        // We close it, (last point we
        // added is collinear, so it will be removed).
        l.close()?;
        assert_eq!(l.len(), 3);

        // Sanitizing now should return a closed polygon.
        l = l.sanitize()?;
        assert!(l.closed());

        Ok(())
    }

    #[test]
    fn test_weird_loop() -> Result<(), String> {
        let mut l = Loop3D::new();
        l.push(Point3D {
            x: 0.,
            y: 1.3500000000000001,
            z: 0.0,
        })?;
        l.push(Point3D {
            x: 0.,
            y: 2.0899999999999999,
            z: 0.82999999999999996,
        })?;
        l.push(Point3D {
            x: 0.,
            y: 2.9900000000000002,
            z: 0.82999999999999996,
        })?;
        l.push(Point3D {
            x: 0.,
            y: 2.9900000000000002,
            z: 2.29,
        })?;
        l.push(Point3D {
            x: 0.,
            y: 2.0899999999999999,
            z: 2.29,
        })?;
        l.push(Point3D {
            x: 0.,
            y: 1.3500000000000001,
            z: 2.7000000000000002,
        })?;
        l.push(Point3D {
            x: 0.,
            y: 3.7400000000000002,
            z: 2.7000000000000002,
        })?;
        l.push(Point3D {
            x: 0.,
            y: 3.7400000000000002,
            z: 0.0,
        })?;

        l.close()?;

        let poly = Polygon3D::new(l)?;
        let _: Triangulation3D = poly.try_into()?;
        Ok(())
    }

    #[test]
    fn test_weird_loop_2() -> Result<(), String> {
        let mut outer = Loop3D::new();
        outer.push(Point3D {
            x: 8.7699999999999996,
            y: 3.7400000000000002,
            z: 0.0,
        })?; // 0
        outer.push(Point3D {
            x: 8.7699999999999996,
            y: 3.7400000000000002,
            z: 2.7000000000000002,
        })?; // 1
        outer.push(Point3D {
            x: 8.7699999999999996,
            y: 0.0,
            z: 2.7000000000000002,
        })?; // 1
        outer.push(Point3D {
            x: 8.7699999999999996,
            y: 0.0,
            z: 0.0,
        })?; // 1
        outer.close()?;

        let mut inner = Loop3D::new();

        inner.push(Point3D {
            x: 8.7699999999999996,
            y: 1.2649999999999999,
            z: 0.69999999999999996,
        })?; // 2
        inner.push(Point3D {
            x: 8.7699999999999996,
            y: 1.2649999999999999,
            z: 2.29,
        })?; // 3
        inner.push(Point3D {
            x: 8.7699999999999996,
            y: 2.4750000000000001,
            z: 2.29,
        })?; // 4
        inner.push(Point3D {
            x: 8.7699999999999996,
            y: 2.4740000000000002,
            z: 0.69999999999999996,
        })?; // 5

        inner.close()?;

        let mut poly = Polygon3D::new(outer)?;
        poly.cut_hole(inner)?;
        let _: Triangulation3D = poly.try_into()?;

        Ok(())
    }

    #[test]
    fn test_weird_loop_3() -> Result<(), String> {
        let mut outer = Loop3D::new();
        outer.push(Point3D {
            x: 2.98,
            y: 1.3500000000000001,
            z: 2.7000000000000002,
        })?; // 0
        outer.push(Point3D {
            x: 4.0199999999999996,
            y: 1.3500000000000001,
            z: 2.7000000000000002,
        })?; // 1
        outer.push(Point3D {
            x: 4.0199999999999996,
            y: 6.0300000000000002,
            z: 2.7000000000000002,
        })?; // 1
        outer.push(Point3D {
            x: 2.98,
            y: 6.0300000000000002,
            z: 2.7000000000000002,
        })?; // 1

        outer.push(Point3D {
            x: 2.98,
            y: 5.7599999999999998,
            z: 2.7000000000000002,
        })?; // 2
        outer.push(Point3D {
            x: 1.9199999999999999,
            y: 5.7599999999999998,
            z: 2.7000000000000002,
        })?; // 3
        outer.push(Point3D {
            x: 1.9199999999999999,
            y: 4.7999999999999998,
            z: 2.7000000000000002,
        })?; // 4
        outer.push(Point3D {
            x: 2.98,
            y: 4.7999999999999998,
            z: 2.7000000000000002,
        })?; // 5

        outer.close()?;

        let poly = Polygon3D::new(outer)?;
        let _: Triangulation3D = poly.try_into()?;
        Ok(())
    }

    #[test]
    fn test_project_into_plane_self() -> Result<(), String> {
        // Project into self.
        let lstr = "[
            0, 0, 0,  
            1, 0, 0,  
            1, 1, 0,
            0, 1, 0
        ]";

        let this_loop: Loop3D = serde_json::from_str(lstr).map_err(|e| e.to_string())?;

        let mut this_loop_projected = this_loop.clone();

        let other_loop = this_loop.clone();
        this_loop_projected.project_into_plane(&other_loop)?;

        assert_eq!(this_loop.len(), this_loop_projected.len());
        for (i, this_p) in this_loop.vertices.iter().enumerate() {
            assert!(this_p.compare(this_loop_projected.vertices[i]))
        }
        Ok(())
    }

    #[test]
    fn test_project_into_plane_fails() -> Result<(), String> {
        let mut this_loop = Loop3D::new();

        this_loop.push(Point3D::new(0.0, 0.0, 0.0))?;
        this_loop.push(Point3D::new(1.0, 0.0, 0.0))?;
        this_loop.push(Point3D::new(1.0, 1.0, 0.0))?;
        this_loop.push(Point3D::new(0.0, 1.0, 0.0))?;

        let mut other_loop: Loop3D = this_loop.clone();

        // None of them are not closed... this should fail
        assert!(this_loop.project_into_plane(&other_loop).is_err());

        // Other is not closed... should fail
        let mut this_closed = this_loop.clone();
        this_closed.close()?;
        assert!(this_closed.project_into_plane(&other_loop).is_err());

        // This is not closed... shold fail
        other_loop.close()?;
        assert!(this_loop.project_into_plane(&other_loop).is_err());
        Ok(())
    }

    #[test]
    fn test_normal_distance_fails() -> Result<(), String> {
        let mut this_loop = Loop3D::new();

        this_loop.push(Point3D::new(0.0, 0.0, 0.0))?;
        this_loop.push(Point3D::new(1.0, 0.0, 0.0))?;
        this_loop.push(Point3D::new(1.0, 1.0, 0.0))?;
        this_loop.push(Point3D::new(0.0, 1.0, 0.0))?;

        let mut other_loop: Loop3D = this_loop.clone();

        // None of them are not closed... this should fail
        assert!(this_loop.normal_distance(&other_loop).is_err());

        // Other is not closed... should fail
        let mut this_closed = this_loop.clone();
        this_closed.close()?;
        assert!(this_closed.normal_distance(&other_loop).is_err());

        // This is not closed... shold fail
        other_loop.close()?;
        assert!(this_loop.normal_distance(&other_loop).is_err());
        Ok(())
    }

    #[test]
    fn test_project_into_plane_z() -> Result<(), String> {
        // Project into a Loop above it.
        let this_str = "[
            0, 0, 0,  
            1, 0, 0,  
            1, 1, 0,
            0, 1, 0
        ]";

        let other_str = "[
            0, 0, 2.0,  
            1, 0, 2.0,  
            1, 1, 2.0,
            0, 1, 2.0
        ]";

        let mut this_loop: Loop3D = serde_json::from_str(this_str).map_err(|e| e.to_string())?;
        let other_loop: Loop3D = serde_json::from_str(other_str).map_err(|e| e.to_string())?;

        // Both closed... is should work now
        this_loop.project_into_plane(&other_loop)?;

        assert_eq!(this_loop.len(), other_loop.len());
        for (i, this_p) in this_loop.vertices.iter().enumerate() {
            assert!(this_p.compare(other_loop.vertices[i]))
        }
        Ok(())
    }

    #[test]
    fn test_project_into_plane_neg_z() -> Result<(), String> {
        // Project into a Loop above it.
        let this_str = "[
            0, 0, 0,  
            1, 0, 0,  
            1, 1, 0,
            0, 1, 0
        ]";

        let other_str = "[
            0, 0, -2.0,  
            1, 0, -2.0,  
            1, 1, -2.0,
            0, 1, -2.0
        ]";

        let mut this_loop: Loop3D = serde_json::from_str(this_str).map_err(|e| e.to_string())?;
        let other_loop: Loop3D = serde_json::from_str(other_str).map_err(|e| e.to_string())?;

        // Both closed... is should work now
        this_loop.project_into_plane(&other_loop)?;

        assert_eq!(this_loop.len(), other_loop.len());
        for (i, this_p) in this_loop.vertices.iter().enumerate() {
            assert!(this_p.compare(other_loop.vertices[i]))
        }
        Ok(())
    }

    #[test]
    fn test_normal_distance() -> Result<(), String> {
        // Project into a Loop above it.
        let this_str = "[
            0, 0, 0,  
            1, 0, 0,  
            1, 1, 0,
            0, 1, 0
        ]";

        let other_str = "[
            0, 0, 2.0,  
            1, 0, 2.0,  
            1, 1, 2.0,
            0, 1, 2.0
        ]";

        let this_loop: Loop3D = serde_json::from_str(this_str).map_err(|e| e.to_string())?;
        let other_loop: Loop3D = serde_json::from_str(other_str).map_err(|e| e.to_string())?;

        // Both closed... is should work now
        let d = this_loop.normal_distance(&other_loop)?.abs();
        assert!((d - 2.0).abs() < 1e-4);
        Ok(())
    }

    #[test]
    fn test_normal_fail_not_normal() -> Result<(), String> {
        // Project into a Loop above it.
        let this_str = "[
            0, 0, 0,  
            1, 0, 0,  
            1, 1, 0
        ]";

        let other_str = "[
            0, 0, 2.0,  
            1, 0, 1.0,  
            1, 1, 2.0
        ]";

        let this_loop: Loop3D = serde_json::from_str(this_str).map_err(|e| e.to_string())?;
        let other_loop: Loop3D = serde_json::from_str(other_str).map_err(|e| e.to_string())?;

        // Both closed... is should work now
        assert!(this_loop.normal_distance(&other_loop).is_err());
        Ok(())
    }

    #[test]
    fn test_snap_too_far() -> Result<(), String> {
        // Project into a Loop above it.
        let this_str = "[
            0, 0, 0,  
            1, 0, 0,  
            1, 1, 0
        ]";

        let other_str = "[
            0, 0, 2.0,  
            1, 0, 2.0,  
            1, 1, 2.0
        ]";

        let mut this_loop: Loop3D = serde_json::from_str(this_str).map_err(|e| e.to_string())?;
        let other_loop: Loop3D = serde_json::from_str(other_str).map_err(|e| e.to_string())?;

        let res = this_loop.snap(&other_loop.vertices(), 0.1);
        assert!(res.is_none());
        Ok(())
    }

    #[test]
    fn test_snap() -> Result<(), String> {
        // Project into a Loop above it.
        let this_str = "[
            0, 0, 0,  
            1, 0, 0,  
            1, 1, 0
        ]";

        let other_str = "[
            0, 0, 0.001,  
            1, 0, 0.001,  
            1, 1, 0.001 
        ]";

        let mut this_loop: Loop3D = serde_json::from_str(this_str).map_err(|e| e.to_string())?;
        let other_loop: Loop3D = serde_json::from_str(other_str).map_err(|e| e.to_string())?;

        let res = this_loop.snap(&other_loop.vertices(), 0.1);
        assert_eq!(res, Some(3));

        assert_eq!(this_loop.len(), other_loop.len());
        for (i, this_p) in this_loop.vertices.iter().enumerate() {
            assert!(this_p.compare(other_loop.vertices[i]))
        }
        Ok(())
    }

    #[test]
    fn test_snap_2() -> Result<(), String> {
        // Project into a Loop above it.
        let this_str = "[
            0, 0, 0,  
            1, 0, 0,  
            1, 1, 0
        ]";

        let other_str = "[
            0, 0, 0.001,  
            1, 0, 0.001,  
            1, 1, 1.01 
        ]";

        let mut this_loop: Loop3D = serde_json::from_str(this_str).map_err(|e| e.to_string())?;
        let other_loop: Loop3D = serde_json::from_str(other_str).map_err(|e| e.to_string())?;

        let res = this_loop.snap(&other_loop.vertices(), 0.1);
        assert_eq!(res, Some(2));
        Ok(())
    }

    #[test]
    fn test_intersect() -> Result<(), String> {
        let this_str = "[
            0, 0, 0,  
            1, 0, 0,  
            1, 1, 0,
            0, 1, 0
        ]";

        let this_loop: Loop3D = serde_json::from_str(this_str).map_err(|e| e.to_string())?;

        let seg = Segment3D::new(Point3D::new(0.5, -1., 0.), Point3D::new(0.5, 2., 0.));
        let intersections = this_loop.intersect(&seg);
        assert_eq!(intersections.len(), 2);
        assert!(
            (intersections[0].1.compare(Point3D::new(0.5, 0.0, 0.0))
                || intersections[0].1.compare(Point3D::new(0.5, 1.0, 0.0)))
        );
        assert!(
            (intersections[1].1.compare(Point3D::new(0.5, 0.0, 0.0))
                || intersections[1].1.compare(Point3D::new(0.5, 1.0, 0.0)))
        );

        let seg = Segment3D::new(Point3D::new(0.5, -1., 1.), Point3D::new(0.5, 2., 1.));
        let intersections = this_loop.intersect(&seg);
        assert_eq!(intersections.len(), 0);

        let seg = Segment3D::new(Point3D::new(0.5, -1., 0.), Point3D::new(0.5, 0.5, 0.));
        let intersections = this_loop.intersect(&seg);
        assert_eq!(intersections.len(), 1);
        assert!(intersections[0].1.compare(Point3D::new(0.5, 0., 0.)));
        Ok(())
    }

    #[test]
    fn test_clip() -> Result<(), String> {
        let this_str = "[
            0, 0, 0,
            1, 0, 0,
            1, 1, 0,
            0, 1, 0
        ]";

        let this_loop: Loop3D = serde_json::from_str(this_str).map_err(|e| e.to_string())?;

        let other_str = "[
            -0.5, -1, 0,
            0.5,  -1, 0,
            0.5, 2, 0,
            -0.5, 2, 0
        ]";
        let other: Loop3D = serde_json::from_str(other_str).map_err(|e| e.to_string())?;

        let v = this_loop.clip(&other)?;
        // assert_eq!(v.len(), 2);

        let exp_pts = vec![
            Point3D::new(0., 0., 0.),
            Point3D::new(0.5, 0., 0.),
            Point3D::new(0.5, 1., 0.),
            Point3D::new(0., 1., 0.),
        ];
        for (i, p) in v.vertices.iter().enumerate() {
            assert!(p.compare(exp_pts[i]))
        }
        Ok(())
    }

    #[test]
    fn test_split_vertical() -> Result<(), String> {
        let this_str = "[
            0, 0, 0,
            1, 0, 0,
            1, 1, 0,
            0, 1, 0
        ]";

        let this_loop: Loop3D = serde_json::from_str(this_str).map_err(|e| e.to_string())?;

        let seg = Segment3D::new(Point3D::new(0.5, -1.0, 0.0), Point3D::new(0.5, 2.0, 0.0));

        let v = this_loop.split(&seg)?;
        assert_eq!(v.len(), 2);

        let left = &v[0];
        let left_exp_pts = vec![
            Point3D::new(0., 0., 0.),
            Point3D::new(0.5, 0., 0.),
            Point3D::new(0.5, 1., 0.),
            Point3D::new(0., 1., 0.),
        ];
        println!("Left");
        for (i, p) in left.vertices.iter().enumerate() {
            println!("    {}", p);
            assert!(p.compare(left_exp_pts[i]))
        }

        let right = &v[1];
        println!("Right");
        let right_exp_pts = vec![
            Point3D::new(0.5, 0., 0.),
            Point3D::new(1., 0., 0.),
            Point3D::new(1., 1., 0.),
            Point3D::new(0.5, 1., 0.),
        ];
        for (i, p) in right.vertices.iter().enumerate() {
            println!("    {}", p);
            assert!(p.compare(right_exp_pts[i]))
        }
        Ok(())
    }

    #[test]
    fn test_split_horizontal() -> Result<(), String> {
        let this_str = "[
            0, 0, 0,
            1, 0, 0,
            1, 1, 0,
            0, 1, 0
        ]";

        let this_loop: Loop3D = serde_json::from_str(this_str).map_err(|e| e.to_string())?;

        let seg = Segment3D::new(Point3D::new(-0.5, 0.5, 0.0), Point3D::new(1.5, 0.5, 0.0));

        let v = this_loop.split(&seg)?;
        assert_eq!(v.len(), 2);

        let left = &v[0];
        let left_exp_pts = vec![
            Point3D::new(0., 0., 0.),
            Point3D::new(1.0, 0., 0.),
            Point3D::new(1.0, 0.5, 0.),
            Point3D::new(0.0, 0.5, 0.),
        ];
        println!("Left");
        for (i, p) in left.vertices.iter().enumerate() {
            println!("    {}", p);
            assert!(p.compare(left_exp_pts[i]))
        }

        let right = &v[1];
        println!("Right");
        let right_exp_pts = vec![
            Point3D::new(1.0, 0.5, 0.),
            Point3D::new(1., 1., 0.),
            Point3D::new(0., 1., 0.),
            Point3D::new(0.0, 0.5, 0.),
        ];
        for (i, p) in right.vertices.iter().enumerate() {
            println!("    {}", p);
            assert!(p.compare(right_exp_pts[i]))
        }
        Ok(())
    }

    #[test]
    fn test_split_edge() -> Result<(), String> {
        let this_str = "[
            0, 0, 0,
            1, 0, 0,
            1, 1, 0,
            0, 1, 0
        ]";

        let this_loop: Loop3D = serde_json::from_str(this_str).map_err(|e| e.to_string())?;

        let seg = Segment3D::new(Point3D::new(0., 0., 0.0), Point3D::new(1., 0., 0.0));

        let v = this_loop.split(&seg)?;
        assert_eq!(v.len(), 1);

        let left = &v[0];
        let left_exp_pts = vec![
            Point3D::new(0., 0., 0.),
            Point3D::new(1.0, 0., 0.),
            Point3D::new(1.0, 1.0, 0.),
            Point3D::new(0.0, 1., 0.),
        ];
        for (i, p) in left.vertices.iter().enumerate() {
            println!("    {}", p);
            assert!(p.compare(left_exp_pts[i]))
        }
        Ok(())
    }

    #[test]
    fn test_split_longer_edge() -> Result<(), String> {
        let this_str = "[
            0, 0, 0,
            1, 0, 0,
            1, 1, 0,
            0, 1, 0
        ]";

        let this_loop: Loop3D = serde_json::from_str(this_str).map_err(|e| e.to_string())?;

        let seg = Segment3D::new(Point3D::new(-10., 0., 0.0), Point3D::new(10., 0., 0.0));

        let v = this_loop.split(&seg)?;
        assert_eq!(v.len(), 1);

        let left = &v[0];
        let left_exp_pts = vec![
            Point3D::new(0., 0., 0.),
            Point3D::new(1.0, 0., 0.),
            Point3D::new(1.0, 1.0, 0.),
            Point3D::new(0.0, 1., 0.),
        ];
        for (i, p) in left.vertices.iter().enumerate() {
            println!("    {}", p);
            assert!(p.compare(left_exp_pts[i]))
        }
        Ok(())
    }

    #[test]
    fn test_split_corner() -> Result<(), String> {
        let this_str = "[
            0, 0, 0,
            1, 0, 0,
            1, 1, 0,
            0, 1, 0
        ]";

        let this_loop: Loop3D = serde_json::from_str(this_str).map_err(|e| e.to_string())?;

        let seg = Segment3D::new(Point3D::new(-10., 10., 0.0), Point3D::new(10., -10., 0.0));

        let v = this_loop.split(&seg)?;
        assert_eq!(v.len(), 1);

        let left = &v[0];
        let left_exp_pts = vec![
            Point3D::new(0., 0., 0.),
            Point3D::new(1.0, 0., 0.),
            Point3D::new(1.0, 1.0, 0.),
            Point3D::new(0.0, 1., 0.),
        ];
        for (i, p) in left.vertices.iter().enumerate() {
            println!("    {}", p);
            assert!(p.compare(left_exp_pts[i]))
        }
        Ok(())
    }

    #[test]
    fn test_split_from_slice() -> Result<(), String> {
        let this_str = "[
            0, 0, 0,
            1, 0, 0,
            1, 1, 0,
            0, 1, 0
        ]";

        let this_loop: Loop3D = serde_json::from_str(this_str).map_err(|e| e.to_string())?;

        let v = this_loop.split_from_slice(&vec![
            Segment3D::new(Point3D::new(0.5, -1.0, 0.0), Point3D::new(0.5, 2.0, 0.0)),
            Segment3D::new(Point3D::new(-0.5, 0.5, 0.0), Point3D::new(1.5, 0.5, 0.0)),
        ])?;
        assert_eq!(v.len(), 4);

        let exp = vec![
            vec![
                Point3D::new(0.00000, 0.00000, 0.00000),
                Point3D::new(0.50000, 0.00000, 0.00000),
                Point3D::new(0.50000, 0.50000, 0.00000),
                Point3D::new(0.00000, 0.50000, 0.00000),
            ],
            vec![
                Point3D::new(0.50000, 0.50000, 0.00000),
                Point3D::new(0.50000, 1.00000, 0.00000),
                Point3D::new(0.00000, 1.00000, 0.00000),
                Point3D::new(0.00000, 0.50000, 0.00000),
            ],
            vec![
                Point3D::new(0.50000, 0.00000, 0.00000),
                Point3D::new(1.00000, 0.00000, 0.00000),
                Point3D::new(1.00000, 0.50000, 0.00000),
                Point3D::new(0.50000, 0.50000, 0.00000),
            ],
            vec![
                Point3D::new(1.00000, 0.50000, 0.00000),
                Point3D::new(1.00000, 1.00000, 0.00000),
                Point3D::new(0.50000, 1.00000, 0.00000),
                Point3D::new(0.50000, 0.50000, 0.00000),
            ],
        ];

        for (i, l) in v.iter().enumerate() {
            println!("Loop {}", i);
            for (j, p) in l.vertices.iter().enumerate() {
                println!("     {}", p);

                assert!(p.compare(exp[i][j]))
            }
        }
        Ok(())
    }

    #[test]
    fn test_segments() -> Result<(), String> {
        let this_str = "[
            0, 0, 0,
            1, 0, 0,
            1, 1, 0,
            0, 1, 0
        ]";

        let this_loop: Loop3D = serde_json::from_str(this_str).map_err(|e| e.to_string())?;

        let segments = this_loop.segments();

        assert_eq!(segments.len(), 4);

        let expected = vec![
            Segment3D::new(Point3D::new(0., 0., 0.), Point3D::new(1., 0., 0.)),
            Segment3D::new(Point3D::new(1., 0., 0.), Point3D::new(1., 1., 0.)),
            Segment3D::new(Point3D::new(1., 1., 0.), Point3D::new(0., 1., 0.)),
            Segment3D::new(Point3D::new(0., 1., 0.), Point3D::new(0., 0., 0.)),
        ];

        for i in 0..4 {
            assert!(expected[i].compare(&segments[i]));
        }
        Ok(())
    }

    #[test]
    fn test_reverse() -> Result<(), String> {
        let mut l = Loop3D::with_capacity(4);
        let a = Point3D::new(0., 0., 0.);
        let b = Point3D::new(1., 1., 0.);
        let c = Point3D::new(0., 1., 0.);

        l.push(a)?;
        l.push(b)?;
        l.push(c)?;
        l.close()?;

        let normal = l.normal();

        // reverse
        l.reverse();

        let v = l.vertices();
        assert!((normal * -1.0).compare(l.normal()));
        assert!(a.compare(v[2]));
        assert!(b.compare(v[1]));
        assert!(c.compare(v[0]));
        Ok(())
    }

    #[test]
    fn test_get_reversed() -> Result<(), String> {
        let mut l = Loop3D::with_capacity(4);
        let a = Point3D::new(0., 0., 0.);
        let b = Point3D::new(1., 1., 0.);
        let c = Point3D::new(0., 1., 0.);

        l.push(a)?;
        l.push(b)?;
        l.push(c)?;
        l.close()?;

        // reverse
        let rev_l = l.get_reversed();

        assert_eq!(rev_l.len(), l.len());
        assert!((l.normal() * -1.0).compare(rev_l.normal()));
        let v = l.vertices();
        let rev_v = rev_l.vertices();
        let n = v.len();

        for i in 0..n {
            let p = v[i];
            let rev_p = rev_v[n - 1 - i];
            assert!(p.compare(rev_p));
        }
        Ok(())
    }

    #[test]
    fn test_bbox() -> Result<(), String> {
        let mut l = Loop3D::with_capacity(4);
        let a = Point3D::new(0., 0., 0.);
        let b = Point3D::new(1., 1., 0.);
        let c = Point3D::new(0., 1., 0.);

        l.push(a)?;
        let bbox = l.bbox()?;
        assert!(bbox.min.compare(a));
        assert!(bbox.max.compare(a));

        l.push(b)?;
        let bbox = l.bbox()?;
        assert!(bbox.min.compare(a));
        assert!(bbox.max.compare(b));

        l.push(c)?;
        let bbox = l.bbox()?;
        assert!(bbox.min.compare(a));
        assert!(bbox.max.compare(b));

        Ok(())
    }
}
