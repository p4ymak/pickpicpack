use super::loader::{load_new_items, Pic};
use super::utils::*;
use crunch::{pack, Item, PackedItems, Rect, Rotation};
use eframe::egui::DroppedFile;
use image::imageops::{replace, resize, thumbnail, FilterType};
use image::{DynamicImage, ImageResult, RgbaImage};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

struct PackingResult {
    total_w: usize,
    max_w: usize,
    max_h: usize,
    positions: Vec<(Rect, Pic)>,
}

pub struct Packer {
    pub items: Vec<Vec<Item<Pic>>>,
    pub preview_width: f32,
    pub aspect: AspectRatio,
    pub equal: bool,
    pub margin: usize,
    pub scale: ImageScaling,
    pub preview: RgbaImage,
    pub actual_size: RectSize,
    pub cached: bool,
    // pub bg_color: Color32,
    packing_result: Option<PackingResult>,
    cache: HashMap<PathBuf, ImageResult<DynamicImage>>,
}
impl Default for Packer {
    fn default() -> Self {
        Packer {
            items: Vec::<Vec<Item<Pic>>>::new(),
            preview_width: f32::default(),
            aspect: AspectRatio::Square,
            equal: false,
            margin: 0,
            scale: ImageScaling::default(),
            preview: RgbaImage::new(1, 1),
            actual_size: RectSize::default(),
            // bg_color: Color32::TRANSPARENT,
            packing_result: None,
            cached: false,
            cache: HashMap::<PathBuf, ImageResult<DynamicImage>>::new(),
        }
    }
}
impl Packer {
    pub fn new(preview_width: f32, aspect: AspectRatio, scale: ImageScaling, equal: bool) -> Self {
        Packer {
            preview_width,
            aspect,
            scale,
            equal,
            ..Default::default()
        }
    }

