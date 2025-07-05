//! Initialises the application

mod app;

use egui::{Style, Vec2, Visuals, vec2};

use crate::app::RefractionApp;

fn main() -> eframe::Result {
    env_logger::init();

    const WINDOW_SIZE: Vec2 = vec2(1200.0, 800.0);
    const MIN_WINDOW_SIZE: Vec2 = vec2(100.0, 100.0);

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(WINDOW_SIZE)
            .with_min_inner_size(MIN_WINDOW_SIZE),

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
