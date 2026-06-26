use eframe::egui;
use egui::{Color32, Pos2, Rect, Stroke, Vec2};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1000.0, 700.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Strategy Map Editor",
        options,
        Box::new(|_cc| Box::new(MapApp::new())),
    )
}

struct Unit {
    pos: Pos2,
}

struct MapApp {
    camera_offset: Vec2,
    zoom: f32,
    is_dragging: bool,
    last_mouse: Option<Pos2>,
    units: Vec<Unit>,
}

impl MapApp {
    fn new() -> Self {
        Self {
            camera_offset: Vec2::new(0.0, 0.0),
            zoom: 1.0,
            is_dragging: false,
            last_mouse: None,
            units: vec![],
        }
    }

    fn screen_to_world(&self, screen: Pos2) -> Pos2 {
        (screen.to_vec2() - self.camera_offset) / self.zoom
    }

    fn world_to_screen(&self, world: Pos2) -> Pos2 {
        world * self.zoom + self.camera_offset
    }
}

impl eframe::App for MapApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let (response, painter) =
                ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());

            let rect = response.rect;

            // Handle zoom
            if let Some(scroll) = ui.input(|i| i.scroll_delta.y) {
                let zoom_factor = 1.0 + scroll * 0.001;
                self.zoom = (self.zoom * zoom_factor).clamp(0.2, 5.0);
            }

            let mouse_pos = ui.input(|i| i.pointer.hover_pos());

            // Handle panning
            if response.dragged() {
                if let Some(pos) = mouse_pos {
                    if let Some(last) = self.last_mouse {
                        let delta = pos - last;
                        self.camera_offset += delta.to_vec2();
                    }
                    self.last_mouse = Some(pos);
                }
            } else {
                self.last_mouse = mouse_pos;
            }

            if response.drag_stopped() {
                self.last_mouse = None;
            }

            // Click to place unit
            if response.clicked() {
                if let Some(pos) = mouse_pos {
                    let world = self.screen_to_world(pos);
                    self.units.push(Unit { pos: world });
                }
            }

            // Background
            painter.rect_filled(rect, 0.0, Color32::from_rgb(20, 20, 25));

            // Draw grid
            let grid_size = 50.0 * self.zoom;

            let offset_x = self.camera_offset.x % grid_size;
            let offset_y = self.camera_offset.y % grid_size;

            let mut x = rect.left() + offset_x;
            while x < rect.right() {
                painter.line_segment(
                    [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
                    Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 20)),
                );
                x += grid_size;
            }

            let mut y = rect.top() + offset_y;
            while y < rect.bottom() {
                painter.line_segment(
                    [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                    Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 20)),
                );
                y += grid_size;
            }

            // Draw units
            for unit in &self.units {
                let screen = self.world_to_screen(unit.pos);

                painter.circle_filled(
                    screen,
                    6.0,
                    Color32::from_rgb(200, 80, 80),
                );
            }

            // HUD text
            painter.text(
                rect.left_top() + Vec2::new(10.0, 10.0),
                egui::Align2::LEFT_TOP,
                "Drag: pan | Scroll: zoom | Click: place unit",
                egui::FontId::proportional(14.0),
                Color32::WHITE,
            );
        });

        ctx.request_repaint();
    }
}
