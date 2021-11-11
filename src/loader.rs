use crunch::{Item, Rotation};
use eframe::egui::DroppedFile;
// use image::GenericImageView;
use imagesize::size;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct Pic {
    // pub raw_image: DynamicImage,
    pub file: PathBuf,
    pub width: u32,
    pub height: u32,
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
        if let Ok(dimensions) = size(&file) {
            new_items.push(Item::new(
                Pic {
                    file: file.to_owned(),
                    width: dimensions.width as u32,
                    height: dimensions.height as u32,
                },
                dimensions.width,
                dimensions.height,
                Rotation::None,
            ));
        }
    }
    new_items
}
