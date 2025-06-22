use clap::Parser;
use log::{info, warn};
use xkcd::{download_comic, get_wallpaper_from_img, FgColor, ScreenDimensions};

// TODO: Write tests
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

// TODO Generic help text

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    // Command-line interface to download and create wallpapers based on XKCD comics (xkcd.com).
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
    )] // TODO:
    // Stronger
    // constraints
    fg: String,
    #[arg(
        long,
        help = "Optional comic number, by default the latest xkcd will be used."
    )]
    comic: Option<u32>, // TODO: Use this
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
        "dark" => FgColor::Dark,
        _ => FgColor::Light,
    };

    info!("starting comic download");
    let filename = download_comic(cli.comic, &cli.output);

    info!("converting xkcd image into wallpaper");
    let wallpaper_buffer = get_wallpaper_from_img(&filename, fg_color, cli.bg, screen_dimensions);

    info!("saving wallpaper to file {}", filename);
    let _ = wallpaper_buffer.save(filename);
}
