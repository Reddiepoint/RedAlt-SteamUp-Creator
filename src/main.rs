#![windows_subsystem = "windows"]
mod modules;

use eframe::egui::ViewportBuilder;

use crate::modules::app::RedAltSteamUpCreator;

fn main() {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_drag_and_drop(true)
            .with_resizable(true)
            .with_inner_size((800.0, 900.0)),
        centered: true,
        ..Default::default()
    };

    let _ = eframe::run_native(
        "RedAlt SteamUp Creator",
        options,
        Box::new(|_cc| Box::<RedAltSteamUpCreator>::default()),
    );
}
