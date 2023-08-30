use std::path::PathBuf;

use clap::Parser;
use image::Rgba;

use mlaa_impl::{mlaa_metrics, mlaa_painter};

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

    mlaa_metrics(
        input_image.width() as usize,
        input_image.height() as usize,
        |x, y| {
            input_image
                .get_pixel_checked(x as u32, y as u32)
                .unwrap_or(&Rgba([0, 0, 0, 0]))
                .to_owned()
        },
        0.0,
        |gradient| {
            mlaa_painter(
                |c1, c2, t| {
                    // The `image` crate doesn't give a fuck about gamma correctness.
                    // Have to use `smol-rgb` instead of their blending functions.

                    fn lerp(a: f32, b: f32, t: f32) -> f32 {
                        a * (1.0 - t) + b * t
                    }

                    Rgba([
                        smol_rgb::linear_to_encoded(lerp(
                            smol_rgb::encoded_to_linear(c1.0[0]),
                            smol_rgb::encoded_to_linear(c2.0[0]),
                            t,
                        )),
                        smol_rgb::linear_to_encoded(lerp(
                            smol_rgb::encoded_to_linear(c1.0[1]),
                            smol_rgb::encoded_to_linear(c2.0[1]),
                            t,
                        )),
                        smol_rgb::linear_to_encoded(lerp(
                            smol_rgb::encoded_to_linear(c1.0[2]),
                            smol_rgb::encoded_to_linear(c2.0[2]),
                            t,
                        )),
                        lerp(c1.0[3] as f32, c2.0[3] as f32, t) as u8,
                    ])
                },
                |x, y, c| {
                    output_image.put_pixel(x as u32, y as u32, c);
                },
                &gradient,
            );
        },
    );

    output_image
        .save(args.output_path)
        .expect("Failed to save output image");
}
