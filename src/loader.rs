use eframe::egui::DroppedFile;
use image::GenericImageView;
// use imagesize::size;
use std::fs;
use std::path::{Path, PathBuf};

pub type PicId = usize;
pub struct Pic {
    // pub raw_image: DynamicImage,
    pub file: PathBuf,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub id: PicId,
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
        if let Ok(image) = image::open(&file) {
            id += 1;
            new_pics.push(Pic {
                file: file.to_owned(),
                width: image.width(),
                height: image.height(),
                depth: 1,
                id,
            });
        }
    }
    new_pics
}
