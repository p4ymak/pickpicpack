use super::utils::random_gray;
use crunch::{Item, Rotation};
use eframe::egui::DroppedFile;
use image::{io::Reader, ImageResult};
use std::fs;
use std::path::{Path, PathBuf};
#[derive(Clone)]
pub struct Pic {
    pub file: PathBuf,
    pub width: u32,
    pub height: u32,
    pub color: image::Rgba<u8>,
    pub avg_width: u32,
    pub avg_height: u32,
}

fn get_dimensions(path: &Path) -> ImageResult<(u32, u32)> {
    Reader::open(path)?.with_guessed_format()?.into_dimensions()
}
fn equal_fit_dimensions(width: u32, height: u32) -> (u32, u32) {
    let fit = 512.0;
    let ratio = height as f32 / width as f32;
    let (avg_width, avg_height) = match ratio <= 1.0 {
        true => (fit, (fit * ratio)),
        false => ((fit / ratio), fit),
    };
    (avg_width as u32, avg_height as u32)
}

pub fn get_all_files(path: &Path) -> Vec<PathBuf> {
    let mut result = Vec::<PathBuf>::new();
    if let Ok(metadata) = fs::metadata(&path) {
        if metadata.is_file() {
            result.push(PathBuf::from(path));
        } else if metadata.is_dir() {
            if let Ok(dir_content) = fs::read_dir(path) {
                for entry in dir_content.flatten() {
                    result.extend(get_all_files(&entry.path()));
                }
            }
        }
    }
    result
}

pub fn load_new_items(dropped_items: &[DroppedFile]) -> Vec<Item<Pic>> {
    let mut all_files = Vec::<PathBuf>::new();
    let mut new_items = Vec::<Item<Pic>>::new();
    for dropped in dropped_items {
        if let Some(path) = &dropped.path {
            all_files.extend(get_all_files(path))
        }
    }
    for file in all_files {
        if let Ok(dimensions) = get_dimensions(&file) {
            if dimensions.0 > 0 && dimensions.1 > 0 {
                let (avg_width, avg_height) = equal_fit_dimensions(dimensions.0, dimensions.1);
                new_items.push(Item::new(
                    Pic {
                        file: file.to_owned(),
                        width: dimensions.0,
                        height: dimensions.1,
                        color: random_gray(),
                        avg_width,
                        avg_height,
                    },
                    dimensions.0 as usize,
                    dimensions.1 as usize,
                    Rotation::None,
                ));
            }
        }
    }
    new_items
}
