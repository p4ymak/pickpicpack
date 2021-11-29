use super::packer::*;
use super::utils::*;
use core::time::Duration;
use eframe::{egui, epi};
use egui::*;
use epi::Storage;
// use image::RgbaImage;
use plot::{Plot, PlotImage, Polygon, Value, Values};
use std::path::PathBuf;

#[derive(Debug)]
struct Settings {
    screen_size: RectSize,
    width: f32,
    preview_size: RectSize,
    ratio_string: String,
    ratio_custom: (usize, usize),
    zip: bool,
    export_path: PathBuf,
}
impl Default for Settings {
    fn default() -> Settings {
        Settings {
            screen_size: RectSize::default(),
            width: 512.0,
            preview_size: RectSize::default(),
            ratio_string: "2 : 1".to_string(),
            ratio_custom: (2, 1),
            zip: false,
            export_path: default_path(),
        }
    }
}

#[derive(Default)]
struct Counter {
    recent: isize,
    total: isize,
}
impl Counter {
    fn reset(&mut self) {
        self.recent = -1;
        self.total = -1;
    }
    fn renew(&mut self, num: usize) {
        self.recent = -1;
        self.total = num as isize;
    }
    fn update(&mut self) -> Option<isize> {
        if self.total == -1 {
            self.total = 0;
            return Some(0);
        }
        if self.recent >= self.total {
            None
        } else {
            self.recent += 1;
            Some(self.recent)
        }
    }
    fn finished(&self) -> bool {
        self.recent >= self.total
    }
}

#[derive(Default)]
pub struct P3App {
    settings: Settings,
    packer: Packer,
    texture: Option<egui::TextureId>,
    counter: Counter,
    shortcuts: bool,
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
            false,
        );
        self.load(storage);
        self.packer.update(&[]);
        self.counter.reset();

        self.settings.preview_size = RectSize::by_scale_and_ratio(
            &ImageScaling::Preview(self.settings.width),
            &self.packer.aspect,
        );
    }

    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, "PPP_scale", &self.packer.scale);
        epi::set_value(storage, "PPP_equal", &self.packer.equal);
        epi::set_value(storage, "PPP_ratio", &self.packer.aspect);
        epi::set_value(storage, "PPP_export_path", &self.settings.export_path);
        epi::set_value(storage, "PPP_zip", &self.settings.zip);
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        let screen_rect = ctx.input().screen_rect();
        let w = screen_rect.max.x;
        let h = screen_rect.max.y;
        let ratio = self.packer.aspect.div();
        let (box_w, box_h) = match ratio <= h / w {
            true => (w, (w * ratio)),
            false => ((h / ratio), h),
        };
        self.packer.preview_width = box_w;

        if let Some(id) = self.counter.update() {
            self.packer.combine_thumbnails(id);
            self.allocate_texure(frame);
            self.settings.preview_size = RectSize::by_scale_and_ratio(
                &ImageScaling::Preview(self.settings.width),
                &self.packer.aspect,
            );
        }

        egui::Area::new("image")
            .order(Order::Background)
            .anchor(egui::Align2::LEFT_TOP, [0.0, 0.0])
            .drag_bounds(screen_rect)
            .show(ctx, |ui| {
                if let Some(texture) = self.texture {
                    // let box_bg = Polygon::new(Values::from_values(vec![
                    //     Value::new(-box_w / 2.0, -box_h / 2.0),
                    //     Value::new(box_w / 2.0, -box_h / 2.0),
                    //     Value::new(box_w / 2.0, box_h / 2.0),
                    //     Value::new(-box_w / 2.0, box_h / 2.0),
                    // ]))
                    // .color(self.packer.bg_color)
                    // .fill_alpha(self.packer.bg_color.a() as f32 / 255.0);
                    let image_preview =
                        PlotImage::new(texture, Value::new(0.0, 0.0), [box_w, box_h]);
                    let box_frame = Polygon::new(Values::from_values(vec![
                        Value::new(-box_w / 2.0, -box_h / 2.0),
                        Value::new(box_w / 2.0, -box_h / 2.0),
                        Value::new(box_w / 2.0, box_h / 2.0),
                        Value::new(-box_w / 2.0, box_h / 2.0),
                    ]))
                    .color(Color32::DARK_GRAY)
                    .fill_alpha(0.0);

                    ui.add(
                        Plot::new("preview")
                            // .polygon(box_bg)
                            .polygon(box_frame)
                            .image(image_preview)
                            .width(screen_rect.max.x)
                            .height(screen_rect.max.y)
                            .allow_drag(false)
                            .data_aspect(1.0)
                            .view_aspect(1.0 / self.packer.aspect.div())
                            .show_x(false)
                            .show_y(false)
                            .center_x_axis(true)
                            .center_x_axis(true)
                            .allow_drag(false)
                            .allow_zoom(false)
                            .show_background(false)
                            .show_axes([false, false])
                            .legend(plot::Legend {
                                text_style: TextStyle::Small,
                                background_alpha: 0.0,
                                position: plot::Corner::RightBottom,
                            }),
                    );
                }
            });
        //Draw GUI if mouse hovered window
        if self.packer.items.is_empty() || ctx.input().pointer.has_pointer() {
            self.hud(ctx, frame);
        }

        self.detect_files_being_dropped(ctx);
        if self.shortcuts {
            self.handle_keys(ctx);
        }
        if !self.counter.finished() {
            ctx.request_repaint();
        }
    }
}

