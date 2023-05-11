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

use serde::{Deserialize, Deserializer, Serialize};

use crate::Float;
use crate::{Loop3D, Point3D, Segment3D, Vector3D};

/// A 3-dimensional Polygon which can contain holes.
///
/// It is a structure containing several [`Loop3D`]. One of them is
/// the `outer` [`Loop3D`], and there there can be several `inner` [`Loop3D`].
#[derive(Debug, Clone)]
pub struct Polygon3D {
    /// The outer [`Loop3D`]
    outer: Loop3D,

    /// A vector of inner [`Loop3D`]
    inner: Vec<Loop3D>,

    /// The area of the `Polygon3D`    
    area: Float,

    /// The normal of the `Polygon3D`, following a right-hand convention
    normal: Vector3D,
}

impl std::convert::From<Loop3D> for Polygon3D {
    fn from(outer: Loop3D) -> Self {
        let area = outer
            .area()
            .expect("Trying to convert a non-closed Loop3D into a Polygon3D");
        let normal = outer.normal();

        Self {
            outer,
            inner: Vec::with_capacity(0),
            area,
            normal,
        }
    }
}

impl<'de> Deserialize<'de> for Polygon3D {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let data: Loop3D = Deserialize::deserialize(deserializer)?;

        Ok(data.into())
    }
}

impl Serialize for Polygon3D {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let l = self.get_closed_loop();
        l.serialize(serializer)
    }
}

impl Polygon3D {
    /// Creates a new [`Loop3D`] without any holes
    pub fn new(outer: Loop3D) -> Result<Polygon3D, String> {
        if !outer.is_closed() {
            return Err("Trying to create a Polygon3D from a loop that is not closed".to_string());
        }

        let area = outer.area()?;
        let normal = outer.normal();

        Ok(Polygon3D {
            outer,
            area,
            normal,
            inner: Vec::new(),
        })
    }

    // pub fn normal_at_intersection(&self, ray: &Ray3D, t: Float) -> (Vector3D, SurfaceSide) {
    //     let p = self.outer[0];
    //     SurfaceSide::get_side(self.normal, ray.direction)
    // }

    /// Retrieves the normal to the Polygon
    pub fn normal(&self) -> Vector3D {
        self.normal
    }

    /// Calculates the average of the [`Point3D`] in the Outer [`Loop3D`]
    pub fn outer_centroid(&self) -> Point3D {
        let mut centroid = Point3D::new(0., 0., 0.);
        let outer = self.outer();
        for v in outer.vertices() {
            centroid += *v;
        }
        centroid / (outer.len() as Float)
    }

    /// Borrows the Outer [`Loop3D`]
    pub fn outer(&self) -> &Loop3D {
        &self.outer
    }

    /// Clones the Outer [`Loop3D`]
    pub fn clone_outer(&self) -> Loop3D {
        self.outer.clone()
    }

    /// Counts the inner [`Loop3D`]
    pub fn n_inner_loops(&self) -> usize {
        self.inner.len()
    }

    /// Borrows an inner [`Loop3D`]
    pub fn inner(&self, i: usize) -> Result<&Loop3D, String> {
        if i < self.inner.len() {
            return Ok(&self.inner[i]);
        }
        let msg = "Index out of bounds when trying to retrieve inner loop".to_string();
        Err(msg)
    }

    /// Checks whether a [`Point3D`] is inside the [`Polygon3D`]
    pub fn test_point(&self, p: Point3D) -> Result<bool, String> {
        // Must be within the outer loop
        if !self.outer.test_point(p)? {
            return Ok(false);
        }

        // And outside all inner loops.
        for i in 0..self.inner.len() {
            let lp = &self.inner[i];

            let result = lp.test_point(p);

            let is_in = match result {
                Ok(b) => b,
                Err(e) => return Err(e),
            };
            // if it is in one of the inner loops, then it is outside
            // the shape.
            if is_in {
                return Ok(false);
            }
        }
        // If it was not in any of the inner loops, then it is in.
        Ok(true)
    }

