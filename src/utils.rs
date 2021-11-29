use chrono::Local;
use directories::UserDirs;
use eframe::egui::Rect;
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

#[derive(PartialEq, Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AspectRatio {
    Square,
    FourThree,
    ThreeTwo,
    SixteenNine,
    Window(f32),
    Custom((usize, usize)),
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
            AspectRatio::ThreeTwo => 2.0 / 3.0,
            AspectRatio::SixteenNine => 9.0 / 16.0,
            AspectRatio::Window(r) => *r,
            AspectRatio::Custom((a, b)) => *b as f32 / *a as f32,
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
        zip.write_all(&*buffer)?;
        buffer.clear();
    }
    zip.finish()?;
    Ok(())
}

pub fn parse_custom_ratio(raw_text: &str) -> (usize, usize) {
    // const DEFAULT: (usize, usize) = (1, 1);
    if let Ok(f) = raw_text.parse::<f32>() {
        return fuzzy_fraction((f * 100.0).floor() as usize, 100);
    }

    let input_vec: Vec<&str> = raw_text
        .split(|c: char| !c.is_ascii_digit())
        .filter(|n| !n.is_empty())
        .collect();

    let a = input_vec
        .get(0)
        .unwrap_or(&"")
        .parse::<usize>()
        .unwrap_or(0);
    let b = input_vec
        .get(1)
        .unwrap_or(&"")
        .parse::<usize>()
        .unwrap_or(0);
    match (a, b) {
        (0, 0) => (2, 1),
        (a, 0) => (a, 1),
        (0, b) => (1, b),
        (a, b) => fuzzy_fraction(a, b),
    }
}

// fn gcd(a: usize, b: usize) -> usize {
//     match ((a, b), (a & 1, b & 1)) {
//         ((x, y), _) if x == y => y,
//         ((0, x), _) | ((x, 0), _) => x,
//         ((x, y), (0, 1)) | ((y, x), (1, 0)) => gcd(x >> 1, y),
//         ((x, y), (0, 0)) => gcd(x >> 1, y >> 1) << 1,
//         ((x, y), (1, 1)) => {
//             let (x, y) = (x.min(y), x.max(y));
//             gcd((y - x) >> 1, x)
//         }
//         _ => unreachable!(),
//     }
// }

//I'm not proud of it, but it works just like it shold - fast and dirty.
fn fuzzy_fraction(a: usize, b: usize) -> (usize, usize) {
    println!("{} {}", a, b);
    match (a, b) {
        (0, 0) => return (a, b),
        (0, _) => return (0, 1),
        (_, 0) => return (1, 0),
        // (a, b)
        //     if ((a.max(b) - a.min(b)) as f32 / ((a + b) as f32 / 2.0) * 100.0) as usize <= 10 =>
        // {
        //     return (1, 1)
        // }
        (a, b) if a as f32 / b as f32 >= 100.0 => return (100, 1),
        (a, b) if b as f32 / a as f32 >= 100.0 => return (1, 100),
        (_, _) => (),
    };

    let switch = a < b;
    let max = a.max(b);
    let min = a.min(b);
    let float = max as f32 / min as f32;
    let whole = float.floor();
    let d = ((float - whole) * 100.0) as usize;
    if d <= 10 || float.round() >= 10.0 {
        match switch {
            true => return (1, float.round() as usize),
            false => return (float.round() as usize, 1),
        };
    }
    let fraction = {
        if d < 47 {
            if d < 25 {
                if d < 16 {
                    if d < 12 {
                        if d < 11 {
                            (1, 10)
                        } else {
                            (1, 9)
                        }
                    } else if d < 14 {
                        (1, 8)
                    } else {
                        (1, 7)
                    }
                } else if d < 19 {
                    (1, 6)
                } else if d < 22 {
                    (1, 5)
                } else {
                    (2, 9)
                }
            } else if d < 37 {
                if d < 28 {
                    (1, 4)
                } else if d < 31 {
                    (2, 7)
                } else {
                    (1, 3)
                }
            } else if d < 42 {
                if d < 40 {
                    (3, 8)
                } else {
                    (2, 5)
                }
            } else if d < 44 {
                (3, 7)
            } else {
                (4, 9)
            }
        } else if d < 71 {
            if d < 60 {
                if d < 55 {
                    (1, 2)
                } else if d < 57 {
                    (5, 9)
                } else {
                    (4, 7)
                }
            } else if d < 62 {
                (3, 5)
            } else if d < 66 {
                (5, 8)
            } else {
                (2, 3)
            }
        } else if d < 80 {
            if d < 74 {
                (5, 7)
            } else if d < 77 {
                (3, 4)
            } else {
                (7, 9)
            }
        } else if d < 85 {
            if d < 83 {
                (4, 5)
            } else {
                (5, 6)
            }
        } else if d < 87 {
            (6, 7)
        } else if d < 88 {
            (7, 8)
        } else if d < 90 {
            (8, 9)
        } else {
            (9, 10)
        }
    };

    let answer = (whole as usize * fraction.1 + fraction.0, fraction.1);
    if switch {
        return (answer.1, answer.0);
    }
    answer
}
