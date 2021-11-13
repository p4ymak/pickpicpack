mod app;
mod loader;
mod packer;
mod utils;
use app::*;
use eframe::egui;
use utils::{get_screen_size, window_width, WINDOW_SCALE};
fn main() {
    // let icon = eframe::epi::IconData {
    //     rgba: image::open("./icon.png")
    //         .unwrap()
    //         .to_rgba8()
    //         // .pixels()
    //         .to_vec(),
    //     width: 512,
    //     height: 512,
    // };
    let screen_size = get_screen_size();
    let side = window_width(screen_size, WINDOW_SCALE);
    let start_state = P3App::new(screen_size);

    let options = eframe::NativeOptions {
        always_on_top: false,
        decorated: true,
        resizable: true,
        maximized: false,
        initial_window_size: Some(egui::Vec2 { x: side, y: side }),
        drag_and_drop_support: true,
        transparent: true,
        icon_data: None, //Some(icon),
    };
    eframe::run_native(Box::new(start_state), options);
}
