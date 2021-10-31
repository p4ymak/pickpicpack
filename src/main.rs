mod app;
mod loader;
mod packer;
use app::*;
use eframe::egui;

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
    let start_state = P3App::default();
    let options = eframe::NativeOptions {
        always_on_top: true,
        // maximized: true,
        resizable: false,
        initial_window_size: Some(egui::Vec2 { x: 500.0, y: 500.0 }),
        drag_and_drop_support: true,
        transparent: true,
        icon_data: Some(icon),
        ..Default::default()
    };
    eframe::run_native(Box::new(start_state), options);
}
