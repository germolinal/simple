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

/*
THIS CODE IS HEAVILY INSPIRED (rip-off?) IN
Physically Based Rendering: From Theory To Implementation, © 2004-2021 Matt Pharr, Wenzel Jakob, and Greg Humphreys
https://pbr-book.org/3ed-2018/Primitives_and_Intersection_Acceleration/Bounding_Volume_Hierarchies
 */

use crate::scene::Scene;
use crate::triangle::*;
use crate::Float;
use geometry::{BBox3D, BBoxAxis, Point3D, Ray3D, Vector3D};
use std::cmp::Ordering;

use crate::{ax, ay, az, bx, by, bz, cx, cy, cz};

#[derive(Copy, Clone)]
struct BucketInfo {
    count: usize,
    bounds: Option<BBox3D>,
}

fn get_bucket_index(
    centroid: Point3D,
    len_axis: Float,
    split_axis: BBoxAxis,
    n_buckets: usize,
    min_centroid: Point3D,
) -> usize {
    // Identify which bucket contains this object's centroid
    let (centroid_pos, min) = match split_axis {
        BBoxAxis::X => (centroid.x, min_centroid.x),
        BBoxAxis::Y => (centroid.y, min_centroid.y),
        BBoxAxis::Z => (centroid.z, min_centroid.z),
    };
    let mut bucket_index = ((centroid_pos - min) * n_buckets as Float / len_axis).floor() as usize;
    debug_assert!(bucket_index <= n_buckets);
    if bucket_index >= n_buckets {
        // If we are in the upper limit, this can happen
        bucket_index = n_buckets - 1;
    }

    bucket_index
}

/// A struct that is instrumental for building the  `BoundingVolumeTree`
struct ObjectInfo {
    index: usize,
    bounds: BBox3D,
    centroid: Point3D,
}

impl ObjectInfo {
    fn new(scene: &Scene, index: usize) -> Self {
        let bounds = world_bounds(&scene.triangles[index]);
        let centroid = (bounds.max + bounds.min) * 0.5;
        Self {
            index,
            bounds,
            centroid,
        }
    }
}

struct Interior {
    bounds: BBox3D,
    split_axis: BBoxAxis,
    children: (Box<Node>, Box<Node>),
}

enum Node {
    Interior(Interior),
    Leaf(Leaf),
}

struct DestructuredTriangle {
    ax: Float,
    ay: Float,
    az: Float,
    bx: Float,
    by: Float,
    bz: Float,
    cx: Float,
    cy: Float,
    cz: Float,
}

impl DestructuredTriangle {
    fn new(scene: &Scene, index: usize) -> Self {
        let t = scene.triangles[index];
        Self {
            ax: ax![t],
            ay: ay![t],
            az: az![t],
            bx: bx![t],
            by: by![t],
            bz: bz![t],
            cx: cx![t],
            cy: cy![t],
            cz: cz![t],
        }
    }

    fn into_array(&self) -> Triangle {
        [
            self.ax, self.ay, self.az, self.bx, self.by, self.bz, self.cx, self.cy, self.cz,
        ]
    }
}

impl Node {
    fn bounds(&self) -> BBox3D {
        match self {
            Self::Leaf(l) => l.bounds,
            Self::Interior(i) => i.bounds,
        }
    }

    fn new_leaf(first: usize, n_prims: usize, bounds: BBox3D) -> Self {
        Self::Leaf(Leaf {
            bounds,
            first_prim_offset: first,
            n_prims,
        })
    }

