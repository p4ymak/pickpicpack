#![windows_subsystem = "windows"]

mod app;
mod loader;
mod packer;
mod utils;
use app::*;

fn main() {
    let icon = eframe::epi::IconData {
        rgba: image::open("./icon/128x128@2x.png")
            .unwrap()
            .to_rgba8()
            .to_vec(),
        width: 256,
        height: 256,
    };
    let start_state = P3App::default();
    let options = eframe::NativeOptions {
        always_on_top: false,
        decorated: true,
        resizable: true,
        maximized: false,
        drag_and_drop_support: true,
        transparent: true,
        icon_data: Some(icon),
        ..Default::default()
    };
    eframe::run_native(Box::new(start_state), options);
}
