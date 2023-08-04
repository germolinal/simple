use clap::{Parser, ValueEnum};
use rendering::colourmap::Colourmap;

use rendering::image::ImageBuffer;
use rendering::Float;

#[derive(ValueEnum, Clone)]
enum ArgColourMap {
    Radiance,
    Inferno,
    Magma,
    Plasma,
    Viridis,
}

#[derive(Parser)]
struct Inputs {
    #[clap(short, long)]
    /// The image to falsecolorize
    pub input: String,

    #[clap(short, long)]
    /// The output file (it is a JPEG)
    pub output: String,

    /// The maximum value in the scale
    #[clap(short = 's', long)]
    pub max: Option<Float>,

    /// The minimum value in the scale
    #[clap(long)]
    pub min: Option<Float>,

    /// Use a log10 scale
    #[clap(short, long)]
    pub log: bool,

    /// The colour scale to use
    #[clap(short, long)]
    map: ArgColourMap,
}

fn main() {
    let inputs = Inputs::parse();

    let input = inputs.input;

    let image = match ImageBuffer::from_file(std::path::Path::new(&input)) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let scale = match inputs.map {
        ArgColourMap::Radiance => Colourmap::Radiance,
        ArgColourMap::Inferno => Colourmap::Inferno,
        ArgColourMap::Magma => Colourmap::Magma,
        ArgColourMap::Plasma => Colourmap::Plasma,
        ArgColourMap::Viridis => Colourmap::Viridis,
    };

    if inputs.log {
        image.save_log_falsecolour(
            inputs.min,
            inputs.max,
            scale,
            std::path::Path::new(&inputs.output),
        )
    } else {
        image.save_falsecolour(
            inputs.min,
            inputs.max,
            scale,
            std::path::Path::new(&inputs.output),
        )
    }
}
