use super::packer::*;
use super::utils::*;
use core::time::Duration;
use eframe::{egui, epi};
use egui::*;
use epi::Storage;
// use futures::executor::block_on;
use native_dialog::FileDialog;
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct Settings {
    screen_size: RectSize,
    width: f32,
    preview_size: RectSize,
    zip: bool,
    export_path: PathBuf,
}
impl Default for Settings {
    fn default() -> Settings {
        Settings {
            screen_size: RectSize::default(),
            width: 512.0,
            preview_size: RectSize::default(),
            zip: false,
            export_path: std::env::current_dir().unwrap_or_default(),
        }
    }
}

#[derive(Default)]
// #[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
// #[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct P3App {
    settings: Settings,
    packer: Packer,
    texture: Option<(egui::Vec2, egui::TextureId)>,
    to_update: bool,
    fader: Option<String>,
}

impl epi::App for P3App {
    fn name(&self) -> &str {
        OUTPUT_NAME
    }
    fn warm_up_enabled(&self) -> bool {
        true
    }
    fn persist_native_window(&self) -> bool {
        true
    }
    fn persist_egui_memory(&self) -> bool {
        false
    }
    fn auto_save_interval(&self) -> Duration {
        Duration::MAX
    }
    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        _frame: &mut epi::Frame<'_>,
        storage: Option<&dyn Storage>,
    ) {
        self.packer = Packer::new(
            self.settings.width,
            AspectRatio::default(),
            ImageScaling::default(),
        );
        self.load(storage);
        self.packer.update(&[]);
        self.to_update = true;
    }

    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, "PPP_scale", &self.packer.scale);
        epi::set_value(storage, "PPP_ratio", &self.packer.aspect);
        epi::set_value(storage, "PPP_export_path", &self.settings.export_path);
        epi::set_value(storage, "PPP_zip", &self.settings.zip);
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        // if let Some(message) = &self.fader {
        //     let painter = ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("fader")));
        //     let screen_rect = ctx.input().screen_rect();
        //     painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(192));
        //     painter.text(
        //         screen_rect.center(),
        //         Align2::CENTER_CENTER,
        //         message,
        //         TextStyle::Heading,
        //         Color32::WHITE,
        //     );
        // }
        if self.to_update {
            // self.fader = None;
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

            self.settings.preview_size = RectSize::by_scale_and_ratio(
                &ImageScaling::Preview(self.settings.width),
                &self.packer.aspect,
            );
            let size = egui::Vec2::new(
                self.settings.preview_size.w as f32,
                self.settings.preview_size.h as f32,
            );
            frame.set_window_size(size);

            self.to_update = false;
        }

        //Draw Image
        egui::Area::new("image")
            .order(Order::Background)
            .show(ctx, |ui| {
                if let Some((size, texture)) = self.texture {
                    if ui
                        .add(egui::Image::new(texture, size).sense(Sense::drag()))
                        .dragged()
                    {
                        frame.drag_window();
                    }
                }
            });

        let painter = ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("fader")));
        let screen_rect = ctx.input().screen_rect();
        painter.rect_stroke(screen_rect, 0.0, egui::Stroke::new(2.0, Color32::GRAY));
        if self.packer.items.is_empty() {
            painter.text(
                Pos2::new(screen_rect.max.x / 2.0, screen_rect.max.y / 2.0),
                Align2::CENTER_CENTER,
                "DROP HERE",
                TextStyle::Heading,
                Color32::DARK_GRAY,
            );
        }
        //Draw GUI if mouse hovered window
        if self.packer.items.is_empty() || ctx.input().pointer.has_pointer() {
            self.hud(ctx, frame);
        }

        self.detect_files_being_dropped(ctx);
        self.handle_keys(ctx);
    }
}

