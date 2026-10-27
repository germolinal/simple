//! Ray-triangle intersections at the scales found in buildings
use geometry::{Point3D, Ray3D, Triangle3D, Vector3D};
use rendering::material::{Material, Plastic};
use rendering::primitive::Primitive;
use rendering::{Float, Scene, Spectrum};

/// A scene with the given triangles, all made of a gray plastic
fn scene(triangles: &[[(Float, Float, Float); 3]]) -> Scene {
    let mut scene = Scene::new();
    let m = scene.push_material(Material::Plastic(Plastic {
        colour: Spectrum::gray(0.5),
        specularity: 0.,
        roughness: 0.,
    }));
    for [a, b, c] in triangles {
        let p = |(x, y, z): (Float, Float, Float)| Point3D::new(x, y, z);
        let t = Triangle3D::new(p(*a), p(*b), p(*c)).unwrap();
        scene.push_object(m, m, Primitive::Triangle(t));
    }
    scene.build_accelerator();
    scene
}

/// Rays used to miss small triangles (and thin slivers) at oblique angles,
/// because they were considered parallel to them based on an absolute
/// threshold. (`Triangle3D::new` does not accept triangles much smaller than
/// these.)
#[test]
fn small_triangles_are_hit() {
    let theta = (60. as Float).to_radians();
    let direction = Vector3D::new(theta.sin(), 0., -theta.cos());
    let triangles = [
        [(0., 0., 0.), (4e-3, 0., 0.), (0., 4e-3, 0.)],
        [(0., 0., 0.), (1e-2, 0., 0.), (0., 1e-2, 0.)],
        [(0., 0., 0.), (1., 0., 0.), (0.5, 1.5e-5, 0.)], // a sliver
        [(0., 0., 0.), (1., 0., 0.), (0., 1., 0.)],
        [(0., 0., 0.), (100., 0., 0.), (0., 100., 0.)],
    ];
    for triangle in triangles {
        let scene = scene(&[triangle]);
        let [a, b, c] = triangle;
        let target = Point3D::new((a.0 + b.0 + c.0) / 3., (a.1 + b.1 + c.1) / 3., 0.);
        let ray = Ray3D {
            origin: target - direction,
            direction,
        };
        let mut aux = [0; 32];
        let hit = scene.cast_ray(ray, &mut aux);
        assert!(hit.is_some(), "missed the triangle {triangle:?}");
    }
}

/// Shadow rays used to only check the closest triangle in each BVH leaf, so
/// a triangle very close to the origin (rejected as a self-intersection) hid
/// other ones in the same leaf that do block the light
#[test]
fn shadow_rays_check_all_triangles_in_leaf() {
    // A horizontal triangle 0.5 mm below the origin, and a tilted one
    // crossing the ray 0.5 m below. Both have the same bounding box (the
    // blocker's is centred at `z0` in z), so the BVH puts them in the same
    // leaf.
    let z0 = -0.0005;
    let (z_top, x_max) = (2. * z0 + 1., 2. * z0 + 3.);
    let near = [(-1., -1., z0), (-1., 1., z0), (x_max, 0., z0)];
    let blocker = [(-1., -1., -1.), (-1., 1., -1.), (x_max, 0., z_top)];
    let scene = scene(&[near, blocker]);
    let ray = Ray3D {
        origin: Point3D::new(0., 0., 0.),
        direction: Vector3D::new(0., 0., -1.),
    };
    let mut aux = [0; 32];
    // A light 2 m below cannot be seen through the tilted triangle
    assert!(!scene.unobstructed_distance(ray, 4., &mut aux));
    // ...but one 0.3 m below can
    assert!(scene.unobstructed_distance(ray, 0.09, &mut aux));
}
