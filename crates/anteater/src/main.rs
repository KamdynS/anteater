//! Anteater - A Rust-aware debugger
//!
//! Main binary that composes all the crates together

use anteater_ui::AnteaterApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1600.0, 900.0])
            .with_title("Anteater Debugger"),

        // VSync is enabled by default on most platforms
        // This caps FPS to display refresh rate (usually 60Hz)
        // Set to false for uncapped FPS (useful for testing performance)
        vsync: true,

        // Hardware acceleration is enabled by default
        hardware_acceleration: eframe::HardwareAcceleration::Preferred,

        ..Default::default()
    };

    eframe::run_native(
        "Anteater",
        options,
        Box::new(|cc| Ok(Box::new(AnteaterApp::new(cc)))),
    )
}
