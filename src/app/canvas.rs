//! Helper struct for drawing objects in world space onto the screen.
//! This maintains aspect ratio, so `visible_world` is clipped to fill the screen area.

use egui::{Color32, Pos2, Rangef, Rect, Stroke, Ui, epaint::CircleShape};

const SUPPRESS_ZERO_POINTS_THRESHOLD: f32 = 0.005;

pub struct Canvas<'a> {
    ui: &'a Ui,
    screen_extent: Rect,
    range: Rect,
    scale: f32,
}

impl<'a> Canvas<'a> {
    pub fn new(ui: &'a Ui, screen_extent: Rect, visible_x_axis: Rangef) -> Self {
        let y_span = visible_x_axis.span() / screen_extent.aspect_ratio();

        let range = Rect::from_x_y_ranges(visible_x_axis, Rangef::new(-y_span / 2.0, y_span / 2.0));

        let scale = screen_extent.width() / range.width();

        Canvas {
            ui,
            screen_extent,
            range,
            scale,
        }
    }

    fn world_to_screen_pos(&self, pos: &Pos2) -> Pos2 {
        self.screen_extent.min + self.scale * (*pos - self.range.min)
    }

    fn world_to_screen_x(&self, x: f32) -> f32 {
        self.screen_extent.min.x + self.scale * (x - self.range.min.x)
    }

    fn world_to_screen_y(&self, y: f32) -> f32 {
        self.screen_extent.min.y + self.scale * (y - self.range.min.y)
    }

    fn world_to_screen_scale(&self) -> f32 {
        self.scale
    }

    pub fn draw_filled_circle(&self, pos: &Pos2, radius: f32, colour: Color32) {
        let screen_pos = self.world_to_screen_pos(pos);
        let screen_radius = radius * self.world_to_screen_scale();
        self.ui
            .painter()
            .add(CircleShape::filled(screen_pos, screen_radius, colour));
    }

    pub fn draw_grid_lines(&self) {
        const MAX_GRIDLINES: f32 = 20.0;
        let step = (self.range.x_range().span() / MAX_GRIDLINES).round();

        let mut y = step * (self.range.min.y / step).round();
        while y < self.range.max.y {
            self.ui.painter().hline(
                self.screen_extent.x_range(),
                self.world_to_screen_y(y),
                Stroke::new(1.0, Color32::from_rgb(15, 15, 15)),
            );
            y += step;
        }
        let mut x = step * (self.range.min.x / step).round();
        while x < self.range.max.x {
            self.ui.painter().vline(
                self.world_to_screen_x(x),
                self.screen_extent.y_range(),
                Stroke::new(1.0, Color32::from_rgb(15, 15, 15)),
            );
            x += step;
        }
    }

    pub fn draw_axes(&self) {
        self.ui.painter().vline(
            self.world_to_screen_x(0.0),
            self.screen_extent.y_range(),
            Stroke::new(2.0, Color32::from_rgb(20, 20, 20)),
        );
        self.ui.painter().hline(
            self.screen_extent.x_range(),
            self.world_to_screen_y(0.0),
            Stroke::new(2.0, Color32::from_rgb(20, 20, 20)),
        );
    }

    pub fn draw_points(&self, x_points: &[f32], y_points: &[f32], colour: &Color32) {
        if (x_points.len() < 2) || (x_points.len() != y_points.len()) {
            log::error!("Slices passed to draw_points have invalid sizes");
            return;
        }
        let mut screen_points = Vec::with_capacity(x_points.len());
        for (x, y) in x_points.iter().zip(y_points) {
            if y.abs() < SUPPRESS_ZERO_POINTS_THRESHOLD {
                continue;
            }
            screen_points.push(Pos2::new(
                self.world_to_screen_x(*x),
                self.world_to_screen_y(*y),
            ));
        }
        self.ui
            .painter()
            .line(screen_points, Stroke::new(2.0, *colour));
    }
}
