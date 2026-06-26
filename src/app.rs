use eframe::egui;
use crate::engine::{Game, input::handle_input};
use crate::rendering::draw_scene;

pub struct RtsApp {
    game: Game,
}

impl RtsApp {
    pub fn new() -> Self {
        Self {
            game: Game::new(),
        }
    }
}

impl eframe::App for RtsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let input = ui.input(|i| i.clone());
            handle_input(&mut self.game, &input);

            self.game.update();

            let (rect, painter) =
                ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());

            draw_scene(&self.game, rect, painter);
        });

        ctx.request_repaint();
    }
}