    /// Makes a hole shaped as a [`Loop3D`] in the [`Polygon3D`].
    pub fn cut_hole(&mut self, hole: Loop3D) -> Result<(), String> {
        // Check that the normals are the same

        if !self.normal.is_parallel(hole.normal()) {
            let msg = "The hole you are trying to cut and the Polygon3D that should receive it do not have parallel normals".to_string();
            return Err(msg);
        }

        // Check that all the points are inside the polygon.point3d
        for p in hole.vertices() {
            if !self.test_point(*p)? {
                let msg = "At least one of the points in your hole are not inside the polygon"
                    .to_string();
                return Err(msg);
            }
        }

        // Check that this hole does not contain another hole in it.
        let n_inner = self.inner.len();
        for n_loop in 0..n_inner {
            let inner = &self.inner[n_loop];

            for inner_p in inner.vertices() {
                // Check point in the hole we are making
                if hole.test_point(*inner_p)? {
                    let msg = "Apparently another hole in your Polygon3D would be inside the new hole you are making".to_string();
                    return Err(msg);
                }
            }
        }

        // All good now! Add it.
        let hole_area = hole.area()?;

        // reduce area
        self.area -= hole_area;

        // Add it to the inner
        self.inner.push(hole);

        Ok(())
    }

    /// Gets the area of the [`Polygon3D`]
    pub fn area(&self) -> Float {
        self.area
    }

    /// Gets a single [`Loop3D`] representing the same [`Polygon3D`]
    /// geometry, but without any holes
    ///
    /// This is a pretty terible algorithm... but I doubt
    /// it is the bottleneck in my applications.
    ///
    /// It is also not 100% guaranteed to work all the time, but
    /// it has proven to be pretty robust for relatively normal
    /// geometries.

    pub fn get_closed_loop(&self) -> Loop3D {
        //get the number of interior loops
        let n_inner_loops = self.inner.len();

        //initialize the loop by cloning the current outer loop
        let mut ret_loop = self.outer.clone();
        ret_loop.open();

        // We will use this for checking whether the inner
        // loop should be reversed or not.
        let outer_normal = self.outer.normal();

        let mut processed_inner_loops: Vec<usize> = Vec::new();

        let mut inner_loop_id = 0;
        let mut inner_vertex_id = 0;

        // This is done once per inner loop
        for _i in 0..n_inner_loops {
            // find the minimum distance
            // from interior to exterior
            let mut min_distance = 9E14;
            let mut min_inner_loop_id = 0;
            let mut min_ext_vertex_id = 0;
            //let mut min_int_vertex_id = 0;

            let n_ext_vertices = ret_loop.len();
            for j in 0..n_ext_vertices {
                let ext_vertex = ret_loop[j];

                for k in 0..n_inner_loops {
                    // continue if already processed
                    if processed_inner_loops.contains(&k) {
                        continue;
                    }

                    let inner_loop = &self.inner[k];
                    let n_inner_vertices = inner_loop.len();
                    for l in 0..n_inner_vertices {
                        let inner_vertex = inner_loop[l];

                        // we work with squared distances... the result is
                        // the same but the calculation is faster.
                        let distance = ext_vertex.squared_distance(inner_vertex);

                        if distance < min_distance {
                            min_distance = distance;
                            min_ext_vertex_id = j;
                            //min_int_vertex_id = l;
                            min_inner_loop_id = k;
                            inner_loop_id = k;
                            inner_vertex_id = l;
                        }
                    } //end iterating inner vertices
                } // end iterating inner loops
            } // end iterating exterior vertices

            // Now, pass the inner loop to the exterior loop
            // by connecting them
            let mut aux = Loop3D::new();

            for i in 0..n_ext_vertices {
                // Sequentially add all exterior vertices.
                let ext_vertex = ret_loop[i];

                // Add
                aux.push(ext_vertex).unwrap();

                // If we are in the vertex through which we want
                // to connect the interior loop, then go inside.
                if i == min_ext_vertex_id {
                    // add the interior loop... adding the first
                    // point twice (hence the +1)
                    let n_inner_loop_vertices = self.inner[min_inner_loop_id].len();
                    let inner_normal = self.inner[min_inner_loop_id].normal();

                    for j in 0..n_inner_loop_vertices + 1 {
                        let vertex_to_add = if outer_normal.is_same_direction(inner_normal) {
                            // If both in the same direction, then we need to
                            // add the interior in reverse.
                            (inner_vertex_id as i32 - j as i32) as usize % n_inner_loop_vertices
                        } else {
                            (inner_vertex_id as i32 + j as i32) as usize % n_inner_loop_vertices
                        };

                        let inner_vertex = self.inner[min_inner_loop_id][vertex_to_add];

                        let x = inner_vertex.x;
                        let y = inner_vertex.y;
                        let z = inner_vertex.z;

                        aux.push(Point3D::new(x, y, z)).unwrap();
                    }

                    //return to exterior ret_loop
                    aux.push(ext_vertex).unwrap();
                }
            }

            ret_loop = aux;

            // flag ret_loop as processed (instead of deleting it)
            processed_inner_loops.push(inner_loop_id);
        } // end iterating inner loops
          // ret_loop.close().unwrap();
        ret_loop
    } // end of get_closed_polygon

