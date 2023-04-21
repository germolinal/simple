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

use clap::Parser;
use rendering::{RayTracer, Scene, Wavelengths};

use geometry::{Point3D, Vector3D};
use rendering::camera::{Film, Pinhole, View};
use rendering::Float;

#[derive(Debug, Clone)]
struct Triplet(Float, Float, Float);

impl std::fmt::Display for Triplet {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} {} {}", self.0, self.1, self.2)
    }
}

impl std::str::FromStr for Triplet {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let st: Vec<&str> = s.trim().split(' ').collect();
        if st.len() != 3 {
            return Err(format!(
                "Expecting three values—e.g., '1. 2. 3'—... found '{}'",
                s
            ));
        }
        let a = match st[0].parse::<Float>() {
            Ok(v) => v,
            Err(_) => {
                return Err(format!(
                    "Expecting value 1 in triplet to be a number... found '{}'",
                    st[0]
                ))
            }
        };
        let b = match st[1].parse::<Float>() {
            Ok(v) => v,
            Err(_) => {
                return Err(format!(
                    "Expecting value 2 in triplet to be a number... found '{}'",
                    st[1]
                ))
            }
        };
        let c = match st[2].parse::<Float>() {
            Ok(v) => v,
            Err(_) => {
                return Err(format!(
                    "Expecting value 3 in triplet to be a number... found '{}'",
                    st[2]
                ))
            }
        };
        Ok(Self(a, b, c))
    }
}

/// A program for Rendering an image from a .rad (i.e., Radiance) or .spl (i.e., Simple)
/// formats
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Inputs {
    #[clap(short, long)]
    /// The file to load the model from
    pub input: String,

    #[clap(short, long)]
    /// The output of the final image (in rgbe format)
    pub output: String,

    /* Ray-tracer data */
    /// The number of bounces before a ray is terminated (-ab in Radiance lingo)
    #[clap(short = 'b', long = "max_depth", default_value_t = 2)]
    pub max_depth: usize,

    /// The number of shadow rays per light source sent from the first
    /// interaction. From the second interaction and on, only 1 shadow
    /// ray is sent     
    #[clap(short = 's', long = "shadow_samples", default_value_t = 10)]
    pub n_shadow_samples: usize,

    /// The number of secondary rays sent from the first interaction.
    /// From the second interaction and on, this number is reduced
    #[clap(short = 'a', long = "ambient_samples", default_value_t = 70)]
    pub n_ambient_samples: usize,

    /// A lower value makes the Russian roulette less deadly
    #[clap(short = 'w', long = "limit_weight", default_value_t = 1e-3)]
    pub limit_weight: Float,

    /// The probability of counting purely specular bounces as an actual bounce.
    /// (Specular bounces seldom count because that allows achieving caustics better...
    /// and they are cheap)
    #[clap(short = 'c', long = "count_specular", default_value_t = 0.3)]
    pub count_specular_bounce: Float,

    /* Film */
    /// The Horizontal resolution of the final image
    #[clap(short = 'x', long, default_value_t = 512)]
    pub x: usize,

    /// The Vertical resolution of the final image
    #[clap(short = 'y', long, default_value_t = 512)]
    pub y: usize,

    /* VIEW */
    /// The view point (e.g., '-p 0. 1. 2')
    #[clap(short = 'p', long)]
    pub view_point: Triplet,

    /// The view direction (Does not need to be normalized. e.g., '-d -3. 1. 2')
    #[clap(short = 'd', long)]
    pub view_direction: Triplet,

    /// The view up (e.g., '-u 0. 1. 0')
    #[clap(short='u', long, default_value_t=Triplet(0., 1., 0.))]
    pub view_up: Triplet,

    /// The horizontal field of view, in degrees
    #[clap(short = 'h', long = "view_horizontal", default_value_t = 60.)]
    pub field_of_view: Float,
}

fn main() {
    let inputs = Inputs::parse();

    let input_file = inputs.input;
    let mut scene = if input_file.ends_with(".rad") {
        Scene::from_radiance(input_file)
    } else if input_file.ends_with(".spl") {
        let (model, _header) = model::Model::from_file(input_file).unwrap();
        Scene::from_simple_model(&model, Wavelengths::Visible).unwrap()
    }
    // else if input.ends_with(".obj"){
    //     Scene::from_simple_model(input)
    // }
    else {
        panic!("Unkwown format in file {}", input_file);
    };

    scene.build_accelerator();

    // Create camera
    let film = Film {
        // resolution: (512, 367),
        // resolution: (1024, 768),
        resolution: (inputs.x, inputs.y),
    };

    // Create view
    let dir = inputs.view_direction;
    let p = inputs.view_point;
    let up = inputs.view_up;
    let view = View {
        view_direction: Vector3D::new(dir.0, dir.1, dir.2).get_normalized(),
        view_point: Point3D::new(p.0, p.1, p.2),
        view_up: Vector3D::new(up.0, up.1, up.2),
        field_of_view: inputs.field_of_view,
    };

    // Create camera
    let camera = Pinhole::new(view, film);

    let integrator = RayTracer {
        n_ambient_samples: inputs.n_ambient_samples,
        n_shadow_samples: inputs.n_shadow_samples,
        max_depth: inputs.max_depth,
        limit_weight: inputs.limit_weight,
        count_specular_bounce: inputs.count_specular_bounce,
    };

    let buffer = integrator.render(&scene, &camera);

    buffer.save_hdre(std::path::Path::new(&inputs.output));
}
