use crate::{Float, Scene};
use geometry::{
    intersection::{IntersectionInfo, SurfaceSide},
    BBox3D, Point3D, Ray3D, Sphere3D, Triangle3D, Vector3D,
};

mod fallback;

pub(crate) const LEAF_SIZE: usize = 24;

/// The smallest definition of a Triangle I could think of
pub type Triangle = [Float; 9];

#[allow(clippy::too_many_arguments)]
pub fn triangle_area(
    ax: Float,
    ay: Float,
    az: Float,
    bx: Float,
    by: Float,
    bz: Float,
    cx: Float,
    cy: Float,
    cz: Float,
) -> Float {
    let ab = ((bx - ax).powi(2) + (by - ay).powi(2) + (bz - az).powi(2)).sqrt();
    let bc = ((cx - bx).powi(2) + (cy - by).powi(2) + (cz - bz).powi(2)).sqrt();
    let ca = ((ax - cx).powi(2) + (ay - cy).powi(2) + (az - cz).powi(2)).sqrt();

    ((ca + bc + ab)
        * ((ca + bc + ab) / 2. - ab)
        * ((ca + bc + ab) / 2. - bc)
        * ((ca + bc + ab) / 2. - ca)
        / 2.)
        .sqrt()
}

#[allow(clippy::too_many_arguments)]
pub fn triangle_solid_angle_pdf(
    ax: Float,
    ay: Float,
    az: Float,
    bx: Float,
    by: Float,
    bz: Float,
    cx: Float,
    cy: Float,
    cz: Float,
    point: Point3D,
    normal: Vector3D,
    ray: &Ray3D,
) -> Float {
    let d2 = (point - ray.origin).length_squared();
    let cos_theta = ray.origin * normal;
    // debug_assert!(cos_theta > 0.);
    if cos_theta < 1e-7 {
        return 0.0;
    }
    let area = triangle_area(ax, ay, az, bx, by, bz, cx, cy, cz);
    // return
    d2 / cos_theta.abs() / area
}

/// Gets the BBox of a Triangle
pub fn world_bounds(t: &Triangle) -> BBox3D {
    let a = Point3D::new(t[0], t[1], t[2]);
    let bbox = BBox3D::from_point(a);

    let b = Point3D::new(t[3], t[4], t[5]);
    let bbox = BBox3D::from_union_point(&bbox, b);

    let c = Point3D::new(t[6], t[7], t[8]);
    BBox3D::from_union_point(&bbox, c)
}

/// Intersects a `Ray3D` and a [`Triangle`], returning the [`IntersectionInfo`]
/// (or `None` if they don't intersect)
pub fn intersect_triangle_slice(
    scene: &Scene,
    ray: &geometry::Ray3D,
    ini: usize,
    fin: usize,
) -> Option<(usize, IntersectionInfo)> {
    fallback::intersect_triangle_slice(scene, ray, ini, fin)
}

/// Intersects a `Ray3D` and a [`Triangle`], returning the `Point3D` of
/// intersection
pub fn simple_triangle_intersect(
    scene: &Scene,
    ray: &geometry::Ray3D,
    ini: usize,
    fin: usize,
) -> Option<(usize, geometry::Point3D)> {
    fallback::simple_intersect_triangle_slice(scene, ray, ini, fin)
}

pub struct Intersection {
    pub e1: Vector3D,
    pub e2: Vector3D,
    pub normal: Vector3D,
    pub point: Point3D,
    // pub tri_index: usize,
    pub side: SurfaceSide,
    pub u: Float,
    pub v: Float,
}

