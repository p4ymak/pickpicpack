use super::loader::{load_new_items, Pic};
use super::utils::*;
use crunch::{pack, Item, PackedItems, Rect};
use eframe::egui::{Color32, DroppedFile};
use image::imageops::{replace, resize, thumbnail, FilterType};
use image::RgbaImage;
use std::path::Path;

const SCALE_MULTIPLIER: f32 = 1.1; //std::f32::consts::E / 2.0;

#[derive(Debug)]
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
    // pub preview_size: RectSize,
    // pub export_size: RectSize,
    pub preview_width: f32,
    pub aspect: AspectRatio,
    pub scale: ImageScaling,
    pub preview: Option<Preview>,
    pic_placement: Result<(usize, usize, PackedItems<Pic>), ()>,
}
impl Default for Packer {
    fn default() -> Self {
        Packer {
            items: Vec::<Vec<Item<Pic>>>::new(),
            // preview_size: RectSize::default(),
            // export_size: RectSize::default(),
            preview_width: f32::default(),
            aspect: AspectRatio::Square,
            scale: ImageScaling::default(),
            preview: None,
            pic_placement: Err(()),
        }
    }
}
impl Packer {
    pub fn new(preview_width: f32, aspect: AspectRatio) -> Self {
        Packer {
            preview_width,
            aspect,
            ..Default::default()
        }
    }

    pub fn update(&mut self, dropped_items: &[DroppedFile]) {
        if !dropped_items.is_empty() {
            let new_pics = load_new_items(dropped_items);
            self.add_items(new_pics);
        }
        // let side = (self.area_min as f32).sqrt() as u32;
        // self.preview_size =
        //     size_by_side_and_ratio(&ImageScaling::Preview(self.preview_width), &self.aspect);
        self.pack();
        self.preview();
    }

    pub fn undo(&mut self) {
        if !self.items.is_empty() {
            self.items.pop();
            self.update(&[]);
        }
        // if self.items.len() <= 1 {
        //     self.pic_placement = Err(());
        //     self.preview = None;
        // }
    }

    fn add_items(&mut self, new_items: Vec<Item<Pic>>) {
        // for item in &new_items {
        //     self.area_min += item.w * item.h;
        // }

        self.items.push(new_items);
    }
    // fn heuristic(&self) -> f32 {
    //     let mut perimeter = 0;
    //     for v in
    // }

    fn pack(&mut self) {
        if !self.items.is_empty() {
            let items_flat: Vec<Item<Pic>> = self.items.clone().into_iter().flatten().collect();
            let width = (items_flat.iter().map(|r| r.w * r.h).sum::<usize>() as f32).sqrt();
            // println!("HEURISTIC: {}", perimeter);
            // let width = (perimeter as f32).sqrt() * self.aspect.div();
            // let width = self.heuristic();
            self.pic_placement = pack_to_ratio(&items_flat, self.aspect.div(), width);
        }
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
            let crop = (max_w as f32)
                .max(max_h as f32 / self.aspect.div())
                .min(packed.0 as f32);
            // let size = RectSize::new(max_w, (max_w as f32 * self.aspect.div()) as usize);

            // let div = image_size.w as f32 / packed.0 as f32;
            let div = image_size.w as f32 / crop;
            let mut combined = RgbaImage::new(image_size.w as u32, image_size.h as u32);
            for item in &packed.2 {
                if let Ok(image) = image::open(&item.1.file) {
                    let thumbnail = match is_preview {
                        true => thumbnail(
                            &image,
                            (item.1.width as f32 * div) as u32,
                            (item.1.height as f32 * div) as u32,
                        ),
                        false => resize(
                            &image,
                            (item.1.width as f32 * div) as u32,
                            (item.1.height as f32 * div) as u32,
                            FilterType::CatmullRom,
                        ),
                    };
                    let loc = item.0;
                    let (dx, dy) = ((loc.x as f32 * div) as u32, (loc.y as f32 * div) as u32);
                    // println!("{:?} - {} {}", pic.id, loc.x(), loc.y());
                    replace(&mut combined, &thumbnail, dx, dy);
                }
            }
            return Some(combined);
        }
        None
    }
    fn preview(&mut self) {
        // self.preview = None;
        let size = size_by_side_and_ratio(&ImageScaling::Preview(self.preview_width), &self.aspect);
        if let Some(combined) = self.combine_image(CombineTask::Preview(size)) {
            self.preview = Some(Preview {
                size,
                pixels: combined
                    .pixels()
                    .map(|p| Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
                    .collect(),
            });
        } else {
            self.preview = Some(Preview {
                size,
                pixels: vec![Color32::TRANSPARENT; size.w * size.h],
            })
        }
    }
    pub fn export(&mut self, path: &Path) {
        let size = size_by_side_and_ratio(&self.scale, &self.aspect);
        if let Some(combined) = self.combine_image(CombineTask::Export(size)) {
            let result = combined.save(export_file_path(path, "png"));
            match result {
                Ok(_) => println!("Combined image saved!"),
                Err(_) => println!("Couldn't save image!"),
            }
        }
    }
}

fn pack_to_ratio(
    items: &[Item<Pic>],
    ratio: f32,
    width: f32,
) -> Result<(usize, usize, PackedItems<Pic>), ()> {
    let height = width * ratio;
    println!("TRY pack to {}:{}", width, height);
    let rect = Rect::of_size(width as usize, height as usize);
    let packed = pack(rect, items.to_owned());
    if let Ok(packed_items) = packed {
        Ok((width as usize, height as usize, packed_items))
    } else {
        pack_to_ratio(items, ratio, width * SCALE_MULTIPLIER)
    }
}