impl P3App {
    //GUI reaction
    fn hud(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        egui::Window::new("Settings")
            .anchor(egui::Align2::CENTER_TOP, [0.0, 0.0])
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            // .auto_sized()
            .show(ctx, |panel| {
                panel.vertical(|ui| {
                    //RADIO - SET RATIO
                    ui.horizontal(|ratio| {
                        let tooltip_ratio =
                            "Aspect ratio of packaging area. Updates package on change..";
                        ratio.label("Ratio:").on_hover_text(tooltip_ratio);
                        if ratio
                            .selectable_value(&mut self.packer.aspect, AspectRatio::Square, "1 : 1")
                            .clicked()
                            || ratio
                                .selectable_value(
                                    &mut self.packer.aspect,
                                    AspectRatio::FourThree,
                                    "4 : 3",
                                )
                                .clicked()
                            || ratio
                                .selectable_value(
                                    &mut self.packer.aspect,
                                    AspectRatio::ThreeFour,
                                    "3 : 4",
                                )
                                .clicked()
                            || ratio
                                .selectable_value(
                                    &mut self.packer.aspect,
                                    AspectRatio::ThreeTwo,
                                    "3 : 2",
                                )
                                .clicked()
                            || ratio
                                .selectable_value(
                                    &mut self.packer.aspect,
                                    AspectRatio::TwoThree,
                                    "2 : 3",
                                )
                                .clicked()
                            || ratio
                                .selectable_value(
                                    &mut self.packer.aspect,
                                    AspectRatio::SixteenNine,
                                    "16 : 9",
                                )
                                .clicked()
                            || ratio
                                .selectable_value(
                                    &mut self.packer.aspect,
                                    AspectRatio::NineSixteen,
                                    "9 : 16",
                                )
                                .clicked()
                        {
                            self.update_packer(&[]);
                            self.to_update = true;
                        }

                        if ratio
                            .add(egui::SelectableLabel::new(
                                self.packer.aspect == AspectRatio::Zero,
                                "0 : 0",
                            ))
                            .clicked()
                        {
                            // #[cfg(feature = "persistence")]
                            // self.save();
                            frame.quit();
                        }
                    });
                    //RADIO - EXPORT SIZE
                    ui.separator();
                    ui.horizontal(|export_size| {
                        let tooltip_size = "Maximum dimension of exported image..";
                        export_size
                            .label("Export Size:")
                            .on_hover_text(tooltip_size);
                        if export_size
                            .selectable_value(
                                &mut self.packer.scale,
                                ImageScaling::FitScreen,
                                "Fit Screen",
                            )
                            .clicked()
                            || export_size
                                .selectable_value(
                                    &mut self.packer.scale,
                                    ImageScaling::HalfK,
                                    "512",
                                )
                                .clicked()
                            || export_size
                                .selectable_value(
                                    &mut self.packer.scale,
                                    ImageScaling::OneK,
                                    "1024",
                                )
                                .clicked()
                            || export_size
                                .selectable_value(
                                    &mut self.packer.scale,
                                    ImageScaling::TwoK,
                                    "2048",
                                )
                                .clicked()
                            || export_size
                                .selectable_value(
                                    &mut self.packer.scale,
                                    ImageScaling::FourK,
                                    "4096",
                                )
                                .clicked()
                            || export_size
                                .selectable_value(
                                    &mut self.packer.scale,
                                    ImageScaling::Actual,
                                    "Actual",
                                )
                                .clicked()
                        {
                            // self.to_update = true;
                            // println!("{:?}", self.packer.scale);
                        }
                    });
                    ui.separator();
                    //BUTTONS - EXPORT
                    ui.horizontal(|buttons| {
                        if buttons
                            .button("Clear")
                            .on_hover_text("Start from scratch..\nShortcut: [Escape]")
                            .clicked()
                        {
                            self.clear();
                        }
                        if buttons
                            .button("Undo")
                            .on_hover_text("Remove last drop..\nShortcut: [Backspace]")
                            .clicked()
                        {
                            self.undo();
                        }

                        buttons.separator();
                        let button_path = buttons
                            .button("Directory...")
                            .on_hover_text("Where to place resulting image..");
                        if button_path.clicked() {
                            if let Ok(Some(path)) = FileDialog::new()
                                .set_location(&self.settings.export_path)
                                .show_open_single_dir()
                            {
                                self.settings.export_path = path;
                            }
                            // if let Some(path) = block_on(open_dialog(&self.settings.export_path)) {
                            //     self.settings.export_path = path;
                            // }
                        }
                        buttons.separator();
                        buttons
                            .checkbox(&mut self.settings.zip, "ZIP")
                            .on_hover_text("Also pack all source images to archive..");

                        buttons.separator();
                        if buttons
                            .button("Export Result")
                            .on_hover_text("Save result to file..\nShortcut: [Enter]")
                            .clicked()
                        {
                            self.export();
                        };
                    });
                    // println!("HEADER: {:?}", ui.min_rect());
                })
            });

        if self.packer.items.is_empty() {
            egui::Window::new("About")
                .anchor(egui::Align2::CENTER_BOTTOM, [0.0, 0.0])
                // .frame(egui::containers::Frame::default())
                .title_bar(false)
                .resizable(false)
                .collapsible(false)
                .show(ctx, |about| {
                    about.vertical_centered(|ui| {
                        // ui.add(
                        //     egui::Hyperlink::new("https://github.com/emilk/egui")
                        //         .text("My favorite repo"),
                        // );
                        ui.label(format!(
                            "PickPicPack v{} by Roman Chumak",
                            env!("CARGO_PKG_VERSION"),
                        ));
                    });
                });
        }
    }

