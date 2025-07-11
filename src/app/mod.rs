//! Contains all application code, including application state and drawing logic

mod canvas;
mod simulation;

use canvas::Canvas;
use simulation::Simulation;

use egui::{Color32, Pos2, Rect, Style};
use std::time::SystemTime;

const MONITOR_REFRESH_RATE: u32 = 60;
const SIMULATION_FPS: u32 = if MONITOR_REFRESH_RATE < 60 {
    MONITOR_REFRESH_RATE
} else {
    60
};

pub struct RefractionApp {
    simulation: Simulation,
    paused: bool,
    frame: u32,
    zoom: f32,
    frame_skip: u32,
    last_n_frames_start: SystemTime,
    last_n_frames_time_micros: f32,
}

impl RefractionApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            simulation: Simulation::new(),
            paused: true,
            frame: 0,
            zoom: 1.0,
            frame_skip: SIMULATION_FPS / 5,
            last_n_frames_start: SystemTime::now(),
            last_n_frames_time_micros: 1e6,
        }
    }
}

impl eframe::App for RefractionApp {
    /// Called by the framework to save state before shutdown.

    /// Called each time the UI needs repainting
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.frame % SIMULATION_FPS == SIMULATION_FPS - 1 {
            self.last_n_frames_time_micros =
                self.last_n_frames_start.elapsed().unwrap().as_micros() as f32;
            self.last_n_frames_start = SystemTime::now();
        }

        if !self.paused && (self.frame % MONITOR_REFRESH_RATE / SIMULATION_FPS == 0) {
            if self.simulation.update() {
                // sim complete, reset
                self.paused = true;
                self.simulation = Simulation::new();
                self.frame = 0;
            }
            self.frame += 1;
        }

        // draws simulation controls at the bottom of the window
        let controls = egui::TopBottomPanel::bottom("controls");
        let r = controls.show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(self.paused, egui::Button::new("▶"))
                    .on_hover_text("Play simulation")
                    .clicked()
                {
                    self.paused = false;
                }
                if ui
                    .add_enabled(!self.paused, egui::Button::new("⏸"))
                    .on_hover_text("Pause simulation")
                    .clicked()
                {
                    self.paused = true;
                }
                if ui
                    .add_enabled(self.simulation.time() > 0.0, egui::Button::new("⟲"))
                    .on_hover_text("Restart simulation")
                    .clicked()
                {
                    self.paused = true;
                    self.simulation = Simulation::new();
                    self.frame = 0;
                }
                if ui
                    .add_enabled(self.paused, egui::Button::new("⏭"))
                    .on_hover_text("Advance simulation by one step")
                    .clicked()
                {
                    if self.paused {
                        for _ in 0..self.frame_skip {
                            self.simulation.update();
                        }
                    }
                }
                ui.add(egui::DragValue::new(&mut self.frame_skip))
                    .on_hover_text("Number of frames to advance per step");
                ui.separator();
                ui.label("Speed").on_hover_text(
                    "Warning: changing speed during a simulation run is not stable!",
                );
                ui.add(egui::Slider::new(&mut self.simulation.speed, 0.1..=10.0))
                    .on_hover_text(
                        "Warning: changing speed during a simulation run is not stable!",
                    );
                if ui.button("↺").on_hover_text("Reset").clicked() {
                    self.simulation.speed = 1.0;
                }
                ui.separator();
                ui.label("Zoom");
                ui.add(egui::Slider::new(&mut self.zoom, 0.1..=10.0));
                if ui.button("↺").on_hover_text("Reset").clicked() {
                    self.zoom = 1.0;
                }
                ui.separator();
                ui.label(format!("{0:.0} FPS", 6e7 / self.last_n_frames_time_micros));
            });
        });

        let canvas_extent = Rect::from_two_pos(
            Pos2::new(ctx.screen_rect().left(), ctx.screen_rect().top()),
            Pos2::new(ctx.screen_rect().right(), r.response.rect.top()),
        );
        let world_width = 2.0 * self.simulation.size() / self.zoom;

        // draws the simulation in the main panel of the window
        let style = Style::default();
        egui::CentralPanel::default()
            .frame(egui::Frame::canvas(&style))
            .show(ctx, |ui| {
                let canvas = Canvas::from_centre(ui, canvas_extent, world_width);

                canvas.draw_grid_lines();
                canvas.draw_axes();

                for electron in self.simulation.electrons() {
                    canvas.draw_filled_circle(
                        electron.position(),
                        0.25,
                        Color32::from_rgb(255, 175, 0),
                    );
                    canvas.draw_points(
                        self.simulation.x_intervals(),
                        electron.retarded_velocity(),
                        &Color32::from_rgb(0, 255, 0),
                    );
                    /*
                    canvas.draw_points(
                        self.simulation.x_intervals(),
                        electron.field(),
                        &Color32::from_rgb(20, 100, 255),
                    );
                    */
                }

                canvas.draw_points(
                    self.simulation.x_intervals(),
                    self.simulation.applied_field(),
                    &Color32::from_rgb(255, 50, 50),
                );
                /*
                canvas.draw_points(
                    self.simulation.x_intervals(),
                    self.simulation.resultant_field(),
                    &Color32::from_rgb(180, 20, 180),
                );
                */

                //canvas.draw_function(f32::sin, &Color32::from_rgb(255, 0, 0));
                //canvas.draw_function(f32::cos, &Color32::from_rgb(0, 0, 255));
            });

        ctx.request_repaint();
    }
}
