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
use clap::Parser;
use rendering::{colour_matrix::save_colour_matrix, Scene};
use weather::ReinhartSky;
// use rendering::from_radiance::from
use geometry::{Point3D, Ray3D, Vector3D};
use rendering::daylight_coefficients::DCFactory;
use rendering::Wavelengths;
use utils::ProgressBar;

/// Calculates the Daylight Coefficients
#[derive(Debug, Parser)]
struct Inputs {
    #[clap(short, long)]
    /// The file to load the model from
    pub input: String,

    #[clap(short, long)]
    /// The file where the matrix will be stored
    pub output: String,

    #[clap(short = 'm', long = "sky_subdivision", default_value_t = 1)]
    pub mf: usize,

    /// The number of bounces before a ray is terminated (-ab in Radiance lingo)
    #[clap(short = 'b', long = "max_depth", default_value_t = 120)]
    pub max_depth: usize,

    /// The number of secondary rays sent from the first interaction.
    /// From the second interaction and on, this number is reduced
    #[clap(short = 'a', long = "ambient_samples", default_value_t = 30000)]
    pub n_ambient_samples: usize,

    /// The number of sensors to receive in the standard input
    #[clap(short = 'n', long, default_value_t = 64)]
    pub n_sensors: usize,
}

fn main() -> Result<(), String> {
    // time cargo run --release --package simple --bin sfluxmtx -- -i tests/cold_apartment/cold.spl -o check.csv -b 5 -a 5000
    // cargo instruments --release --template Allocations --package simple --bin sfluxmtx -- -i tests/cold_apartment/cold.spl -o check.csv -b 5 -a 5000
    // cargo instruments --release --template 'CPU Profiler' --package simple --bin sfluxmtx -- -i tests/cold_apartment/cold.spl -o check.csv -b 5 -a 5000

    let inputs = Inputs::parse();

    let input_file = inputs.input;
    let mut scene = if input_file.ends_with(".rad") {
        Scene::from_radiance(input_file)?
    } else if input_file.ends_with(".spl") {
        let (model, _header) = model::Model::from_file(input_file)?;
        Scene::from_simple_model(&model, Wavelengths::Solar)?
    } else {
        return Err(format!("Unkwown format in file {}", input_file));
    };

    scene.build_accelerator();

    let factory = DCFactory {
        max_depth: inputs.max_depth,
        n_ambient_samples: inputs.n_ambient_samples,
        reinhart: ReinhartSky::new(inputs.mf),
    };

    let rays = vec![
        Ray3D {
            origin: Point3D::new(1., 6., 1.),
            direction: Vector3D::new(0., 0., 1.),
        },
        Ray3D {
            origin: Point3D::new(1., 6., 1.),
            direction: Vector3D::new(0., 0., 1.),
        },
        Ray3D {
            origin: Point3D::new(1., 6., 1.),
            direction: Vector3D::new(0., 0., 1.),
        },
        Ray3D {
            origin: Point3D::new(1., 6., 1.),
            direction: Vector3D::new(0., 0., 1.),
        },
        Ray3D {
            origin: Point3D::new(1., 6., 1.),
            direction: Vector3D::new(0., 0., 1.),
        },
        Ray3D {
            origin: Point3D::new(1., 6., 1.),
            direction: Vector3D::new(0., 0., 1.),
        },
    ];

    let progress = ProgressBar::new(
        "Calculating Daylight Coefficients".to_string(),
        rays.len() * factory.n_ambient_samples,
    );

    let dc_matrix = factory.calc_dc(&rays, &scene, Some(&progress));
    progress.done();
    save_colour_matrix(&dc_matrix, std::path::Path::new(&inputs.output))?;

    Ok(())
}

//fn load_sensor_file() -> Vec<Ray3D> {
//     let mut rays: Vec<Ray3D> = Vec::with_capacity(inputs.n_sensors);
//     let mut buffer = String::new();
//     let mut ln = 0;
//     dbg!("Before while");
//     while let Ok(n) = std::io::stdin().read_line(&mut buffer) {
//         if n == 0 {
//             break; // Reached EOF
//         }
//         ln += 1;
//         {
//             let st: Vec<&str> = buffer.trim().split(' ').collect();
//             // let st : Vec<&str> = buffer.split_ascii_whitespace().collect();

//             if st.len() != 6 {
//                 eprintln!(
//                     "Expecting six values—e.g., '1. 2. 3. 4. 5. 6.'—... line {} contains '{:?}'",
//                     ln, st
//                 );
//                 std::process::exit(1);
//             }
//             let ox = match st[0].parse::<Float>() {
//                 Ok(v) => v,
//                 Err(_) => {
//                     eprintln!(
//                         "Expecting value 1 in sensor line to be a number... found '{}'",
//                         st[0]
//                     );
//                     std::process::exit(1);
//                 }
//             };
//             let oy = match st[1].parse::<Float>() {
//                 Ok(v) => v,
//                 Err(_) => {
//                     eprintln!(
//                         "Expecting value 2 in sensor line to be a number... found '{}'",
//                         st[1]
//                     );
//                     std::process::exit(1);
//                 }
//             };
//             let oz = match st[2].parse::<Float>() {
//                 Ok(v) => v,
//                 Err(_) => {
//                     eprintln!(
//                         "Expecting value 3 in sensor line to be a number... found '{}'",
//                         st[2]
//                     );
//                     std::process::exit(1);
//                 }
//             };
//             let dx = match st[3].parse::<Float>() {
//                 Ok(v) => v,
//                 Err(_) => {
//                     eprintln!(
//                         "Expecting value 4 in sensor line to be a number... found '{}'",
//                         st[3]
//                     );
//                     std::process::exit(1);
//                 }
//             };
//             let dy = match st[4].parse::<Float>() {
//                 Ok(v) => v,
//                 Err(_) => {
//                     eprintln!(
//                         "Expecting value 5 in sensor line to be a number... found '{}'",
//                         st[4]
//                     );
//                     std::process::exit(1);
//                 }
//             };
//             let dz = match st[5].parse::<Float>() {
//                 Ok(v) => v,
//                 Err(_) => {
//                     eprintln!(
//                         "Expecting value 6 in sensor line to be a number... found '{}'",
//                         st[5]
//                     );
//                     std::process::exit(1);
//                 }
//             };
//             rays.push(Ray3D {
//                 origin: Point3D::new(ox, oy, oz),
//                 direction: Vector3D::new(dx, dy, dz).get_normalized(),
//             })
//         }
//         buffer.truncate(0);
//     }
//     rays
// }
