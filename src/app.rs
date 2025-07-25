//! Contains all application code, including application state and drawing logic

mod canvas;
mod simulation;

use canvas::Canvas;
use simulation::variables::{ELECTRON_DAMPING, ELECTRON_MASS, ELECTRON_SPACING, SPRING_CONSTANT};
use simulation::{Simulation, Waveform};

use egui::{Color32, Rangef, Rect, Response, Sense, Style, pos2};
use std::time::SystemTime;

const MONITOR_REFRESH_RATE: u32 = 60;
const SIMULATION_FPS: u32 = if MONITOR_REFRESH_RATE < 60 {
    MONITOR_REFRESH_RATE
} else {
    60
};

fn zoom_to(range: &Rangef, zoom: f32, centre: f32) -> Rangef {
    Rangef {
        min: centre - range.span() / (2.0 * zoom),
        max: centre + range.span() / (2.0 * zoom),
    }
}

fn electron_colour(a: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(255, 175, 0, (a * a * 255.0) as u8)
}
fn electron_field_colour(a: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(20, 100, 255, (a * a * 255.0) as u8)
}
fn applied_field_colour(a: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(255, 50, 50, (a * a * 255.0) as u8)
}
fn resultant_field_colour(a: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(180, 20, 180, (a * a * 255.0) as u8)
}

pub struct RefractionApp {
    simulation: Simulation,
    paused: bool,
    speed: f32,
    requested_frames: f32,
    frame: u32,
    zoom: f32,
    world_centre: f32,
    zoom_centre: Option<f32>,
    dragging: Option<f32>,
    frame_skip: u32,
    last_n_frames_start: SystemTime,
    last_n_frames_time_micros: f32,

    applied_field_opacity: f32,
    resultant_field_opacity: f32,
    electron_field_opacity: f32,
}