#[allow(clippy::too_many_arguments)]
pub fn new_info(
    ax: Float,
    ay: Float,
    az: Float,
    bx: Float,
    by: Float,
    bz: Float,
    cx: Float,
    cy: Float,
    cz: Float,
    point: Point3D,
    _u: Float,
    _v: Float,
    ray_dir: Vector3D,
) -> IntersectionInfo {
    let dpdu = Vector3D::new(bx - ax, by - ay, bz - az);
    let dpdv = Vector3D::new(cx - ax, cy - ay, cz - az);
    // eprintln!("dpdu = {} | dpdv = {}", dpdu, dpdv);
    let normal = dpdu.cross(dpdv).get_normalized();
    // eprintln!("normal = {}", normal);
    let (normal, side) = SurfaceSide::get_side(normal, ray_dir);
    let e1 = dpdu.get_normalized();
    let e2 = normal.cross(e1).get_normalized();
    debug_assert!((1.0 - normal.length()).abs() < 1e-5);
    debug_assert!((1.0 - e1.length()).abs() < 1e-5);
    debug_assert!((1.0 - e2.length()).abs() < 1e-5);
    IntersectionInfo {
        normal,
        dpdu: e1,
        dpdv: e2,
        p: point,
        side,
        #[cfg(feature = "textures")]
        u: _u,
        #[cfg(feature = "textures")]
        v: _v,
        #[cfg(feature = "textures")]
        dndu: Vector3D::new(0., 0., 0.),
        #[cfg(feature = "textures")]
        dndv: Vector3D::new(0., 0., 0.),
    }
}

/// Transforms a `Triangle3D` and transforms it into a `Vec<Triangle>` and their
/// respective normals
pub fn mesh_triangle(tr: &Triangle3D) -> (Vec<Triangle>, Vec<(Vector3D, Vector3D, Vector3D)>) {
    // Become a single triangle... dah!
    let s1 = tr.b() - tr.a();
    let s2 = tr.c() - tr.a();

    // All vertices have the same normal
    let normal = s1.cross(s2).get_normalized();
    let normals = vec![(normal, normal, normal)];

    // Push triangle
    let triangles = vec![[
        tr.a().x,
        tr.a().y,
        tr.a().z,
        tr.b().x,
        tr.b().y,
        tr.b().z,
        tr.c().x,
        tr.c().y,
        tr.c().z,
    ]];
    (triangles, normals)
}

pub fn mesh_sphere(s: &Sphere3D) -> (Vec<Triangle>, Vec<(Vector3D, Vector3D, Vector3D)>) {
    const N_REFINEMENTS: u32 = 5;

    let r = s.radius;
    let c = s.centre();
    // check if partial
    let bounds = s.bounds();
    let zmin = bounds.min.z;
    let zmax = bounds.max.z;
    if zmin > -r || zmax < r {
        eprintln!(
            "Warning: Partial Sphere Meshing is not supported yet... adding it as a full sphere."
        )
    }

    // Initialize: set basic coordinates
    let midtop = r * (60. as Float).to_radians().cos();
    let midr = r * (60. as Float).to_radians().sin();
    let midbottom = -midtop;
    // Points
    let top = Point3D::new(0., 0., r) + c;
    let bottom = Point3D::new(0., 0., -r) + c;
    let midtop: Vec<Point3D> = [36., 3. * 36., 5. * 36., 7. * 36., 9. * 36.]
        .iter()
        .map(|angle: &Float| {
            Point3D::new(
                midr * angle.to_radians().sin(),
                midr * angle.to_radians().cos(),
                midtop,
            ) + c
        })
        .collect();
    let midbottom: Vec<Point3D> = [0., 72., 2. * 72., 3. * 72., 4. * 72.]
        .iter()
        .map(|angle: &Float| {
            Point3D::new(
                midr * angle.to_radians().sin(),
                midr * angle.to_radians().cos(),
                midbottom,
            ) + c
        })
        .collect();

    let mut triangles: Vec<(Point3D, Point3D, Point3D)> =
        Vec::with_capacity((4_usize).pow(N_REFINEMENTS) * 20);

    // In reverse (to respect the triangle's normal direction)
    triangles.push((midtop[0], midtop[4], top));
    triangles.push((midtop[4], midtop[3], top));
    triangles.push((midtop[3], midtop[2], top));
    triangles.push((midtop[2], midtop[1], top));
    triangles.push((midtop[1], midtop[0], top));

    triangles.push((midbottom[0], midbottom[1], bottom));
    triangles.push((midbottom[1], midbottom[2], bottom));
    triangles.push((midbottom[2], midbottom[3], bottom));
    triangles.push((midbottom[3], midbottom[4], bottom));
    triangles.push((midbottom[4], midbottom[0], bottom));

    triangles.push((midtop[4], midtop[0], midbottom[0]));
    triangles.push((midtop[0], midtop[1], midbottom[1]));
    triangles.push((midtop[1], midtop[2], midbottom[2]));
    triangles.push((midtop[2], midtop[3], midbottom[3]));
    triangles.push((midtop[3], midtop[4], midbottom[4]));

    triangles.push((midbottom[1], midbottom[0], midtop[0]));
    triangles.push((midbottom[2], midbottom[1], midtop[1]));
    triangles.push((midbottom[3], midbottom[2], midtop[2]));
    triangles.push((midbottom[4], midbottom[3], midtop[3]));
    triangles.push((midbottom[0], midbottom[4], midtop[4]));

    // Refine
    let centre = s.centre();
    let mut refine = || {
        let n = triangles.len();
        for i in 0..n {
            let (a, b, c) = triangles[i];
            // interpolate
            let ab = (a + b) / 2.;
            let ac = (a + c) / 2.;
            let bc = (b + c) / 2.;
            // project into the sphere
            let ab = centre + (ab - centre).get_normalized() * r;
            let ac = centre + (ac - centre).get_normalized() * r;
            let bc = centre + (bc - centre).get_normalized() * r;

            // Replace existing one
            triangles[i] = (a, ab, ac);

            // push others at the back
            triangles.push((ab, b, bc));
            triangles.push((bc, c, ac));
            triangles.push((ab, bc, ac));
        }
    };

    for _ in 0..N_REFINEMENTS {
        refine()
    }

    // Transform
    let normals: Vec<(Vector3D, Vector3D, Vector3D)> = triangles
        .iter()
        .map(|vertex| {
            let n0 = (vertex.0 - centre).get_normalized();
            let n1 = (vertex.1 - centre).get_normalized();
            let n2 = (vertex.2 - centre).get_normalized();
            (n0, n1, n2)
        })
        .collect();
    let triangles: Vec<Triangle> = triangles
        .iter()
        .map(|vertex| {
            [
                vertex.0.x, vertex.0.y, vertex.0.z, vertex.1.x, vertex.1.y, vertex.1.z, vertex.2.x,
                vertex.2.y, vertex.2.z,
            ]
        })
        .collect();
    (triangles, normals)
}

