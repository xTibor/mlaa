use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::path::PathBuf;

use clap::Parser;
use image::{ImageFormat, Rgba};

use mlaa_impl::{mlaa_metrics, mlaa_painter, MlaaOptions};

#[derive(Parser)]
struct MlaaArgs {
    #[clap(short = 'i', long = "input")]
    pub input_path: Option<PathBuf>,

    #[clap(short = 'o', long = "output")]
    pub output_path: Option<PathBuf>,

    #[clap(short = 'c', long = "config")]
    pub config_path: Option<PathBuf>,
}

// cargo run --release --bin mlaa_image -- -i test/input.png -o test/output.png

fn main() {
    let args = MlaaArgs::parse();

    let input_image = {
        let mut reader: Box<dyn Read> = if let Some(input_path) = args.input_path.as_ref() {
            Box::new(File::open(input_path).unwrap())
        } else {
            Box::new(std::io::stdin())
        };

        let image_format = args
            .input_path
            .and_then(ImageFormat::from_extension)
            .unwrap_or(ImageFormat::Png);

        let mut image_data = Vec::new();
        reader.read_to_end(&mut image_data).unwrap();

        image::load_from_memory_with_format(&image_data, image_format).unwrap()
    };

    let input_image = input_image.to_rgba8();
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
        &MlaaOptions::default(),
        |mlaa_feature| {
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
                &mlaa_feature,
            );
        },
    );

    {
        let mut writer: Box<dyn Write> = if let Some(output_path) = args.output_path.as_ref() {
            Box::new(File::create(output_path).unwrap())
        } else {
            Box::new(std::io::stdout())
        };

        let image_format = args
            .output_path
            .and_then(ImageFormat::from_extension)
            .unwrap_or(ImageFormat::Png);

        let mut image_data = Vec::new();
        output_image
            .write_to(&mut Cursor::new(&mut image_data), image_format)
            .unwrap();

        writer.write_all(&image_data).unwrap();
    }
}
