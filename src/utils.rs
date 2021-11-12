use chrono::Local;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use winit::event_loop::EventLoop;
pub const OUTPUT_NAME: &str = "PickPicPack";
pub const WINDOW_SCALE: f32 = 2.0;

pub fn window_width(div: f32) -> f32 {
    if let Some(monitor) = EventLoop::new().primary_monitor() {
        let size = monitor.size();
        let side = (size.width.min(size.height) as f32 / (100.0 * div)).round() * 100.0;
        // println!("SIDE: {}", side);
        return side;
    };
    360.0
}

pub fn get_screen_size() -> RectSize {
    if let Some(monitor) = EventLoop::new().primary_monitor() {
        let size = monitor.size();
        return RectSize::new(size.width as usize, size.height as usize);
    };
    RectSize::new(1280, 720)
}

pub fn export_file_path(path: &Path, ext: &str) -> PathBuf {
    let time_stamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
    Path::new(path).join(format!("{}_{}.{}", OUTPUT_NAME, time_stamp, ext))
}

pub fn size_by_side_and_ratio(side: &ImageScaling, aspect: &AspectRatio) -> RectSize {
    let ratio = aspect.div();

    let k = 1024.0;
    let side = match side {
        ImageScaling::Preview(w) => *w,
        ImageScaling::FitScreen => get_screen_size().w as f32,
        ImageScaling::HalfK => k / 2.0,
        ImageScaling::OneK => k,
        ImageScaling::TwoK => k * 2.0,
        ImageScaling::FourK => k * 4.0,
        _ => 0.0,
    };
    if ratio > 1.0 {
        RectSize::new(side as usize, (side * ratio) as usize)
    } else {
        RectSize::new((side / ratio) as usize, side as usize)
    }
}

#[derive(PartialEq, Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AspectRatio {
    Square,
    Screen,
    FourThree,
    ThreeFour,
    SixteenNine,
    NineSixteen,
    Zero,
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
            AspectRatio::Screen => {
                let screen = get_screen_size();
                screen.h as f32 / screen.w as f32
            }
            AspectRatio::FourThree => 3.0 / 4.0,
            AspectRatio::ThreeFour => 4.0 / 3.0,
            AspectRatio::SixteenNine => 9.0 / 16.0,
            AspectRatio::NineSixteen => 16.0 / 9.0,
            _ => 0.0,
        }
    }
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub enum ImageScaling {
    Preview(f32),
    FitScreen,
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
}
