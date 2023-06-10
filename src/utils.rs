use chrono::Local;
use directories::UserDirs;
use eframe::egui::Rect;
use fuzzy_fraction::fuzzy_fraction;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use zip::{result::ZipResult, ZipWriter};
use zip::{write::FileOptions, CompressionMethod};

pub const OUTPUT_NAME: &str = "PickPicPack";

pub fn file_timestamp() -> String {
    let time_stamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
    format!("{}_{}", OUTPUT_NAME, time_stamp)
}

pub fn default_path() -> PathBuf {
    if let Some(user_dirs) = UserDirs::new() {
        if let Some(pics_dir) = user_dirs.picture_dir() {
            pics_dir.to_owned()
        } else {
            PathBuf::default()
        }
    } else {
        PathBuf::default()
    }
}

pub fn random_gray() -> image::Rgba<u8> {
    let mut rng = rand::thread_rng();
    let r: u8 = rng.gen_range(60..200);
    //image::Rgba::<u8>::from([rng.gen(), rng.gen(), rng.gen(), 255])
    image::Rgba::<u8>::from([r, r / 4, 0, 128])
}

#[derive(Default, PartialEq, Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AspectRatio {
    #[default]
    Square,
    FourThree,
    ThreeTwo,
    SixteenNine,
    Window(f32),
    Custom((usize, usize)),
}

impl AspectRatio {
    pub fn div(&self) -> f32 {
        match self {
            AspectRatio::Square => 1.0,
            AspectRatio::FourThree => 3.0 / 4.0,
            AspectRatio::ThreeTwo => 2.0 / 3.0,
            AspectRatio::SixteenNine => 9.0 / 16.0,
            AspectRatio::Window(r) => *r,
            AspectRatio::Custom((a, b)) => *b as f32 / *a as f32,
        }
    }
}

#[derive(Default, PartialEq, Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ImageScaling {
    Preview(f32),
    FitScreen(Rect),
    HalfK,
    #[default]
    OneK,
    TwoK,
    FourK,
    Actual,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct RectSize {
    pub w: usize,
    pub h: usize,
}
impl RectSize {
    pub fn new(w: usize, h: usize) -> Self {
        RectSize { w, h }
    }

    pub fn by_scale_and_ratio(scaling: &ImageScaling, aspect: &AspectRatio) -> Self {
        let ratio = aspect.div();

        let k = 1024.0;
        let side = match scaling {
            ImageScaling::Preview(w) => {
                return RectSize::new(*w as usize, (*w * ratio) as usize);
            }
            ImageScaling::FitScreen(size) => {
                let w = size.max.x;
                let h = size.max.y;
                if ratio <= h / w {
                    return RectSize::new(w as usize, (w * ratio) as usize);
                } else {
                    return RectSize::new((h / ratio) as usize, h as usize);
                }
            }
            ImageScaling::HalfK => k / 2.0,
            ImageScaling::OneK => k,
            ImageScaling::TwoK => k * 2.0,
            ImageScaling::FourK => k * 4.0,
            _ => 0.0, //Actual is not covered because it is unknown.
        };

        if ratio <= 1.0 {
            RectSize::new(side as usize, (side * ratio) as usize)
        } else {
            RectSize::new((side / ratio) as usize, side as usize)
        }
    }
}

pub fn fit_to_square(width: u32, height: u32, side: u32) -> (u32, u32) {
    let side = side as f32;
    let ratio = height as f32 / width as f32;
    let (avg_width, avg_height) = match ratio <= 1.0 {
        true => (side, (side * ratio)),
        false => ((side / ratio), side),
    };
    (avg_width as u32, avg_height as u32)
}

pub fn archive_files(files: Vec<&PathBuf>, path: PathBuf) -> ZipResult<()> {
    let zip_file = File::create(&path)?;
    let mut zip = ZipWriter::new(zip_file);
    let options = FileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .large_file(true);
    let mut buffer = Vec::new();
    let prefix_len = format!("{}", files.len()).len();
    for (i, file) in files.iter().enumerate() {
        let file_name = format!(
            "{:0width$}_{}",
            i + 1,
            file.file_name().unwrap().to_string_lossy(),
            width = prefix_len
        );
        zip.start_file(file_name, options)?;
        let mut f = File::open(file)?;
        f.read_to_end(&mut buffer)?;
        zip.write_all(&buffer)?;
        buffer.clear();
    }
    zip.finish()?;
    Ok(())
}

pub fn parse_custom_ratio(raw_text: &str) -> (usize, usize) {
    // const DEFAULT: (usize, usize) = (1, 1);
    if let Ok(f) = raw_text.parse::<f32>() {
        let (x, y) = fuzzy_fraction((f.abs() * 100.0).floor() as usize, 100);
        return (x.min(100), y.min(100));
    }

    let input_vec: Vec<&str> = raw_text
        .split(|c: char| !c.is_ascii_digit())
        .filter(|n| !n.is_empty())
        .collect();

    let a = input_vec
        .first()
        .unwrap_or(&"")
        .parse::<usize>()
        .unwrap_or(0);
    let b = input_vec
        .get(1)
        .unwrap_or(&"")
        .parse::<usize>()
        .unwrap_or(0);

    let (x, y) = match (a, b) {
        (0, 0) => (2, 1),
        (a, 0) => (a, 1),
        (0, b) => (1, b),
        (a, b) => fuzzy_fraction(a, b),
    };

    (x.min(100), y.min(100))
}
