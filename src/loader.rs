// extern crate fs_extra;
use super::packer::Pic;
use eframe::egui::DroppedFile;
// use fs_extra::dir;
// use size_format::SizeFormatterSI;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn get_all_files(path: &Path) -> Vec<PathBuf> {
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
pub fn load_all_images(dropped_items: &[DroppedFile]) {
    let mut all_files = Vec::<PathBuf>::new();
    for dropped in dropped_items {
        if let Some(path) = &dropped.path {
            all_files.extend(get_all_files(path))
        }
    }
    for file in all_files {
        println!("{:?}", file);
    }
}
//fn walkdir(cur_dir: &str) -> Vec<Path> {
//    let mut found_file = false;
//    let mut links = Vec::<String>::new();
//    for entry in fs::read_dir(cur_dir).unwrap() {
//        let entry = entry.unwrap();
//        let path = entry.path();
//        let metadata = fs::metadata(&path).unwrap();

//        if metadata.is_file() {
//            let file_name = path
//                .file_name()
//                .unwrap()
//                .to_string_lossy()
//                .to_string()
//                .to_lowercase();
//            if file_name.ends_with(".gitignore") {
//                found_file = true;
//            } else {
//                for (i, ftype) in ftypes.iter().enumerate() {
//                    if file_name.ends_with(ftype) {
//                        //println!("Found {:?}", file_name);
//                        counter[i] += 1;
//                        if !found_file {
//                            collected_dirs.push(cur_dir.to_string());
//                            *size_total += dir::get_size(cur_dir).unwrap();
//                        }
//                        found_file = true;
//                    }
//                }
//            }
//        } else if metadata.is_dir() {
//            let path_name = path.to_string_lossy().to_string();
//            links.push(path_name);
//        }
//    }
//    if !found_file {
//        for link in links {
//            walkdir(
//                &link,
//                &mut collected_dirs,
//                &ftypes,
//                &mut counter,
//                &mut size_total,
//            );
//        }
//    }
//    todo!();
//}
