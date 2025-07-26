//! Contains all application code, including application state and drawing logic

mod canvas;
mod simulation;

use canvas::Canvas;
use simulation::variables::{ELECTRON_DAMPING, ELECTRON_MASS, ELECTRON_SPACING, SPRING_CONSTANT};
use simulation::{Simulation, Waveform};

use egui::{Color32, Rangef, Rect, Response, Sense, Style, pos2};
use std::time::SystemTime;
use strum::IntoEnumIterator;

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
            requested_frames: 1.0,
            frame: 1,
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

        // advance simulation when not paused and with (hardcoded) vsync in order to not run sim faster than app can draw
        if !self.paused && (self.frame % MONITOR_REFRESH_RATE / SIMULATION_FPS == 0) {
            // speed works by allowing a fractional number of requested frames per update.
            // this means that when using a speed different to 1, each redraw may have a varying number of simulation updates.
            self.requested_frames += self.speed;
            while (self.frame as f32) < self.requested_frames {
                // update sim until frame number satisfies requests
                if self.simulation.update() {
                    // sim complete, reset
                    self.paused = true;
                    self.simulation.reset();
                    self.frame = 0;
                    self.requested_frames = 0.0;
                }
                self.frame += 1;
            }
        }

        // recorded for checking if any change to these this redraw -
        // only want to update sim when these values change as it's an expensive thing to do
        let electron_count = self.simulation.electron_count;
        let electron_spacing = self.simulation.electron_spacing;

        // draws simulation settings at the top of the window
        let settings = egui::TopBottomPanel::top("settings");
        let settings_drawn: Response = settings
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // dropdown to select applied wave type
                    ui.label("Waveform:");
                    egui::ComboBox::from_id_salt("Wave")
                        .selected_text(format!("{:?}", &self.simulation.waveform))
                        .show_ui(ui, |ui| {
                            for form in Waveform::iter() {
                                ui.selectable_value(
                                    &mut self.simulation.waveform,
                                    form,
                                    format!("{:?}", form),
                                );
                            }
                        });

                    ui.separator();

                    // number of electons, allow only the amount that can appear onscreen at once
                    let max_e = self.simulation.max_electrons();
                    ui.label("Electrons:");
                    ui.add(
                        egui::DragValue::new(&mut self.simulation.electron_count).range(1..=max_e),
                    );

                    // distance between each electron
                    ui.label("Spacing:");
                    ui.add(egui::Slider::new(
                        &mut self.simulation.electron_spacing,
                        ELECTRON_SPACING.min..=ELECTRON_SPACING.max,
                    ));
                    if ui.button("↺").on_hover_text("Reset electrons").clicked() {
                        self.simulation.electron_count = 1;
                        self.simulation.electron_spacing = ELECTRON_SPACING.initial;
                    }

                    ui.separator();

                    // electron properties

                    ui.label("M").on_hover_text("Particle mass");
                    ui.add(egui::Slider::new(
                        &mut self.simulation.electron_mass,
                        ELECTRON_MASS.min..=ELECTRON_MASS.max,
                    ));
                    if ui.button("↺").on_hover_text("Reset").clicked() {
                        self.simulation.electron_mass = ELECTRON_MASS.initial;
                    }

                    ui.separator();

                    ui.label("k").on_hover_text("Particle spring constant");
                    ui.add(egui::Slider::new(
                        &mut self.simulation.spring_constant,
                        SPRING_CONSTANT.min..=SPRING_CONSTANT.max,
                    ));
                    if ui.button("↺").on_hover_text("Reset").clicked() {
                        self.simulation.spring_constant = SPRING_CONSTANT.initial;
                    }

                    ui.separator();

                    ui.label("Damping")
                        .on_hover_text("Electron motion damping factor");
                    ui.add(egui::Slider::new(
                        &mut self.simulation.damping,
                        ELECTRON_DAMPING.min..=ELECTRON_DAMPING.max,
                    ));
                    if ui.button("↺").on_hover_text("Reset").clicked() {
                        self.simulation.damping = ELECTRON_DAMPING.initial;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Field opacities:");
                    ui.label(egui::RichText::new("◼").color(applied_field_colour(0.7)))
                        .on_hover_text("Initial electric field");
                    ui.add(egui::Slider::new(
                        &mut self.applied_field_opacity,
                        0.0..=1.0,
                    ));
                    ui.label(egui::RichText::new("◼").color(resultant_field_colour(0.7)))
                        .on_hover_text("Resultant electric field");
                    ui.add(egui::Slider::new(
                        &mut self.resultant_field_opacity,
                        0.0..=1.0,
                    ));
                    ui.label(egui::RichText::new("◼").color(electron_field_colour(0.7)))
                        .on_hover_text("Induced electric field of particles");
                    ui.add(egui::Slider::new(
                        &mut self.electron_field_opacity,
                        0.0..=1.0,
                    ));
                });
            })
            .response;

        // draws simulation controls at the bottom of the window
        let controls = egui::TopBottomPanel::bottom("controls");
        let controls_drawn: Response = controls
            .show(ctx, |ui| {
                ui.horizontal(|ui| {

                    if ui
                        .add( egui::Button::new(if self.paused {"▶"} else {"⏸"}))
                        .on_hover_text(if self.paused {"Play simulation"} else {"Pause simulation"})
                        .clicked()
                    {
                        self.paused = !self.paused;
                    }
                    // button for stepping the simulation by a configurable number of updates
                    if ui
                        .add_enabled(self.paused, egui::Button::new("⏭"))
                        .on_hover_text("Advance simulation by a number of frames")
                        .clicked()
                    {
                        if self.paused {
                            for _ in 0..self.frame_skip {
                                self.simulation.update();
                            }
                        }
                    }
                    ui.add(egui::DragValue::new(&mut self.frame_skip))
                        .on_hover_text("Number of updates to advance per step");

                    ui.label(format!("{0:.2}s @ {1}", self.simulation.time(), self.frame)).on_hover_text("[Elapsed time]s @ [number of frames]");

                    if ui
                        .add_enabled(self.simulation.time() > 0.0, egui::Button::new("⟲"))
                        .on_hover_text("Restart simulation")
                        .clicked()
                    {
                        self.paused = true;
                        self.frame = 0;
                        self.requested_frames = 0.0;
                        self.simulation.reset();
                    }

                    ui.separator();

                    // ratio of simulation UPS to screen FPS
                    ui.label("Speed");
                    ui.add(egui::Slider::new(&mut self.speed, 0.1..=10.0));
                    if ui.button("↺").on_hover_text("Reset").clicked() {
                        self.speed = 1.0;
                    }

                    ui.separator();

                    ui.label("Zoom").on_hover_text("You can also zoom using the mouse wheel, and move around by dragging with the mouse.");
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

        // adds/removes/modifies electrons only if required
        let electron_count_changed = self.simulation.electron_count != electron_count;
        let electron_spacing_changed = self.simulation.electron_spacing != electron_spacing;
        if electron_count_changed || electron_spacing_changed {
            self.simulation.update_electrons(electron_spacing_changed);
        }

        // the space on the screen in points between the settings/control bars
        let canvas_extent = Rect::from_two_pos(
            pos2(ctx.screen_rect().left(), settings_drawn.rect.bottom()),
            pos2(ctx.screen_rect().right(), controls_drawn.rect.top()),
        );

        // dimensions on the x axis of the amount of the simulation that is visible on-screen
        let mut visible_world = zoom_to(self.simulation.size(), self.zoom, self.world_centre);

        // position of mouse pointer
        let pointer_pos = ctx.pointer_latest_pos().unwrap_or(pos2(0.0, 0.0)); // in screen space (measured in points)
        let pointer_world_pos = ((pointer_pos.x - canvas_extent.left()) * visible_world.span() // in world space (measured in simulation units)
            / canvas_extent.width())
            + visible_world.min;

        // get the amount the user has scrolled the mouse wheel this frame
        let mut scroll_delta: f32 = 0.0;
        ctx.input(|input| {
            scroll_delta = input.smooth_scroll_delta.y;
        });
        if scroll_delta == 0.0 && self.zoom_centre.is_some() {
            // no scrolling, stop remembering the centre we were zooming into
            self.zoom_centre = None;
        }
        if scroll_delta != 0.0 {
            if canvas_extent.contains(pointer_pos) {
                // the dimensions of the visible part of the simulation after this frame's zoom is applied
                let future_visible_world = zoom_to(
                    self.simulation.size(),
                    (self.zoom + scroll_delta / 100.0).max(1.0),
                    self.world_centre,
                );

                // If this is the first frame of a zoom action, remember the centre we are zooming on.
                // This is because the mouse pointer will change where in world space it is located but we want to stay
                // locked onto the same centre for the whole zoom action.
                if self.zoom_centre.is_none() {
                    self.zoom_centre = Some(pointer_world_pos);
                }

                // if zoom centre is near the edges, clamp centre so no no part of the canvas is outside the simulation's bounds
                self.world_centre = self
                    .zoom_centre
                    .unwrap()
                    .min(self.simulation.size().max - future_visible_world.span() / 2.0)
                    .max(self.simulation.size().min + future_visible_world.span() / 2.0);

                // change zoom level based on scroll amount
                self.zoom = (self.zoom + scroll_delta / 100.0).max(1.0);
            }

            // apply new zoom
            visible_world = zoom_to(self.simulation.size(), self.zoom, self.world_centre);
        }

        // draws the simulation in the main panel of the window
        let style = Style::default();
        let _ = egui::CentralPanel::default()
            .frame(egui::Frame::canvas(&style))
            .show(ctx, |ui| {
                // this class draws objects in screen space based on coordinates given in simulation (world) space
                let canvas = Canvas::new(ui, canvas_extent, visible_world);

                // detects user dragging canvas with the mouse and shifts visible world accordingly
                if ui
                    .interact(canvas_extent, egui::Id::new("canvas-drag"), Sense::drag())
                    .dragged()
                {
                    // get shift in pointer based on remembered mouse position last frame
                    let diff = self.dragging.unwrap_or(pointer_pos.x) - pointer_pos.x;
                    // Change world centre, changing from screen space diff to world space diff
                    // This will apply on the next frame, due to necessary 1 frame delay to calculate initial mouse movement from start of dragging
                    self.world_centre += diff * visible_world.span() / canvas_extent.width();
                    // clamp centre so no part of the canvas is outside simulation bounds
                    self.world_centre = self
                        .world_centre
                        .min(self.simulation.size().max - visible_world.span() / 2.0)
                        .max(self.simulation.size().min + visible_world.span() / 2.0);
                    // remember pointer position for next frame
                    self.dragging = Some(pointer_pos.x);
                } else {
                    // drag ended, stop remembering mouse position
                    self.dragging = None;
                }

                // draw lines on the canvas
                canvas.draw_grid_lines();
                canvas.draw_axes();

                // draw electrons and fields
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

        // immediately redraw so simulation is constantly updated as fast as monitor refresh
        ctx.request_repaint();
    }
}
