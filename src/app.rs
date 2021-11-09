use super::packer::*;
use super::utils::*;
use eframe::{egui, epi};
use egui::*;
use epi::Storage;
use std::path::PathBuf;

#[derive(Default)]
pub struct P3App {
    aspect: AspectRatio,
    export_scale: ImageScaling,
    preview_size: RectSize,
    export_size: RectSize,
    ratio: f32,
    packer: Packer,
    texture: Option<(egui::Vec2, egui::TextureId)>,
    to_update: bool,
    export_path: PathBuf,
}

impl epi::App for P3App {
    fn name(&self) -> &str {
        OUTPUT_NAME
    }

    fn setup(
        &mut self,
        ctx: &egui::CtxRef,
        frame: &mut epi::Frame<'_>,
        _storage: Option<&dyn Storage>,
    ) {
        // self.packer.side = frame.margin.x;
        self.preview_size = RectSize::new(512, 512);
        self.export_size = RectSize::new(512, 512);
    }
    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        if self.to_update {
            if let Some(texture) = self.texture {
                frame.tex_allocator().free(texture.1);
            }
            if let Some(preview) = &self.packer.preview {
                // Allocate a texture:
                let texture = frame
                    .tex_allocator()
                    .alloc_srgba_premultiplied((preview.size.w, preview.size.h), &preview.pixels);
                let size = egui::Vec2::new(preview.size.w as f32, preview.size.h as f32);
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
        egui::Window::new("Settings")
            // .frame(egui::containers::Frame::default())
            // .anchor(egui::Align2::CENTER_TOP, [0.0, 0.0])
            .anchor(egui::Align2::LEFT_TOP, [0.0, 0.0])
            //.default_pos([0.0, 0.0])
            .resizable(false)
            // .collapsible(false)
            .show(ctx, |panel| {
                panel.vertical(|ui| {
                    //RADIO - SET RATIO
                    ui.horizontal_top(|ratio| {
                        ratio.label("Ratio:");
                        if ratio
                            .selectable_value(&mut self.aspect, AspectRatio::Square, "Square")
                            .clicked()
                            || ratio
                                .selectable_value(&mut self.aspect, AspectRatio::Screen, "Screen")
                                .clicked()
                            || ratio
                                .selectable_value(&mut self.aspect, AspectRatio::FourThree, "4 : 3")
                                .clicked()
                            || ratio
                                .selectable_value(&mut self.aspect, AspectRatio::ThreeFour, "3 : 4")
                                .clicked()
                            || ratio
                                .selectable_value(
                                    &mut self.aspect,
                                    AspectRatio::SixteenNine,
                                    "16 : 9",
                                )
                                .clicked()
                            || ratio
                                .selectable_value(
                                    &mut self.aspect,
                                    AspectRatio::NineSixteen,
                                    "9 : 16",
                                )
                                .clicked()
                        {
                            // self.to_update = true;
                            println!("{:?}", self.aspect);
                        }
                    });
                    //RADIO - EXPORT SIZE
                    ui.horizontal_top(|export_size| {
                        export_size.label("Size:");
                        if export_size
                            .selectable_value(
                                &mut self.export_scale,
                                ImageScaling::FitScreen,
                                "Fit Screen",
                            )
                            .clicked()
                            || export_size
                                .selectable_value(
                                    &mut self.export_scale,
                                    ImageScaling::HalfK,
                                    "512",
                                )
                                .clicked()
                            || export_size
                                .selectable_value(
                                    &mut self.export_scale,
                                    ImageScaling::OneK,
                                    "1024",
                                )
                                .clicked()
                            || export_size
                                .selectable_value(
                                    &mut self.export_scale,
                                    ImageScaling::TwoK,
                                    "2048",
                                )
                                .clicked()
                            || export_size
                                .selectable_value(
                                    &mut self.export_scale,
                                    ImageScaling::FourK,
                                    "4096",
                                )
                                .clicked()
                            || export_size
                                .selectable_value(
                                    &mut self.export_scale,
                                    ImageScaling::Actual,
                                    "Actual",
                                )
                                .clicked()
                        {
                            // self.to_update = true;
                            println!("{:?}", self.export_scale);
                        }
                    });
                    //BUTTON - SET PATH
                    ui.horizontal_wrapped(|export_path| {
                        export_path.label("Export path:");
                        let button_path = export_path.button("Choose directory...");
                        if button_path.clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .set_directory(&self.export_path)
                                .pick_folder()
                            {
                                self.export_path = path;
                            }
                        }
                    });
                    //BUTTON - EXPORT
                    ui.separator();
                    ui.horizontal_wrapped(|apply| {
                        let button_apply = apply.button("Apply");

                        if button_apply.clicked() {
                            self.packer.preview_size = self.preview_size;
                            self.packer.export_size = self.export_size;
                            self.to_update = true;
                        }
                    });
                })
            });
        egui::Area::new("Export")
            // .frame(egui::containers::Frame::default())
            // .anchor(egui::Align2::CENTER_TOP, [0.0, 0.0])
            .anchor(egui::Align2::RIGHT_BOTTOM, [0.0, 0.0])
            //.default_pos([0.0, 0.0])
            // .collapsible(false)
            .show(ctx, |panel| {
                if panel.button("Export").clicked() {
                    self.packer.export(&self.export_path);
                };
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
        // Preview hvering files:
        // for file in &ctx.input().raw.hovered_files {
        //     println!("{:?}", file.mime);
        // }

        // Collect dropped files:
        if !ctx.input().raw.dropped_files.is_empty() {
            self.fader(ctx, "packing");

            self.packer
                .update(&ctx.input().raw.dropped_files, self.preview_size);
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
        self.packer = Packer::new(self.preview_size, self.export_size, self.ratio);
        self.to_update = true;
    }
    fn undo(&mut self, ctx: &egui::CtxRef) {
        self.fader(ctx, "undo");
        self.packer.undo();
        self.to_update = true;
    }
}
