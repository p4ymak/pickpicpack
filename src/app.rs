use super::packer::*;
use eframe::{egui, epi};
use egui::*;
// use epi::Storage;

// enum Aspect {
//     Square,
//     Screen,
//     FourThree,
//     Custom(u32, u32),
//     None,
// }
// impl Default for Aspect {
//     fn default() -> Aspect {
//         Aspect::None
//     }
// }

// #[derive(Default)]
pub struct P3App {
    packer: Packer,
    texture: Option<(egui::Vec2, egui::TextureId)>,
    to_update: bool,
}

impl Default for P3App {
    fn default() -> P3App {
        P3App {
            packer: Packer::new(),
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
            if let Some(preview) = &self.packer.preview {
                // Allocate a texture:
                let texture = frame
                    .tex_allocator()
                    .alloc_srgba_premultiplied(preview.size, &preview.pixels);
                let size = egui::Vec2::new(preview.size.0 as f32, preview.size.1 as f32);
                self.texture = Some((size, texture));
            }

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
        self.handle_keys(ctx);
    }
}

impl P3App {
    fn detect_files_being_dropped(&mut self, ctx: &egui::CtxRef) {
        // Preview hovering files:
        // if !ctx.input().raw.hovered_files.is_empty() {
        // self.fader(ctx, "drop here");
        // }

        // Collect dropped files:
        if !ctx.input().raw.dropped_files.is_empty() {
            self.fader(ctx, "packing");
            self.packer.update(&ctx.input().raw.dropped_files);
            self.to_update = true;
        }
    }

    fn handle_keys(&mut self, ctx: &egui::CtxRef) {
        for event in &ctx.input().raw.events {
            match event {
                Event::Key {
                    key: egui::Key::Backspace,
                    pressed: false,
                    ..
                } => self.undo(ctx),

                Event::Key {
                    key: egui::Key::Escape,
                    pressed: true,
                    ..
                } => self.clear(ctx),

                _ => (),
            }
            self.to_update = true;
        }
    }

    //UI reaction
    fn fader(&mut self, ctx: &egui::CtxRef, text: &str) {
        let painter = ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("fader")));
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
    // Key Functions
    fn clear(&mut self, ctx: &egui::CtxRef) {
        self.fader(ctx, "clear");
        self.packer = Packer::new();
    }
    fn undo(&mut self, ctx: &egui::CtxRef) {
        self.fader(ctx, "undo");
        self.packer.undo();
    }
}
