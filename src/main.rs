use std::fs::File;
use std::io::{copy, BufReader};

use clap::Parser;
use image::imageops::overlay;
use image::{ImageBuffer, ImageReader};
use serde::Deserialize;

#[derive(Deserialize)]
struct XkcdMetadata {
    safe_title: String,
    img: String,
    day: String,
    month: String,
    year: String,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(long)]
    width: u32,
    #[arg(long)]
    height: u32,
    #[arg(short, default_value_t = 31)]
    r: u8,
    #[arg(short, default_value_t = 36)]
    g: u8,
    #[arg(short, default_value_t = 31)]
    b: u8,
}

fn get_metadata(url: &str) -> Result<XkcdMetadata, ureq::Error> {
    let recv_body = ureq::get(url)
        .call()?
        .body_mut()
        .read_json::<XkcdMetadata>()?;

    Ok(recv_body)
}

fn download_img(original_url: &str, output_filename: &str) -> Result<(), ureq::Error> {
    let scaled_url = original_url.replace(".png", "_2x.png");

    let mut response = match ureq::get(scaled_url).call() {
        Ok(res) => res,
        Err(_) => ureq::get(original_url).call()?,
    };

    let mut reader = BufReader::new(response.body_mut().with_config().reader());
    let mut file = File::create(output_filename).expect("Failed to create file.");
    copy(&mut reader, &mut file).expect("Failed to save image.");

    Ok(())
}

fn main() {
    let cli = Cli::parse();

    let metadata = match get_metadata("https://xkcd.com/info.0.json") {
        Ok(result) => result,
        Err(_) => {
            panic!("Could not retrieve metadata from xkcd website.")
        }
    };

    let filename = format!(
        "{0}_{1}_{2}_{3}.png",
        metadata.year, metadata.month, metadata.day, metadata.safe_title
    );

    match download_img(&metadata.img, &filename) {
        Ok(_) => {}
        Err(_) => panic!("Failed to download image."),
    }

    let mut comic_img = ImageReader::open(&filename)
        .expect("Failed to open image.")
        .decode()
        .expect("Failed to decode img.");

    comic_img.invert();
    let mut comic_buffer = comic_img.into_rgba8();

    let background_pixel = image::Rgba([cli.r, cli.g, cli.b, 255]);

    for (_x, _y, pixel) in comic_buffer.enumerate_pixels_mut() {
        if *pixel == image::Rgba([0, 0, 0, 255]) {
            *pixel = background_pixel
        }
    }

    // Place comic in the middle of the background buffer
    let mut background_buffer = ImageBuffer::from_pixel(cli.width, cli.height, background_pixel);
    overlay(
        &mut background_buffer,
        &comic_buffer,
        (cli.width / 2 - comic_buffer.width() / 2).into(),
        (cli.height / 2 - comic_buffer.height() / 2).into(),
    );
    let _ = background_buffer.save(filename);
}
