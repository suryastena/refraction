//! Contains all simulation logic
#![allow(dead_code)]

mod field;

use egui::{Pos2, Vec2};

use crate::app::simulation::field::{Field, SimpleField};

pub const WORLD_SIZE: f32 = 5.0;
pub const DIVISIONS: usize = 501;
pub const ELECTRON_MASS: f32 = 1.0;
pub const C: f32 = 1.0;
pub const EM_STRENGTH: f32 = 20.0;
pub const SPRING_CONSTANT: f32 = 2.0;
pub const TIME_STEP: f32 = 1.0 / (crate::app::SIMULATION_FPS as f32);

fn wave_packet(x: f32, t: f32) -> f32 {
    let xp = x + C * t - WORLD_SIZE;
    (-(xp * xp)).exp() * (10.0 * xp).sin()
}

pub struct Electron {
    position: Pos2,
    velocity: f32,
    field: SimpleField,
    velocity_history: Vec<(f32, f32)>,
    ret_v: Vec<f32>,
}

impl Electron {
    pub fn new(position: Pos2) -> Self {
        Electron {
            position,
            velocity: 0.0,
            field: SimpleField::new(),
            velocity_history: Vec::new(),
            ret_v: vec![0.0; DIVISIONS],
        }
    }

    fn update_induced_field(&mut self, t: f32) {
        for i in 0..DIVISIONS {
            let x = self.field.position_at(i);
            let r = self.position - Vec2::new(x, 0.0);
            /*if (r.x * r.x + r.y * r.y) < 0.0001 {
                self.field[i] = 0.0;
                continue;
            }*/
            let vy = self.retarded_velocity(x, t);
            let v_perp_y = vy - vy * r.y * r.y / (r.x * r.x + r.y * r.y).powf(1.0);
            self.field[i] = -v_perp_y;
        }
    }

    fn update_position(&mut self, applied_field_strength: f32, t: f32, delta_t: f32) {
        let force = EM_STRENGTH * applied_field_strength - SPRING_CONSTANT * self.position.y;
        self.velocity += delta_t * (force / ELECTRON_MASS);
        self.position.y += delta_t * (self.velocity);
        self.velocity_history.push((t, self.velocity));

        self.ret_v.clear();
        for x in self.field.intervals() {
            let ret_v = self.retarded_velocity(*x, t);
            self.ret_v.push(ret_v);
        }
    }

    pub fn position(&self) -> &Pos2 {
        &self.position
    }

    pub fn field(&self) -> &[f32] {
        self.field.values()
    }

    fn retarded_velocity(&self, x: f32, t: f32) -> f32 {
        if self.velocity_history.len() < 2 {
            return self.velocity;
        }
        let distance = (x - self.position.x).abs();
        let past_t = (t - distance / C).max(0.0);
        let now = (t, self.velocity);
        let mut past_v: f32 = 0.0;
        for i in (0..self.velocity_history.len()).rev() {
            let (t1, v1) = &self.velocity_history[i];
            if *t1 < past_t {
                let (t2, v2) = &self.velocity_history.get(i + 1).unwrap_or(&now);
                //return match (t1-t).abs() < (t2-t).abs() {
                //    true => *v1,
                //    false => *v2
                //};
                let interpolation_factor = (past_t - t1)/(t2-t1);
                past_v = v1 * (1.0 - interpolation_factor) + v2 * interpolation_factor;
                break;
            }
        }
        return past_v;
    }

    pub fn ret_v(&self) -> &[f32] {
        &self.ret_v
    }
}

/*
=================================================================================
*/

pub struct Simulation {
    pub speed: f32,
    t: f32,
    pub photon: f32,
    applied_field: SimpleField,
    resultant_field: SimpleField,
    electrons: Vec<Electron>,
}

impl Simulation {
    pub fn new() -> Self {
        Simulation {
            t: 0.0,
            speed: 1.0,
            photon: WORLD_SIZE,
            applied_field: SimpleField::new(),
            resultant_field: SimpleField::new(),
            electrons: vec![Electron::new(Pos2::new(0.0, 0.0))],
        }
    }

    pub fn reset(&mut self) {
        self.t = 0.0;
        self.photon = WORLD_SIZE;
        self.applied_field = SimpleField::new();
        self.resultant_field = SimpleField::new();
        self.electrons.clear();
        self.electrons.push(Electron::new(Pos2::new(0.0, 0.0)));
    }

    pub fn size(&self) -> f32 {
        WORLD_SIZE
    }

    pub fn update(&mut self) -> bool {
        self.applied_field.set_from_function(wave_packet, self.t);
        self.resultant_field.set_from_function(wave_packet, self.t);

        let time_step = self.time_step();
        for i in 0..self.electrons.len() {
            let e_y = self.applied_field.value_at(self.electrons[i].position.x);
            let e = self.electrons.get_mut(i).unwrap();
            e.update_position(e_y, self.t, time_step);
            e.update_induced_field(self.t);
            self.resultant_field.add(&e.field);
        }

        self.photon -= C * time_step;
        self.t += time_step;

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
        self.applied_field.intervals()
    }

    pub fn applied_field(&self) -> &[f32] {
        self.applied_field.values()
    }

    pub fn resultant_field(&self) -> &[f32] {
        self.resultant_field.values()
    }
}