    /// Checks whether the [`Polygon3D`] contains a certain [`Segment3D`]
    /// in any of its loops
    pub fn contains_segment(&self, s: &Segment3D) -> bool {
        if self.outer.contains_segment(s) {
            return true;
        }
        for inner in self.inner.iter() {
            if inner.contains_segment(s) {
                return true;
            }
        }
        false
    }
}

/***********/
/* TESTING */
/***********/

#[cfg(test)]
mod testing {
    use super::*;

    #[test]
    fn serde_ok() {
        let a = "[
            0.0,0,0,  
            1.0,1,1,  
            2,3,-1
        ]";

        let pol: Polygon3D = serde_json::from_str(a).unwrap();
        let p = &pol.outer;

        assert_eq!(p.len(), 3);

        assert_eq!(p[0].x, 0.);
        assert_eq!(p[0].y, 0.);
        assert_eq!(p[0].z, 0.);

        assert_eq!(p[1].x, 1.);
        assert_eq!(p[1].y, 1.);
        assert_eq!(p[1].z, 1.);

        assert_eq!(p[2].x, 2.);
        assert_eq!(p[2].y, 3.);
        assert_eq!(p[2].z, -1.);

        println!("{}", serde_json::to_string(&pol).unwrap());
    }

    // use crate::vector3d::Vector3D;

    // #[test]
    // fn test_polygon_intersect() {
    //     // It should not work if we don't close it.
    //     let mut the_loop = Loop3D::new();
    //     let l = 20. as Float;
    //     the_loop.push(Point3D::new(-l, -l, 0.)).unwrap();
    //     the_loop.push(Point3D::new(l, -l, 0.)).unwrap();
    //     the_loop.push(Point3D::new(l, l, 0.)).unwrap();
    //     the_loop.push(Point3D::new(-l, l, 0.)).unwrap();

    //     the_loop.close().unwrap();

    //     let polygon = Polygon3D::new(the_loop).unwrap();

    //     let ray = Ray3D {
    //         origin: Point3D::new(2.1, -3.1, 100.),
    //         direction: Vector3D::new(0., 0., -1.),
    //     };
    //     if let Some(t) = polygon.intersect(&ray) {
    //         let (normal, side) = polygon.normal_at_intersection(&ray, t);
    //         assert_eq!(side, SurfaceSide::Front);
    //         assert_eq!(normal, Vector3D::new(0., 0., 1.));
    //         assert_eq!(t, 100.);
    //     } else {
    //         panic!("Did not intersect!")
    //     }

    //     let ray = Ray3D {
    //         origin: Point3D::new(2.1, -3.1, -100.),
    //         direction: Vector3D::new(0., 0., 1.),
    //     };
    //     if let Some(t) = polygon.intersect(&ray) {
    //         let (normal, side) = polygon.normal_at_intersection(&ray, t);
    //         assert_eq!(side, SurfaceSide::Back);
    //         assert_eq!(normal, Vector3D::new(0., 0., -1.));
    //         assert_eq!(t, 100.);
    //     } else {
    //         panic!("Did not intersect!")
    //     }
    // }

    #[test]
    fn test_new() {
        // It should not work if we don't close it.
        let mut the_loop = Loop3D::new();
        let l = 2. as Float;
        the_loop.push(Point3D::new(-l, -l, 0.)).unwrap();
        the_loop.push(Point3D::new(l, -l, 0.)).unwrap();
        the_loop.push(Point3D::new(l, l, 0.)).unwrap();
        the_loop.push(Point3D::new(-l, l, 0.)).unwrap();

        assert!(!Polygon3D::new(the_loop).is_ok());

        // It should work if we close it.
        let mut the_loop = Loop3D::new();
        let l = 2. as Float;
        the_loop.push(Point3D::new(-l, -l, 0.)).unwrap();
        the_loop.push(Point3D::new(l, -l, 0.)).unwrap();
        the_loop.push(Point3D::new(l, l, 0.)).unwrap();
        the_loop.push(Point3D::new(-l, l, 0.)).unwrap();
        the_loop.close().unwrap();

        let poly = Polygon3D::new(the_loop).unwrap();
        assert!((2. * l * 2. * l - poly.area).abs() < 1e-4);
        assert!(!poly.normal.is_zero());
    }

    #[test]
    fn test_test_cut_hole() {
        // A square with the center at the origin.
        /*****/
        let mut outer_loop = Loop3D::new();
        let l = 2. as Float;
        outer_loop.push(Point3D::new(-l, -l, 0.)).unwrap();
        outer_loop.push(Point3D::new(l, -l, 0.)).unwrap();
        outer_loop.push(Point3D::new(l, l, 0.)).unwrap();
        outer_loop.push(Point3D::new(-l, l, 0.)).unwrap();
        outer_loop.close().unwrap();

        let mut poly = Polygon3D::new(outer_loop).unwrap();
        assert!((2. * l * 2. * l - poly.area).abs() < 1e-4);
        assert_eq!(poly.inner.len(), 0);

        // Add hole
        let mut hole = Loop3D::new();
        let l = 2. as Float;
        hole.push(Point3D::new(-l / 2., -l / 2., 0.)).unwrap();
        hole.push(Point3D::new(l / 2., -l / 2., 0.)).unwrap();
        hole.push(Point3D::new(l / 2., l / 2., 0.)).unwrap();
        hole.push(Point3D::new(-l / 2., l / 2., 0.)).unwrap();
        hole.close().unwrap();

        poly.cut_hole(hole).unwrap();

        assert!(
            (poly.area - (2. * l * 2. * l - l * l)).abs() < 1e-4,
            "err is {}",
            (poly.area - 2. * l * 2. * l - l * l).abs()
        );
        assert_eq!(poly.inner.len(), 1);

        /* Add hole in a different plane */
        /*****/

        let mut outer_loop = Loop3D::new();
        let l = 2. as Float;
        outer_loop.push(Point3D::new(-l, -l, 0.)).unwrap();
        outer_loop.push(Point3D::new(l, -l, 0.)).unwrap();
        outer_loop.push(Point3D::new(l, l, 0.)).unwrap();
        outer_loop.push(Point3D::new(-l, l, 0.)).unwrap();
        outer_loop.close().unwrap();

        let mut poly = Polygon3D::new(outer_loop).unwrap();

        let mut hole = Loop3D::new();
        let l = 2. as Float;
        hole.push(Point3D::new(-l / 2., -l / 2., 0.)).unwrap();
        hole.push(Point3D::new(l / 2., -l / 2., 0.)).unwrap();
        hole.push(Point3D::new(0., l / 2., l)).unwrap();
        hole.close().unwrap();

        // should fail
        assert!(!poly.cut_hole(hole).is_ok());

        /* Add hole that contains another whole */
        /*****/

        let mut outer_loop = Loop3D::new();
        let l = 2. as Float;
        outer_loop.push(Point3D::new(-l, -l, 0.)).unwrap();
        outer_loop.push(Point3D::new(l, -l, 0.)).unwrap();
        outer_loop.push(Point3D::new(l, l, 0.)).unwrap();
        outer_loop.push(Point3D::new(-l, l, 0.)).unwrap();
        outer_loop.close().unwrap();
        let mut poly = Polygon3D::new(outer_loop).unwrap();

        let mut hole = Loop3D::new();
        let l = 2. as Float;
        hole.push(Point3D::new(-l / 2., -l / 2., 0.)).unwrap();
        hole.push(Point3D::new(l / 2., -l / 2., 0.)).unwrap();
        hole.push(Point3D::new(l / 2., l / 2., 0.)).unwrap();
        hole.push(Point3D::new(-l / 2., l / 2., 0.)).unwrap();
        hole.close().unwrap();
        poly.cut_hole(hole).unwrap();

        let mut hole = Loop3D::new();
        let l = 2. as Float;
        hole.push(Point3D::new(-l / 1.5, -l / 1.5, 0.)).unwrap();
        hole.push(Point3D::new(l / 1.5, -l / 1.5, 0.)).unwrap();
        hole.push(Point3D::new(l / 1.5, l / 1.5, 0.)).unwrap();
        hole.push(Point3D::new(-l / 1.5, l / 1.5, 0.)).unwrap();
        hole.close().unwrap();

        // should fail
        assert!(!poly.cut_hole(hole).is_ok());
    }

    #[test]
    fn test_get_closed_loop() {
        let mut outer = Loop3D::new();

        outer.push(Point3D::new(-2., -2., 0.)).unwrap();
        outer.push(Point3D::new(6., -2., 0.)).unwrap();
        outer.push(Point3D::new(6., 6., 0.)).unwrap();
        outer.push(Point3D::new(-2., 6., 0.)).unwrap();

        outer.close().unwrap();

        let mut p = Polygon3D::new(outer).unwrap();

        let mut inner = Loop3D::new();
        inner.push(Point3D::new(-1., -1., 0.)).unwrap(); // 3
        inner.push(Point3D::new(1., -1., 0.)).unwrap(); // 2
        inner.push(Point3D::new(1., 1., 0.)).unwrap(); // 1
        inner.push(Point3D::new(-1., 1., 0.)).unwrap(); // 0
        inner.close().unwrap();

        p.cut_hole(inner).unwrap();

        let closed = p.get_closed_loop();

        assert_eq!(closed.len(), 10);

        // 0
        let p = closed[0];
        assert!(p.compare(Point3D::new(-2., -2., 0.)));

        // 1
        let p = closed[1];
        assert!(p.compare(Point3D::new(-1., -1., 0.)));

        // 2
        let p = closed[2];
        assert!(p.compare(Point3D::new(-1., 1., 0.)));

        // 3
        let p = closed[3];
        assert!(p.compare(Point3D::new(1., 1., 0.)));

        // 4
        let p = closed[4];
        assert!(p.compare(Point3D::new(1., -1., 0.)));

        // 5
        let p = closed[5];
        assert!(p.compare(Point3D::new(-1., -1., 0.)));

        // 6... Back to exterior
        let p = closed[6];
        assert!(p.compare(Point3D::new(-2., -2., 0.)));

        // 7.
        let p = closed[7];
        assert!(p.compare(Point3D::new(6., -2., 0.)));

        // 8
        let p = closed[8];
        assert!(p.compare(Point3D::new(6., 6., 0.)));

        // 9
        let p = closed[9];
        assert!(p.compare(Point3D::new(-2., 6., 0.)));
    }

    #[test]
    fn test_get_closed_loop_with_clean() {
        let mut outer = Loop3D::new();
        outer.push(Point3D::new(-2., -2., 0.)).unwrap();
        outer.push(Point3D::new(0., -2., 0.)).unwrap(); // colinear point
        outer.push(Point3D::new(6., -2., 0.)).unwrap();
        outer.push(Point3D::new(6., 6., 0.)).unwrap();
        outer.push(Point3D::new(-2., 6., 0.)).unwrap();
        outer.close().unwrap();

        let mut p = Polygon3D::new(outer).unwrap();

        let mut inner_loop = Loop3D::new();
        inner_loop.push(Point3D::new(1., 1., 0.)).unwrap(); // 0
        inner_loop.push(Point3D::new(1., -1., 0.)).unwrap(); // 1
        inner_loop.push(Point3D::new(0., -1., 0.)).unwrap(); // 2. colinear point
        inner_loop.push(Point3D::new(-1., -1., 0.)).unwrap(); // 3
        inner_loop.push(Point3D::new(-1., 1., 0.)).unwrap(); // 4
        inner_loop.push(Point3D::new(0., 1., 0.)).unwrap(); // 5. colinear point
        inner_loop.close().unwrap();

        p.cut_hole(inner_loop).unwrap();

        let closed = p.get_closed_loop();

        assert_eq!(closed.len(), 10);

        // 0
        let p = closed[0];
        assert!(p.compare(Point3D::new(-2., -2., 0.)));

        // 1
        let p = closed[1];
        assert!(p.compare(Point3D::new(-1., -1., 0.)));

        // 2
        let p = closed[2];
        assert!(p.compare(Point3D::new(-1., 1., 0.)));

        // 3
        let p = closed[3];
        assert!(p.compare(Point3D::new(1., 1., 0.)));

        // 4
        let p = closed[4];
        assert!(p.compare(Point3D::new(1., -1., 0.)));

        // 5
        let p = closed[5];
        assert!(p.compare(Point3D::new(-1., -1., 0.)));

        // 6... Back to exterior
        let p = closed[6];
        assert!(p.compare(Point3D::new(-2., -2., 0.)));

        // 7.
        let p = closed[7];
        assert!(p.compare(Point3D::new(6., -2., 0.)));

        // 8
        let p = closed[8];
        assert!(p.compare(Point3D::new(6., 6., 0.)));

        // 9
        let p = closed[9];
        assert!(p.compare(Point3D::new(-2., 6., 0.)));
    }

    #[test]
    fn test_contains_segment() {
        let mut outer = Loop3D::new();
        let p0 = Point3D::new(-2., -2., 0.);
        let p1 = Point3D::new(2., -2., 0.);
        let p2 = Point3D::new(2., 2., 0.);
        let p3 = Point3D::new(-2., 2., 0.);

        outer.push(p0).unwrap(); // 0
        outer.push(p1).unwrap(); // 1
        outer.push(p2).unwrap(); // 2
        outer.push(p3).unwrap(); // 3
        outer.close().unwrap();

        let mut inner = Loop3D::new();
        let ip0 = Point3D::new(-1., -1., 0.);
        let ip1 = Point3D::new(1., -1., 0.);
        let ip2 = Point3D::new(1., 1., 0.);
        let ip3 = Point3D::new(-1., 1., 0.);
        inner.push(ip0).unwrap();
        inner.push(ip1).unwrap();
        inner.push(ip2).unwrap();
        inner.push(ip3).unwrap();
        inner.close().unwrap();

        let mut l = Polygon3D::new(outer).unwrap();
        l.cut_hole(inner).unwrap();

        // Existing segments, in both directions
        assert!(l.contains_segment(&Segment3D::new(p0, p1)));
        assert!(l.contains_segment(&Segment3D::new(p1, p2)));
        assert!(l.contains_segment(&Segment3D::new(p2, p3)));
        assert!(l.contains_segment(&Segment3D::new(p3, p0)));
        assert!(l.contains_segment(&Segment3D::new(p3, p2)));
        assert!(l.contains_segment(&Segment3D::new(p2, p1)));
        assert!(l.contains_segment(&Segment3D::new(p1, p0)));
        assert!(l.contains_segment(&Segment3D::new(p0, p3)));

        assert!(l.contains_segment(&Segment3D::new(ip0, ip1)));
        assert!(l.contains_segment(&Segment3D::new(ip1, ip2)));
        assert!(l.contains_segment(&Segment3D::new(ip2, ip3)));
        assert!(l.contains_segment(&Segment3D::new(ip3, ip0)));
        assert!(l.contains_segment(&Segment3D::new(ip3, ip2)));
        assert!(l.contains_segment(&Segment3D::new(ip2, ip1)));
        assert!(l.contains_segment(&Segment3D::new(ip1, ip0)));
        assert!(l.contains_segment(&Segment3D::new(ip0, ip3)));

        // Diagonals
        assert!(!l.contains_segment(&Segment3D::new(p1, p3)));
        assert!(!l.contains_segment(&Segment3D::new(p3, p1)));
        assert!(!l.contains_segment(&Segment3D::new(p0, p2)));
        assert!(!l.contains_segment(&Segment3D::new(p2, p0)));

        assert!(!l.contains_segment(&Segment3D::new(ip1, ip3)));
        assert!(!l.contains_segment(&Segment3D::new(ip3, ip1)));
        assert!(!l.contains_segment(&Segment3D::new(ip0, ip2)));
        assert!(!l.contains_segment(&Segment3D::new(ip2, ip0)));

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
    }

    
}