    fn fader(&mut self, text: &str) {
        if text.is_empty() {
            self.fader = None;
        } else {
            self.fader = Some(text.to_string());
        }
    }

    fn detect_files_being_dropped(&mut self, ctx: &egui::CtxRef) {
        // Preview hvering files:
        // for file in &ctx.input().raw.hovered_files {
        //     println!("{:?}", file.mime);
        // }

        // Collect dropped files:
        if !ctx.input().raw.dropped_files.is_empty() {
            self.fader("packing");
            ctx.request_repaint();
            self.update_packer(&ctx.input().raw.dropped_files);
            self.fader("");
            self.to_update = true;
        }
    }
    fn update_packer(&mut self, files: &[DroppedFile]) {
        self.packer.update(files);
    }
    fn handle_keys(&mut self, ctx: &egui::CtxRef) {
        for event in &ctx.input().raw.events {
            match event {
                Event::Key {
                    key: egui::Key::Backspace,
                    pressed: false,
                    ..
                } => self.undo(),

                Event::Key {
                    key: egui::Key::Escape,
                    pressed: true,
                    ..
                } => self.clear(),

                Event::Key {
                    key: egui::Key::Enter,
                    pressed: true,
                    ..
                } => self.export(),
                _ => (),
            }
        }
    }

    // Key Functions
    fn clear(&mut self) {
        self.fader("clear");
        self.packer = Packer::new(self.settings.width, self.packer.aspect, self.packer.scale);
        self.fader("");
        self.to_update = true;
    }
    fn undo(&mut self) {
        self.fader("undo");
        if self.packer.items.len() <= 1 {
            self.clear();
        } else {
            self.packer.undo();
        }
        self.fader("");
        self.to_update = true;
    }
    fn export(&mut self) {
        self.fader("exporting");
        self.packer
            .export(&self.settings.export_path, self.settings.zip);
        self.fader("");
    }

    // Load state
    fn load(&mut self, storage: Option<&dyn epi::Storage>) {
        if let Some(storage) = storage {
            self.packer.scale = epi::get_value(storage, "PPP_scale").unwrap_or_default();
            self.packer.aspect = epi::get_value(storage, "PPP_ratio").unwrap_or_default();
            self.settings.export_path =
                epi::get_value(storage, "PPP_export_path").unwrap_or_default();
            self.settings.zip = epi::get_value(storage, "PPP_zip").unwrap_or_default();
        }
    }

    pub fn new(screen_size: RectSize) -> Self {
        let width = window_width(screen_size, WINDOW_SCALE);
        P3App {
            settings: Settings {
                screen_size,
                width,
                ..Default::default()
            },
            packer: Packer::new(width, AspectRatio::default(), ImageScaling::default()),
            texture: None,
            to_update: false,
            fader: None,
        }
    }
}

// async fn open_dialog(def: &Path) -> Option<PathBuf> {
//     let dialog = rfd::AsyncFileDialog::new()
//         .set_directory(def)
//         .pick_folder()
//         .await;

//     dialog.map(|d| d.path().to_owned())
// }