impl P3App {
    fn allocate_texure(&mut self, frame: &mut epi::Frame<'_>) {
        if let Some(texture) = self.texture {
            frame.tex_allocator().free(texture);
        }
        // let preview = match self.counter.finished() {
        //     false => image::imageops::thumbnail(
        //         &self.packer.preview,
        //         self.packer.preview.width() / 10,
        //         self.packer.preview.height() / 10,
        //     ),
        //     true => self.packer.preview.to_owned(),
        // };
        let pixels = self
            .packer
            .preview
            .pixels()
            .map(|p| Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
            .collect::<Vec<Color32>>();

        let texture = frame.tex_allocator().alloc_srgba_premultiplied(
            (
                self.packer.preview.width() as usize,
                self.packer.preview.height() as usize,
            ),
            &pixels,
        );
        self.texture = Some(texture);
    }

    //GUI reaction
    fn hud(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        egui::Window::new("Menu")
            .anchor(egui::Align2::LEFT_TOP, [0.0, 0.0])
            // .title_bar(false)
            .resizable(false)
            // .collapsible(false)
            // .auto_sized()
            .frame(Frame {
                margin: Vec2::new(8.0, 8.0),
                corner_radius: 0.0,
                shadow: epaint::Shadow::small_dark(),
                fill: Color32::from_rgb(33, 33, 33),
                stroke: Stroke::new(1.0, Color32::DARK_GRAY),
            })
            .show(ctx, |panel| {
                panel.vertical(|ui| {
                    //RADIO - SET RATIO
                    ui.horizontal(|ratio| {
                        let tooltip_ratio =
                            "Aspect ratio of packaging area.\nUpdates package on change..";
                        ratio.label("Ratio:").on_hover_text(tooltip_ratio);
                        let ratio_input = ratio.add(
                            egui::TextEdit::singleline(&mut self.settings.ratio_string)
                                .desired_width(60.0),
                        );
                        if ratio_input.gained_focus() {
                            self.settings.ratio_string = String::new();
                            self.shortcuts = false;
                        }
                        if ratio_input.lost_focus() {
                            self.shortcuts = true;
                        }
                        if ratio_input.lost_focus()
                        //&& ctx.input().key_pressed(egui::Key::Enter) {
                        {
                            self.settings.ratio_custom =
                                parse_custom_ratio(&self.settings.ratio_string);
                            self.settings.ratio_string = format!(
                                "{} : {}",
                                self.settings.ratio_custom.0, self.settings.ratio_custom.1
                            );
                            // self.shortcuts = true;
                        }
                        // if ratio_input.lost_focus() {
                        //     self.shortcuts = true;
                        // }

                        if ratio
                            .selectable_value(
                                &mut self.packer.aspect,
                                AspectRatio::Custom(self.settings.ratio_custom),
                                format!(
                                    "{} : {}",
                                    self.settings.ratio_custom.0, self.settings.ratio_custom.1
                                ),
                            )
                            .clicked()
                        {
                            self.update_packer(&[]);
                        }

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
                                    AspectRatio::ThreeTwo,
                                    "3 : 2",
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
                                    AspectRatio::Window(
                                        ctx.input().screen_rect.max.y
                                            / ctx.input().screen_rect.max.x,
                                    ),
                                    "Window",
                                )
                                .on_hover_text("Get aspect ratio of window..\nShortcut: [Space]")
                                .clicked()
                        {
                            self.update_packer(&[]);
                        }
                    });
                    //Thumbnails scaling options
                    ui.separator();
                    ui.horizontal(|scaling| {
                        let tooltip_scale =
                            "Scaling options for each image..\nUpdates package on change..";
                        scaling.label("Scale:").on_hover_text(tooltip_scale);
                        if scaling
                            .checkbox(&mut self.packer.equal, "Equal")
                            .on_hover_text("Scale images to equal size..")
                            .clicked()
                        {
                            self.update_packer(&[]);
                        };
                        scaling.separator();

                        let tooltip_margin = "Space between images..\nUpdates package on change..";
                        scaling.label("Margin:").on_hover_text(tooltip_margin);
                        let zero = match self.packer.margin {
                            0 => scaling.add_enabled(false, Button::new("0")),
                            _ => scaling.add_enabled(true, Button::new("0")),
                        };
                        if zero.clicked() {
                            self.packer.margin = 0;
                            self.update_packer(&[]);
                        }
                        let minus = match self.packer.margin {
                            0 => scaling.add_enabled(false, Button::new("-")),
                            _ => scaling.add_enabled(true, Button::new("-")),
                        };
                        if minus.clicked() {
                            self.packer.margin = match self.packer.margin {
                                1 => 0,
                                x => x / 2,
                            };
                            self.update_packer(&[]);
                        }
                        let plus = match self.packer.margin {
                            1024 => scaling.add_enabled(false, Button::new("+")),
                            _ => scaling.add_enabled(true, Button::new("+")),
                        };
                        if plus.clicked() {
                            self.packer.margin = match self.packer.margin {
                                0 => 1,
                                x => (x * 2).min(1024),
                            };
                            self.update_packer(&[]);
                        }

                        scaling.separator();

                        let tooltip_margin = "Total packed images..";
                        scaling
                            .label(format!(
                                "Loaded: {} / {}",
                                self.counter.recent.max(0),
                                self.counter.total.max(0)
                            ))
                            .on_hover_text(tooltip_margin);
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
                                ImageScaling::FitScreen(ctx.input().screen_rect()),
                                "Current",
                            )
                            .on_hover_text("Scale exported image to fit window.")
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
                                .on_hover_text("Keep opriginal images size.")
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

                        #[cfg(target_os = "macos")]
                        let button_path =
                            buttons.add_enabled(false, egui::Button::new("Exports to Pictures"));
                        #[cfg(not(target_os = "macos"))]
                        let button_path = buttons
                            .button("Set Directory...")
                            .on_hover_text("Where to place resulting image..");
                        if button_path.clicked() {
                            if self.settings.export_path.exists() {
                                if let Some(path) = rfd::FileDialog::new()
                                    .set_directory(&self.settings.export_path)
                                    .pick_folder()
                                {
                                    self.settings.export_path = path;
                                }
                            } else if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                self.settings.export_path = path;
                            }
                        }

                        buttons.separator();
                        buttons
                            .checkbox(&mut self.settings.zip, "ZIP")
                            .on_hover_text("Also pack all source images to archive..");

                        buttons.separator();
                        if self.counter.total > 0 {
                            if buttons
                                .button("Export Result")
                                .on_hover_text({
                                    let size = match self.packer.scale {
                                        ImageScaling::Actual => self.packer.actual_size,
                                        _ => RectSize::by_scale_and_ratio(
                                            &self.packer.scale,
                                            &self.packer.aspect,
                                        ),
                                    };
                                    format!(
                                        "Save result to file..\n{} x {}\nShortcut: [Enter]",
                                        size.w, size.h
                                    )
                                })
                                .clicked()
                            {
                                self.export();
                            };
                        } else {
                            buttons.add_enabled(false, Button::new("Export Result"));
                        }
                    });

                    ui.separator();
                    ui.vertical_centered(|ui| {
                        ui.add(
                            egui::Hyperlink::new("http://www.p43d.com/pickpicpack").text(
                                // );
                                // ui.label(
                                format!(
                                    "PickPicPack v{} by Roman Chumak",
                                    env!("CARGO_PKG_VERSION"),
                                ),
                            ),
                        );
                    });
                })
            });
    }

    fn fader(&mut self, text: &str) {
        if text.is_empty() {
            self.fader = None;
        } else {
            self.fader = Some(text.to_string());
        }
    }

    fn detect_files_being_dropped(&mut self, ctx: &egui::CtxRef) {
        if !ctx.input().raw.dropped_files.is_empty() {
            self.fader("packing");
            self.update_packer(&ctx.input().raw.dropped_files);
            self.fader("");
        }
    }

    fn update_packer(&mut self, files: &[DroppedFile]) {
        self.counter.renew(self.packer.update(files));
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
                Event::Key {
                    key: egui::Key::Space,
                    pressed: true,
                    ..
                } => self.window_ratio(ctx),
                _ => (),
            }
        }
    }

    // Shortcut Functions
    fn clear(&mut self) {
        self.fader("clear");
        self.packer = Packer::new(
            self.settings.width,
            self.packer.aspect,
            self.packer.scale,
            self.packer.equal,
        );
        self.fader("");
        self.counter.reset();
    }
    fn undo(&mut self) {
        self.fader("undo");
        if self.packer.items.len() <= 1 {
            self.clear();
            self.counter.reset();
        } else {
            self.counter.renew(self.packer.undo());
        }
        self.fader("");
    }
    fn export(&mut self) {
        self.fader("exporting");
        self.packer
            .export(&self.settings.export_path, self.settings.zip);
        self.fader("");
    }
    fn window_ratio(&mut self, ctx: &egui::CtxRef) {
        self.packer.aspect =
            AspectRatio::Window(ctx.input().screen_rect.max.y / ctx.input().screen_rect.max.x);
        self.update_packer(&[]);
    }

    // Load state
    fn load(&mut self, storage: Option<&dyn epi::Storage>) {
        if let Some(storage) = storage {
            self.packer.scale = epi::get_value(storage, "PPP_scale").unwrap_or_default();
            self.packer.equal = epi::get_value(storage, "PPP_equal").unwrap_or_default();
            self.packer.aspect = epi::get_value(storage, "PPP_ratio").unwrap_or_default();
            self.settings.export_path =
                epi::get_value(storage, "PPP_export_path").unwrap_or_else(default_path);
            self.settings.zip = epi::get_value(storage, "PPP_zip").unwrap_or_default();
        }
    }
}
