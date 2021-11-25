use super::loader::{load_new_items, Pic};
use super::utils::*;
use crunch::{pack, Item, PackedItems, Rect};
use eframe::egui::{Color32, DroppedFile};
use image::imageops::{replace, resize, thumbnail, FilterType};
use image::RgbaImage;
use std::path::{Path, PathBuf};

// #[derive(Debug)]

// fn to_vec(&self) -> Vec<Color32> {
//     self
//         .pixels()
//         .map(|p| Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
//         .collect()

// enum CombineTask {
//     Layout(RectSize),
//     Preview(RectSize, usize),
//     Export(RectSize),
// }

pub struct Packer {
    pub items: Vec<Vec<Item<Pic>>>,
    pub preview_width: f32,
    pub aspect: AspectRatio,
    pub scale: ImageScaling,
    pub preview: RgbaImage,
    pub actual_size: RectSize,
    pic_placement: Result<(usize, usize, PackedItems<Pic>), ()>,
}
impl Default for Packer {
    fn default() -> Self {
        Packer {
            items: Vec::<Vec<Item<Pic>>>::new(),
            preview_width: f32::default(),
            aspect: AspectRatio::Square,
            scale: ImageScaling::default(),
            preview: RgbaImage::new(1, 1),
            actual_size: RectSize::default(),
            pic_placement: Err(()),
        }
    }
}
impl Packer {
    pub fn new(preview_width: f32, aspect: AspectRatio, scale: ImageScaling) -> Self {
        Packer {
            preview_width,
            aspect,
            scale,
            ..Default::default()
        }
    }

    pub fn update(&mut self, dropped_items: &[DroppedFile]) -> usize {
        if !dropped_items.is_empty() {
            let new_pics = load_new_items(dropped_items);
            self.add_items(new_pics);
        }
        let num = self.pack();
        self.combine_thumbnails(0);
        num
    }

    pub fn undo(&mut self) -> usize {
        if !self.items.is_empty() {
            self.items.pop();
            return self.update(&[]);
        }
        0
    }

    fn add_items(&mut self, new_items: Vec<Item<Pic>>) {
        self.items.push(new_items);
    }

    fn pack(&mut self) -> usize {
        if !self.items.is_empty() {
            let items_flat: Vec<Item<Pic>> = self.items.clone().into_iter().flatten().collect();

            let width = (items_flat.iter().map(|r| r.w * r.h).sum::<usize>() as f32
                / self.aspect.div())
            .sqrt();
            self.pic_placement = pack_to_ratio(&items_flat, self.aspect.div(), width, width, 1);
            return items_flat.len();
        }
        0
    }

    pub fn combine_thumbnails(&mut self, loaded: isize) {
        let image_size = match loaded {
            0 => RectSize::by_scale_and_ratio(
                &ImageScaling::Preview(self.preview_width),
                &self.aspect,
            ),
            _ => RectSize {
                w: self.preview.width() as usize,
                h: self.preview.height() as usize,
            },
        };

        if let Ok(packed) = &self.pic_placement {
            let mut max_w = 0;
            let mut max_h = 0;
            for item in &packed.2 {
                max_w = max_w.max(item.0.w + item.0.x);
                max_h = max_h.max(item.0.h + item.0.y);
            }
            let crop = (max_w as f32)
                .max(max_h as f32 / self.aspect.div())
                .min(packed.0 as f32);
            let div = image_size.w as f32 / crop;

            self.actual_size = RectSize::new(max_w, max_h);

            if loaded == 0 {
                //Create Layout Preview
                self.preview = RgbaImage::new(image_size.w as u32, image_size.h as u32);
                for item in &packed.2 {
                    let color_box = RgbaImage::from_pixel(
                        (item.1.width as f32 * div).floor() as u32,
                        (item.1.height as f32 * div).floor() as u32,
                        item.1.color,
                    );
                    let loc = item.0;
                    let (dx, dy) = (
                        (loc.x as f32 * div).floor() as u32,
                        (loc.y as f32 * div).floor() as u32,
                    );
                    replace(&mut self.preview, &color_box, dx, dy);
                }
            } else {
                //Update Layout Preview with new loaded image
                let mut id = 0;
                for item in &packed.2 {
                    id += 1;
                    if id == loaded {
                        if let Ok(image) = image::open(&item.1.file) {
                            let thumbnail = thumbnail(
                                &image,
                                (item.1.width as f32 * div).floor() as u32,
                                (item.1.height as f32 * div).floor() as u32,
                            );
                            let loc = item.0;
                            let (dx, dy) = (
                                (loc.x as f32 * div).floor() as u32,
                                (loc.y as f32 * div).floor() as u32,
                            );
                            replace(&mut self.preview, &thumbnail, dx, dy);
                        }
                    }
                }
            }
        } else {
            self.preview = RgbaImage::new(image_size.w as u32, image_size.h as u32);
        }
    }

