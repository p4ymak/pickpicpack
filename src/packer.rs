use image::{DynamicImage, GenericImageView, Rgba, RgbaImage};
use rectangle_pack::{
    contains_smallest_box, pack_rects, volume_heuristic, GroupedRectsToPlace, RectToInsert,
    RectanglePackError, RectanglePackOk, TargetBin,
};

use std::collections::BTreeMap;
type PicId = usize;
type BinId = u8;

pub struct Pic {
    raw_image: DynamicImage,
    width: u32,
    height: u32,
    depth: u32,
    id: PicId,
}

pub struct Packer {
    pics_to_pack: Vec<Pic>,
    width: u32,
    height: u32,
    bin_id: BinId,
    target_bins: BTreeMap<BinId, TargetBin>,
    rects_to_place: GroupedRectsToPlace<PicId, BinId>,
    pic_placement: Result<RectanglePackOk<PicId, BinId>, RectanglePackError>,
    result: Option<RgbaImage>,
    preview: Option<RgbaImage>,
}

impl Packer {
    pub fn new() -> Self {
        let mut bins = BTreeMap::new();
        bins.insert(1, TargetBin::new(1, 1, 0));
        Packer {
            pics_to_pack: Vec::<Pic>::new(),
            width: 1,
            height: 1,
            bin_id: 1,
            target_bins: bins,
            rects_to_place: GroupedRectsToPlace::new(),
            pic_placement: Err(RectanglePackError::NotEnoughBinSpace),
            result: None,
            preview: None,
        }
    }
    pub fn add_pics(&mut self, new_pics: Vec<Pic>) {
        for pic in new_pics {
            self.rects_to_place.push_rect(
                pic.id,
                Some(vec![self.bin_id]),
                RectToInsert::new(pic.width, pic.height, pic.depth),
            );
            self.pics_to_pack.push(pic);
        }
    }
    pub fn pack(&mut self) {
        self.pic_placement = pack_rects(
            &self.rects_to_place,
            &mut self.target_bins,
            &volume_heuristic,
            &contains_smallest_box,
        );
    }
}