#[cfg(test)]
mod testing {
    use crate::{material::Metal, Spectrum};

    use super::*;
    use validate::assert_close;

    #[test]
    fn test_triangle_area() {
        // in XY
        assert_close!(0.5, triangle_area(0., 0., 0., 1., 0., 0., 0., 1., 0.));

        assert_close!(2., triangle_area(0., 0., 0., 2., 0., 0., 0., 2., 0.));

        // in XZ
        assert_close!(0.5, triangle_area(0., 0., 0., 1., 0., 0., 0., 0., 1.));

        assert_close!(2., triangle_area(0., 0., 0., 2., 0., 0., 0., 0., 2.));

        // in YZ
        assert_close!(0.5, triangle_area(0., 0., 0., 0., 1., 0., 0., 0., 1.));

        assert_close!(2., triangle_area(0., 0., 0., 0., 2., 0., 0., 0., 2.));
    }

    #[test]
    fn test_mesh_triangle() -> Result<(), String> {
        let a: (Float, Float, Float) = (0., 1., 2.);
        let b: (Float, Float, Float) = (3., 4., 5.);
        let c: (Float, Float, Float) = (6., -7., 8.);
        let tri: Triangle3D = Triangle3D::new(
            Point3D::new(a.0, a.1, a.2),
            Point3D::new(b.0, b.1, b.2),
            Point3D::new(c.0, c.1, c.2),
        )?;
        let input: Triangle = [a.0, a.1, a.2, b.0, b.1, b.2, c.0, c.1, c.2];
        let (output, normals) = mesh_triangle(&tri);
        assert_eq!(1, output.len());
        assert_eq!(1, normals.len());
        assert_eq!(input, output[0]);
        assert_eq!(normals[0].0, tri.normal());
        assert_eq!(normals[0].1, tri.normal());
        assert_eq!(normals[0].2, tri.normal());

        Ok(())
    }