    pub fn export(&mut self, path: &Path, to_zip: bool) {
        todo!();
    }
}

//fn combine_image(&mut self, task: CombineTask) -> Option<RgbaImage> {
//    if let Ok(packed) = &self.pic_placement {
//        let mut max_w = 0;
//        let mut max_h = 0;
//        for item in &packed.2 {
//            max_w = max_w.max(item.0.w + item.0.x);
//            max_h = max_h.max(item.0.h + item.0.y);
//        }
//        let mut image_size = match task {
//            CombineTask::Layout(rect) => rect,
//            CombineTask::Preview(rect, _) => rect,
//            CombineTask::Export(rect) => rect,
//        };
//        let crop = (max_w as f32)
//            .max(max_h as f32 / self.aspect.div())
//            .min(packed.0 as f32);
//        let mut div = image_size.w as f32 / crop;

//        //In case of pixel perfect big picture
//        if let CombineTask::Export(_) = task {
//            if self.scale == ImageScaling::Actual {
//                div = 1.0;
//                image_size = RectSize::new(max_w, max_h);
//            }
//        }
//        self.actual_size = RectSize::new(max_w, max_h);
//        let mut combined = RgbaImage::new(image_size.w as u32, image_size.h as u32);

//        if let CombineTask::Layout(_) = task {
//            for item in &packed.2 {
//                let color_box = RgbaImage::from_pixel(
//                    (item.1.width as f32 * div).floor() as u32,
//                    (item.1.height as f32 * div).floor() as u32,
//                    item.1.color,
//                );
//                let loc = item.0;
//                let (dx, dy) = (
//                    (loc.x as f32 * div).floor() as u32,
//                    (loc.y as f32 * div).floor() as u32,
//                );
//                replace(&mut combined, &color_box, dx, dy);
//            }
//        } else {
//            for item in &packed.2 {
//                if let Ok(image) = image::open(&item.1.file) {
//                    let thumbnail = match task {
//                        CombineTask::Preview(_, _) | CombineTask::Layout(_) => thumbnail(
//                            &image,
//                            (item.1.width as f32 * div).floor() as u32,
//                            (item.1.height as f32 * div).floor() as u32,
//                        ),
//                        CombineTask::Export(_) => resize(
//                            &image,
//                            (item.1.width as f32 * div).floor() as u32,
//                            (item.1.height as f32 * div).floor() as u32,
//                            FilterType::CatmullRom,
//                        ),
//                    };
//                    let loc = item.0;
//                    let (dx, dy) = (
//                        (loc.x as f32 * div).floor() as u32,
//                        (loc.y as f32 * div).floor() as u32,
//                    );
//                    replace(&mut combined, &thumbnail, dx, dy);
//                }
//            }
//        }

//    }
//}

// pub fn preview(&mut self, loaded: usize) {
//     let size =
//         RectSize::by_scale_and_ratio(&ImageScaling::Preview(self.preview_width), &self.aspect);

//     let task = match loaded {
//         0 => CombineTask::Layout(size),
//         _ => CombineTask::Preview(size, loaded - 1),
//     };
//     self.combine_image(task).to_vec()
// }

// pub fn export(&mut self, path: &Path, to_zip: bool) {
//     let size = RectSize::by_scale_and_ratio(&self.scale, &self.aspect);
//     let file_name = file_timestamp();
//     if let Some(combined) = self.combine_image(CombineTask::Export(size)) {
//         let img_result =
//             combined.save(Path::new(path).join(format!("{}.{}", file_name, "png")));
//         match img_result {
//             Ok(_) => println!("Combined image saved!"),
//             Err(err) => println!("Couldn't save image!\n{}", err),
//         }
//         if to_zip {
//             let files: Vec<&PathBuf> = self
//                 .items
//                 .iter()
//                 .flatten()
//                 .map(|item| &item.data.file)
//                 .collect();
//             let zip_result = archive_files(
//                 files,
//                 Path::new(path).join(format!("{}.{}", file_name, "zip")),
//             );
//             match zip_result {
//                 Ok(_) => println!("Zip archive saved!"),
//                 Err(err) => println!("Couldn't save archive!\n{}", err),
//             }
//         }
//     }
// }
// }

fn pack_to_ratio(
    items: &[Item<Pic>],
    ratio: f32,
    width: f32,
    original_width: f32,
    step: usize,
) -> Result<(usize, usize, PackedItems<Pic>), ()> {
    let height = width * ratio;
    let rect = Rect::of_size(width as usize, height as usize);
    let packed = pack(rect, items.to_owned());
    if let Ok(packed_items) = packed {
        Ok((width as usize, height as usize, packed_items))
    } else {
        pack_to_ratio(
            items,
            ratio,
            original_width + original_width * step as f32 * 0.01, // add 1% per step
            original_width,
            step + 1,
        )
    }
}
