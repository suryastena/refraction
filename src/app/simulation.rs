//! Contains all simulation logic
#![allow(dead_code)]

use egui::{Pos2, Vec2};
use ndarray::{Array, Array1, Array2, Ix1, Ix2, s};

pub const WORLD_SIZE: f32 = 5.0;
pub const DIVISIONS: usize = 200;
pub const ELECTRON_MASS: f32 = 1.0;
pub const C: f32 = 1.0;
pub const EM_STRENGTH: f32 = 20.0;
pub const SPRING_CONSTANT: f32 = 2.0;
pub const TIME_STEP: f32 = 1.0 / (crate::app::SIMULATION_FPS as f32);

fn index_of(x: f32) -> usize {
    const STEP: f32 = 2.0 * WORLD_SIZE / (DIVISIONS as f32);
    ((x + WORLD_SIZE) / STEP).round() as usize
}

fn field_at(field: &Array1<f32>, x: f32) -> f32 {
    *field.get(index_of(x)).unwrap_or(&0.0)
}

pub struct Electron {
    position: Pos2,
    velocity: f32,
    field: Array1<f32>,
    x_intervals: Array1<f32>,
    retarded_velocity: Array2<f32>,
    frame_in_flight: usize,
}

impl Electron {
    pub fn new(position: Pos2) -> Self {
        Electron {
            position,
            velocity: 0.0,
            x_intervals: Array::linspace(-WORLD_SIZE, WORLD_SIZE, DIVISIONS),
            field: Array::zeros(Ix1(DIVISIONS)),
            retarded_velocity: Array2::zeros(Ix2(2, DIVISIONS)),
            frame_in_flight: 0,
        }
    }

    fn update_induced_field(&mut self) {
        for i in 0..DIVISIONS {
            let r = self.position - Vec2::new(self.x_intervals[i], 0.0);
            if (r.x * r.x + r.y * r.y) < 0.0001 {
                self.field[i] = 0.0;
                continue;
            }
            let vy = self.retarded_velocity()[i];
            let v_perp_y = vy - vy * r.y * r.y / (r.x * r.x + r.y * r.y).powf(1.0);
            self.field[i] = -v_perp_y;
        }
    }

    fn update_position(&mut self, applied_field_strength: f32, time_step: f32) {
        let force = EM_STRENGTH * applied_field_strength - SPRING_CONSTANT * self.position.y;
        self.velocity += time_step * (force / ELECTRON_MASS);
        self.position.y += time_step * (self.velocity);

        // propagate stored seen velocity at each point in space at the speed of light
        for _i in 0..DIVISIONS {}

        self.frame_in_flight = (self.frame_in_flight + 1) % 2;
        let x_pos = self.position.x;
        self.retarded_velocity_mut()[index_of(x_pos)] = self.velocity;
    }

    pub fn position(&self) -> &Pos2 {
        &self.position
    }

    pub fn field(&self) -> &[f32] {
        self.field.slice(s![..]).to_slice().unwrap()
    }

    pub fn retarded_velocity(&self) -> &[f32] {
        self.retarded_velocity
            .row(self.frame_in_flight)
            .to_slice()
            .unwrap()
    }

    pub fn retarded_velocity_mut(&mut self) -> &mut [f32] {
        self.retarded_velocity
            .row_mut(self.frame_in_flight)
            .into_slice()
            .unwrap()
    }

    pub fn retarded_velocity_past(&self) -> &[f32] {
        self.retarded_velocity
            .row((self.frame_in_flight + 1) % 2)
            .to_slice()
            .unwrap()
    }
}

pub struct Simulation {
    pub speed: f32,
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
            speed: 1.0,
            x_intervals: Array::linspace(-WORLD_SIZE, WORLD_SIZE, DIVISIONS),
            applied_field: Array::zeros(Ix1(DIVISIONS)),
            resultant_field: Array::zeros(Ix1(DIVISIONS)),
            electrons: vec![Electron::new(Pos2::new(0.0, 0.0))],
        }
    }

    pub fn size(&self) -> f32 {
        WORLD_SIZE
    }

    pub fn update(&mut self) -> bool {
        self.applied_field = Array::from_vec(self.function_to_points(Self::wave_packet));
        self.resultant_field = Array::zeros(Ix1(DIVISIONS)) + &self.applied_field;

        let time_step = self.time_step();
        for e in &mut self.electrons {
            e.update_position(field_at(&self.applied_field, e.position.x), time_step);
            e.update_induced_field();
            self.resultant_field += &e.field;
        }

        self.t += self.time_step();

        return self.t > (2.0 * WORLD_SIZE / C);
    }

    fn time_step(&self) -> f32 {
        self.speed * TIME_STEP
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
