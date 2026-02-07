#![windows_subsystem = "windows"]

mod app;
mod loader;
mod packer;
mod utils;

use app::*;
use clap::{App, Arg};
use eframe::egui::{self, DroppedFile, IconData};
use packer::Packer;
use std::path::PathBuf;
use utils::*;

struct CLIArgsParsed {
    pub input: PathBuf,
    pub output: PathBuf,
    pub ratio: AspectRatio,
    pub equal: bool,
    pub scale: ImageScaling,
    pub zip: bool,
}

fn run_cli(args: CLIArgsParsed) {
    let size = match args.scale {
        ImageScaling::Preview(side) => side,
        _ => 512.0,
    };
    let mut packer = Packer::new(size, args.ratio, args.scale, args.equal);
    let dropped = DroppedFile {
        mime: "Image".to_string(),
        path: Some(args.input),
        name: String::new(),
        last_modified: None,
        bytes: None,
    };
    packer.update(&[dropped]);
    packer.export(&args.output, args.zip);
}

fn run_gui() -> Result<(), eframe::Error> {
    let icon = IconData {
        rgba: image::load_from_memory(include_bytes!("../icon/128x128@2x.png"))
            .unwrap()
            .to_rgba8()
            .to_vec(),
        width: 256,
        height: 256,
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_icon(icon)
            .with_taskbar(true)
            .with_app_id(OUTPUT_NAME)
            .with_decorations(true)
            .with_transparent(false)
            .with_resizable(true)
            .with_maximized(false)
            .with_drag_and_drop(true)
            .with_inner_size([340.0, 624.0])
            .with_transparent(true)
            .with_min_inner_size([280.0, 280.0]),
        ..Default::default()
    };

    eframe::run_native(
        OUTPUT_NAME,
        options,
        Box::new(|_cc| Ok(Box::new(P3App::default()))),
    )
}

fn main() {
    let cli = App::new(OUTPUT_NAME)
        .version(env!("CARGO_PKG_VERSION"))
        .author("by Roman Chumak")
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("INPUT_DIR")
                .help("Sets directory to get images from.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("OUTPUT_DIR")
                .help("Sets directory to put the result. Default is Pictures folder.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("ratio")
                .short("r")
                .long("ratio")
                .value_name("RATIO")
                .help("Sets aspect ratio of package. Can be a float or a pair of integers. Default is 1:1.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("equal")
                .short("e")
                .long("equal")
                .value_name("EQUAL")
                .help("Scale images to equal size.")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("size")
                .short("s")
                .long("size")
                .value_name("SIZE")
                .help("Sets maximum dimension of exported image. Default is Actual.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("zip")
                .short("z")
                .long("zip")
                .value_name("ZIP")
                .help("Also pack all given images to ZIP archive.")
                .takes_value(false),
        )
        .get_matches();

    if let Some(dir) = cli.value_of("input") {
        run_cli(CLIArgsParsed {
            input: PathBuf::from(dir),
            output: match cli.value_of("output") {
                Some(out) => PathBuf::from(out),
                None => default_path(),
            },
            ratio: match cli.value_of("ratio") {
                Some(ratio) => AspectRatio::Custom(parse_custom_ratio(ratio)),
                None => AspectRatio::Square,
            },
            equal: cli.is_present("equal"),
            scale: match cli.value_of("size") {
                Some(size) => {
                    ImageScaling::Preview(size.parse::<f32>().unwrap_or(1024.0).max(32.0))
                }
                None => ImageScaling::Actual,
            },
            zip: cli.is_present("zip"),
        });
    } else {
        run_gui().ok();
    }
}
