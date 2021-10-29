use super::loader::get_all_files;
use super::packer::*;
use eframe::{egui, epi};
use egui::*;
use epi::Storage;
use image::{DynamicImage, GenericImageView, Rgba, RgbaImage};
use std::path::PathBuf;

enum Aspect {
    Square,
    Screen,
    FourThree,
    Custom(u32, u32),
    None,
}
impl Default for Aspect {
    fn default() -> Aspect {
        Aspect::None
    }
}

// #[derive(Default)]
pub struct P3App {
    packer: Packer,
    preview: RgbaImage,
    texture: Option<(egui::Vec2, egui::TextureId)>,
    to_update: bool,
}

impl Default for P3App {
    fn default() -> P3App {
        P3App {
            packer: Packer::new(),
            preview: RgbaImage::new(300, 300),
            texture: None,
            to_update: true,
        }
    }
}

impl epi::App for P3App {
    fn name(&self) -> &str {
        "PickPicPack"
    }

    // fn setup(
    //     &mut self,
    //     ctx: &egui::CtxRef,
    //     frame: &mut epi::Frame<'_>,
    //     _storage: Option<&dyn Storage>,
    // ) {
    //     println!("{:?}", self.max_size_points());
    // }
    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        if self.to_update {
            if let Some(texture) = self.texture {
                frame.tex_allocator().free(texture.1);
            }
            let (size, pixels) = self.image_prepare();
            // Allocate a texture:
            let texture = frame
                .tex_allocator()
                .alloc_srgba_premultiplied(size, &pixels);
            let size = egui::Vec2::new(size.0 as f32, size.1 as f32);
            self.texture = Some((size, texture));
            self.to_update = false;
        }

        //DRAW GUI

        egui::CentralPanel::default()
            .frame(Frame::default())
            .show(ctx, |ui| {
                if let Some((size, texture)) = self.texture {
                    ui.image(texture, size);
                }
            });

        self.detect_files_being_dropped(ctx);
    }
}

impl P3App {
    fn image_prepare(&self) -> ((usize, usize), Vec<Color32>) {
        let image = &self.preview;
        // let image_buffer = image.to_rgba8();
        let size = (image.width() as usize, image.height() as usize);
        let pixels = image.clone().into_vec();
        assert_eq!(size.0 * size.1 * 4, pixels.len());
        let pixels: Vec<_> = pixels
            .chunks_exact(4)
            .map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
            .collect();
        (size, pixels)
    }
    fn detect_files_being_dropped(&mut self, ctx: &egui::CtxRef) {
        // Preview hovering files:
        if !ctx.input().raw.hovered_files.is_empty() {
            let text = "Dropping files!".to_owned();

            let painter =
                ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));

            let screen_rect = ctx.input().screen_rect();
            painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(192));
            painter.text(
                screen_rect.center(),
                Align2::CENTER_CENTER,
                text,
                TextStyle::Heading,
                Color32::WHITE,
            );
        }

        // Collect dropped files:
        if !ctx.input().raw.dropped_files.is_empty() {
            // self.packer
            // .add_pics(
            self.packer.update(&ctx.input().raw.dropped_files);
            println!("{}", self.packer.pics.len());
            println!("Side: {}", self.packer.width);
            self.preview = self.packer.preview.to_owned().unwrap();
            // self.preview = self
            //     .packer
            //     .pics_to_pack
            //     .last()
            //     .unwrap()
            //     .raw_image
            //     .clone()
            // .into_rgba8();
            self.to_update = true;
        }
    }
}
