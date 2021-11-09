use super::loader::{load_new_items, Pic};
use super::utils::*;
use crunch::{pack_into_po2, Item, PackedItems};
use eframe::egui::{Color32, DroppedFile};
use image::imageops::{replace, resize, thumbnail, FilterType};
use image::RgbaImage;
use std::path::Path;

pub struct Preview {
    pub size: RectSize,
    pub pixels: Vec<Color32>,
}

enum CombineTask {
    Preview(RectSize),
    Export(RectSize),
}

pub struct Packer {
    pub items: Vec<Vec<Item<Pic>>>,
    pub preview_size: RectSize,
    pub export_size: RectSize,
    pub ratio: f32,
    pub preview: Option<Preview>,
    pic_placement: Result<(usize, usize, PackedItems<Pic>), ()>,
}
impl Default for Packer {
    fn default() -> Self {
        Packer {
            items: Vec::<Vec<Item<Pic>>>::new(),
            preview_size: RectSize::default(),
            export_size: RectSize::default(),
            ratio: f32::default(),
            preview: None,
            pic_placement: Err(()),
        }
    }
}
impl Packer {
    pub fn new(preview_size: RectSize, export_size: RectSize, ratio: f32) -> Self {
        Packer {
            items: Vec::<Vec<Item<Pic>>>::new(),
            preview_size,
            export_size,
            ratio,
            pic_placement: Err(()),
            preview: None,
        }
    }

    pub fn update(&mut self, dropped_items: &[DroppedFile], preview_size: RectSize) {
        self.preview_size = preview_size;
        if !dropped_items.is_empty() {
            let new_pics = load_new_items(dropped_items);
            self.add_items(new_pics);
        }
        // let side = (self.area_min as f32).sqrt() as u32;
        self.pack();
        self.preview();
    }

    pub fn undo(&mut self) {
        if !self.items.is_empty() {
            self.items.pop();
            self.update(&[], self.preview_size);
        }
    }

    fn add_items(&mut self, new_items: Vec<Item<Pic>>) {
        // for item in &new_items {
        //     self.area_min += item.w * item.h;
        // }

        self.items.push(new_items);
    }
    fn pack(&mut self) {
        let items_flat = self.items.clone().into_iter().flatten();
        self.pic_placement = pack_into_po2(usize::MAX, items_flat);
    }

    fn combine_image(&mut self, task: CombineTask) -> Option<RgbaImage> {
        if let Ok(packed) = &self.pic_placement {
            let mut max_w = 0;
            let mut max_h = 0;
            for item in &packed.2 {
                max_w = max_w.max(item.0.w + item.0.x);
                max_h = max_h.max(item.0.h + item.0.y);
            }
            let (is_preview, image_size) = match task {
                CombineTask::Preview(rect) => (true, rect),
                CombineTask::Export(rect) => (false, rect),
            };
            let div_x = image_size.w as f32 / max_w as f32;
            let div_y = image_size.w as f32 / max_w as f32;

            let mut combined = RgbaImage::new(image_size.w as u32, image_size.h as u32);
            for item in &packed.2 {
                if let Ok(image) = image::open(&item.1.file) {
                    let thumbnail = match is_preview {
                        true => thumbnail(
                            &image,
                            (item.1.width as f32 * div_x) as u32,
                            (item.1.height as f32 * div_y) as u32,
                        ),
                        false => resize(
                            &image,
                            (item.1.width as f32 * div_x) as u32,
                            (item.1.height as f32 * div_y) as u32,
                            FilterType::CatmullRom,
                        ),
                    };
                    let loc = item.0;
                    let (dx, dy) = ((loc.x as f32 * div_x) as u32, (loc.y as f32 * div_y) as u32);
                    // println!("{:?} - {} {}", pic.id, loc.x(), loc.y());
                    replace(&mut combined, &thumbnail, dx, dy);
                }
            }
            return Some(combined);
        }
        None
    }
    fn preview(&mut self) {
        self.preview = None;
        if let Some(combined) = self.combine_image(CombineTask::Preview(self.preview_size)) {
            self.preview = Some(Preview {
                size: self.preview_size,
                pixels: combined
                    .pixels()
                    .map(|p| Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
                    .collect(),
            });
        }
    }
    pub fn export(&mut self, path: &Path) {
        println!("{:?}", self.export_size);
        if let Some(combined) = self.combine_image(CombineTask::Export(self.export_size)) {
            let result = combined.save(export_file_path(path, "png"));
            match result {
                Ok(_) => println!("Combined image saved!"),
                Err(_) => println!("Couldn't save image!"),
            }
        }
    }
}
