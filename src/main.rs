use clap::Parser;
use log::info;
use xkcd_wallpaper::{
    download_comic, get_wallpaper_from_comic, save_img_to_file, ForegroundColor, ScreenDimensions,
};

#[derive(Parser)]
#[command(
    version,
    long_about,
    after_help = "Examples:

    Generate a 2560x1440 wallpaper from comic number 3084
    with a dark green background and white colored drawings

        xkcd-wallpaper \\
            --width 2560 --height 1440 \\
            --bg \"#1F241F\" \\
            --fg light \\
            --comic 3084

    Generate a 1920x1080 wallpaper from the latest issue
    with and write it to a specific output folder with
    a Year-Month-Day-Title format, e.g. 2025-06-20-SomeTitle.

        xkcd-wallpaper \\
            --width 1920 --height 1080 \\
            --output ./output/%y-%m-%d-%t

Format string format:
    You can use the following placeholders in the format string:
        %y   Two-digit year (e.g., 25)
        %m   Two-digit month (e.g., 06)
        %d   Two-digit day (e.g., 22)
        %n   Comic number
        %t   Title   
"
)]
/// Download xkcd wallpapers
///
/// To use simply call `xkcd-wallpaper --width 1920 --height 1080`
struct Cli {
    #[arg(long, help = "Width of output wallpaper")]
    width: u32,
    #[arg(long, help = "Height of output wallpaper")]
    height: u32,
    #[arg(long, value_parser=parse_hex_color, default_value = "#1F241F", help="Background color in HEX format")]
    bg: image::Rgba<u8>,
    #[arg(
        long,
        default_value = "light",
        help = "Foreground color, either dark or light"
    )] // TODO: Stronger constraints
    fg: String,
    #[arg(
        long,
        help = "Optional comic number, by default the latest xkcd will be used."
    )]
    comic: Option<u32>,
    #[arg(short, long, default_value = "./%y-%m-%d_%t.png")]
    output: String,
}

fn main() {
    env_logger::init();
    info!("parsing CLI arguments");
    let cli = Cli::parse();

    let screen_dimensions = ScreenDimensions {
        width: cli.width,
        height: cli.height,
    };

    let fg_color = match cli.fg.as_str() {
        "dark" => ForegroundColor::Dark,
        _ => ForegroundColor::Light,
    };

    info!("starting comic download");
    let comic_img = match download_comic(cli.comic) {
        Ok(img) => img,
        Err(e) => {
            eprintln!("Failed to download comic: {e}");
            std::process::exit(1);
        }
    };

    info!("converting xkcd image into wallpaper");
    let wallpaper_img = get_wallpaper_from_comic(comic_img, fg_color, cli.bg, screen_dimensions);

    save_img_to_file(&wallpaper_img, &cli.output);
}

/// Parse a colour in “#RRGGBB”
fn parse_hex_color(s: &str) -> Result<image::Rgba<u8>, String> {
    let hex = s.trim_start_matches('#');
    let full = match hex.len() {
        6 => format!("{hex}FF"),
        _ => return Err("Hex colour must be 6 hex digits (e.g. #1e90ff)".into()),
    };
    let v = u32::from_str_radix(&full, 16).map_err(|_| "Invalid hex digits")?;

    Ok(image::Rgba([
        ((v >> 24) & 0xFF) as u8, // R
        ((v >> 16) & 0xFF) as u8, // G
        ((v >> 8) & 0xFF) as u8,  // B
        (v & 0xFF) as u8,         // A
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("#FF0000", 255, 0, 0)]
    #[case("FF0000", 255, 0, 0)]
    #[case("#FF69B4", 255, 105, 180)]
    fn hex_parse_ok(#[case] input: &str, #[case] r: u8, #[case] g: u8, #[case] b: u8) {
        let rgba = image::Rgba([r, g, b, 255]);
        assert_eq!(parse_hex_color(input), Ok(rgba));
    }

    #[rstest]
    #[case("FF00")]
    #[case("ZZ0000")]
    #[case("")]
    fn hex_parse_error(#[case] input: &str) {
        assert!(parse_hex_color(input).is_err())
    }
}
