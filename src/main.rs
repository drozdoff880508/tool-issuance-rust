mod data;
mod gui;

use eframe::egui;
use gui::App;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("Система выдачи инструментов"),
        ..Default::default()
    };

    eframe::run_native(
        "Tool Issuance System",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}
