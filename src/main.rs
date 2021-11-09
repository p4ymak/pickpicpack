mod app;
mod loader;
mod packer;
mod utils;
use app::*;
use eframe::egui;
use utils::window_width;
fn main() {
    let icon = eframe::epi::IconData {
        rgba: image::open("/home/p4ymak/Work/00_P4/Rust/PickPicPack/icon/icon/PickPicPack.png")
            .unwrap()
            .to_rgba8()
            // .pixels()
            .to_vec(),
        width: 512,
        height: 512,
    };
    let side = window_width(2.0);
    let start_state = P3App::default();
    let options = eframe::NativeOptions {
        always_on_top: true,
        resizable: false,
        initial_window_size: Some(egui::Vec2 { x: side, y: side }),
        drag_and_drop_support: true,
        transparent: true,
        icon_data: Some(icon),
        ..Default::default()
    };
    eframe::run_native(Box::new(start_state), options);
}
