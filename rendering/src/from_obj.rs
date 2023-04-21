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

use crate::Float;

use crate::scene::Scene;

use geometry::{Point3D, Vector3D};

use std::fs;

use obj::raw::object::Polygon;
use obj::{load_obj, FromRawVertex, Obj, ObjResult};

#[allow(dead_code)]
struct ObjTriangle {
    vertices: [Point3D; 3],
    normals: Option<[Vector3D; 3]>,
    textures: Option<[(Float, Float, Float); 3]>,
}

impl<I> FromRawVertex<I> for ObjTriangle {
    fn process(
        vertices: Vec<(f32, f32, f32, f32)>,
        normals: Vec<(f32, f32, f32)>,
        tex_coords: Vec<(f32, f32, f32)>,
        polygons: Vec<Polygon>,
    ) -> ObjResult<(Vec<Self>, Vec<I>)> {
        let mut ret: Vec<Self> = Vec::with_capacity(polygons.len());

        for poly in polygons {
            let p: Vec<usize>;
            let mut t: Option<Vec<usize>> = None;
            let mut n: Option<Vec<usize>> = None;
            match poly {
                Polygon::P(pos) => {
                    p = pos;
                }
                Polygon::PT(pos_texts) => {
                    p = pos_texts.iter().map(|v| v.0).collect();
                    t = Some(pos_texts.iter().map(|v| v.1).collect());
                }
                Polygon::PN(pos_normal) => {
                    p = pos_normal.iter().map(|v| v.0).collect();
                    n = Some(pos_normal.iter().map(|v| v.1).collect());
                }
                Polygon::PTN(pos_text_normal) => {
                    p = pos_text_normal.iter().map(|v| v.0).collect();
                    t = Some(pos_text_normal.iter().map(|v| v.1).collect());
                    n = Some(pos_text_normal.iter().map(|v| v.2).collect());
                }
            }
            // assert!(p.len() == 4 || p.len() == 3, "Only faces of 3 and 4 vertices are allowed when reading OBJ files for now... sorry");
            if p.len() > 4 {
                continue;
            }
            let p: Vec<Point3D> = p
                .iter()
                .map(|index| {
                    let vert = vertices
                        .get(*index)
                        .expect("Malformed OBJ file... face references an inexistend vertex");
                    Point3D::new(vert.0 as Float, vert.1 as Float, vert.2 as Float)
                })
                .collect();

            // let t: Option<Vec<(Float, Float, Float)>> = match t{
            //     None => None,
            //     Some(te)=>{Some(te.iter().map(|x| {
            //         let cord = tex_coords.get(*x).expect("Malformed OBJ file... face references an inexistend texture");
            //         (cord.0 as Float, cord.1 as Float, cord.2 as Float)
            //     }).collect())}
            // };
            let t: Option<Vec<(Float, Float, Float)>> = t.map(|te| {
                te.iter()
                    .map(|x| {
                        let cord = tex_coords
                            .get(*x)
                            .expect("Malformed OBJ file... face references an inexistend texture");
                        (cord.0 as Float, cord.1 as Float, cord.2 as Float)
                    })
                    .collect()
            });

            // let n: Option<Vec<Vector3D>> = match n{
            //     None => None,
            //     Some(norm)=>{Some(norm.iter().map(|x| {
            //         let vert = normals.get(*x).expect("Malformed OBJ file... face references an inexistend normal");
            //         Vector3D::new(vert.0 as Float, vert.1 as Float, vert.2 as Float)
            //     }).collect())}
            // };
            let n: Option<Vec<Vector3D>> = n.map(|norm| {
                norm.iter()
                    .map(|x| {
                        let vert = normals
                            .get(*x)
                            .expect("Malformed OBJ file... face references an inexistend normal");
                        Vector3D::new(vert.0 as Float, vert.1 as Float, vert.2 as Float)
                    })
                    .collect()
            });

            // Add the bottom half
            let (x, y, z) = (0, 1, 2);
            let vertices = [p[x], p[y], p[z]];
            // let normals = match &n {
            //     Some(no)=>Some([no[x], no[y], no[z]]),
            //     None => None
            // };
            let normals = n.as_ref().map(|no| [no[x], no[y], no[z]]);

            // let textures = match &t {
            //     Some(no)=>Some([no[x], no[y], no[z]]),
            //     None => None
            // };
            let textures = t.as_ref().map(|no| [no[x], no[y], no[z]]);
            ret.push(ObjTriangle {
                vertices,
                normals,
                textures,
            });
            if p.len() == 4 {
                // add second half
                let (x, y, z) = (0, 2, 3);
                let vertices = [p[x], p[y], p[z]];
                // let normals = match &n {
                //     Some(no)=>Some([no[x], no[y], no[z]]),
                //     None => None
                // };
                let normals = n.as_ref().map(|no| [no[x], no[y], no[z]]);
                // let textures = match &t {
                //     Some(no)=>Some([no[x], no[y], no[z]]),
                //     None => None
                // };
                let textures = t.as_ref().map(|no| [no[x], no[y], no[z]]);
                ret.push(ObjTriangle {
                    vertices,
                    normals,
                    textures,
                });
            }
        }

        Ok((ret, Vec::new()))
    }
}

impl Scene {
    /// Reads a Radiance file and builds a scene.
    pub fn add_from_obj(
        &mut self,
        filename: String,
        front_material_index: usize,
        back_material_index: usize,
    ) {
        let src = fs::read(filename).unwrap();

        self.add_from_obj_source(&src, front_material_index, back_material_index);
    }

    /// Creates a scene from a slice of bytes read from a
    /// Radiance file
    pub fn add_from_obj_source(
        &mut self,
        source: &[u8],
        front_material_index: usize,
        back_material_index: usize,
    ) {
        let obj: Obj<ObjTriangle> = load_obj(source).unwrap();

        let triangles = obj.vertices;

        for triangle in triangles {
            let a = triangle.vertices[0];
            let b = triangle.vertices[1];
            let c = triangle.vertices[2];
            if let Ok(tri) = geometry::Triangle3D::new(a, b, c) {
                self.push_object(
                    front_material_index,
                    back_material_index,
                    crate::primitive::Primitive::Triangle(tri),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::colour::Spectrum;
    use crate::material::*;

    #[test]
    fn test_load_model() {
        let src = b"
        mtllib sponza.mtl

#
# object arcs_floor
#

v  10.927240 2.957242 2.370587
v  10.927240 3.238548 2.282929
v  10.927240 3.149897 2.049175
v  12.927240 3.149897 2.049175

g arcs_03
usemtl sp_00_luk_mali
f 1 2 3 4 
f 1 2 3


        ";
        let mut scene = Scene::default();
        let gray = scene.push_material(Material::Plastic(Plastic {
            colour: Spectrum::gray(0.3),
            specularity: 0.,
            roughness: 0.,
        }));

        // let mut scene = Scene::from_obj("./tests/scenes/sponza.obj".to_string());
        scene.add_from_obj_source(src, gray, gray);

        assert_eq!(3, scene.triangles.len());
    }
}
