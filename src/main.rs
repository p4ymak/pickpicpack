use eframe::{egui, epi};

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

#[derive(Default)]
struct MyApp {
    aspect: Aspect,
    dropped_files: Vec<egui::DroppedFile>,
    // picked_path: Option<String>,
}

impl epi::App for MyApp {
    fn name(&self) -> &str {
        "PickPicPack"
    }

    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Aspect::None = self.aspect {
                ui.label("Aspect NOT chosen!");
            }

            if self.dropped_files.is_empty() {
                ui.label("Drag-and-drop files onto the window!");
            }
            //             if cfg!(target_os = "macos") {
            //                 // Awaiting fix of winit bug: https://github.com/rust-windowing/winit/pull/2027
            //             } else if ui.button("Open file…").clicked() {
            //                 if let Some(path) = rfd::FileDialog::new().pick_file() {
            //                     self.picked_path = Some(path.display().to_string());
            //                 }
            //             }

            //             if let Some(picked_path) = &self.picked_path {
            //                 ui.horizontal(|ui| {
            //                     ui.label("Picked file:");
            //                     ui.monospace(picked_path);
            //                 });
            //             }

            // Show dropped files (if any):
            if !self.dropped_files.is_empty() {
                ui.group(|ui| {
                    ui.label("Dropped files:");

                    for file in &self.dropped_files {
                        let mut info = if let Some(path) = &file.path {
                            path.display().to_string()
                        } else if !file.name.is_empty() {
                            file.name.clone()
                        } else {
                            "???".to_owned()
                        };
                        if let Some(bytes) = &file.bytes {
                            info += &format!(" ({} bytes)", bytes.len());
                        }
                        ui.label(info);
                    }
                });
            }
        });

        self.detect_files_being_dropped(ctx);
    }
}

impl MyApp {
    fn detect_files_being_dropped(&mut self, ctx: &egui::CtxRef) {
        use egui::*;

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
            self.dropped_files
                .extend(ctx.input().raw.dropped_files.clone());
            println!("EXECUTE PACKING!!");
        }
    }
}

fn main() {
    let options = eframe::NativeOptions {
        drag_and_drop_support: true,
        ..Default::default()
    };
    eframe::run_native(Box::new(MyApp::default()), options);
}
