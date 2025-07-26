//! Initialises the application

mod app;
use crate::app::RefractionApp;

use egui::{Pos2, Style, Vec2, Visuals, pos2, vec2};

fn main() -> eframe::Result {
    env_logger::init();

    const WINDOW_POSITION: Pos2 = pos2(50.0, 50.0);
    const WINDOW_SIZE: Vec2 = vec2(1500.0, 900.0);
    const MIN_WINDOW_SIZE: Vec2 = vec2(100.0, 100.0);

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(WINDOW_SIZE)
            .with_min_inner_size(MIN_WINDOW_SIZE)
            .with_position(WINDOW_POSITION),
        ..Default::default()
    };

    eframe::run_native(
        "Refraction",
        native_options,
        Box::new(|cc| {
            let style = Style {
                visuals: Visuals::dark(),
                ..Style::default()
            };
            cc.egui_ctx.set_style(style);
            Ok(Box::new(RefractionApp::new(cc)))
        }),
    )
}
