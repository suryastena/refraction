//! Contains all application code, including application state and drawing logic

mod canvas;
mod simulation;

use canvas::Canvas;
use simulation::Simulation;

use egui::{Color32, Pos2, Rect, Style};
use std::time::SystemTime;

pub struct RefractionApp {
    simulation: Simulation,
    paused: bool,
    frame: u32,
    speed: f32,
    zoom: f32,
    last_60_frames_start: SystemTime,
    last_60_frames_time_micros: f32,
}

impl RefractionApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            simulation: Simulation::new(),
            paused: true,
            frame: 0,
            speed: 1.0,
            zoom: 1.0,
            last_60_frames_start: SystemTime::now(),
            last_60_frames_time_micros: 1e6,
        }
    }
}

impl eframe::App for RefractionApp {
    /// Called by the framework to save state before shutdown.

    /// Called each time the UI needs repainting
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.frame % 60 == 59 {
            self.last_60_frames_time_micros =
                self.last_60_frames_start.elapsed().unwrap().as_micros() as f32;
            self.last_60_frames_start = SystemTime::now();
        }

        if !self.paused {
            self.simulation.update(self.speed);
        }

        // draws simulation controls at the bottom of the window
        let controls = egui::TopBottomPanel::bottom("controls");
        let r = controls.show(ctx, |ui| {

            ui.horizontal(|ui| {
                if ui.button("▶").clicked() {
                    self.paused = false;
                }
                if ui.button("⏸").clicked() {
                    self.paused = true;
                }
                if ui.button("⏭").clicked() {
                    if self.paused {                    
                        self.simulation.update(self.speed);
                    }
                }
                if ui.button("⟲").clicked() {
                    self.paused = true;
                    self.simulation = Simulation::new();
                    self.frame = 0;
                }
                ui.separator();
                ui.label("Speed");
                ui.add(egui::Slider::new(&mut self.speed, 0.1..=10.0));
                if ui.button("↺").clicked() {
                    self.speed = 1.0;
                }
                ui.separator();
                ui.label("Zoom");
                ui.add(egui::Slider::new(&mut self.zoom, 0.1..=10.0));    
                if ui.button("↺").clicked() {
                    self.zoom = 1.0;
                }            
                ui.separator();  
                ui.label(format!("{0:.0} FPS", 6e7 / self.last_60_frames_time_micros));
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

                canvas.draw_filled_circle(Pos2::new(0.0, 0.0), 0.25, Color32::from_rgb(255, 175, 0));
            });

        self.frame += 1;

        //ctx.request_repaint();
    }
}
