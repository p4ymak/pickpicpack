use super::packer::Pic;
use eframe::egui::DroppedFile;
use image::{DynamicImage, GenericImageView, Rgba, RgbaImage};
use std::fs;
use std::path::{Path, PathBuf};

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

pub fn load_new_pics(dropped_items: &[DroppedFile], last_id: usize) -> Vec<Pic> {
    let mut all_files = Vec::<PathBuf>::new();
    let mut new_pics = Vec::<Pic>::new();
    for dropped in dropped_items {
        if let Some(path) = &dropped.path {
            all_files.extend(get_all_files(path))
        }
    }
    let mut id = last_id;
    for file in all_files {
        if let Ok(image) = image::open(file) {
            id += 1;
            new_pics.push(Pic {
                width: image.width(),
                height: image.height(),
                raw_image: image,
                depth: 1,
                id,
            });
        }
    }
    new_pics
}
