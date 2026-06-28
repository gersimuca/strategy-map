mod app;
mod engine;
mod rendering;
mod math;

use app::RtsApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1440.0, 900.0])
            .with_title("Strategy RTS Engine"),
        ..Default::default()
    };
    eframe::run_native(
        "Strategy RTS Engine",
        options,
        Box::new(|cc| {
            let fonts = egui::FontDefinitions::default();
            cc.egui_ctx.set_fonts(fonts);
            Ok(Box::new(RtsApp::new(&cc.egui_ctx)))
        }),
    )
}