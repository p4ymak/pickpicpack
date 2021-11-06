use super::packer::*;
use eframe::{egui, epi};
use egui::*;
use std::path::{Path, PathBuf};
// use epi::Storage;

impl Default for Aspect {
    fn default() -> Aspect {
        Aspect::Square
    }
}

// #[derive(Default)]
pub struct P3App {
    aspect: Aspect,
    packer: Packer,
    texture: Option<(egui::Vec2, egui::TextureId)>,
    to_update: bool,
    export_path: PathBuf,
    // side: f32,
}

impl Default for P3App {
    fn default() -> P3App {
        P3App {
            aspect: Aspect::default(),
            packer: Packer::new(),
            texture: None,
            to_update: false,
            export_path: PathBuf::new(),
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
    //     // self.packer.side = frame.margin.x;
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

        //Draw Image
        egui::Area::new("image")
            .order(Order::Background)
            .show(ctx, |ui| {
                if let Some((size, texture)) = self.texture {
                    ui.image(texture, size);
                }
            });

        //Draw GUI if mouse hovered window
        if self.packer.items.is_empty() || ctx.input().pointer.has_pointer() {
            self.hud(ctx);
        }

        self.detect_files_being_dropped(ctx);
        self.handle_keys(ctx);
    }
}

impl P3App {
    //GUI reaction
    fn hud(&mut self, ctx: &egui::CtxRef) {
        egui::Area::new("menu")
            .order(Order::Foreground)
            .movable(false)
            .show(ctx, |panel| {
                panel.horizontal_top(|ui| {
                    //RADIO - SET RATIO
                    ui.vertical(|ratio| {
                        if ratio
                            .selectable_value(&mut self.aspect, Aspect::Square, "Square")
                            .clicked()
                            || ratio
                                .selectable_value(&mut self.aspect, Aspect::Screen, "Screen")
                                .clicked()
                            || ratio
                                .selectable_value(&mut self.aspect, Aspect::FourThree, "4 : 3")
                                .clicked()
                            || ratio
                                .selectable_value(&mut self.aspect, Aspect::ThreeFour, "3 : 4")
                                .clicked()
                            || ratio
                                .selectable_value(&mut self.aspect, Aspect::SixteenNine, "16 : 9")
                                .clicked()
                            || ratio
                                .selectable_value(&mut self.aspect, Aspect::NineSixteen, "9 : 16")
                                .clicked()
                        {
                            // self.to_update = true;
                            println!("{:?}", self.aspect);
                        }
                    });

                    //BUTTON - SET PATH
                    let button_path = ui.button("Export to...");
                    if button_path.clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .set_directory(&self.export_path)
                            .pick_folder()
                        {
                            self.export_path = path;
                        }
                    }

                    //BUTTON - EXPORT
                    let button_export = ui.button("Export");
                    if button_export.clicked() {
                        todo!();
                    }
                })
            });
    }

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

    fn detect_files_being_dropped(&mut self, ctx: &egui::CtxRef) {
        // Preview hovering files:
        // for file in &ctx.input().raw.hovered_files {
        //     println!("{:?}", file.mime);
        // }

        // Collect dropped files:
        if !ctx.input().raw.dropped_files.is_empty() {
            self.fader(ctx, "packing");
            self.packer.side = ctx.input().screen_rect().width();
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
        }
    }

    // Key Functions
    fn clear(&mut self, ctx: &egui::CtxRef) {
        self.fader(ctx, "clear");
        self.packer = Packer::new();
        self.to_update = true;
    }
    fn undo(&mut self, ctx: &egui::CtxRef) {
        self.fader(ctx, "undo");
        self.packer.undo();
        self.to_update = true;
    }
}
