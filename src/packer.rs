use super::loader::{load_new_pics, Pic, PicId};
use eframe::egui::{Color32, DroppedFile};
use image::imageops::{replace, thumbnail};
use image::RgbaImage;
use rectangle_pack::{
    contains_smallest_box, pack_rects, volume_heuristic, GroupedRectsToPlace, RectToInsert,
    RectanglePackError, RectanglePackOk, TargetBin,
};
use std::collections::BTreeMap;

const UPSCALE: f32 = 1.5;
const SIDE: usize = 500;

type BinId = u8;
const BINID: BinId = 0;
type Bin = BTreeMap<BinId, TargetBin>;

pub struct Preview {
    pub size: (usize, usize),
    pub pixels: Vec<Color32>,
}

pub struct Packer {
    pub pics: Vec<Vec<Pic>>,
    width: u32,
    height: u32,
    area_min: u32,
    last_id: usize,
    pic_placement: Result<RectanglePackOk<PicId, BinId>, RectanglePackError>,
    result: Option<RgbaImage>,
    pub preview: Option<Preview>,
}

impl Packer {
    pub fn new() -> Self {
        Packer {
            pics: Vec::<Vec<Pic>>::new(),
            width: 0,
            height: 0,
            area_min: 0,
            last_id: 1,
            pic_placement: Err(RectanglePackError::NotEnoughBinSpace),
            result: None,
            preview: None,
        }
    }

    pub fn update(&mut self, dropped_items: &[DroppedFile]) {
        if !dropped_items.is_empty() {
            let new_pics = load_new_pics(dropped_items, self.last_id);
            self.last_id += new_pics.len();
            self.add_pics(new_pics);
        }
        // let side = (self.area_min as f32).sqrt() as u32;
        self.pack();
        self.combine_pic();
    }

    pub fn undo(&mut self) {
        if !self.pics.is_empty() {
            self.last_id -= self.pics.last().unwrap().len();
            self.pics.pop();
            self.update(&[]);
        }
    }

    fn add_pics(&mut self, new_pics: Vec<Pic>) {
        for pic in &new_pics {
            self.area_min += pic.width * pic.height;
        }

        self.pics.push(new_pics);
    }

    fn pack(&mut self) {
        let mut area = 0;
        let mut rects_to_place = GroupedRectsToPlace::new();
        for pic in self.pics.iter().flatten() {
            rects_to_place.push_rect(
                pic.id,
                Some(vec![BINID]),
                RectToInsert::new(pic.width, pic.height, pic.depth),
            );
            area += pic.width * pic.height;
        }
        let side = (area as f32).sqrt() as u32;
        println!("Trying to PACK!!");
        let (pic_placement, side) = try_pack(side, &rects_to_place);
        self.pic_placement = pic_placement;
        self.width = side;
        self.height = side;
    }

    fn combine_pic(&mut self) {
        self.result = None;
        self.preview = None;
        if let Ok(packed) = &self.pic_placement {
            if !self.pics.is_empty() {
                let mut combined = RgbaImage::new(self.width, self.height);
                for pic in (&self.pics).iter().flatten() {
                    if let Ok(image) = image::open(&pic.file) {
                        let loc = packed.packed_locations()[&pic.id].1;
                        let (dx, dy) = (loc.x(), loc.y());
                        // println!("{:?} - {} {}", pic.id, loc.x(), loc.y());
                        replace(&mut combined, &image, dx, dy);
                    }
                }

                self.preview = Some(Preview {
                    size: (SIDE, SIDE),
                    pixels: thumbnail(&combined, SIDE as u32, SIDE as u32)
                        .pixels()
                        .map(|p| Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
                        .collect(),
                });
                self.result = Some(combined);
            }
        }
    }
}

fn bin(width: u32, height: u32) -> Bin {
    BTreeMap::<BinId, TargetBin>::from([(BINID, TargetBin::new(width, height, 1))])
}
fn try_pack(
    side: u32,
    rects_to_place: &GroupedRectsToPlace<PicId, BinId>,
) -> (
    Result<RectanglePackOk<PicId, BinId>, RectanglePackError>,
    u32,
) {
    println!("BIN SIDE: {}", side);
    let mut bin = bin(side, side);
    let pic_placement = pack_rects(
        rects_to_place,
        &mut bin,
        &volume_heuristic,
        &contains_smallest_box,
    );

    if pic_placement == Err(RectanglePackError::NotEnoughBinSpace) && side < u32::MAX {
        return try_pack((side as f32 * UPSCALE) as u32, rects_to_place);
    }
    (pic_placement, side)
}
