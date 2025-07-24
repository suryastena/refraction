//! Contains all simulation logic

mod field;
pub mod variables;

use field::Field;
use variables::{ELECTRON_DAMPING, ELECTRON_MASS, ELECTRON_SPACING, SPRING_CONSTANT, WORLD_SIZE};

use egui::{Pos2, Rangef, pos2, vec2};
/*
=================================================================================
*/
pub const DIVISIONS: usize = 1001;
pub const C: f32 = 1.0;
pub const TIME_STEP: f32 = 1.0 / (crate::app::SIMULATION_FPS as f32);
/*
=================================================================================
*/
#[derive(Debug, PartialEq)]
pub enum Waveform {
    Gaussian,
    GaussianPacket,
    PlaneWave,
}

fn gaussian_wave(x: f32, t: f32) -> f32 {
    let xp = x + C * t - 4.0;
    -0.5*(-(xp * xp)).exp()
}

fn gaussian_packet_wave(x: f32, t: f32) -> f32 {
    let xp = x + C * t - 4.0;
    (-(xp * xp)).exp() * (5.0 * xp).sin()
}

fn plane_wave(x: f32, t: f32) -> f32 {
    let xp = x + C * t - 4.0;
    (8.0 * xp).sin()
}

/*
=================================================================================
*/

struct PointInTime {
    t: f32,
    y: f32,
    v: f32,
    a: f32,
}

pub struct Electron {
    mass: f32,
    spring_constant: f32,
    position: Pos2,
    velocity: f32,
    acceleration: f32,
    damping: f32,
    field: Field,
    history: Vec<PointInTime>,
}

impl Electron {
    pub fn new(position: Pos2, field_size: Rangef) -> Self {
        Electron {
            mass: ELECTRON_MASS.default,
            spring_constant: SPRING_CONSTANT.default,
            damping: ELECTRON_DAMPING.default,
            position,
            velocity: 0.0,
            acceleration: 0.0,
            field: Field::new(field_size),
            history: Vec::new(),
        }
    }

    fn update_induced_field(&mut self, t: f32) {
        for i in 0..DIVISIONS {
            // derived from second time-derivative term of Heaviside-Feynman formula
            let x = self.field.position_at(i);
            let e_rva = self.retarded_rva(x, t);
            let r = vec2(self.position.x - x, e_rva.y);
            let mod_r = r.length();
            let v = e_rva.v;
            let a = e_rva.a;
            let r_dot_v = r.y * v;
            self.field[i] = match r.x.abs() < self.field.size() / (DIVISIONS - 1) as f32 {
                true => 0.0,
                false => v * r_dot_v / (mod_r * mod_r) - a / mod_r,
            };
        }
    }

    fn update_position(&mut self, applied_field_strength: f32, t: f32, delta_t: f32) {
        let force = applied_field_strength
            - self.spring_constant * self.position.y
            - self.damping * self.velocity;
        self.acceleration = force / self.mass;
        self.velocity += delta_t * self.acceleration;
        self.position.y += delta_t * self.velocity;
        self.history.push(self.snapshot(t));
    }

    fn snapshot(&self, t: f32) -> PointInTime {
        PointInTime {
            t,
            y: self.position.y,
            v: self.velocity,
            a: self.acceleration,
        }
    }

    pub fn position(&self) -> &Pos2 {
        &self.position
    }

    pub fn field(&self) -> &[f32] {
        self.field.values()
    }

    fn retarded_rva(&self, x: f32, t: f32) -> PointInTime {
        let now = self.snapshot(t);
        if self.history.len() < 2 {
            return now;
        }
        let distance = (x - self.position.x).abs();
        let past_t = (t - distance / C).max(0.0);
        let mut point = PointInTime {
            t: past_t,
            y: 0.0,
            v: 0.0,
            a: 0.0,
        };
        for i in (0..self.history.len()).rev() {
            let t1 = self.history.get(i).unwrap();
            if t1.t < past_t {
                let t2 = self.history.get(i + 1).unwrap_or(&now);
                let interpolation_factor = (past_t - t1.t) / (t2.t - t1.t);
                point.y = t1.y * (1.0 - interpolation_factor) + t2.y * interpolation_factor;
                point.v = t1.v * (1.0 - interpolation_factor) + t2.v * interpolation_factor;
                point.a = t1.a * (1.0 - interpolation_factor) + t2.a * interpolation_factor;
                break;
            }
        }
        return point;
    }
}

