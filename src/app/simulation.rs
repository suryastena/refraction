//! Contains all simulation logic

use egui::{Pos2, Vec2};
use ndarray::{Array, Array1, s};

pub const WORLD_SIZE: f32 = 10.0;
pub const DIVISIONS: usize = 501; // should be odd. not odd odd, just not even.
pub const ELECTRON_MASS: f32 = 10.0;
pub const C: f32 = 2.0;
pub const TIME_STEP: f32 = 1.0 / (crate::app::SIMULATION_FPS as f32);

static_assertions::const_assert!(DIVISIONS % 2 == 1);

fn field_at(field: &Array1<f32>, x: f32) -> f32 {
    let step = 2.0 * WORLD_SIZE / (DIVISIONS as f32);
    let idx = (x + WORLD_SIZE) / step;
    let (upper, lower) = (idx.floor(), idx.ceil());
    return (idx - lower) * field[lower as usize] + (1.0 - idx + lower) * field[upper as usize];
}

pub struct Electron {
    position: Pos2,
    retarded_velocity: Array1<f32>,
    field: Array1<f32>,
    x_intervals: Array1<f32>,
}

impl Electron {
    pub fn new(position: Pos2) -> Self {
        Electron {
            position,
            retarded_velocity: Array::linspace(0.0, 0.0, DIVISIONS),
            field: Array::linspace(0.0, 0.0, DIVISIONS),
            x_intervals: Array::linspace(-WORLD_SIZE, WORLD_SIZE, DIVISIONS),
        }
    }

    fn update_induced_field(&mut self) {
        for i in 0..DIVISIONS {
            if i == DIVISIONS/2 {
                continue;
            }
            let r = self.position - Vec2::new(self.x_intervals[i], 0.0);
            let vy = self.retarded_velocity[i];
            let v_perp_y = vy - vy * r.y * r.y / (r.x * r.x + r.y * r.y).powf(1.5);
            self.field[i] = -v_perp_y;
        }
    }

    fn update_position(&mut self, applied_field_strength: f32) {
        for i in (DIVISIONS / 2 + 1..DIVISIONS).rev() {
            let velocity_in_past = field_at(
                &self.retarded_velocity,
                (self.x_intervals[i] - C * TIME_STEP).min(0.0),
            );
            self.retarded_velocity[i] = velocity_in_past;
        }
        for i in 0..DIVISIONS / 2 {
            let velocity_in_past = field_at(
                &self.retarded_velocity,
                (self.x_intervals[i] + C * TIME_STEP).max(0.0),
            );
            self.retarded_velocity[i] = velocity_in_past;
        }

        self.retarded_velocity[DIVISIONS / 2] += applied_field_strength / ELECTRON_MASS;
        self.position.y += self.retarded_velocity[DIVISIONS / 2];
    }

    pub fn position(&self) -> &Pos2 {
        &self.position
    }

    pub fn field(&self) -> &[f32] {
        self.field.slice(s![..]).to_slice().unwrap()
    }
}

pub struct Simulation {
    t: f32,
    x_intervals: Array1<f32>,
    applied_field: Array1<f32>,
    resultant_field: Array1<f32>,
    electrons: Vec<Electron>,
}

impl Simulation {
    fn wave_packet(x: f32, t: f32) -> f32 {
        let xp = x + C * t - WORLD_SIZE;
        (-(xp * xp)).exp() * (10.0 * xp).sin()
    }

    pub fn new() -> Self {
        Simulation {
            t: 0.0,
            x_intervals: Array::linspace(-WORLD_SIZE, WORLD_SIZE, DIVISIONS),
            applied_field: Array::linspace(0.0, 0.0, DIVISIONS),
            resultant_field: Array::linspace(0.0, 0.0, DIVISIONS),
            electrons: vec![Electron::new(Pos2::new(0.0, 0.0))],
        }
    }

    pub fn size(&self) -> f32 {
        WORLD_SIZE
    }

    pub fn update(&mut self, speed_factor: f32) -> bool {
        self.applied_field = Array::from_vec(self.function_to_points(Self::wave_packet));
        self.resultant_field = Array::linspace(0.0, 0.0, DIVISIONS) + &self.applied_field;

        for e in &mut self.electrons {
            e.update_induced_field();
            self.resultant_field += &e.field;
        }
        for e in &mut self.electrons {
            e.update_position(field_at(&self.resultant_field, e.position.x));
        }

        self.t += speed_factor * TIME_STEP;

        return self.t > (2.0 * WORLD_SIZE / C);
    }

    pub fn electrons(&self) -> &[Electron] {
        &self.electrons
    }

    pub fn time(&self) -> f32 {
        self.t
    }

    pub fn x_intervals(&self) -> &[f32] {
        self.x_intervals.slice(s![..]).to_slice().unwrap()
    }

    pub fn applied_field(&self) -> &[f32] {
        self.applied_field.slice(s![..]).to_slice().unwrap()
    }

    pub fn resultant_field(&self) -> &[f32] {
        self.resultant_field.slice(s![..]).to_slice().unwrap()
    }

    pub fn function_to_points(&self, f: fn(f32, f32) -> f32) -> Vec<f32> {
        let mut v = Vec::with_capacity(DIVISIONS);
        for x in self.x_intervals() {
            v.push(f(*x, self.t));
        }
        return v;
    }
}
