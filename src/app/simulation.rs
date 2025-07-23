//! Contains all simulation logic

mod field;

use egui::{Pos2, Rangef, Vec2};

use field::Field;

pub const DIVISIONS: usize = 501;
pub const C: f32 = 1.0;
pub const TIME_STEP: f32 = 1.0 / (crate::app::SIMULATION_FPS as f32);
pub const K_DEFAULT: f32 = 0.5;
pub const M_DEFAULT: f32 = 0.2;

fn wave_packet(x: f32, t: f32) -> f32 {
    let xp = x + C * t - 4.0;
    (-(xp * xp)).exp() * (10.0 * xp).sin()
}

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
    field: Field,
    history: Vec<PointInTime>,
}

impl Electron {
    pub fn new(position: Pos2, field_size: Rangef) -> Self {
        Electron {
            mass: M_DEFAULT,
            spring_constant: K_DEFAULT,
            position,
            velocity: 0.0,
            acceleration: 0.0,
            field: Field::new(field_size),
            history: Vec::new(),
        }
    }

    fn update_induced_field(&mut self, t: f32) {
        for i in 0..DIVISIONS {
            /*
            let r = Vec2::new(self.field.position_at(i), 0.0);
            let e_rva = self.retarded_rva(r.x, t);
            let e_to_r = r - Vec2::new(self.position.x, e_rva.y);
            let n_s = e_to_r.normalized();
            let d = e_to_r.length();
            let v_s = Vec2::new(0.0, e_rva.v);
            let ns_dot_vs = n_s.dot(v_s);

            // derived from Heaviside–Feynman formula
            self.field[i] = (
                //(n_s - v_s) / (d*d*(1.0 - ns_dot_vs).powi(3)) +
                Vec2::new(n_s.x*n_s.y, -n_s.x*n_s.x)*e_rva.a / (d*(1.0 - ns_dot_vs).powi(3))
            ).y;



            */
            // derived from second time-derivative term of Heaviside-Feynman formula
            let x = self.field.position_at(i);
            let e_rva = self.retarded_rva(x, t);
            let r = Vec2::new(self.position.x - x, e_rva.y);
            let mod_r = r.length();
            let v = e_rva.v;
            let a = e_rva.a;
            let r_dot_v = r.y * v;
            self.field[i] = match r.x.abs() < self.field.size() / (DIVISIONS-1) as f32 {
                true => 0.0,
                false => v*r_dot_v / (mod_r*mod_r) - a / mod_r
            };
        }
    }

    fn update_position(&mut self, applied_field_strength: f32, t: f32, delta_t: f32) {
        let force = applied_field_strength - self.spring_constant * self.position.y;
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
                //return match (t1-t).abs() < (t2-t).abs() {
                //    true => *v1,
                //    false => *v2
                //};
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
    pub spring_constant: f32,
    pub electron_mass: f32,
    applied_field: Field,
    resultant_field: Field,
    electrons: Vec<Electron>,
}

impl Simulation {
    pub fn new(size: Rangef) -> Self {
        Simulation {
            t: 0.0,
            speed: 1.0,
            size,
            spring_constant: K_DEFAULT,
            electron_mass: M_DEFAULT,
            applied_field: Field::new(size),
            resultant_field: Field::new(size),
            electrons: vec![Electron::new(Pos2::new(0.0, 0.0), size)],
        }
    }

    pub fn reset(&mut self) {
        self.t = 0.0;
        self.applied_field = Field::new(self.size);
        self.resultant_field = Field::new(self.size);
        self.electrons.clear();
        self.electrons
            .push(Electron::new(Pos2::new(0.0, 0.0), self.size));
    }

    pub fn size(&self) -> &Rangef {
        &self.size
    }

    pub fn update(&mut self) -> bool {
        self.applied_field.set_from_function(wave_packet, self.t);
        self.resultant_field.set_from_function(wave_packet, self.t);

        let time_step = self.time_step();
        for i in 0..self.electrons.len() {
            let e_y = self.resultant_field.value_at(self.electrons[i].position.x);
            let e = self.electrons.get_mut(i).unwrap();
            e.mass = self.electron_mass;
            e.spring_constant = self.spring_constant;
            e.update_position(e_y, self.t, time_step);
            e.update_induced_field(self.t);
            self.resultant_field.add(&e.field);
        }

        self.t += time_step;

        return self.t > (1.3 * self.size.span() / C);
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
