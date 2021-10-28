use image::{DynamicImage, GenericImageView, Rgba, RgbaImage};
use rectangle_pack::{
    contains_smallest_box, pack_rects, volume_heuristic, GroupedRectsToPlace, RectToInsert,
    TargetBin,
};

struct PicPack {
    raw_image: DynamicImage,
    width: u32,
    height: u32,
    id: usize,
}
pub struct Packer {
    images_to_pack: Vec<PicPack>,
    preview: Option<RgbaImage>,
}
