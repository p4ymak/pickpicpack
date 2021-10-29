mod app;
mod loader;
mod packer;
use app::*;
use eframe::egui;

fn main() {
    let start_state = P3App::default();
    let options = eframe::NativeOptions {
        always_on_top: true,
        // maximized: true,
        resizable: false,
        initial_window_size: Some(egui::Vec2 { x: 500.0, y: 500.0 }),
        drag_and_drop_support: true,
        transparent: true,
        ..Default::default()
    };
    eframe::run_native(Box::new(start_state), options);
}