    fn new_interior(split_axis: BBoxAxis, child1: Node, child2: Node) -> Self {
        let bounds = BBox3D::from_union(&child1.bounds(), &child2.bounds());

        Self::Interior(Interior {
            bounds,
            split_axis,
            children: (Box::new(child1), Box::new(child2)),
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn recursive_build(
        scene: &Scene,
        primitives_info: &mut [ObjectInfo],
        start: usize,
        end: usize,
        total_nodes: &mut usize,
        ordered_triangles: &mut Vec<DestructuredTriangle>,
        ordered_mapping: &mut Vec<usize>,
        ordered_front_materials: &mut Vec<usize>,
        ordered_back_materials: &mut Vec<usize>,
        ordered_normals: &mut Vec<(Vector3D, Vector3D, Vector3D)>,
    ) -> Self {
        let front_materials = &scene.front_material_indexes;
        let back_materials = &scene.back_material_indexes;
        let normals = &scene.normals;

        debug_assert!(start < end);
        *total_nodes += 1;
        // The whole point of most of this function is to idenfity
        // the value this 'mid' variable should have... we are creating
        // a binary tree, you know, so we split things in halves
        let mut mid: Option<usize> = None;

        // Get a BBOX containing EVERYTHING within scope
        let mut bounds = primitives_info[start].bounds;

        for info in primitives_info.iter().take(end).skip(start + 1) {
            bounds = BBox3D::from_union(&bounds, &info.bounds);
        }
        let n_primitives = end - start;
        if n_primitives == 1 {
            // Create Leaf
            let first_prim_offset = ordered_triangles.len();
            for info in primitives_info.iter().take(end).skip(start) {
                let index = info.index;
                ordered_triangles.push(DestructuredTriangle::new(scene, index));
                ordered_mapping.push(index);
                ordered_back_materials.push(back_materials[index]);
                ordered_front_materials.push(front_materials[index]);
                ordered_normals.push(normals[index]);
            }
            return Node::new_leaf(first_prim_offset, n_primitives, bounds);
        }

        // Calculate the the BBOX of the centroids
        let mut centroids_bbox = BBox3D::from_point(primitives_info[start].centroid);
        for prim_info in primitives_info.iter().take(end).skip(start + 1) {
            centroids_bbox = BBox3D::from_union_point(&centroids_bbox, prim_info.centroid);
        }

        let split_axis = centroids_bbox.max_extent();
        // the extent of the centroids in the largest dimension
        let len_axis = match split_axis {
            BBoxAxis::X => centroids_bbox.max.x - centroids_bbox.min.x,
            BBoxAxis::Y => centroids_bbox.max.y - centroids_bbox.min.y,
            BBoxAxis::Z => centroids_bbox.max.z - centroids_bbox.min.z,
        };
        // Define some auxiliary functions
        let cmp_float = |a: Float, b: Float| -> Ordering {
            if a < b {
                Ordering::Less
            } else if a > b {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        };
        let cmp_centroids = |a: &ObjectInfo, b: &ObjectInfo| -> Ordering {
            match split_axis {
                BBoxAxis::X => cmp_float(a.centroid.x, b.centroid.x),
                BBoxAxis::Y => cmp_float(a.centroid.y, b.centroid.y),
                BBoxAxis::Z => cmp_float(a.centroid.z, b.centroid.z),
            }
        };

        const TOO_FEW_TO_BUCKET: usize = 4;
        const N_BUCKETS: usize = 12;
        const RELATIVE_TRANSVERSAL_COST: Float = 10.;
        // Now, Proceed
        if len_axis < 1e-8 {
            // All primitives seem to be aligned in all directions (i.e., overlapping)
            // Put them al together in a Leaf
            let first_prim_offset = ordered_triangles.len();
            // for i in start..end{
            for prim_info in primitives_info.iter().take(end).skip(start) {
                let index = prim_info.index;
                ordered_triangles.push(DestructuredTriangle::new(scene, index));
                ordered_mapping.push(index);
                ordered_back_materials.push(back_materials[index]);
                ordered_front_materials.push(front_materials[index]);
                ordered_normals.push(normals[index]);
            }
            return Node::new_leaf(first_prim_offset, n_primitives, bounds);
        } else {
            // Line 306 of https://github.com/mmp/pbrt-v3/blob/master/src/accelerators/bvh.cpp
            if n_primitives <= TOO_FEW_TO_BUCKET {
                // too few... just split in half,
                let this_mid = (end - start) / 2;

                mid = Some(this_mid); // this operation rounds results
                                      // and sort the relevant range in the array
                primitives_info[start..end].select_nth_unstable_by(this_mid, cmp_centroids);
            } else {
                // Use the Surface Area Heuristic...

                // First, put all the elements in a bucket
                let mut buckets: Vec<BucketInfo> = vec![
                    BucketInfo {
                        count: 0,
                        bounds: None
                    };
                    N_BUCKETS
                ];

                for prim_info in primitives_info.iter().take(end).skip(start) {
                    // Identify which bucket contains this object's centroid
                    let bucket_index = get_bucket_index(
                        prim_info.centroid,
                        len_axis,
                        split_axis,
                        N_BUCKETS,
                        centroids_bbox.min,
                    );

                    // Register
                    buckets[bucket_index].count += 1;
                    match buckets[bucket_index].bounds {
                        Some(b) => {
                            buckets[bucket_index].bounds =
                                Some(BBox3D::from_union_point(&b, prim_info.centroid))
                        }
                        None => {
                            buckets[bucket_index].bounds =
                                Some(BBox3D::from_point(prim_info.centroid))
                        }
                    }
                }

                // Compute costs of splitting after each bucket
                let mut min_cost = Float::MAX;
                let mut min_cost_bucket = 0;
                for i in 0..N_BUCKETS - 1 {
                    // Properties before... initialize as first bucket
                    let mut before = buckets[0].bounds.unwrap();
                    let mut count_before = buckets[0].count;

                    for bucket in buckets.iter().take(i + 1).skip(1) {
                        if let Some(b) = &bucket.bounds {
                            before = BBox3D::from_union(&before, b);
                            count_before += bucket.count;
                        }
                    }
                    // Properties after... initialize as last bucket
                    let mut after = buckets[N_BUCKETS - 1].bounds.unwrap();
                    let mut count_after = buckets[N_BUCKETS - 1].count;
                    // for j in (i+1)..(N_BUCKETS-1){
                    for bucket in buckets.iter().take(N_BUCKETS - 1).skip(i + 1) {
                        if let Some(b) = &bucket.bounds {
                            after = BBox3D::from_union(&after, b);
                            count_after += bucket.count;
                        }
                    }

                    let c = RELATIVE_TRANSVERSAL_COST
                        + (count_before as Float * before.surface_area()
                            + count_after as Float * after.surface_area())
                            / bounds.surface_area();
                    if c < min_cost {
                        min_cost = c;
                        min_cost_bucket = i;
                    }
                } // end of calculating bucket's costs

                // Decide whether to keep splitting or not.
                let leaf_cost = n_primitives as Float;
                if n_primitives > crate::triangle::LEAF_SIZE || min_cost < leaf_cost {
                    // We need or want to subdivide... create interior

                    // Sort based on centroid position
                    primitives_info[start..end].sort_unstable_by(cmp_centroids);
                    // Identify the first primitive that is
                    for (index, i) in (start..end).enumerate() {
                        let bucket_index = get_bucket_index(
                            primitives_info[i].centroid,
                            len_axis,
                            split_axis,
                            N_BUCKETS,
                            centroids_bbox.min,
                        );
                        if bucket_index <= min_cost_bucket {
                            mid = Some(index + 1) // update mid.
                        } else {
                            break; // we are past (these are sorted)... just break and prepare to return.
                        }
                    }
                } else {
                    // Don't subdivide... create leaf
                    let first = ordered_triangles.len();
                    let n_prims = n_primitives;
                    for prim in primitives_info.iter().take(end).skip(start) {
                        let prim_num = prim.index;
                        ordered_triangles.push(DestructuredTriangle::new(scene, prim_num));
                        ordered_mapping.push(prim_num);
                        ordered_back_materials.push(back_materials[prim_num]);
                        ordered_front_materials.push(front_materials[prim_num]);
                        ordered_normals.push(normals[prim_num]);
                    }
                    return Node::new_leaf(first, n_prims, bounds);
                }
            }
        }
        // If we have not returned a Leaf yet... split!
        let mid = mid.unwrap() + start;
        let child1 = Self::recursive_build(
            scene,
            primitives_info,
            start,
            mid,
            total_nodes,
            ordered_triangles,
            ordered_mapping,
            ordered_front_materials,
            ordered_back_materials,
            ordered_normals,
        );
        let child2 = Self::recursive_build(
            scene,
            primitives_info,
            mid,
            end,
            total_nodes,
            ordered_triangles,
            ordered_mapping,
            ordered_front_materials,
            ordered_back_materials,
            ordered_normals,
        );
        Node::new_interior(split_axis, child1, child2)
    }
}

#[derive(Clone)]
struct Leaf {
    bounds: BBox3D,
    n_prims: usize,
    first_prim_offset: usize,
}

#[derive(Clone, Debug)]
struct FlatNode {
    /// The Bounding Box of this node
    bounds: BBox3D,
    /// The number of primitives in the node. Interior Nodes
    /// have Zero; Leafs should always have more
    n_prims: i16,
    /// The axis in which this was split. This value should
    /// not be used in leafs
    axis: BBoxAxis,
    /// The 'next' node to check. This is the equivalent to
    /// what in PBR is called `primitivesOffset` for Leafs, and
    /// `secondChildOffset` for Interior nodes
    next: i32,
}

impl FlatNode {
    fn is_leaf(&self) -> bool {
        self.n_prims > 0
    }
}

#[derive(Debug, Default, Clone)]
pub struct BoundingVolumeTree {
    nodes: Vec<FlatNode>,
}

impl BoundingVolumeTree {
    pub fn new(scene: &mut Scene) -> (Self, Vec<usize>) {
        let n_objects = scene.triangles.len();
        if n_objects == 0 {
            return (Self::default(), Vec::with_capacity(0));
        }
        /*
        STEP 1:  First, bounding information about each primitive is computed and
        stored in an array that will be used during tree construction
        */
        let mut primitives_info: Vec<ObjectInfo> = Vec::with_capacity(n_objects);

        for i in 0..n_objects {
            primitives_info.push(ObjectInfo::new(scene, i))
        }

        /*
        STEP 2:  Next, the tree is built using the algorithm choice
        encoded in splitMethod. The result is a binary tree where
        each interior node holds pointers to its children and each
        leaf node holds references to one or more primitives.
        */
        let mut total_nodes = 0;
        let mut ordered_triangles: Vec<DestructuredTriangle> = Vec::with_capacity(n_objects);
        let mut ordered_mapping: Vec<usize> = Vec::with_capacity(n_objects);
        let mut ordered_front_materials: Vec<usize> = Vec::with_capacity(n_objects);
        let mut ordered_back_materials: Vec<usize> = Vec::with_capacity(n_objects);
        let mut ordered_normals: Vec<(Vector3D, Vector3D, Vector3D)> =
            Vec::with_capacity(n_objects);
        let root = Node::recursive_build(
            scene,
            &mut primitives_info,
            0,
            n_objects,
            &mut total_nodes,
            &mut ordered_triangles,
            &mut ordered_mapping,
            &mut ordered_front_materials,
            &mut ordered_back_materials,
            &mut ordered_normals,
        );

        // scene.triangles = ordered_triangles; // Update the Scene with the ordered primitive.
        scene.triangles = ordered_triangles.iter().map(|t| t.into_array()).collect();
        scene.front_material_indexes = ordered_front_materials;
        scene.back_material_indexes = ordered_back_materials;
        scene.normals = ordered_normals;

        /*
        STEP 3: Finally, this tree is converted to a more compact
        (and thus more efficient) pointerless representation for
        use during rendering.
        */
        let mut nodes: Vec<FlatNode> = Vec::with_capacity(total_nodes);
        Self::flatten_node(&root, &mut nodes);

        // return
        (Self { nodes }, ordered_mapping)
    }

    fn flatten_node(node: &Node, nodes: &mut Vec<FlatNode>) -> usize {
        let this_offset = nodes.len();
        match node {
            Node::Leaf(l) => {
                nodes.push(FlatNode {
                    bounds: l.bounds,
                    n_prims: l.n_prims as i16,
                    next: l.first_prim_offset as i32,
                    axis: BBoxAxis::X, // We won't use this
                });
            }
            Node::Interior(i) => {
                let (child1, child2) = &i.children;
                // nodes.push(FlatNode::Interior(FlatInterior {
                //     second_child_offset: 0, // We will fill this in a minute.
                //     bounds: i.bounds,
                //     axis: i.split_axis,
                // }));
                nodes.push(FlatNode {
                    bounds: i.bounds,
                    n_prims: 0,
                    next: 0, // We will patch this
                    axis: i.split_axis,
                });
                Self::flatten_node(child1, nodes);
                // Patch second offset
                nodes[this_offset].next = Self::flatten_node(child2, nodes) as i32;
            }
        }
        // return
        this_offset
    }

    #[inline(always)]
    fn pop_node<const N: usize>(
        &self,
        stack_size: &mut usize,
        stack: &[usize; N],
    ) -> Option<usize> {
        if *stack_size == 0 {
            None
        } else {
            let v = stack[*stack_size - 1];
            *stack_size -= 1;
            Some(v)
        }
    }

    #[inline(always)]
    fn push_node<const N: usize>(&self, stack_size: &mut usize, stack: &mut [usize; N], v: usize) {
        stack[*stack_size] = v;
        *stack_size += 1;
    }

    /// Returns an `Option<usize>`, containing the index of the [`Triangle`]
    /// to be hit by the ray, if any. The ray given will have the `interaction`
    ///
    pub fn intersect<const N: usize>(
        &self,
        scene: &Scene,
        ray: Ray3D,
        nodes_to_visit: &mut [usize; N], //&mut Vec<usize>,
    ) -> Option<(usize, Point3D)> {
        const MIN_T: Float = 0.0000001;

        if self.nodes.is_empty() {
            return None;
        }
        let mut stack_size = 0;

        let mut prim_index: Option<(usize, Point3D)> = None;
        let mut t_squared = Float::MAX;

        let inv_x = 1. / ray.direction.x;
        let inv_y = 1. / ray.direction.y;
        let inv_z = 1. / ray.direction.z;

        let inv_dir = Vector3D::new(inv_x, inv_y, inv_z);
        let dir_is_neg = (inv_dir.x < 0., inv_dir.y < 0., inv_dir.z < 0.);

        let mut current_node = 0;

        loop {
            let node = &self.nodes[current_node];
            if node.bounds.intersect(ray, &inv_dir) {
                if node.is_leaf() {
                    let offset = node.next;

                    // Check all the objects in this Node
                    let ini = offset as usize;
                    let fin = ini + node.n_prims as usize;

                    if let Some((i, p)) = simple_intersect_triangle_slice(scene, ray, ini, fin) {
                        // If hit, check the distance.
                        let this_t_squared = (p - ray.origin).length_squared();
                        // if the distance is less than the prevous one, update the info
                        if this_t_squared > MIN_T && this_t_squared < t_squared {
                            // If the distance is less than what we had, update return data
                            t_squared = this_t_squared;
                            prim_index = Some((i, p));
                        }
                    }

                    // update node we need to visit next, if any... otherwise, finish
                    if let Some(i) = self.pop_node(&mut stack_size, nodes_to_visit) {
                        current_node = i;
                    } else {
                        break;
                    }
                } else {
                    // is interior... choose first or second child,
                    // add to the stack
                    let is_neg = match node.axis {
                        BBoxAxis::X => dir_is_neg.0,
                        BBoxAxis::Y => dir_is_neg.1,
                        BBoxAxis::Z => dir_is_neg.2,
                    };
                    if is_neg {
                        self.push_node(&mut stack_size, nodes_to_visit, current_node + 1);
                        current_node = node.next as usize;
                    } else {
                        self.push_node(&mut stack_size, nodes_to_visit, node.next as usize);
                        current_node += 1;
                    }
                }
            } else if let Some(i) = self.pop_node(&mut stack_size, nodes_to_visit) {
                current_node = i;
            } else {
                break;
            }
        } // End loop

        // return
        // if let Some(_index, pt) = prim_index {
        //     let t = t_squared.sqrt();
        //     // ray.interaction.point = ray.geometry.project(t);
        //     // ray.interaction.wo = ray.geometry.direction * -1.;
        // }
        prim_index
    }

    /// Checks if a ray can travel a certain distance without hitting anything
    pub fn unobstructed_distance<const N: usize>(
        &self,
        scene: &Scene,
        ray: Ray3D,
        distance_squared: Float,
        nodes_to_visit: &mut [usize; N],
    ) -> bool {
        if self.nodes.is_empty() {
            return true;
        }
        // reset
        // nodes_to_visit.truncate(0);
        let mut stack_size = 0;

        // let d_squared = distance * distance;
        const MIN_T: Float = 0.000001;

        let inv_dir = Vector3D::new(
            1. / ray.direction.x,
            1. / ray.direction.y,
            1. / ray.direction.z,
        );
        let dir_is_neg = (inv_dir.x < 0., inv_dir.y < 0., inv_dir.z < 0.);
        let mut current_node = 0;

        loop {
            let node = &self.nodes[current_node];
            if node.bounds.intersect(ray, &inv_dir) {
                if node.is_leaf() {
                    let offset = node.next;

                    // Check all the objects in this Node
                    let ini = offset as usize;
                    let fin = ini + node.n_prims as usize;

                    /* NON_SIMD */

                    if let Some((_, p)) = simple_triangle_intersect(scene, ray, ini, fin) {
                        // If hit, check the distance.
                        let this_t_squared = (p - ray.origin).length_squared();

                        if this_t_squared > MIN_T
                            && this_t_squared + MIN_T < distance_squared
                            && (distance_squared - this_t_squared).abs() > 0.0001
                        {
                            return false;
                        }
                    }

                    if let Some(i) = self.pop_node(&mut stack_size, nodes_to_visit) {
                        current_node = i;
                    } else {
                        break;
                    }
                } else {
                    let is_neg = match node.axis {
                        BBoxAxis::X => dir_is_neg.0,
                        BBoxAxis::Y => dir_is_neg.1,
                        BBoxAxis::Z => dir_is_neg.2,
                    };
                    if is_neg {
                        // nodes_to_visit.push(current_node + 1);
                        self.push_node(&mut stack_size, nodes_to_visit, current_node + 1);
                        current_node = node.next as usize;
                    } else {
                        self.push_node(&mut stack_size, nodes_to_visit, node.next as usize);
                        current_node += 1;
                    }
                }
            } else if let Some(i) = self.pop_node(&mut stack_size, nodes_to_visit) {
                current_node = i;
            } else {
                break;
            }
        } // End loop

        // otherwise, return true
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::colour::Spectrum;
    use crate::material::Material;
    use crate::material::Plastic;

    use geometry::{Point3D, Ray3D, Sphere3D};

    use crate::primitive::Primitive;

    #[test]
    fn test_pop_node() {
        let mut scene = Scene::new();
        let (bvh, _) = BoundingVolumeTree::new(&mut scene);
        let mut stack_size = 0;
        let mut stack = [0; 10];
        assert!(bvh.pop_node(&mut stack_size, &mut stack).is_none());

        let mut stack_size = 1;
        let mut stack = [1, 2, 3, 13, 14];
        let node = bvh.pop_node(&mut stack_size, &mut stack);
        assert!(node.is_some());
        assert_eq!(node.unwrap(), 1);
        assert_eq!(stack_size, 0);

        let mut stack_size = 3;
        let mut stack = [1, 2, 3, 13, 14];
        let node = bvh.pop_node(&mut stack_size, &mut stack);
        assert!(node.is_some());
        assert_eq!(node.unwrap(), 3);
        assert_eq!(stack_size, 2);
    }

    #[test]
    fn test_push_node() {
        let mut scene = Scene::new();
        let (bvh, _) = BoundingVolumeTree::new(&mut scene);
        let mut stack_size = 0;
        let mut stack = [0; 2];

        bvh.push_node(&mut stack_size, &mut stack, 2);
        assert_eq!(stack_size, 1);
        assert_eq!([2, 0], stack);

        bvh.push_node(&mut stack_size, &mut stack, 12);
        assert_eq!(stack_size, 2);
        assert_eq!([2, 12], stack)
    }

    #[test]
    fn test_empty() {
        let mut scene = Scene::new();
        let ray = Ray3D {
            origin: Point3D::new(0., 0., 0.),
            direction: Vector3D::new(0., 0., 0.),
        };
        let (bvh, _) = BoundingVolumeTree::new(&mut scene);
        let mut aux = [0; 32];
        assert!(bvh.intersect(&scene, ray, &mut aux).is_none());
    }

    /// A simple scene with two 0.5-r-spheres; one at x = -1 and the other
    /// at x = 1. This should lead to three nodes:
    /// * The main one, being an interior node (e.g., n_prims == 0)
    /// * First child --> Leaf with one element
    /// * Second child --> Leaf with one element

    fn get_horizontal_scene() -> Scene {
        let mut scene = Scene::new();

        let plastic = Plastic {
            colour: Spectrum::from(2.),
            specularity: 1.,
            roughness: 0.,
        };
        let plastic = scene.push_material(Material::Plastic(plastic));

        // One sphere
        let sphere = Sphere3D::new(0.5, Point3D::new(-1., 0., 0.));
        scene.push_object(plastic, plastic, Primitive::Sphere(sphere));

        // Another sphere
        let sphere = Sphere3D::new(0.5, Point3D::new(1., 0., 0.));
        scene.push_object(plastic, plastic, Primitive::Sphere(sphere));

        scene
    }

    fn get_vertical_scene() -> Scene {
        let mut scene = Scene::new();

        let plastic = Plastic {
            colour: Spectrum::from(2.),
            specularity: 1.,
            roughness: 0.,
        };
        let plastic = scene.push_material(Material::Plastic(plastic));

        // One sphere
        let sphere = Sphere3D::new(0.5, Point3D::new(0., 0., -1.));
        scene.push_object(plastic, plastic, Primitive::Sphere(sphere));

        // Another sphere
        let sphere = Sphere3D::new(0.5, Point3D::new(0., 0., 1.));
        scene.push_object(plastic, plastic, Primitive::Sphere(sphere));

        scene
    }

    #[test]
    fn test_build_horizontal_bvh() {
        let original_scene = get_horizontal_scene();
        let mut scene = get_horizontal_scene();
        let (bvh, mapping) = BoundingVolumeTree::new(&mut scene);
        for (i, original_i) in mapping.into_iter().enumerate() {
            assert_eq!(&scene.triangles[i], &original_scene.triangles[original_i]);
        }

        let node = &bvh.nodes[0];
        assert_eq!(node.n_prims, 0);
        assert_eq!(node.axis, BBoxAxis::X);
    }

    #[test]
    fn test_build_vertical_bvh() {
        let original_scene = get_vertical_scene();
        let mut scene = get_vertical_scene();
        let (bvh, mapping) = BoundingVolumeTree::new(&mut scene);
        for (i, original_i) in mapping.into_iter().enumerate() {
            assert_eq!(&scene.triangles[i], &original_scene.triangles[original_i]);
        }
        // assert_eq!(bvh.nodes.len(), 3);

        let node = &bvh.nodes[0];
        assert_eq!(node.n_prims, 0);
        assert_eq!(node.axis, BBoxAxis::Z);
    }

    #[test]
    fn test_intersect_horizontal() {
        let original_scene = get_horizontal_scene();
        let mut scene = get_horizontal_scene();
        let mapping = scene.build_accelerator();
        // let (bvh, mapping) = BoundingVolumeTree::new(&mut scene);
        for (i, original_i) in mapping.into_iter().enumerate() {
            assert_eq!(&scene.triangles[i], &original_scene.triangles[original_i]);
        }

        let ray = Ray3D {
            origin: Point3D::new(-1., -10., 0.),
            direction: Vector3D::new(0., 1., 0.),
        };
        let mut aux = [0; 32];
        let res = scene.cast_ray(ray, &mut aux);
        assert!(res.is_some());
        let (_, interaction) = res.unwrap();

        assert!(
            (interaction.point - Point3D::new(-1., -0.5, 0.)).length() < 1e-5,
            "diff is {}",
            (interaction.point - Point3D::new(-1., -0.5, 0.)).length()
        );

        let ray = Ray3D {
            origin: Point3D::new(1., -10., 0.),
            direction: Vector3D::new(0., 1., 0.),
        };
        let mut aux = [0; 32];
        let ret = scene.cast_ray(ray, &mut aux);
        assert!(ret.is_some());
        let (_, interaction) = ret.unwrap();

        assert!((interaction.point - Point3D::new(1., -0.5, 0.)).length() < 1e-5);
    }

    #[test]
    fn test_intersect_vertical() {
        let mut scene = get_vertical_scene();
        scene.build_accelerator();
        // let (bvh, ..) = BoundingVolumeTree::new(&mut scene);

        let ray = Ray3D {
            origin: Point3D::new(0., -10., -1.),
            direction: Vector3D::new(0., 1., 0.),
        };
        let mut aux = [0; 32];
        let ret = scene.cast_ray(ray, &mut aux);
        assert!(ret.is_some());
        let (_, interaction) = ret.unwrap();
        assert!(
            (interaction.point - Point3D::new(0., -0.5, -1.)).length() < 1e-9,
            "Point was {}",
            interaction.point
        );

        let ray = Ray3D {
            origin: Point3D::new(0., -10., 1.),
            direction: Vector3D::new(0., 1., 0.),
        };
        let mut aux = [0; 32];
        let ret = scene.cast_ray(ray, &mut aux);
        assert!(ret.is_some());
        let (_, interaction) = ret.unwrap();

        assert!((interaction.point - Point3D::new(0., -0.5, 1.)).length() < 1e-9);
    }
}