    #[test]
    fn test_mesh_sphere() {
        let centre = Point3D::new(1., 6., -2.);
        let radius = 5.21;

        let sphere = Sphere3D::new(radius, centre);

        let (triangles, normals) = mesh_sphere(&sphere);
        assert_eq!(triangles.len(), normals.len());

        for (trindex, tri) in triangles.iter().enumerate() {
            let a = Point3D::new(tri[0], tri[1], tri[2]);
            let b = Point3D::new(tri[3], tri[4], tri[5]);
            let c = Point3D::new(tri[6], tri[7], tri[8]);

            let ra = a - centre;
            let rb = b - centre;
            let rc = c - centre;

            assert!(
                (ra.length() - radius).abs() < 1e-5,
                "Expecting ra to be {}... found {}",
                radius,
                ra.length()
            );
            assert!(
                (rb.length() - radius).abs() < 1e-5,
                "Expecting rb to be {}... found {}",
                radius,
                rb.length()
            );
            assert!(
                (rc.length() - radius).abs() < 1e-5,
                "Expecting rc to be {}... found {}",
                radius,
                rc.length()
            );

            assert_eq!(ra.get_normalized(), normals[trindex].0);
            assert_eq!(rb.get_normalized(), normals[trindex].1);
            assert_eq!(rc.get_normalized(), normals[trindex].2);
        }
    }

    const UP: Vector3D = Vector3D {
        x: 0.,
        y: 0.,
        z: 1.,
    };
    const DOWN: Vector3D = Vector3D {
        x: 0.,
        y: 0.,
        z: -1.,
    };

    #[test]
    fn test_triangle_intersect() -> Result<(), String> {
        let a = Point3D::new(0., 0., 0.);
        let b = Point3D::new(1., 0., 0.);
        let c = Point3D::new(0., 1., 0.);

        let mut scene = Scene::new();
        scene.push_material(crate::material::Material::Metal(Metal {
            colour: Spectrum([0.2, 0.2, 0.2]),
            specularity: 0.0,
            roughness: 0.0,
        }));
        scene.push_object(
            0,
            0,
            crate::primitive::Primitive::Triangle(Triangle3D::new(a, b, c)?),
        );

        scene.build_accelerator();

        let test_hit = |pt: Point3D,
                        dir: Vector3D,
                        expect_pt: Option<Point3D>,
                        exp_side: SurfaceSide|
         -> Result<(), String> {
            let ray = Ray3D {
                origin: pt,
                direction: dir,
            };

            if let Some((.., info)) = intersect_triangle_slice(&scene, &ray, 0, 1) {
                let phit = info.p;

                if let Some(exp_p) = expect_pt {
                    if !phit.compare(exp_p) {
                        return Err(format!(
                            "Hit in incorrect point...: pt = {}, dir = {}, phit = {}",
                            pt, dir, phit
                        ));
                    }
                } else {
                    return Err(format!("Was NOT expecting hit: pt = {}, dir = {}", pt, dir));
                }
                if exp_side != info.side {
                    return Err(format!(
                        "Expecing a hit at the {:?} (dir = {}, pt = {})",
                        exp_side, dir, pt
                    ));
                }
            } else {
                if expect_pt.is_some() {
                    return Err(format!("WAS expecting hit: pt = {}, dir = {}", pt, dir));
                }
            }

            Ok(())
        }; // end of closure

        /* FROM THE BOTTOM, GOING UP */
        // Vertex A
        test_hit(a + DOWN, UP, Some(a), SurfaceSide::Back)?;

        // Vertex B.
        test_hit(b + DOWN, UP, Some(b), SurfaceSide::Back)?;

        // Vertex C.
        test_hit(c + DOWN, UP, Some(c), SurfaceSide::Back)?;

        // Segment AB.
        let p = Point3D::new(0.5, 0., 0.);
        test_hit(p + DOWN, UP, Some(p), SurfaceSide::Back)?;

        // Segment AC.
        let p = Point3D::new(0., 0.5, 0.);
        test_hit(p + DOWN, UP, Some(p), SurfaceSide::Back)?;

        // Segment BC.
        let p = Point3D::new(0.5, 0.5, 0.);
        test_hit(p + DOWN, UP, Some(p), SurfaceSide::Back)?;

        // Point outside
        let p = Point3D::new(0., -1., 0.);
        test_hit(p + DOWN, UP, None, SurfaceSide::Back)?;

        // Point inside
        let p = Point3D::new(0.1, 0.1, 0.);
        test_hit(p + DOWN, UP, Some(p), SurfaceSide::Back)?;

        /* FROM THE TOP, GOING DOWN */
        // Vertex A
        test_hit(a + UP, DOWN, Some(a), SurfaceSide::Front)?;

        // Vertex B.
        test_hit(b + UP, DOWN, Some(b), SurfaceSide::Front)?;

        // Vertex C.
        test_hit(c + UP, DOWN, Some(c), SurfaceSide::Front)?;

        // Segment AB.
        let p = Point3D::new(0.5, 0., 0.);
        test_hit(p + UP, DOWN, Some(p), SurfaceSide::Front)?;

        // Segment AC.
        let p = Point3D::new(0., 0.5, 0.);
        test_hit(p + UP, DOWN, Some(p), SurfaceSide::Front)?;

        // Segment BC.
        let p = Point3D::new(0.5, 0.5, 0.);
        test_hit(p + UP, DOWN, Some(p), SurfaceSide::Front)?;

        // Point outside
        let p = Point3D::new(0., -1., 0.);
        test_hit(p + UP, DOWN, None, SurfaceSide::Front)?;

        // Point inside
        let p = Point3D::new(0.1, 0.1, 0.);
        test_hit(p + UP, DOWN, Some(p), SurfaceSide::Front)?;

        Ok(())
    }

