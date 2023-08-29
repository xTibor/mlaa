use std::path::PathBuf;

use clap::Parser;
use image::Rgba;

use mlaa::{mlaa, Gradient};

#[derive(Parser)]
struct MlaaArgs {
    pub input_path: PathBuf,

    pub output_path: PathBuf,
}

// cargo run --release --bin mlaa_image -- test/input.png test/output.png

fn main() {
    let args = MlaaArgs::parse();

    let input_image = image::open(args.input_path)
        .expect("Failed to open input image")
        .into_rgba8();

    let mut output_image = input_image.clone();

    mlaa(
        input_image.width() as usize,
        input_image.height() as usize,
        |x, y| {
            input_image
                .get_pixel_checked(x as u32, y as u32)
                .unwrap_or(&Rgba([0, 0, 0, 0]))
        },
        0.0,
        |gradient| {
            match gradient {
                Gradient::Vertical { x, y, height, colors } => {
                    // TODO
                    output_image.put_pixel(x as u32, y as u32, Rgba([255, 255, 0, 255]));
                }
                Gradient::Horizontal { x, y, width, colors } => {
                    // TODO
                    output_image.put_pixel(x as u32, y as u32, Rgba([255, 0, 255, 255]));
                }
            }
        },
    );

    output_image
        .save(args.output_path)
        .expect("Failed to save output image");
}
