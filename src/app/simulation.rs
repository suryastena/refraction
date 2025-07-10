//! Contains all simulation logic

use egui::{Pos2, Vec2};
use ndarray::{Array, Array1, s};

pub const WORLD_SIZE: f32 = 5.0;
pub const DIVISIONS: usize = 200;
pub const ELECTRON_MASS: f32 = 1.0;
pub const C: f32 = 1.0;
pub const EM_STRENGTH: f32 = 20.0;
pub const SPRING_CONSTANT: f32 = 2.0;
pub const TIME_STEP: f32 = 1.0 / (crate::app::SIMULATION_FPS as f32);

fn bounded_field_at(
    field: &Array1<f32>,
    x: f32,
    lower_bound: (f32, f32),
    upper_bound: (f32, f32),
) -> f32 {
    const STEP: f32 = 2.0 * WORLD_SIZE / (DIVISIONS as f32);
    let space_to_index = |pos: f32| (pos + WORLD_SIZE) / STEP;

    let idx = space_to_index(x);
    let lower_bound_idx = space_to_index(lower_bound.0);
    let upper_bound_idx = space_to_index(upper_bound.0);

    let bounded_field = |index: f32| -> &f32 {
        if index <= lower_bound_idx {
            return &lower_bound.1;
        }
        if index >= upper_bound_idx {
            return &upper_bound.1;
        }
        return field.get(index as usize).unwrap_or(&0.0);
    };

    let (lower_idx, upper_idx) = (idx.floor(), idx.ceil());
    let lower = bounded_field(lower_idx);
    let upper = bounded_field(upper_idx);
    
    return (idx - lower_idx) * lower + (1.0 - idx + lower_idx) * upper;
}

fn field_at(field: &Array1<f32>, x: f32) -> f32 {
    bounded_field_at(field, x, (-WORLD_SIZE, 0.0), (WORLD_SIZE, 0.0))
}

pub struct Electron {
    position: Pos2,
    velocity: f32,
    retarded_velocity: Array1<f32>,
    field: Array1<f32>,
    x_intervals: Array1<f32>,
}

impl Electron {
    pub fn new(position: Pos2) -> Self {
        Electron {
            position,
            velocity: 0.0,
            retarded_velocity: Array::linspace(0.0, 0.0, DIVISIONS),
            field: Array::linspace(0.0, 0.0, DIVISIONS),
            x_intervals: Array::linspace(-WORLD_SIZE, WORLD_SIZE, DIVISIONS),
        }
    }

    fn update_induced_field(&mut self) {
        for i in 0..DIVISIONS {
            let r = self.position - Vec2::new(self.x_intervals[i], 0.0);
            if (r.x * r.x + r.y * r.y) < 0.001 {
                self.field[i] = 0.0;
                continue;
            }
            let vy = self.retarded_velocity[i];
            let v_perp_y = vy - vy * r.y * r.y / (r.x * r.x + r.y * r.y).powf(1.0);
            self.field[i] = -v_perp_y;
        }
    }

    fn update_position(&mut self, applied_field_strength: f32, time_step: f32) {
        // propagate stored seen velocity at each point in space at the speed of light

        for i in 0..DIVISIONS {
            let x = self.x_intervals[i];
            if x > self.position.x {
                break;
            }
            log::info!("{}", i);
            let past_x = x + C * time_step;
            let ret_v = bounded_field_at(
                    &self.retarded_velocity,
                    past_x,
                    (-WORLD_SIZE, 0.0),
                    (self.position.x, self.velocity),
                );
            self.retarded_velocity[i] = ret_v;
        }
        for i in (0..DIVISIONS).rev() {
            let x = self.x_intervals[i];
            if x < self.position.x {
                break;
            }
            log::info!("{}", i);
            let past_x = x - C * time_step;
            let ret_v = bounded_field_at(
                    &self.retarded_velocity,
                    past_x,
                    (self.position.x, self.velocity),
                    (WORLD_SIZE, 0.0),
                );
            self.retarded_velocity[i] = ret_v;
        }
        log::info!("========================");

        let force = EM_STRENGTH * applied_field_strength - SPRING_CONSTANT * self.position.y;
        self.velocity += time_step * (force / ELECTRON_MASS);
        self.position.y += time_step * (self.velocity);
    }

    pub fn position(&self) -> &Pos2 {
        &self.position
    }

    pub fn field(&self) -> &[f32] {
        self.field.slice(s![..]).to_slice().unwrap()
    }

    pub fn ret_v(&self) -> &[f32] {
        self.retarded_velocity.slice(s![..]).to_slice().unwrap()
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
            applied_field: Array::linspace(0.0, 0.0, DIVISIONS),
            resultant_field: Array::linspace(0.0, 0.0, DIVISIONS),
            electrons: vec![Electron::new(Pos2::new(0.0, 0.0))],
        }
    }

    pub fn size(&self) -> f32 {
        WORLD_SIZE
    }

    pub fn update(&mut self) -> bool {
        self.applied_field = Array::from_vec(self.function_to_points(Self::wave_packet));
        self.resultant_field = Array::linspace(0.0, 0.0, DIVISIONS) + &self.applied_field;

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