    // #[test]
    // fn test_triangle_intersect_pack() -> Result<(),String> {
    //     let a = Point3D::new(0., 0., 0.);
    //     let b = Point3D::new(1., 0., 0.);
    //     let c = Point3D::new(0., 1., 0.);

    //     let triangle: [Triangle; 4] = [
    //         [a.x, a.y, 0.0, b.x, b.y, 0.0, c.x, c.y, 0.0],
    //         [a.x, a.y, 0.1, b.x, b.y, 0.1, c.x, c.y, 0.1],
    //         [a.x, a.y, 0.2, b.x, b.y, 0.2, c.x, c.y, 0.2],
    //         [a.x, a.y, 0.3, b.x, b.y, 0.3, c.x, c.y, 0.3],
    //     ];

    //     let test_hit = |pt: Point3D,
    //                     dir: Vector3D,
    //                     exp_index: usize,
    //                     expect_pt: Option<Point3D>,
    //                     exp_side: SurfaceSide|
    //      -> Result<(), String> {
    //         let ray = Ray3D {
    //             origin: pt,
    //             direction: dir,
    //         };

    //         if let Some((index, info)) = triangle_intersect_pack(&triangle, &ray) {
    //             let phit = info.p;

    //             if let Some(exp_p) = expect_pt {
    //                 if !phit.compare(exp_p) {
    //                     return Err(format!(
    //                         "Hit in incorrect point...: pt = {}, dir = {}, phit = {}",
    //                         pt, dir, phit
    //                     ));
    //                 }
    //             } else {
    //                 return Err(format!("Was NOT expecting hit: pt = {}, dir = {}", pt, dir));
    //             }
    //             if exp_side != info.side {
    //                 return Err(format!(
    //                     "Expecing a hit at the {:?} (dir = {}, pt = {})",
    //                     exp_side, dir, pt
    //                 ));
    //             }
    //             if exp_index != index {
    //                 return Err(format!(
    //                     "Expecing a hit at triangle {} (dir = {}, pt = {})",
    //                     index, dir, pt
    //                 ));
    //             }
    //         } else {
    //             if expect_pt.is_some() {
    //                 return Err(format!("WAS expecting hit: pt = {}, dir = {}", pt, dir));
    //             }
    //         }

    //         Ok(())
    //     }; // end of closure

    //     /* FROM THE BOTTOM, GOING UP */
    //     // Vertex A
    //     test_hit(a + DOWN, UP, 0, Some(a), SurfaceSide::Back)?;

    //     // Vertex B.
    //     test_hit(b + DOWN, UP, 0, Some(b), SurfaceSide::Back)?;

    //     // Vertex C.
    //     test_hit(c + DOWN, UP, 0, Some(c), SurfaceSide::Back)?;

    //     // Segment AB.
    //     let p = Point3D::new(0.5, 0., 0.);
    //     test_hit(p + DOWN, UP, 0, Some(p), SurfaceSide::Back)?;

    //     // Segment AC.
    //     let p = Point3D::new(0., 0.5, 0.);
    //     test_hit(p + DOWN, UP, 0, Some(p), SurfaceSide::Back)?;