impl RefractionApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let simulation = Simulation::new(Waveform::GaussianPacket);
        let world_centre = simulation.size().center();
        Self {
            simulation,
            paused: true,

            speed: 1.0,
            requested_frames: 0.0,
            frame: 0,
            frame_skip: SIMULATION_FPS / 5,
            last_n_frames_start: SystemTime::now(),
            last_n_frames_time_micros: 1e6,

            world_centre,
            zoom: 1.0,
            zoom_centre: None,
            dragging: None,

            applied_field_opacity: 0.8,
            resultant_field_opacity: 0.7,
            electron_field_opacity: 0.2,
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
            self.requested_frames += self.speed;
            while (self.frame as f32) < self.requested_frames {
                if self.simulation.update() {
                    // sim complete, reset
                    self.paused = true;
                    self.simulation.reset();
                    self.frame = 0;
                }
                self.frame += 1;
            }
        }

        let electron_count = self.simulation.electron_count;
        let electron_spacing = self.simulation.electron_spacing;

        // draws simulation controls at the bottom of the window
        let settings = egui::TopBottomPanel::top("settings");
        let settings_drawn: Response = settings
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Waveform:");
                    egui::ComboBox::from_id_salt("Wave")
                        .selected_text(format!("{:?}", &self.simulation.waveform))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.simulation.waveform,
                                Waveform::GaussianPacket,
                                format!("{:?}", Waveform::GaussianPacket),
                            );
                            ui.selectable_value(
                                &mut self.simulation.waveform,
                                Waveform::Gaussian,
                                format!("{:?}", Waveform::Gaussian),
                            );
                            ui.selectable_value(
                                &mut self.simulation.waveform,
                                Waveform::PlaneWave,
                                format!("{:?}", Waveform::PlaneWave),
                            );
                        });

                    ui.separator();

                    let max_e = self.simulation.max_electrons();
                    ui.label("Electrons:");
                    ui.add(
                        egui::DragValue::new(&mut self.simulation.electron_count).range(1..=max_e),
                    );

                    ui.label("Spacing:");
                    ui.add(egui::Slider::new(
                        &mut self.simulation.electron_spacing,
                        ELECTRON_SPACING.min..=ELECTRON_SPACING.max,
                    ));
                    if ui.button("↺").on_hover_text("Reset electrons").clicked() {
                        self.simulation.electron_count = 1;
                        self.simulation.electron_spacing = ELECTRON_SPACING.default;
                    }

                    ui.separator();

                    ui.label("M").on_hover_text("Particle mass");
                    ui.add(egui::Slider::new(
                        &mut self.simulation.electron_mass,
                        ELECTRON_MASS.min..=ELECTRON_MASS.max,
                    ));
                    if ui.button("↺").on_hover_text("Reset").clicked() {
                        self.simulation.electron_mass = ELECTRON_MASS.default;
                    }

                    ui.separator();

                    ui.label("k").on_hover_text("Particle spring constant");
                    ui.add(egui::Slider::new(
                        &mut self.simulation.spring_constant,
                        SPRING_CONSTANT.min..=SPRING_CONSTANT.max,
                    ));
                    if ui.button("↺").on_hover_text("Reset").clicked() {
                        self.simulation.spring_constant = SPRING_CONSTANT.default;
                    }

                    ui.separator();

                    ui.label("Damping")
                        .on_hover_text("Electron motion damping factor");
                    ui.add(egui::Slider::new(
                        &mut self.simulation.damping,
                        ELECTRON_DAMPING.min..=ELECTRON_DAMPING.max,
                    ));
                    if ui.button("↺").on_hover_text("Reset").clicked() {
                        self.simulation.damping = ELECTRON_DAMPING.default;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Opacities:");
                    ui.label(egui::RichText::new("◼").color(applied_field_colour(0.7)));
                    ui.add(egui::Slider::new(
                        &mut self.applied_field_opacity,
                        0.0..=1.0,
                    ));
                    ui.label(egui::RichText::new("◼").color(resultant_field_colour(0.7)));
                    ui.add(egui::Slider::new(
                        &mut self.resultant_field_opacity,
                        0.0..=1.0,
                    ));
                    ui.label(egui::RichText::new("◼").color(electron_field_colour(0.7)));
                    ui.add(egui::Slider::new(
                        &mut self.electron_field_opacity,
                        0.0..=1.0,
                    ));
                });
            })
            .response;

        let controls = egui::TopBottomPanel::bottom("controls");
        let controls_drawn: Response = controls
            .show(ctx, |ui| {
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
                        self.simulation.reset();
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

                    ui.label("Speed");
                    ui.add(egui::Slider::new(&mut self.speed, 0.1..=10.0));
                    if ui.button("↺").on_hover_text("Reset").clicked() {
                        self.speed = 1.0;
                    }

                    ui.separator();

                    ui.label("Zoom");
                    ui.add(egui::Slider::new(&mut self.zoom, 1.0..=10.0));
                    if ui.button("↺").on_hover_text("Reset view").clicked() {
                        self.zoom = 1.0;
                        self.world_centre = self.simulation.size().center();
                        self.zoom_centre = None;
                        self.dragging = None;
                    }

                    ui.separator();

                    ui.label(format!("{0:.0} FPS", 6e7 / self.last_n_frames_time_micros));
                });
            })
            .response;

        let electron_count_changed = self.simulation.electron_count != electron_count;
        let electron_spacing_changed = self.simulation.electron_spacing != electron_spacing;

        if electron_count_changed || electron_spacing_changed {
            self.simulation.update_electrons(electron_spacing_changed);
        }

        let canvas_extent = Rect::from_two_pos(
            pos2(ctx.screen_rect().left(), settings_drawn.rect.bottom()),
            pos2(ctx.screen_rect().right(), controls_drawn.rect.top()),
        );

        let pointer_pos = ctx.pointer_latest_pos().unwrap_or(pos2(0.0, 0.0));
        let mut visible_world = zoom_to(self.simulation.size(), self.zoom, self.world_centre);
        let pointer_world_pos = ((pointer_pos.x - canvas_extent.left()) * visible_world.span()
            / canvas_extent.width())
            + visible_world.min;

        let mut scroll_delta: f32 = 0.0;
        ctx.input(|input| {
            scroll_delta = input.smooth_scroll_delta.y;
        });
        if scroll_delta == 0.0 && self.zoom_centre.is_some() {
            self.zoom_centre = None;
        }
        if scroll_delta != 0.0 {
            if canvas_extent.contains(pointer_pos) {
                let future_visible_world = zoom_to(
                    self.simulation.size(),
                    (self.zoom + scroll_delta / 100.0).max(1.0),
                    self.world_centre,
                );

                if self.zoom_centre.is_none() {
                    self.zoom_centre = Some(pointer_world_pos);
                }

                self.world_centre = self
                    .zoom_centre
                    .unwrap()
                    .min(self.simulation.size().max - future_visible_world.span() / 2.0)
                    .max(self.simulation.size().min + future_visible_world.span() / 2.0);

                self.zoom = (self.zoom + scroll_delta / 100.0).max(1.0);
            }
        }

        visible_world = zoom_to(self.simulation.size(), self.zoom, self.world_centre);

        // draws the simulation in the main panel of the window
        let style = Style::default();
        let _ = egui::CentralPanel::default()
            .frame(egui::Frame::canvas(&style))
            .show(ctx, |ui| {
                let canvas = Canvas::new(ui, canvas_extent, visible_world);

                if ui
                    .interact(canvas_extent, egui::Id::new("canvas-drag"), Sense::drag())
                    .dragged()
                {
                    let diff = self.dragging.unwrap_or(pointer_pos.x) - pointer_pos.x;
                    self.world_centre += diff * visible_world.span() / canvas_extent.width();
                    self.world_centre = self
                        .world_centre
                        .min(self.simulation.size().max - visible_world.span() / 2.0)
                        .max(self.simulation.size().min + visible_world.span() / 2.0);
                    self.dragging = Some(pointer_pos.x);
                } else {
                    self.dragging = None;
                }

                canvas.draw_grid_lines();
                canvas.draw_axes();

                for electron in self.simulation.electrons() {
                    canvas.draw_filled_circle(electron.position(), 0.25, electron_colour(1.0));
                    canvas.draw_points(
                        self.simulation.x_intervals(),
                        electron.field(),
                        &electron_field_colour(self.electron_field_opacity),
                    );
                }

                canvas.draw_points(
                    self.simulation.x_intervals(),
                    self.simulation.applied_field(),
                    &applied_field_colour(self.applied_field_opacity),
                );

                canvas.draw_points(
                    self.simulation.x_intervals(),
                    self.simulation.resultant_field(),
                    &resultant_field_colour(self.resultant_field_opacity),
                );
            })
            .response;

        ctx.request_repaint();
    }
}
