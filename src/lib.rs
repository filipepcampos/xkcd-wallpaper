use std::fs::File;
use std::io::{copy, BufReader};

use image::imageops::overlay;
use image::{DynamicImage, ImageBuffer, ImageReader};
use log::{info, warn};
use serde::Deserialize;
use thiserror::Error;

#[derive(Deserialize)]
/// Metadata obtained through the xkcd API
pub struct Metadata {
    pub num: u64,
    pub safe_title: String,
    pub img: String,
    pub day: String,
    pub month: String,
    pub year: String,
}

/// Wrapper for xkcd image which contains metadata
pub struct Image {
    pub img: DynamicImage,
    pub metadata: Metadata,
}

#[derive(PartialEq)]
/// Foreground color for drawings, either light or dark
pub enum ForegroundColor {
    Light,
    Dark,
}

/// Represents dimensions of a screen
pub struct ScreenDimensions {
    pub width: u32,
    pub height: u32,
}

#[derive(Error, Debug)]
pub enum XkcdError {
    #[error("Network error: {0}")]
    Network(#[from] ureq::Error),
    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Tempfile error: {0}")]
    Tempfile(#[from] tempfile::PersistError),
    #[error("Other error: {0}")]
    Other(String),
}

/// Download a xkcd comic png (specific number or latest) and return a Image object
pub fn download_comic(comic_number: Option<u32>) -> Result<Image, XkcdError> {
    let metadata = get_metadata(comic_number)?;

    // NamedTempFile over tempfile because it requires .png suffix to be supported by ImageReader
    let mut file = tempfile::NamedTempFile::with_suffix(".png")?;
    download_img(&metadata.img, file.as_file_mut())?;

    let img = ImageReader::open(file.path())?.decode()?;

    Ok(Image { img, metadata })
}

/// Use a comic `Image` to obtain a wallpaper, returned as a `Image`.
pub fn get_wallpaper_from_comic(
    comic_img: Image,
    fg_color: ForegroundColor,
    bg_color: image::Rgba<u8>,
    screen_dimensions: ScreenDimensions,
) -> Image {
    let metadata = comic_img.metadata;
    let mut comic_img = comic_img.img.to_owned();

    if fg_color == ForegroundColor::Light {
        info!("inverting image colors");
        comic_img.invert();
    }

    let mut comic_buffer = comic_img.into_rgba8();

    let comic_background_color = match fg_color {
        ForegroundColor::Light => image::Rgba([0, 0, 0, 255]),
        ForegroundColor::Dark => image::Rgba([255, 255, 255, 255]),
    };

    info!("replacing background pixels with background colors");
    for (_x, _y, pixel) in comic_buffer.enumerate_pixels_mut() {
        if *pixel == comic_background_color {
            *pixel = bg_color;
        }
    }

    // Place comic in the middle of the background buffer
    info!("placing comic in center of the background");
    let mut background_buffer =
        ImageBuffer::from_pixel(screen_dimensions.width, screen_dimensions.height, bg_color);
    overlay(
        &mut background_buffer,
        &comic_buffer,
        (screen_dimensions.width / 2 - comic_buffer.width() / 2).into(),
        (screen_dimensions.height / 2 - comic_buffer.height() / 2).into(),
    );

    Image {
        img: DynamicImage::ImageRgba8(background_buffer),
        metadata,
    }
}

/// Save `Image` to a specific output file, supports placeholders.
///
/// # Filename placeholders
/// The output filename can use placeholders which will be substituted with corresponding metadata
///
/// y   Two-digit year (e.g., 25)
/// m   Two-digit month (e.g., 06)
/// d   Two-digit day (e.g., 22)
/// n   Comic number
/// t   Title   
/// For instance `./output/%y-%m-%d-%t` would generated a file `./output/2025-06-20-SomeTitle`.
pub fn save_img_to_file(img: &Image, filename: &str) {
    let filename = convert_fmt_filename(filename, &img.metadata);
    let _ = img.img.save(filename); // TODO: Shouldn't ignore output
}

fn get_metadata(comic_number: Option<u32>) -> Result<Metadata, XkcdError> {
    let metadata_url = match comic_number {
        Some(num) => format!("https://xkcd.com/{}/info.0.json", num),
        None => "https://xkcd.com/info.0.json".to_string(),
    };
    info!("downloading metadata from url {}", metadata_url);

    let recv_body = ureq::get(metadata_url)
        .call()?
        .body_mut()
        .read_json::<Metadata>()?;
    info!("metadata downloaded successfully");

    Ok(recv_body)
}

fn download_img(original_url: &str, mut output_file: &File) -> Result<(), XkcdError> {
    let scaled_url = original_url.replace(".png", "_2x.png");

    info!("downloading img {}", scaled_url);
    let mut response = match ureq::get(scaled_url).call() {
        Ok(res) => res,
        Err(_) => {
            warn!(
                "cannot get image with 2x resolution, falling back to regular res. {}",
                original_url
            );
            ureq::get(original_url).call()?
        }
    };

    info!("reading response into BufReader");
    let mut reader = BufReader::new(response.body_mut().with_config().reader());
    copy(&mut reader, &mut output_file)?;

    Ok(())
}

fn convert_fmt_filename(format_filename: &str, metadata: &Metadata) -> String {
    let replacements = [
        ("%y", metadata.year.as_str()),
        ("%m", metadata.month.as_str()),
        ("%d", metadata.day.as_str()),
        ("%t", metadata.safe_title.as_str()),
        ("%n", &metadata.num.to_string()),
    ];

    let mut output = format_filename.to_owned();
    for (token, value) in &replacements {
        output = output.replace(token, value);
    }

    info!("converted filename from {} to {}", format_filename, output);
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("%y.png", "2025.png")]
    #[case("output/file.png", "output/file.png")]
    #[case("%y-%m-%d_%t", "2025-06-27_Some title")]
    fn convert_filename_ok(#[case] input: &str, #[case] output: &str) {
        let metadata = Metadata {
            num: 42,
            safe_title: "Some title".to_string(),
            year: "2025".to_string(),
            month: "06".to_string(),
            day: "27".to_string(),
            img: "https://example.com".to_string(),
        };

        assert_eq!(convert_fmt_filename(input, &metadata), output);
    }
}
