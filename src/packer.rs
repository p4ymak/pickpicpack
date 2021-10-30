use super::loader::{load_new_items, Pic};
use crunch::{pack_into_po2, Item, PackedItems};
use eframe::egui::{Color32, DroppedFile};
use image::imageops::{replace, thumbnail};
use image::RgbaImage;

const SIDE: usize = 500;

pub struct Preview {
    pub size: (usize, usize),
    pub pixels: Vec<Color32>,
}

pub struct Packer {
    pub items: Vec<Vec<Item<Pic>>>,
    // width: u32,
    // height: u32,
    // area_min: u32,
    pic_placement: Result<(usize, usize, PackedItems<Pic>), ()>,
    _result: Option<RgbaImage>,
    pub preview: Option<Preview>,
}

impl Packer {
    pub fn new() -> Self {
        Packer {
            items: Vec::<Vec<Item<Pic>>>::new(),
            // width: 0,
            // height: 0,
            // area_min: 0,
            pic_placement: Err(()),
            _result: None,
            preview: None,
        }
    }

    pub fn update(&mut self, dropped_items: &[DroppedFile]) {
        if !dropped_items.is_empty() {
            let new_pics = load_new_items(dropped_items);
            self.add_items(new_pics);
        }
        // let side = (self.area_min as f32).sqrt() as u32;
        self.pack();
        self.combine_preview();
    }

    pub fn undo(&mut self) {
        if !self.items.is_empty() {
            self.items.pop();
            self.update(&[]);
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
    // fn pack(&mut self) {
    //     let mut area = 0;
    //     let mut rects_to_place = GroupedRectsToPlace::new();
    //     for pic in self.items.iter().flatten() {
    //         rects_to_place.push_rect(
    //             pic.id,
    //             Some(vec![BINID]),
    //             RectToInsert::new(pic.width, pic.height, pic.depth),
    //         );
    //         area += pic.width * pic.height;
    //     }
    //     let side = (area as f32).sqrt() as u32;
    //     println!("Trying to PACK!!");
    //     let (pic_placement, side) = try_pack(side, &rects_to_place);
    //     self.pic_placement = pic_placement;
    //     // self.width = side;
    //     // self.height = side;
    // }

    fn combine_preview(&mut self) {
        self.preview = None;

        if let Ok(packed) = &self.pic_placement {
            let mut max_dim = 0;
            for item in &packed.2 {
                max_dim = max_dim.max((item.0.w + item.0.x).max(item.0.h + item.0.y));
            }
            let div = SIDE as f32 / max_dim as f32;
            // let div = SIDE as f32 / packed.0 as f32;
            let mut combined = RgbaImage::new(SIDE as u32, SIDE as u32);
            for item in &packed.2 {
                if let Ok(image) = image::open(&item.1.file) {
                    let thumbnail = thumbnail(
                        &image,
                        (item.1.width as f32 * div) as u32,
                        (item.1.height as f32 * div) as u32,
                    );
                    let loc = item.0;
                    let (dx, dy) = ((loc.x as f32 * div) as u32, (loc.y as f32 * div) as u32);
                    // println!("{:?} - {} {}", pic.id, loc.x(), loc.y());
                    replace(&mut combined, &thumbnail, dx, dy);
                }
            }

            self.preview = Some(Preview {
                size: (SIDE, SIDE),
                pixels: combined
                    .pixels()
                    .map(|p| Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
                    .collect(),
            });
        }
    }

    // fn combine_pic(&mut self) {
    //     self.result = None;
    //     self.preview = None;
    //     let div = SIDE as f32 / self.width as f32;

    //     if let Ok(packed) = &self.pic_placement {
    //         if !self.pics.is_empty() {
    //             let mut combined = RgbaImage::new(self.width, self.height);
    //             for pic in (&self.pics).iter().flatten() {
    //                 if let Ok(image) = image::open(&pic.file) {
    //                     let loc = packed.packed_locations()[&pic.id].1;
    //                     let (dx, dy) = (loc.x(), loc.y());
    //                     // println!("{:?} - {} {}", pic.id, loc.x(), loc.y());
    //                     replace(&mut combined, &image, dx, dy);
    //                 }
    //             }

    //             self.preview = Some(Preview {
    //                 size: (SIDE, SIDE),
    //                 pixels: thumbnail(&combined, SIDE as u32, SIDE as u32)
    //                     .pixels()
    //                     .map(|p| Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
    //                     .collect(),
    //             });
    //             self.result = Some(combined);
    //         }
    //     }
    // }
}

// fn bin(width: u32, height: u32) -> Bin {
//     BTreeMap::<BinId, TargetBin>::from([(BINID, TargetBin::new(width, height, 1))])
// }
// fn try_pack(
//     side: u32,
//     rects_to_place: &GroupedRectsToPlace<PicId, BinId>,
// ) -> (
//     Result<RectanglePackOk<PicId, BinId>, RectanglePackError>,
//     u32,
// ) {
//     println!("BIN SIDE: {}", side);
//     let mut bin = bin(side, side);
//     let pic_placement = pack_rects(
//         rects_to_place,
//         &mut bin,
//         &volume_heuristic,
//         &contains_smallest_box,
//     );

//     if pic_placement == Err(RectanglePackError::NotEnoughBinSpace) && side < u32::MAX {
//         return try_pack((side as f32 * UPSCALE) as u32, rects_to_place);
//     }
//     (pic_placement, side)
// }