    //     // Segment BC.
    //     let p = Point3D::new(0.5, 0.5, 0.);
    //     test_hit(p + DOWN, UP, 0, Some(p), SurfaceSide::Back)?;

    //     // Point outside
    //     let p = Point3D::new(0., -1., 0.);
    //     test_hit(p + DOWN, UP, 0, None, SurfaceSide::Back)?;

    //     // Point inside
    //     let p = Point3D::new(0.1, 0.1, 0.);
    //     test_hit(p + DOWN, UP, 0, Some(p), SurfaceSide::Back)?;

    //     /* FROM THE TOP, GOING DOWN */
    //     // Vertex A
    //     test_hit(a + UP, DOWN, 3, Some(a + UP * 0.3), SurfaceSide::Front)?;

    //     // Vertex B.
    //     test_hit(b + UP, DOWN, 3, Some(b + UP * 0.3), SurfaceSide::Front)?;

    //     // Vertex C.
    //     test_hit(c + UP, DOWN, 3, Some(c + UP * 0.3), SurfaceSide::Front)?;

    //     // Segment AB.
    //     let p = Point3D::new(0.5, 0., 0.);
    //     test_hit(p + UP, DOWN, 3, Some(p + UP * 0.3), SurfaceSide::Front)?;

    //     // Segment AC.
    //     let p = Point3D::new(0., 0.5, 0.);
    //     test_hit(p + UP, DOWN, 3, Some(p + UP * 0.3), SurfaceSide::Front)?;

    //     // Segment BC.
    //     let p = Point3D::new(0.5, 0.5, 0.);
    //     test_hit(p + UP, DOWN, 3, Some(p + UP * 0.3), SurfaceSide::Front)?;

    //     // Point outside
    //     let p = Point3D::new(0., -1., 0.);
    //     test_hit(p + UP, DOWN, 3, None, SurfaceSide::Front)?;

    //     // Point inside
    //     let p = Point3D::new(0.1, 0.1, 0.);
    //     test_hit(p + UP, DOWN, 3, Some(p + UP * 0.3), SurfaceSide::Front)?;

    //     /* FROM THE TOP, GOING DOWN... BUT STARTING BETWEEN TRIANGLES */
    //     // Vertex A
    //     test_hit(
    //         a + UP * 0.15,
    //         DOWN,
    //         1,
    //         Some(a + UP * 0.1),
    //         SurfaceSide::Front,
    //     )?;

    //     // Vertex B.
    //     test_hit(
    //         b + UP * 0.15,
    //         DOWN,
    //         1,
    //         Some(b + UP * 0.1),
    //         SurfaceSide::Front,
    //     )?;

    //     // Vertex C.
    //     test_hit(
    //         c + UP * 0.15,
    //         DOWN,
    //         1,
    //         Some(c + UP * 0.1),
    //         SurfaceSide::Front,
    //     )?;

    //     // Segment AB.
    //     let p = Point3D::new(0.5, 0., 0.);
    //     test_hit(
    //         p + UP * 0.15,
    //         DOWN,
    //         1,
    //         Some(p + UP * 0.1),
    //         SurfaceSide::Front,
    //     )?;

    //     // Segment AC.
    //     let p = Point3D::new(0., 0.5, 0.);
    //     test_hit(
    //         p + UP * 0.15,
    //         DOWN,
    //         1,
    //         Some(p + UP * 0.1),
    //         SurfaceSide::Front,
    //     )?;

    //     // Segment BC.
    //     let p = Point3D::new(0.5, 0.5, 0.);
    //     test_hit(
    //         p + UP * 0.15,
    //         DOWN,
    //         1,
    //         Some(p + UP * 0.1),
    //         SurfaceSide::Front,
    //     )?;

    //     // Point outside
    //     let p = Point3D::new(0., -1., 0.);
    //     test_hit(p + UP * 0.15, DOWN, 1, None, SurfaceSide::Front)?;

    //     // Point inside
    //     let p = Point3D::new(0.1, 0.1, 0.);
    //     test_hit(
    //         p + UP * 0.15,
    //         DOWN,
    //         1,
    //         Some(p + UP * 0.1),
    //         SurfaceSide::Front,
    //     )?;
    // }
}
