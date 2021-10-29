use super::loader::load_new_pics;
use eframe::egui::DroppedFile;
use image::imageops::{replace, FilterType};
use image::{DynamicImage, GenericImageView, RgbaImage};
use rectangle_pack::{
    contains_smallest_box, pack_rects, volume_heuristic, GroupedRectsToPlace, RectToInsert,
    RectanglePackError, RectanglePackOk, TargetBin,
};
use std::collections::BTreeMap;

const UPSCALE: f32 = 1.5;
type PicId = usize;
type BinId = u8;

pub struct Pic {
    pub raw_image: DynamicImage,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub id: PicId,
}

pub struct Packer {
    pub pics: Vec<Pic>,
    pub width: u32,
    height: u32,
    area_min: u32,
    bin_id: BinId,
    bins: BTreeMap<BinId, TargetBin>,
    rects_to_place: GroupedRectsToPlace<PicId, BinId>,
    pic_placement: Result<RectanglePackOk<PicId, BinId>, RectanglePackError>,
    result: Option<RgbaImage>,
    pub preview: Option<RgbaImage>,
}

impl Packer {
    pub fn new() -> Self {
        Packer {
            pics: Vec::<Pic>::new(),
            width: 0,
            height: 0,
            area_min: 0,
            bin_id: 0,
            bins: BTreeMap::<BinId, TargetBin>::from([(0, TargetBin::new(0, 0, 0))]),
            rects_to_place: GroupedRectsToPlace::new(),
            pic_placement: Err(RectanglePackError::NotEnoughBinSpace),
            result: None,
            preview: None,
        }
    }

    pub fn update(&mut self, dropped_items: &[DroppedFile]) {
        self.add_pics(load_new_pics(dropped_items, self.pics.len()));
        let side = (self.area_min as f32).sqrt() as u32;
        self.pack(side);
        self.combine_pic();
    }

    fn bin(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.bins =
            BTreeMap::<BinId, TargetBin>::from([(self.bin_id, TargetBin::new(width, height, 1))]);
    }

    fn add_pics(&mut self, new_pics: Vec<Pic>) {
        for pic in new_pics {
            self.area_min += pic.width * pic.height;
            self.rects_to_place.push_rect(
                pic.id,
                Some(vec![self.bin_id]),
                RectToInsert::new(pic.width, pic.height, pic.depth),
            );
            self.pics.push(pic);
        }
    }

    fn pack(&mut self, side: u32) {
        self.bin(side, side);
        self.pic_placement = pack_rects(
            &self.rects_to_place,
            &mut self.bins,
            &volume_heuristic,
            &contains_smallest_box,
        );
        if self.pic_placement == Err(RectanglePackError::NotEnoughBinSpace) && self.width < u32::MAX
        {
            self.pack((side as f32 * UPSCALE) as u32);
        }
    }
    fn combine_pic(&mut self) {
        let mut combined = RgbaImage::new(self.width, self.height);
        println!(
            "-- Combined dims: {}x{}",
            combined.width(),
            combined.height()
        );
        if let Ok(packed) = &self.pic_placement {
            for pic in &self.pics {
                let loc = packed.packed_locations()[&pic.id].1;
                let (dx, dy) = (loc.x(), loc.y());
                println!("{:?} - {} {}", pic.id, loc.x(), loc.y());
                replace(&mut combined, &pic.raw_image, dx, dy);
            }
        }

        self.preview = Some(
            DynamicImage::ImageRgba8(combined.to_owned())
                .resize_exact(500, 500, FilterType::Nearest)
                .to_rgba8(),
        );
        self.result = Some(combined);
    }
}
