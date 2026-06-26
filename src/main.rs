mod app;
mod engine;
mod rendering;
mod math;

use app::RtsApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };

    eframe::run_native(
        "RTS Engine",
        options,
        Box::new(|_cc| Box::new(RtsApp::new())),
    )
}
