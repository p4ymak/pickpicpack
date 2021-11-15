#![windows_subsystem = "windows"]

mod app;
mod loader;
mod packer;
mod utils;
use app::*;

fn main() {
    let start_state = P3App::default();
    let options = eframe::NativeOptions {
        always_on_top: false,
        decorated: true,
        resizable: true,
        maximized: false,
        drag_and_drop_support: true,
        transparent: true,
        ..Default::default()
    };
    eframe::run_native(Box::new(start_state), options);
}
