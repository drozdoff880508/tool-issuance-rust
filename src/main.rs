#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

mod data;
mod gui;

use eframe::egui;
use gui::App;

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("Система выдачи инструментов"),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "Tool Issuance System",
        options,
        Box::new(|cc| Box::new(App::new(cc))),
    );
}
