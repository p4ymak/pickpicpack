mod app;
mod loader;
mod packer;
use app::*;
use eframe::egui;
use winit::event_loop::EventLoop;

fn window_side() -> f32 {
    if let Some(monitor) = EventLoop::new().primary_monitor() {
        let size = monitor.size();
        let side = (size.width.min(size.height) as f32 / 150.0).round() * 100.0;
        // println!("SIDE: {}", side);
        return side;
    };
    512.0
}

fn main() {
    // let icon = eframe::epi::IconData {
    //     rgba: image::open("/home/p4ymak/Work/00_P4/Rust/PickPicPack/icon/icon/PickPicPack.png")
    //         .unwrap()
    //         .to_rgba8()
    //         // .pixels()
    //         .to_vec(),
    //     width: 512,
    //     height: 512,
    // };
    let side = window_side();
    let start_state = P3App::default();
    let options = eframe::NativeOptions {
        always_on_top: true,
        // maximized: true,
        resizable: false,
        initial_window_size: Some(egui::Vec2 { x: side, y: side }),
        drag_and_drop_support: true,
        transparent: true,
        icon_data: None, //Some(icon),
        ..Default::default()
    };
    eframe::run_native(Box::new(start_state), options);
}
