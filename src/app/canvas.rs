//! Helper struct for drawing objects in world space onto the screen.

use egui::{Color32, Pos2, Rangef, Rect, Stroke, Ui, Vec2, epaint::CircleShape, pos2, vec2};

// Don't draw points with absolute y value less than this
const SUPPRESS_ZERO_POINTS_THRESHOLD: f32 = 0.005;

pub struct Canvas<'a> {
    ui: &'a Ui,
    screen_extent: Rect, // screen area to be drawn to
    range: Rect,         // area of simulation to draw from
    scale: Vec2,         // ratios between screen and world space for each axis
}

impl<'a> Canvas<'a> {
    pub fn new(ui: &'a Ui, screen_extent: Rect, visible_x_axis: Rangef) -> Self {
        // calculate world space
        //let y_span = visible_x_axis.span() / screen_extent.aspect_ratio();
        let y_span = 4.0;
        let range = Rect::from_x_y_ranges(visible_x_axis, Rangef::new(-y_span / 2.0, y_span / 2.0));

        let x_scale = screen_extent.width() / range.width();
        let y_scale = screen_extent.height() / range.height();

        Canvas {
            ui,
            screen_extent,
            range,
            scale: vec2(x_scale, y_scale),
        }
    }

    fn world_to_screen_pos(&self, pos: &Pos2) -> Pos2 {
        // convert vector from simulation coords to screen pixel location
        self.screen_extent.min + self.scale * (*pos - self.range.min)
    }

    fn world_to_screen_x(&self, x: f32) -> f32 {
        // convert simulation x coord to screen pixel location
        self.screen_extent.min.x + self.scale.x * (x - self.range.min.x)
    }

    fn world_to_screen_y(&self, y: f32) -> f32 {
        // convert simulation y coord to screen pixel location
        self.screen_extent.min.y + self.scale.y * (y - self.range.min.y)
    }

    fn world_to_screen_scale(&self) -> f32 {
        // when drawing objects with fixed aspect ratio, use x scale for sizing
        self.scale.x
    }

    // draws a circle
    pub fn draw_filled_circle(&self, pos: &Pos2, radius: f32, colour: Color32) {
        let screen_pos = self.world_to_screen_pos(pos);
        let screen_radius = radius * self.world_to_screen_scale();
        self.ui
            .painter()
            .add(CircleShape::filled(screen_pos, screen_radius, colour));
    }

    // draw fine background lines
    pub fn draw_grid_lines(&self) {
        // draw a horizontal line every 0.5 world units
        let mut y = (2.0 * self.range.min.y).round() / 2.0;
        while y < self.range.max.y {
            self.ui.painter().hline(
                self.screen_extent.x_range(),
                self.world_to_screen_y(y),
                Stroke::new(1.0, Color32::from_rgb(15, 15, 15)),
            );
            y += 0.5;
        }

        // try to fit close to this many vertical lines on the screen
        const MAX_GRIDLINES: f32 = 20.0;
        let step = (self.range.x_range().span() / MAX_GRIDLINES).round();
        // starting coordinate
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

    // draw thicker lines at x=0 and y=0
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

    // draw a set of points as a continuous line
    pub fn draw_points(&self, x_points: &[f32], y_points: &[f32], colour: &Color32) {
        // number of elements must match
        if (x_points.len() < 2) || (x_points.len() != y_points.len()) {
            log::error!("Slices passed to draw_points have invalid sizes");
            return;
        }
        // filter out small values for visual clarity
        let mut screen_points = Vec::with_capacity(x_points.len());
        for (x, y) in x_points.iter().zip(y_points) {
            if y.abs() < SUPPRESS_ZERO_POINTS_THRESHOLD {
                continue;
            }
            screen_points.push(pos2(self.world_to_screen_x(*x), self.world_to_screen_y(*y)));
        }
        self.ui
            .painter()
            .line(screen_points, Stroke::new(2.0, *colour));
    }
}
