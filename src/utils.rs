use chrono::Local;
use directories::UserDirs;
use eframe::egui::Rect;
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

#[derive(PartialEq, Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AspectRatio {
    Square,
    FourThree,
    ThreeFour,
    ThreeTwo,
    TwoThree,
    SixteenNine,
    NineSixteen,
    Window(f32),
}
impl Default for AspectRatio {
    fn default() -> AspectRatio {
        AspectRatio::Square
    }
}
impl AspectRatio {
    pub fn div(&self) -> f32 {
        match self {
            AspectRatio::Square => 1.0,
            AspectRatio::FourThree => 3.0 / 4.0,
            AspectRatio::ThreeFour => 4.0 / 3.0,
            AspectRatio::ThreeTwo => 2.0 / 3.0,
            AspectRatio::TwoThree => 3.0 / 2.0,
            AspectRatio::SixteenNine => 9.0 / 16.0,
            AspectRatio::NineSixteen => 16.0 / 9.0,
            AspectRatio::Window(r) => *r,
        }
    }
}

#[derive(PartialEq, Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ImageScaling {
    Preview(f32),
    FitScreen(Rect),
    HalfK,
    OneK,
    TwoK,
    FourK,
    Actual,
}
impl Default for ImageScaling {
    fn default() -> ImageScaling {
        ImageScaling::OneK
    }
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
                if ratio >= 1.0 {
                    return RectSize::new(*w as usize, (*w * ratio) as usize);
                } else {
                    return RectSize::new((*w / ratio) as usize, *w as usize);
                }
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
        zip.write_all(&*buffer)?;
        buffer.clear();
    }
    zip.finish()?;
    Ok(())
}