/*
=================================================================================
*/

pub struct Simulation {
    t: f32,
    size: Rangef,
    pub speed: f32,
    pub waveform: Waveform,
    pub spring_constant: f32,
    pub electron_mass: f32,
    pub damping: f32,
    pub electron_count: usize,
    pub electron_spacing: f32,
    applied_field: Field,
    resultant_field: Field,
    electrons: Vec<Electron>,
}

impl Simulation {
    pub fn new(waveform: Waveform) -> Self {
        let size = WORLD_SIZE;
        Simulation {
            t: 0.0,
            speed: 1.0,
            size,
            waveform,
            electron_count: 1,
            damping: ELECTRON_DAMPING.default,
            spring_constant: SPRING_CONSTANT.default,
            electron_mass: ELECTRON_MASS.default,
            electron_spacing: ELECTRON_SPACING.default,
            applied_field: Field::new(size),
            resultant_field: Field::new(size),
            electrons: vec![Electron::new(pos2(0.0, 0.0), size)],
        }
    }

    pub fn reset(&mut self) {
        self.t = 0.0;
        self.applied_field = Field::new(self.size);
        self.resultant_field = Field::new(self.size);
        self.electrons.clear();
        for i in 0..self.electron_count {
            self.electrons.push(Electron::new(
                pos2(-(i as f32) * self.electron_spacing, 0.0),
                self.size,
            ));
        }
    }

    pub fn size(&self) -> &Rangef {
        &self.size
    }

    pub fn update_electrons(&mut self, update_all: bool) {
        if update_all {
            self.electrons.resize_with(1, || -> Electron {
                Electron::new(pos2(0.0, 0.0), self.size)
            });
        }
        if self.electron_count != self.electrons.len() {
            let mut i = self.electrons.len() - 1;
            self.electrons
                .resize_with(self.electron_count, || -> Electron {
                    i += 1;
                    Electron::new(pos2(-(i as f32) * self.electron_spacing, 0.0), self.size)
                });
        }
    }

    pub fn update(&mut self) -> bool {
        self.applied_field
            .set_from_function(self.applied_wave(), self.t);
        self.resultant_field
            .set_from_function(self.applied_wave(), self.t);

        let time_step = self.time_step();
        for i in 0..self.electrons.len() {
            let e_y = self.resultant_field.value_at(self.electrons[i].position.x);
            let e = self.electrons.get_mut(i).unwrap();
            e.mass = self.electron_mass;
            e.spring_constant = self.spring_constant;
            e.damping = self.damping;
            e.update_position(e_y, self.t, time_step);
            e.update_induced_field(self.t);
            self.resultant_field.add(&e.field);
        }

        self.t += time_step;

        // terminate simulation after wave has cleared the screen
        //return self.t > (1.3 * self.size.span() / C);
        return false;
    }

    pub fn max_electrons(&self) -> u32 {
        (self.size.min.abs() / self.electron_spacing).floor() as u32
    }

    fn applied_wave(&self) -> fn(f32, f32) -> f32 {
        match self.waveform {
            Waveform::Gaussian => gaussian_wave,
            Waveform::GaussianPacket => gaussian_packet_wave,
            Waveform::PlaneWave => plane_wave,
        }
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
        self.applied_field.intervals()
    }

    pub fn applied_field(&self) -> &[f32] {
        self.applied_field.values()
    }

    pub fn resultant_field(&self) -> &[f32] {
        self.resultant_field.values()
    }
}