    pub fn update(&mut self, dropped_items: &[DroppedFile]) -> usize {
        if !dropped_items.is_empty() {
            let new_pics = load_new_items(dropped_items);
            if !new_pics.is_empty() {
                self.add_items(new_pics);
            }
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
            let mean_max_dim = match self.equal {
                true => {
                    self.items
                        .iter()
                        .flatten()
                        .map(|item| item.data.width.max(item.data.height))
                        .sum::<u32>()
                        / (self.items.iter().map(|v| v.len()).sum::<usize>() as u32).max(1)
                }
                false => 0,
            };
            let items_flat: Vec<Item<Pic>> = match self.equal {
                true => self
                    .items
                    .clone()
                    .into_iter()
                    .flatten()
                    .map(|item| {
                        let new_dims =
                            fit_to_square(item.data.width, item.data.height, mean_max_dim);
                        Item::new(
                            Pic {
                                file: item.data.file,
                                width: new_dims.0,
                                height: new_dims.1,
                                color: item.data.color,
                            },
                            new_dims.0 as usize + self.margin,
                            new_dims.1 as usize + self.margin,
                            Rotation::None,
                        )
                    })
                    .collect(),
                false => self
                    .items
                    .clone()
                    .into_iter()
                    .flatten()
                    .map(|item| {
                        Item::new(
                            Pic {
                                file: item.data.file,
                                width: item.data.width,
                                height: item.data.height,
                                color: item.data.color,
                            },
                            item.data.width as usize + self.margin,
                            item.data.height as usize + self.margin,
                            Rotation::None,
                        )
                    })
                    .collect(),
            };

            let width = (items_flat.iter().map(|r| r.w * r.h).sum::<usize>() as f32
                / self.aspect.div())
            .sqrt();
            let pic_placement = pack_to_ratio(&items_flat, self.aspect.div(), width, width, 1);

            if let Ok(packed) = pic_placement {
                let total_w = packed.0;
                let mut max_w = 0;
                let mut max_h = 0;
                let mut positions = Vec::<(Rect, Pic)>::new();
                for item in packed.2 {
                    max_w = max_w.max(item.0.w + item.0.x);
                    max_h = max_h.max(item.0.h + item.0.y);
                    positions.push(item);
                }
                self.packing_result = Some(PackingResult {
                    total_w,
                    max_w,
                    max_h,
                    positions,
                });
            } else {
                self.packing_result = None
            }

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

        if let Some(packed) = &self.packing_result {
            let crop = (packed.max_w as f32)
                .max(packed.max_h as f32 / self.aspect.div())
                .min(packed.total_w as f32);
            let div = (image_size.w) as f32 / crop;

            self.actual_size = RectSize::new(packed.max_w, packed.max_h);

            if loaded == 0 {
                //Create Layout Preview
                self.preview = RgbaImage::new(image_size.w as u32, image_size.h as u32);
                for item in &packed.positions {
                    let color_box = RgbaImage::from_pixel(
                        (item.1.width as f32 * div).floor() as u32,
                        (item.1.height as f32 * div).floor() as u32,
                        item.1.color,
                    );
                    let loc = item.0;
                    let (dx, dy) = (
                        ((loc.x + self.margin / 2) as f32 * div).floor() as u32,
                        ((loc.y + self.margin / 2) as f32 * div).floor() as u32,
                    );
                    replace(&mut self.preview, &color_box, dx, dy);
                }
            } else {
                //Update Layout Preview with new loaded image
                if let Some(item) = &packed.positions.get((loaded - 1) as usize) {
                    let thumbnail = match self.cached {
                        true => {
                            if !self.cache.contains_key(&item.1.file) {
                                self.cache
                                    .insert(item.1.file.clone(), image::open(&item.1.file));
                            }
                            let stored = self.cache.get(&item.1.file).unwrap();
                            if let Ok(image) = stored {
                                Some(thumbnail(
                                    image,
                                    (item.1.width as f32 * div).floor() as u32,
                                    (item.1.height as f32 * div).floor() as u32,
                                ))
                            } else {
                                None
                            }
                        }
                        false => {
                            let loaded = image::open(&item.1.file);
                            if let Ok(image) = loaded {
                                Some(thumbnail(
                                    &image,
                                    (item.1.width as f32 * div).floor() as u32,
                                    (item.1.height as f32 * div).floor() as u32,
                                ))
                            } else {
                                None
                            }
                        }
                    };
                    if let Some(thumbnail) = thumbnail {
                        let loc = item.0;
                        let (dx, dy) = (
                            ((loc.x + self.margin / 2) as f32 * div).floor() as u32,
                            ((loc.y + self.margin / 2) as f32 * div).floor() as u32,
                        );
                        replace(&mut self.preview, &thumbnail, dx, dy);
                    }
                }
            }
        } else {
            self.preview = RgbaImage::new(image_size.w as u32, image_size.h as u32);
        }
    }

    fn combine_image(&mut self) -> Option<RgbaImage> {
        if let Some(packed) = &self.packing_result {
            let max_w = packed.max_w;
            let max_h = packed.max_h;
            let image_size = match self.scale {
                ImageScaling::Actual => RectSize::new(max_w, max_h),
                ImageScaling::Preview(_) => RectSize::by_scale_and_ratio(
                    &ImageScaling::Preview(self.preview_width),
                    &self.aspect,
                ),
                scale => RectSize::by_scale_and_ratio(&scale, &self.aspect),
            };

            let crop = (packed.max_w as f32)
                .max(packed.max_h as f32 / self.aspect.div())
                .min(packed.total_w as f32);
            let div = match self.scale {
                ImageScaling::Actual => 1.0,
                _ => (image_size.w) as f32 / crop,
            };

            self.actual_size = RectSize::new(max_w, max_h);
            let mut combined = RgbaImage::new(image_size.w as u32, image_size.h as u32);

            for item in &packed.positions {
                if let Ok(image) = image::open(&item.1.file) {
                    let thumbnail = resize(
                        &image,
                        (item.1.width as f32 * div).floor() as u32,
                        (item.1.height as f32 * div).floor() as u32,
                        FilterType::CatmullRom,
                    );

                    let loc = item.0;
                    let (dx, dy) = (
                        (loc.x as f32 * div).floor() as u32,
                        (loc.y as f32 * div).floor() as u32,
                    );
                    replace(&mut combined, &thumbnail, dx, dy);
                }
            }
            return Some(combined);
        }
        None
    }

    pub fn export(&mut self, path: &Path, to_zip: bool) {
        let file_name = file_timestamp();
        if let Some(combined) = self.combine_image() {
            let img_result =
                combined.save(Path::new(path).join(format!("{}.{}", file_name, "png")));
            match img_result {
                Ok(_) => println!("Combined image saved!"),
                Err(err) => println!("Couldn't save image!\n{}", err),
            }
            if to_zip {
                let files: Vec<&PathBuf> = self
                    .items
                    .iter()
                    .flatten()
                    .map(|item| &item.data.file)
                    .collect();
                let zip_result = archive_files(
                    files,
                    Path::new(path).join(format!("{}.{}", file_name, "zip")),
                );
                match zip_result {
                    Ok(_) => println!("Zip archive saved!"),
                    Err(err) => println!("Couldn't save archive!\n{}", err),
                }
            }
        }
    }
}

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
