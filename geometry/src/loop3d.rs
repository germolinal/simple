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
/// assert!(the_loop.push(Point3D::new(-l, -l, 0.)).is_ok());
/// assert!(the_loop.push(Point3D::new(-l, l, 0.)).is_ok());
/// assert!(the_loop.push(Point3D::new(l, l, 0.)).is_ok());
/// assert!(the_loop.push(Point3D::new(l, -l, 0.)).is_ok());
///
/// assert!(the_loop.area().is_err());
/// assert!(the_loop.close().is_ok());
///
/// let a = the_loop.area().expect("no area?");
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
/// assert!(the_loop.push(Point3D::new(0., 0., 0.)).is_ok());
/// assert!(the_loop.push(Point3D::new(1., 1., 0.)).is_ok());
/// assert_eq!(2, the_loop.len());
///
/// // Adding a collinear point will not extend.
/// let collinear = Point3D::new(2., 2., 0.);
/// assert!(the_loop.push(collinear).is_ok());
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

impl std::fmt::Display for Loop3D {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let n = self.vertices.len();
        for p in self.vertices.iter().take(n - 1) {
            writeln!(f, "{},{},{},", p.x, p.y, p.z)?;
        }
        let p = self.vertices.last().unwrap();
        write!(f, "{},{},{}", p.x, p.y, p.z)
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
                    Value::Number(x) => {
                        x.as_f64()
                            .ok_or("Could not get X... it does not seem to be a number")
                            .map_err(serde::de::Error::custom)? as Float
                    }
                    _ => panic!("Expecting Polygon3D to be an array of numbers"),
                };
                let y = it.next();
                let y = match y {
                    Some(Value::Number(y)) => {
                        y.as_f64()
                            .ok_or("Could not get Y... it does not seem to be a number")
                            .map_err(serde::de::Error::custom)? as Float
                    }
                    _ => panic!("Expecting Polygon3D to be an array of numbers"),
                };
                let z = it.next();
                let z = match z {
                    Some(Value::Number(z)) => {
                        z.as_f64()
                            .ok_or("Could not get Z... it does not seem to be a number")
                            .map_err(serde::de::Error::custom)? as Float
                    }
                    _ => panic!("Expecting Polygon3D to be an array of numbers"),
                };
                ret.push(Point3D { x, y, z })
                    .map_err(serde::de::Error::custom)?;
            }
        }

        ret.close().map_err(serde::de::Error::custom)?;

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

    /// Check if two Polygon3D have the same vertices (i.e., same shape and position),
    /// ignoring their starting point or their normal.
    ///
    /// # Example
    /// ```
    /// use geometry::{Loop3D, Point3D};
    ///
    /// let mut l = Loop3D::with_capacity(3);
    /// let a = Point3D::new(0., 0., 0.);
    /// let b = Point3D::new(1., 1., 0.);
    /// let c = Point3D::new(0., 1., 0.);
    ///
    /// assert!(l.push(a).is_ok());
    /// assert!(l.push(b).is_ok());
    /// assert!(l.push(c).is_ok());
    /// assert!(l.close().is_ok());
    ///
    /// let mut l2 = Loop3D::with_capacity(4);
    /// assert!(l2.push(b).is_ok());
    /// assert!(l2.push(c).is_ok());
    /// assert!(l2.push(a).is_ok());
    /// assert!(l2.close().is_ok());
    ///
    /// if let Ok(is_equal) = l.is_equal(&l2, 0.001){
    ///     assert!(is_equal);
    /// }else{
    ///     panic!("these should have been equal");
    /// }
    /// ```
    pub fn is_equal(&self, other: &Loop3D, eps: Float) -> Result<bool, String> {
        if !self.closed || !other.closed {
            return Err("Trying to compare two Loop3D that might not be closed".into());
        }
        // Cant be equal if they have different area or n vertices
        if self.len() != other.len() || (self.area - other.area).abs() > 1e-3 {
            return Ok(false);
        }

        // find the vertex in other that matches the first vertex in self.
        let mut anchor = 0;
        let mut found = false;
        for (i, p) in other.vertices.iter().enumerate() {
            if p.compare_by(self.vertices[0], eps) {
                found = true;
                anchor = i;
                break;
            }
        }
        if !found {
            return Ok(false);
        }

        let is_reversed = self.normal * other.normal < 0.0;
        let n = self.len();

        let mut other_i = anchor;
        for this_pt in self.vertices.iter() {
            let other_pt = other.vertices[other_i];
            if !this_pt.compare_by(other_pt, eps) {
                return Ok(false);
            }
            if is_reversed {
                other_i = (other_i + n - 1) % n;
            } else {
                other_i = (other_i + 1) % n;
            }
        }

        Ok(true)
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
    /// assert!(l.push(a).is_ok());
    /// assert!(l.push(b).is_ok());
    /// assert!(l.push(c).is_ok());
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
    /// assert!(l.push(a).is_ok());
    /// assert!(l.push(b).is_ok());
    /// assert!(l.push(c).is_ok());
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
    /// assert!(l.push(Point3D::new(0., 0., 0.)).is_ok());
    /// assert!(l.push(Point3D::new(1., 1., 0.)).is_ok());
    /// assert!(l.push(Point3D::new(0., 1., 0.)).is_ok());
    /// assert!(l.push(Point3D::new(0., 0.5, 0.)).is_ok());
    ///
    /// let res = l.sanitize();
    /// assert!(res.is_ok());
    ///     
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

    /// It is like `push`, but the `avoid_dollinear` parameters let us choose whether we want to
    /// allow multiple collinear points being put together.
    pub fn push_collinear(&mut self, point: Point3D, avoid_collinear: bool) -> Result<(), String> {
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

            if avoid_collinear && a.is_collinear(b, point)? {
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
        self.push_collinear(point, true)
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
        if !self.test_point(s.midpoint())? {
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
        self.vertices.shrink_to_fit(); // save space
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

    /// Checks whether a loop contains a specific [`Point3D`] as one of
    /// its vertices, comparing points with a maximum distance of `eps`.
    pub fn contains_vertex(&self, vertex: Point3D, eps: Float) -> bool {
        self.vertices.iter().any(|v| v.compare_by(vertex, eps))
    }

    /// Returns the index of the first segment that contains
    /// a certain point... if any.
    ///
    /// The first segment is the one that groes from point [0] to [1]
    pub fn containing_segment(&self, point: Point3D) -> Option<usize> {
        let n = self.len();
        for i in 0..n {
            let ini = self[i];
            let fin = self[(i + 1) % n];
            let seg = Segment3D::new(ini, fin);
            if let Ok(true) = seg.contains_point(point) {
                return Some(i);
            }
        }
        None
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

        let anchor = other.vertices.get(0).ok_or("no vertices in anchor")?;
        let normal = other.normal();

        for p in self.vertices.iter_mut() {
            *p = p.project_into_plane(*anchor, normal);
        }
        if self.normal * normal > 0.0 {
            self.normal = normal;
        } else {
            self.normal = -normal;
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
    /// happens, and a `usize` with the index of the edge that is intersected
    /// at that point
    pub fn intersect(&self, s: &Segment3D) -> Vec<(usize, Point3D)> {
        let mut ret = Vec::new();

        if s.length < 1e-3 {
            return ret;
        }

        let n = self.vertices.len();
        let mut inter = Point3D::default();

        let mut add_pt = |i: usize, the_pt: Point3D| {
            let mut already_in = false;
            for (_, p) in ret.iter() {
                if the_pt.compare(*p) {
                    already_in = true;
                    break;
                }
            }
            if !already_in {
                ret.push((i, the_pt))
            }
        };

        for i in 0..n {
            let current_segment = Segment3D::new(self.vertices[i], self.vertices[(i + 1) % n]);
            if current_segment.length < 1e-2 || s.length < 1e-2 {
                continue;
            }

            if current_segment.intersect(s, &mut inter) {
                add_pt(i, inter);
            }
        }

        ret
    }

    fn get_closest_intersection_pt(
        intersections: Vec<(usize, Point3D)>,
        last_pt: Point3D,
    ) -> Option<(usize, Point3D)> {
        let mut other_seg_i = 0;
        let mut inter_pt = None;
        let mut min_d = Float::MAX;
        for (j, this_inter_pt) in intersections.into_iter() {
            let this_d = this_inter_pt.squared_distance(last_pt);
            if this_d > 1e-9 && this_d < min_d {
                min_d = this_d;
                other_seg_i = j;
                inter_pt = Some(this_inter_pt);
            }
        }
        inter_pt.map(|inter_pt| (other_seg_i, inter_pt))
    }
    /// Bites a loop, changing its shape
    pub fn bite(&self, chewer: &Self) -> Result<Self, String> {
        if !self.closed {
            return Err("Trying to bite an open Polygon3D".into());
        }
        if !chewer.closed {
            return Err("Trying to bite a Polygon3D using an open Polygon3D".into());
        }

        debug_assert!((1.0 - self.normal.length()).abs() < 1e-5);
        debug_assert!((1.0 - chewer.normal.length()).abs() < 1e-5);
        if (self.normal * chewer.normal).abs() < 0.999 {
            return Err("Biting a Loop3D with a non-coplanar Loop3D".to_string());
        }
        if !self.is_coplanar(chewer[0])? {
            return Err("Biting a Loop3D with a non-coplanar Loop3D".to_string());
        }
        // We build the chewer... this has no vertices outside of self now
        let chewer = self.intersection(chewer)?;
        if chewer.is_none() {
            // no intersection... bitten Self is Self
            return Ok(self.clone());
        }
        let mut chewer = chewer.expect("We checked that is not None already");

        // They need to circle in different directions.
        if chewer.normal.is_same_direction(self.normal) {
            chewer.reverse();
        }

        // Find a pooint that is part of self but not other.
        let mut walking_through_self = true;
        let first_pt: Option<Point3D>;
        let mut index = 0;
        loop {
            let this_v = self[index];
            if !chewer.test_point(this_v)? {
                first_pt = Some(this_v);
                break;
            }

            index += 1;
            if index == self.len() {
                // no intersection... bitten Self is Self
                return Ok(self.clone());
            }
        }

        // start
        let mut l = Self::new();
        let first_pt = first_pt.expect("This should never be None");
        l.push(first_pt)?;
        index += 1;

        let mut limit = 0;
        loop {
            limit += 1;
            if limit > 99 {
                // no intersection assumed.
                return Ok(self.clone());
            }

            // Get point
            let candidate_pt = if walking_through_self {
                self[index % self.len()]
            } else {
                chewer[index % chewer.len()]
            };

            // Break if we have circled back
            if l.len() > 1 && candidate_pt.compare(first_pt) {
                break;
            }

            let (walking_n, non_walking_n) = if walking_through_self {
                (self.len(), chewer.len())
            } else {
                (chewer.len(), self.len())
            };

            let last_pt = l.vertices[l.len() - 1]; // we know len is at least 1
            if last_pt.compare(candidate_pt) {
                index = (index + 1) % walking_n;
                continue;
            }
            let delta = (candidate_pt - last_pt).get_normalized() * 0.001;

            let candidate_seg = Segment3D::new(last_pt, candidate_pt + delta);

            let intersections = if walking_through_self {
                chewer.intersect(&candidate_seg)
            } else {
                self.intersect(&candidate_seg)
            };

            if intersections.is_empty() {
                l.push(candidate_pt)?;
                index = (index + 1) % walking_n;
            } else if let Some((other_seg_i, inter_pt)) =
                Self::get_closest_intersection_pt(intersections, last_pt)
            {
                l.push(inter_pt)?;
                index = (other_seg_i + 1) % non_walking_n;
                walking_through_self = !walking_through_self;
            } else {
                l.push(candidate_pt)?;
            }
        }

        l.close()?;

        Ok(l)
    }

    /// Finds the intersection between two Polygon3D
    ///
    /// The normal of the output Loop3D will be the same as `self`
    pub fn intersection(&self, other: &Loop3D) -> Result<Option<Loop3D>, String> {
        if !self.closed || !other.closed {
            return Err("Trying to find the intersection between two Polygon3D... but at least one of them is not closed".into());
        }

        debug_assert!((1.0 - self.normal.length()).abs() < 1e-5);
        debug_assert!((1.0 - other.normal.length()).abs() < 1e-5);
        if (self.normal * other.normal).abs() < 0.999 {
            return Ok(None);
        }
        if !self.is_coplanar(other[0])? {
            return Ok(None);
        }

        let mut other = other.clone();
        // They need to circle in the same direction.
        if !other.normal.is_same_direction(self.normal) {
            other.reverse();
        }

        // Find a posint that is part of self and of other.
        let mut walking_through_self = true;
        let first_pt: Option<Point3D>;
        let mut index = 0;
        loop {
            if walking_through_self {
                if index == self.len() {
                    index = 0;
                    walking_through_self = !walking_through_self;
                    continue;
                }
                let this_v = self[index];
                if other.test_point(this_v)? {
                    first_pt = Some(self[index]);
                    break; // found it!
                }
            } else {
                if index == other.len() {
                    // There is no intersection between both polygons
                    return Ok(None);
                }
                let this_v = other[index];
                if self.test_point(this_v)? {
                    first_pt = Some(other[index]);
                    break; // found it!
                }
            }
            index += 1;
        }

        let mut l: Loop3D = Loop3D::new();
        let first_pt = first_pt.expect("This should never be None");
        // add first point
        l.push(first_pt)?;
        index += 1;

        let mut limit = 0;
        loop {
            limit += 1;

            if limit > 99 {
                return Ok(None);
            }

            let candidate_pt = if walking_through_self {
                self[index % self.len()]
            } else {
                other[index % other.len()]
            };

            // Break if we have circled back
            if l.len() > 2
                && (l.vertices[0].compare(candidate_pt)
                    || l.vertices[0].compare(l.vertices[l.len() - 1]))
            {
                break;
            }

            // this should always work, as we push a point before entering this loop.
            let last_pt = l.vertices[l.len() - 1];
            let candidate_seg = Segment3D::new(last_pt, candidate_pt);

            let (intersections, walking_n, non_walking_n) = if walking_through_self {
                (other.intersect(&candidate_seg), self.len(), other.len())
            } else {
                (self.intersect(&candidate_seg), other.len(), self.len())
            };

            // select next point
            if intersections.is_empty() {
                // l.push(candidate_pt)?;
                if self.test_point(candidate_pt)? && other.test_point(candidate_pt)? {
                    l.push(candidate_pt)?;
                } else {
                    // This has happened with some geometries
                    return Ok(None);
                }
                index = (index + 1) % walking_n;
            } else if let Some((other_seg_i, inter_pt)) =
                Self::get_closest_intersection_pt(intersections, last_pt)
            {
                l.push(inter_pt)?;
                walking_through_self = !walking_through_self;
                index = (other_seg_i + 1) % non_walking_n;
            } else if self.test_point(candidate_pt)? && other.test_point(candidate_pt)? {
                l.push(candidate_pt)?;
            } else {
                // This has happened with some geometries
                return Ok(None);
            }
        }

        l.close()?;

        Ok(Some(l))
    }

    /// Splits a [`Loop3D`] into two pieces according to a
    /// [`Segment3D`] that goes through it
    pub fn split(&self, seg: &Segment3D) -> Result<Vec<Self>, String> {
        const TINY: Float = 0.001;
        let mut seg = *seg;

        // Snap Segment to Loop, if they are very close.
        if !self.closed {
            return Err("trying to split an open Loop3D".to_string());
        }
        let normal = self.normal;
        let anchor = self.vertices[0];

        let d_start = seg.start.distance_to_plane(anchor, normal).abs();
        let d_end = seg.end.distance_to_plane(anchor, normal).abs();
        if d_start < TINY && d_end < TINY {
            seg.start = seg.start.project_into_plane(anchor, normal);
            seg.end = seg.end.project_into_plane(anchor, normal);
        }

        if self.contains_segment(&seg) || seg.length() < 1e-6 {
            return Ok(vec![self.clone()]);
        }

        let intersections = self.intersect(&seg);
        let mut the_indices: Vec<usize> = Vec::with_capacity(2);
        let mut the_points: Vec<Point3D> = Vec::with_capacity(2);
        for (index, pt) in intersections {
            if !the_indices.contains(&index) {
                the_indices.push(index);
                the_points.push(pt);
            }
        }

        if the_indices.is_empty() || the_indices.len() == 1 {
            // No intersections, or no cutting... just return original
            return Ok(vec![self.clone()]);
        }

        let mut branch_index = the_indices[0];
        let mut branch_pt = the_points[0];
        let mut merge_index = the_indices[1];
        let mut merge_pt = the_points[1];

        // they cannot be equal
        if branch_index == merge_index {
            return Ok(vec![self.clone()]);
        }

        // Same point? (e.g., corner), just return
        if branch_pt.compare(merge_pt) {
            return Ok(vec![self.clone()]);
        }

        // Ensure that we branch befor emerging
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

    fn try_to_bite(this_str: &str, chewer: &str, exp_str: &str) -> Result<(), String> {
        let this_loop: Loop3D = serde_json::from_str(this_str).map_err(|e| e.to_string())?;
        let chewer: Loop3D = serde_json::from_str(chewer).map_err(|e| e.to_string())?;
        let mut reversed_chewer = chewer.clone();
        reversed_chewer.reverse();
        let exp: Loop3D = serde_json::from_str(exp_str).map_err(|e| e.to_string())?;

        let bitten = this_loop.bite(&chewer)?;
        assert_eq!(bitten.len(), exp.len());
        assert!(bitten.is_equal(&exp, 1e-3)?);

        let reverse_bitten = this_loop.bite(&reversed_chewer)?;
        assert_eq!(reverse_bitten.len(), exp.len());
        assert!(reverse_bitten.is_equal(&exp, 1e-3)?);

        Ok(())
    }

    #[test]
    fn test_bite_1() -> Result<(), String> {
        /*
                Case 1
                ------

        (-0.5,2) ____________ (0.5,2)
                |           |
                | (0,1).....|....(1,1)
                |      :////|   :
                |      :////|   :
                | (0,0):....|.. :(1,0)
                |___________|
        (-0.5,-1)         (0.5,-1)
                 */
        let chewer = "[
                    0, 0, 0,
                    1, 0, 0,
                    1, 1, 0,
                    0, 1, 0
                ]";
        let this_str = "[
                    -0.5, -1, 0,
                    0.5,  -1, 0,
                    0.5, 2, 0,
                    -0.5, 2, 0
                ]";
        let exp_str = "[
            -0.5, -1, 0,
            0.5,  -1, 0,
            0.5,   0, 0,
            0,0,0,
            0,1,0,
            0.5,1,0,
            0.5, 2, 0,
            -0.5, 2, 0
        ]";
        try_to_bite(this_str, chewer, exp_str)
    }

    #[test]
    fn test_bite_2() -> Result<(), String> {
        /*
               Case 2
               ------
        (-0.5,2)____________ (0.5,2)
               |           |
               | (0,1).....|........(1,1)
               |      :////|        :
               |      :////|        :
               |______:....|....... :
          (-0.5,0)  (0,0)  (0.5,0)  (1,0)

                */
        let chewer = "[
                    0, 0, 0,
                    1, 0, 0,
                    1, 1, 0,
                    0, 1, 0
                ]";
        let this_str = "[
                    -0.5, 0, 0,
                    0.5,  0, 0,
                    0.5,  2, 0,
                    -0.5, 2, 0
                ]";
        let exp_str = "[
            -0.5, 0, 0,
            0,0,0,
            0,1,0,
            0.5, 1, 0,            
            0.5,  2, 0,
            -0.5, 2, 0
        ]";
        try_to_bite(this_str, chewer, exp_str)
    }

    #[test]
    fn test_bite_3() -> Result<(), String> {
        /*
             Case 3
             ------
             (-0.5,2)____________ (1,2)
             |                    |
             | (0,1)..............(1,1)
             |      ://///////////:
             |      ://///////////:
             |______:............ :
        (-0.5,0)  (0,0)          (1,0)

              */
        let chewer = "[
                    0, 0, 0,
                    1, 0, 0,
                    1, 1, 0,
                    0, 1, 0
                ]";
        let this_str = "[
                    -0.5, 0, 0,
                    1.0,  0, 0,
                    1.0,  2, 0,
                    -0.5, 2, 0
                ]";
        let exp_str = "[
            -0.5, 0, 0,
            0,0,0,
            0,1,0,
            1,1,0,
            1.0,  2, 0,
            -0.5, 2, 0
        ]";
        try_to_bite(this_str, chewer, exp_str)
    }

    #[test]
    fn test_bite_4() -> Result<(), String> {
        /*
        Case 4
        ------

         ___________       .........
        |          |       :       :
        |          |       :       :
        |          |       :.......:
        |__________|

         */
        let chewer = "[
                4, 4, 0,
                5, 4, 0,
                5, 5, 0,
                4, 5, 0
            ]";
        let this_str = "[
                0,0,0,
                1,0,0,
                1,1,0,
                0,1,0
            ]";
        let exp_str = "[
            0,0,0,
            1,0,0,
            1,1,0,
            0,1,0
        ]";
        try_to_bite(this_str, chewer, exp_str)
    }

    #[test]
    fn test_bite_5() -> Result<(), String> {
        /*
        Case 5 :
        ------

         THEY ARE THE SAME POLYGON

         */
        let chewer = "[
                4, 4, 0,
                5, 4, 0,
                5, 5, 0,
                4, 5, 0
            ]";
        let this_str = chewer.clone();
        let exp_str = chewer.clone();

        try_to_bite(this_str, chewer, exp_str)
    }

    #[test]
    fn test_bite_6() -> Result<(), String> {
        /* CASE 6
        ---------

        Non coplanar
        */

        let this_str = "[
            -1.9148658671263563,-8.9269446089153,-1.106214891352675,
            -5.164369097543792,-1.5891449751178979,-1.106214891352675,
            -5.164369097543792,-1.5891449751178979,1.315362463352675,
            -1.9148658671263563,-8.9269446089153,1.315362463352675
        ]";
        let chewer = "[
            -1.9148659536009056,-8.926944428203846,1.315362463352675,
            -1.9148659536009056,-8.926944428203846,-1.106214891352675,
            2.6397675407798626,-6.909950781785453,-1.106214891352675,
            2.6397675407798626,-6.909950781785453,1.315362463352675
        ]";
        let this_loop: Loop3D = serde_json::from_str(this_str).map_err(|e| e.to_string())?;
        let chewer: Loop3D = serde_json::from_str(chewer).map_err(|e| e.to_string())?;
        let mut reversed_chewer = chewer.clone();
        reversed_chewer.reverse();
        assert!(this_loop.bite(&chewer).is_err());
        assert!(this_loop.bite(&reversed_chewer).is_err());

        Ok(())
    }

    #[test]
    fn test_bite_7() -> Result<(), String> {
        /*
        Case 7 :
        ------



         */
        let chewer = "[
            2.6397675407798626,-6.909950781785453,1.315362463352675,
            4.7744262940712305,-7.826319424405565,1.315362463352675,
            4.120359205911263,-8.115971036089993,1.315362463352675
        ]";

        let this_str = "[
            4.774426294071231,-7.826319424405565,1.315362463352675,
            4.120359205911263,-8.115971036089993,1.315362463352675,
            3.157255631736398,-5.941159318880256,1.315362463352675
        ]";

        let exp_str = "[
            4.7744262940712305,-7.826319424405565,1.315362463352675,
            3.157255631736398,-5.941159318880256,1.315362463352675,
            3.8084522554181546,-7.411645033362059,1.315362463352675
        ]";

        try_to_bite(this_str, chewer, exp_str)
    }

    fn try_to_intersect(this_str: &str, other_str: &str, exp_str: &str) -> Result<(), String> {
        let this_loop: Loop3D = serde_json::from_str(this_str).map_err(|e| e.to_string())?;
        let other: Loop3D = serde_json::from_str(other_str).map_err(|e| e.to_string())?;
        let exp: Loop3D = serde_json::from_str(exp_str).map_err(|e| e.to_string())?;

        let this_clipped = this_loop
            .intersection(&other)?
            .ok_or("No intersection for this_loop")?;

        assert_eq!(this_clipped.len(), exp.len());
        assert!(this_clipped.is_equal(&exp, 1e-3)?);

        let other_clipped = other
            .intersection(&this_loop)?
            .ok_or("No intersection for other")?;
        assert_eq!(other_clipped.len(), exp.len());
        assert!(other_clipped.is_equal(&exp, 1e-3)?);

        let clippity_clip = this_loop
            .intersection(&this_clipped)?
            .ok_or("No intersection for this_clipped")?;
        println!("\n\n{}\n\n", serde_json::to_string(&clippity_clip).unwrap());
        assert_eq!(clippity_clip.len(), exp.len());
        assert!(clippity_clip.is_equal(&exp, 1e-3)?);

        Ok(())
    }

    #[test]
    fn test_intersection_1() -> Result<(), String> {
        /*
                Case 1
                ------

        (-0.5,2) ____________ (0.5,2)
                |           |
                | (0,1).....|....(1,1)
                |      :////|   :
                |      :////|   :
                | (0,0):....|.. :(1,0)
                |___________|
        (-0.5,-1)         (0.5,-1)
                 */
        let this_str = "[
            0, 0, 0,
            1, 0, 0,
            1, 1, 0,
            0, 1, 0
        ]";
        let other_str = "[
            -0.5, -1, 0,
            0.5,  -1, 0,
            0.5, 2, 0,
            -0.5, 2, 0
        ]";
        let exp_str = "[
            0, 0, 0,
            0.5, 0, 0,
            0.5, 1, 0,
            0, 1, 0
        ]";
        try_to_intersect(this_str, other_str, exp_str)
    }

    #[test]
    fn test_intersection_2() -> Result<(), String> {
        /*
               Case 2
               ------
        (-0.5,2)____________ (0.5,2)
               |           |
               | (0,1).....|........(1,1)
               |      :////|        :
               |      :////|        :
               |______:....|....... :
          (-0.5,0)  (0,0)  (0.5,0)  (1,0)

                */
        let this_str = "[
            0, 0, 0,
            1, 0, 0,
            1, 1, 0,
            0, 1, 0
        ]";
        let other_str = "[
            -0.5, 0, 0,
            0.5,  0, 0,
            0.5,  2, 0,
            -0.5, 2, 0
        ]";
        let exp_str = "[
            0, 0, 0,
            0.5, 0, 0,
            0.5, 1, 0,
            0, 1, 0
        ]";
        try_to_intersect(this_str, other_str, exp_str)
    }

    #[test]
    fn test_intersection_3() -> Result<(), String> {
        /*
             Case 3
             ------
             (-0.5,2)____________ (1,2)
             |                    |
             | (0,1)..............(1,1)
             |      ://///////////:
             |      ://///////////:
             |______:............ :
        (-0.5,0)  (0,0)          (1,0)

              */
        let this_str = "[
            0, 0, 0,
            1, 0, 0,
            1, 1, 0,
            0, 1, 0
        ]";
        let other_str = "[
            -0.5, 0, 0,
            1.0,  0, 0,
            1.0,  2, 0,
            -0.5, 2, 0
        ]";
        let exp_str = "[
            0, 0, 0,
            1.0, 0, 0,
            1.0, 1, 0,
            0, 1, 0
        ]";
        try_to_intersect(this_str, other_str, exp_str)
    }

    #[test]
    fn test_intersection_4() -> Result<(), String> {
        /*
        Case 4
        ------

         ___________       .........
        |          |       :       :
        |          |       :       :
        |          |       :.......:
        |__________|

         */
        let this_str = "[
                4, 4, 0,
                5, 4, 0,
                5, 5, 0,
                4, 5, 0
            ]";
        let other_str = "[
                0,0,0,
                1,0,0,
                1,1,0,
                0,1,0
            ]";

        let this_loop: Loop3D = serde_json::from_str(this_str).map_err(|e| e.to_string())?;
        let other: Loop3D = serde_json::from_str(other_str).map_err(|e| e.to_string())?;
        assert!(this_loop.intersection(&other).unwrap().is_none());
        Ok(())
    }

    #[test]
    fn test_intersection_5() -> Result<(), String> {
        /*
        Case 5 :
        ------

         THEY ARE THE SAME POLYGON

         */
        let other_str = "[
                4, 4, 0,
                5, 4, 0,
                5, 5, 0,
                4, 5, 0
            ]";
        let this_str = other_str.clone();
        let exp_str = other_str.clone();
        try_to_intersect(this_str, other_str, exp_str)
    }

    #[test]
    fn test_intersection_6() -> Result<(), String> {
        /*
        Case 6
        ------

         */
        let this_str = "[
                -0.5, 0, 0,
                0.5, 0, 0,
                0.5, 2, 0,
                -0.5, 2, 0
            ]";
        let other_str = "[
                0,1,0,
                1,1,0,
                1,0,0,
                0,0,0
            ]";

        let exp_str = "[
            0,0,0,
            0.5, 0,0,
            0.5, 1, 0,
            0, 1, 0
        ]";
        try_to_intersect(this_str, other_str, exp_str)
    }

    #[test]
    fn test_intersection_7() -> Result<(), String> {
        /* CASE 7
        ---------
        */
        let this_str = "[
            -1.9148658671263563,-8.9269446089153,-1.106214891352675,
            -5.164369097543792,-1.5891449751178979,-1.106214891352675,
            -5.164369097543792,-1.5891449751178979,1.315362463352675,
            -1.9148658671263563,-8.9269446089153,1.315362463352675
        ]";
        let other_str = "[
            -1.9148659536009056,-8.926944428203846,1.315362463352675,
            -1.9148659536009056,-8.926944428203846,-1.106214891352675,
            2.6397675407798626,-6.909950781785453,-1.106214891352675,
            2.6397675407798626,-6.909950781785453,1.315362463352675
        ]";
        let this_loop: Loop3D = serde_json::from_str(this_str).map_err(|e| e.to_string())?;
        let other: Loop3D = serde_json::from_str(other_str).map_err(|e| e.to_string())?;
        assert!(this_loop.intersection(&other).unwrap().is_none());

        Ok(())
    }

    #[test]
    fn test_intersection_8() -> Result<(), String> {
        /*
        CASE 8
        -----
        */
        let this_str = "[
            -0.008626797879489168,-2.6685611144615793,-1.106214891352675,
            -1.7786483547938774,1.32837723131527,-1.106214891352675,
            -1.7786483547938774,1.32837723131527,1.315362463352675,
            -0.008626797879489168,-2.6685611144615793,1.315362463352675
        ]";
        let other_str = "[
            -0.00862690382051734,-2.6685608894532376,-1.106214891352675,
            1.9959152187135276,-7.195077465690807,-1.106214891352675,
            1.9959152187135276,-7.195077465690807,1.315362463352675,
            -0.00862690382051734,-2.6685608894532376,1.315362463352675
        ]";
        let this_loop: Loop3D = serde_json::from_str(this_str).map_err(|e| e.to_string())?;
        let other: Loop3D = serde_json::from_str(other_str).map_err(|e| e.to_string())?;
        let r = this_loop.intersection(&other);
        // let i = r.clone().unwrap().unwrap();
        // let s = serde_json::to_string(&i).unwrap();
        // println!("{}", s);
        assert!(r.unwrap().is_none());

        Ok(())
    }

    #[test]
    fn test_intersection_9() -> Result<(), String> {
        /*
        Case 9
        ------


         */
        let this_str = "[
            4.735886190257991,-9.605338334156942,-1.106215,
            3.9133217092123624,-9.970421664585183,-1.106215,
            3.2400923911335218,-8.453579296160871,-1.106215,
            0.827023652036901,-9.5245856811888,-1.106215,
            0.17662216252091456,-8.059175949412799,-1.106215,
            2.6625769501030905,-6.955819676971018,-1.106215,
            0.4620758072674953,-1.997905860655571,-1.106215,
            1.8211025741736804,-1.3947197681514467,-1.106215,
            2.2930822574289285,-2.458130187957125,-1.106215,
            4.890745461820698,-7.749647974425485,-1.106215,
            4.078713091250105,-8.110056580404846,-1.106215
        ]";
        let other_str = "[
            6.083176155134654,-7.917468659116879,-1.106215,
            5.023166673486083,-8.387603162100461,-1.106215,
            4.786487431882383,-7.85396352065159,-1.106215,
            4.188518380399172,-8.119173206108293,-1.106215,
            3.176839736033784,-5.838147720778049,-1.106215,
            4.834817224226316,-5.102802409572382,-1.106215
        ]";

        let exp_str = "[
            4.890745461820698,-7.749647974425485,-1.106215,
            4.167080001919266,-8.070836228799163,-1.106215,
            3.176839736033784,-5.838147720778049,-1.106215,
            3.8137055203697434,-5.555685340128171,-1.106215
        ]";
        try_to_intersect(this_str, other_str, exp_str)
    }

    #[test]
    fn test_intersection_10() -> Result<(), String> {
        /*
        Case 10
        ------
         */

        let this_str = "[
            3.211889965881089,-8.517330829518581,1.315362463352675,
            0.15559801750397328,-8.010051068340946,1.315362463352675,
            0.8445126043871549,-9.565710884945192,1.315362463352675
        ]";
        let other_str = "[
            0.5657215777665612,-9.689171373805332,1.315362463352675,
            3.874955841552043,-10.014617998386214,1.315362463352675,
            3.211889965881089,-8.517330829518581,1.315362463352675
        ]";

        let this_loop: Loop3D = serde_json::from_str(this_str).map_err(|e| e.to_string())?;
        let other: Loop3D = serde_json::from_str(other_str).map_err(|e| e.to_string())?;
        let r = this_loop.intersection(&other);

        // let i = r.clone().unwrap().unwrap();
        // let s = serde_json::to_string(&i).unwrap();
        // println!("{}", s);

        assert!(r.unwrap().is_none());

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

        let seg = Segment3D::new(Point3D::new(-10., 0., 0.), Point3D::new(10., 0., 0.));

        let v = this_loop.split(&seg)?;
        assert_eq!(v.len(), 1);

        assert!(this_loop.is_equal(&v[0], 1e-3)?);
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

    #[test]
    fn test_containing_segment() -> Result<(), String> {
        fn check(l: &Loop3D, p: Point3D, exp: Option<usize>) -> Result<(), String> {
            let contained = l.containing_segment(p);
            if contained == exp {
                Ok(())
            } else {
                if contained.is_none() && exp.is_some() {
                    return Err("Expecting point to be in a segment".to_string());
                } else if contained.is_some() && exp.is_none() {
                    return Err("NOT expecting point to be in a segment".to_string());
                } else {
                    // the number does not agree.
                    let foundi = contained.unwrap();
                    let expi = exp.unwrap();
                    return Err(format!(
                        "Expecting segment to be {}... found {}",
                        expi, foundi
                    ));
                }
            }
        }

        let loop_str = "[
                    0, 0, 0,
                    1, 0, 0,
                    1, 1, 0,
                    0, 1, 0
                ]";
        let theloop: Loop3D = serde_json::from_str(loop_str).map_err(|e| e.to_string())?;

        check(&theloop, theloop[0], Some(0))?; // is in first segment
        check(&theloop, Point3D::new(0.5, 0., 0.), Some(0))?;
        check(&theloop, theloop[1], Some(0))?; // is also on first segment
        check(&theloop, Point3D::new(1.0, 0.5, 0.), Some(1))?;
        check(&theloop, theloop[2], Some(1))?; // is in second
        check(&theloop, Point3D::new(0.5, 1., 0.), Some(2))?;
        check(&theloop, theloop[3], Some(2))?; // is in third segment
        check(&theloop, Point3D::new(0.0, 0.5, 0.), Some(3))?;

        Ok(())
    }

    #[test]
    fn test_is_equal() -> Result<(), String> {
        let loop_str = "[
                    0, 0, 0,
                    1, 0, 0,
                    1, 1, 0,
                    0, 1, 0
                ]";
        let theloop: Loop3D = serde_json::from_str(loop_str).map_err(|e| e.to_string())?;
        let mut theloop_rev = theloop.clone();
        theloop_rev.reverse();

        println!("{}", theloop);

        assert!(theloop.is_equal(&theloop, 1e-3)?);
        assert!(theloop.is_equal(&theloop_rev, 1e-3)?);

        let other_str = "[
                    0, 1, 0,
                    0, 0, 0,
                    1, 0, 0,
                    1, 1, 0
                ]";
        let other_loop: Loop3D = serde_json::from_str(other_str).map_err(|e| e.to_string())?;
        assert!(theloop.is_equal(&other_loop, 1e-3)?);

        let other_str = "[
                    0, 2, 0,
                    0, 0, 0,
                    1, 0, 0,
                    1, 1, 0
                ]";
        let other_loop: Loop3D = serde_json::from_str(other_str).map_err(|e| e.to_string())?;
        assert!(!theloop.is_equal(&other_loop, 1e-3)?);

        let other_str = "[
                    0, 1, 0,
                    0, 0, 0,
                    1, 0, 0
                ]";
        let other_loop: Loop3D = serde_json::from_str(other_str).map_err(|e| e.to_string())?;
        assert!(!theloop.is_equal(&other_loop, 1e-3)?);

        Ok(())
    }
}
