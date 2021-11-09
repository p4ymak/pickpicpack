use chrono::Local;
use std::path::{Path, PathBuf};
use winit::event_loop::EventLoop;

pub const OUTPUT_NAME: &str = "PickPicPack";

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
        return RectSize {
            w: size.width as usize,
            h: size.height as usize,
        };
    };
    RectSize { w: 1280, h: 720 }
}

pub fn export_file_path(path: &Path, ext: &str) -> PathBuf {
    let time_stamp = Local::now().format("%Y%m%d_%H-%M-%S");
    Path::new(path).join(format!("{}_{}.{}", OUTPUT_NAME, time_stamp, ext))
}
#[derive(PartialEq, Debug)]
pub enum AspectRatio {
    Square,
    Screen,
    FourThree,
    ThreeFour,
    SixteenNine,
    NineSixteen,
}
impl Default for AspectRatio {
    fn default() -> AspectRatio {
        AspectRatio::Square
    }
}

#[derive(PartialEq, Debug)]
pub enum ImageScaling {
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
