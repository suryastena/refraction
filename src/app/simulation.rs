//! Contains all simulation logic
#![allow(dead_code)]

mod field;

use egui::{Pos2, Vec2};

use crate::app::simulation::field::{Field, SimpleField, VelocityField};

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
    retarded_velocity: VelocityField,
}

impl Electron {
    pub fn new(position: Pos2) -> Self {
        Electron {
            position,
            velocity: 0.0,
            field: SimpleField::new(),
            retarded_velocity: VelocityField::new(),
        }
    }

    fn update_induced_field(&mut self) {
        for i in 0..DIVISIONS {
            let r = self.position - Vec2::new(self.field.position_at(i), 0.0);
            /*if (r.x * r.x + r.y * r.y) < 0.0001 {
                self.field[i] = 0.0;
                continue;
            }*/
            let vy = self.retarded_velocity()[i];
            let v_perp_y = vy - vy * r.y * r.y / (r.x * r.x + r.y * r.y).powf(1.0);
            self.field[i] = -v_perp_y;
        }
    }

    fn update_position(&mut self, applied_field_strength: f32, time_step: f32) {
        let force = EM_STRENGTH * applied_field_strength - SPRING_CONSTANT * self.position.y;
        self.velocity += time_step * (force / ELECTRON_MASS);
        self.position.y += time_step * (self.velocity);
        self.retarded_velocity.update(time_step, self.velocity);
    }

    pub fn position(&self) -> &Pos2 {
        &self.position
    }

    pub fn field(&self) -> &[f32] {
        self.field.values()
    }

    pub fn retarded_velocity(&self) -> &[f32] {
        self.retarded_velocity.values()
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
        self.electrons = vec![Electron::new(Pos2::new(0.0, 0.0))];
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
            e.update_position(e_y, time_step);
            e.update_induced_field();
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
