//! Contains all simulation logic

mod field;
pub mod variables;

use field::Field;
use variables::{
    C, DIVISIONS, ELECTRON_DAMPING, ELECTRON_MASS, ELECTRON_SPACING, SPRING_CONSTANT, TIME_STEP,
    WORLD_SIZE,
};

use egui::{Pos2, Rangef, pos2, vec2};
use strum_macros::EnumIter;

/*
== Possible forms for the applied wave ========================================
*/

// Dropdown in the UI will be automatically populated with these options
#[derive(Debug, PartialEq, Clone, Copy, EnumIter)]
pub enum Waveform {
    Gaussian,       // single gaussian pulse
    GaussianPacket, // sine wave modulated by gaussian
    PlaneWave,      // sine wave
}

// mapping between enumeration and function definitions
fn applied_wave(form: Waveform) -> fn(f32, f32) -> f32 {
    match form {
        Waveform::Gaussian => gaussian_wave,
        Waveform::GaussianPacket => gaussian_packet_wave,
        Waveform::PlaneWave => plane_wave,
    }
}

// definitions for waveforms
fn gaussian_wave(x: f32, t: f32) -> f32 {
    let xp = x + C * t - WORLD_SIZE.max;
    (-4.0 * xp * xp).exp()
}
fn gaussian_packet_wave(x: f32, t: f32) -> f32 {
    let xp = x + C * t - WORLD_SIZE.max;
    (-xp * xp).exp() * (5.0 * xp).sin()
}
fn plane_wave(x: f32, t: f32) -> f32 {
    let xp = x + C * t - WORLD_SIZE.max;
    (1.0 * xp).sin()
}

/*
== Logic relating to the particles =========================================================
*/

struct PointInTime {
    t: f32, // point in time
    y: f32, // y displacement as t
    v: f32, // y velocity at t
    a: f32, // y acceleration at t
}

pub struct Electron {
    mass: f32,
    position: Pos2,
    velocity: f32,             // enforce always in y direction
    acceleration: f32,         // enforce always in y direction
    spring_constant: f32,      // treat electron as SHO with this k
    damping: f32,              // SHO damping factor
    field: Field,              // induced electric field from acceleration
    history: Vec<PointInTime>, // for implementing retarded time
}

impl Electron {
    pub fn new(position: Pos2, field_size: Rangef) -> Self {
        Electron {
            mass: ELECTRON_MASS.initial,
            spring_constant: SPRING_CONSTANT.initial,
            damping: ELECTRON_DAMPING.initial,
            position,
            velocity: 0.0,
            acceleration: 0.0,
            field: Field::new(field_size),
            history: Vec::new(),
        }
    }

    pub fn update(&mut self, applied_field_strength: f32, t: f32) {
        self.update_position(applied_field_strength, t);
        self.update_induced_field(t);
    }

    // based on motion of electron, calculate the field seen by all points on x axis
    fn update_induced_field(&mut self, t: f32) {
        for i in 0..DIVISIONS {
            let x = self.field.position_at(i);
            // past motion of electron as seen by point at (x, 0)
            let e_rva = self.retarded_rva(x, t);

            let r = vec2(self.position.x - x, e_rva.y);
            let mod_r = r.length();
            // get perpendicular components of motion
            let cos_theta = r.x.abs() / mod_r;
            let v = e_rva.v * cos_theta;
            let a = e_rva.a * cos_theta;
            let r_dot_v = r.y * v;

            // prevent big spikes in field close to the electron, it looks bad and makes it hard to understand what's going on
            let w = 2.0 * mod_r;
            let pretty_factor = 1.0 / (1.0 / (w * w * w.exp()) + 1.0);

            // derived from second time-derivative term of Heaviside-Feynman formula
            self.field[i] = match r.x.abs() < self.field.size() / (DIVISIONS - 1) as f32 {
                true => 0.0,
                false => pretty_factor * (v * r_dot_v / (mod_r * mod_r) - a / mod_r),
            };
        }
    }

    // update motion of electron based on the field it is experiencing
    fn update_position(&mut self, applied_field_strength: f32, t: f32) {
        // simple harmonic motion
        let force = applied_field_strength
            - self.spring_constant * self.position.y
            - self.damping * self.velocity;
        self.acceleration = force / self.mass;
        self.velocity += TIME_STEP * self.acceleration;
        self.position.y += TIME_STEP * self.velocity;
        // record this instant for retarded time lookup
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

    // motion of this particle as seen by point at (x,0) at time t, due to light delay
    fn retarded_rva(&self, x: f32, t: f32) -> PointInTime {
        let now = self.snapshot(t);
        if self.history.len() < 2 {
            return now;
        }

        // calculate the retarded time that this point is 'seeing' the electron at
        let distance = (x - self.position.x).abs();
        let past_t = (t - distance / C).max(0.0);

        // get index of this time point in electron's history, possible because the simulation increments time by a constant amount
        let i = ((self.history.len() as f32) * past_t / t).floor() as usize;

        // closest time points recorded by electron
        let t1 = self.history.get(i).unwrap_or(&now);
        let t2 = self.history.get(i + 1).unwrap_or(&now);

        // interpolate between the two closest recorded instants
        let interpolation_factor = (past_t - t1.t) / (t2.t - t1.t);
        return PointInTime {
            t: past_t,
            y: t1.y * (1.0 - interpolation_factor) + t2.y * interpolation_factor,
            v: t1.v * (1.0 - interpolation_factor) + t2.v * interpolation_factor,
            a: t1.a * (1.0 - interpolation_factor) + t2.a * interpolation_factor,
        };
    }
}

/*
=================================================================================
*/

pub struct Simulation {
    t: f32,                 // time
    size: Rangef,           // dimensions of x axis
    pub waveform: Waveform, // applied wave
    applied_field: Field,   // applied wave intensity at each x
    resultant_field: Field, // applied wave plus all electron fields

    electrons: Vec<Electron>,
    pub electron_count: usize, // used for updating self.electrons
    pub electron_spacing: f32, // used for updating self.electrons

    pub spring_constant: f32, // need to record this on simulation for slider, updates electrons once per frame
    pub electron_mass: f32, // need to record this on simulation for slider, updates electrons once per frame
    pub damping: f32, // need to record this on simulation for slider, updates electrons once per frame
}

impl Simulation {
    pub fn new(waveform: Waveform) -> Self {
        let size = WORLD_SIZE;
        Simulation {
            t: 0.0,
            size,
            waveform,
            electron_count: 1,
            damping: ELECTRON_DAMPING.initial,
            spring_constant: SPRING_CONSTANT.initial,
            electron_mass: ELECTRON_MASS.initial,
            electron_spacing: ELECTRON_SPACING.initial,
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
            // space electrons evenly starting from origin
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
            // erase all but the origin electron, to be refilled in the next step
            self.electrons.resize_with(1, || -> Electron {
                Electron::new(pos2(0.0, 0.0), self.size)
            });
        }
        // update number of electrons, keeping existing if possible
        if self.electron_count != self.electrons.len() {
            let mut i = self.electrons.len() - 1;
            self.electrons
                .resize_with(self.electron_count, || -> Electron {
                    i += 1;
                    Electron::new(pos2(-(i as f32) * self.electron_spacing, 0.0), self.size)
                });
        }
    }

    // move simulation forward by one time interval
    pub fn update(&mut self) -> bool {
        // set applied and resultant fields from waveform
        self.applied_field
            .set_from_function(applied_wave(self.waveform), self.t);
        self.resultant_field
            .set_from_function(applied_wave(self.waveform), self.t);

        for i in 0..self.electrons.len() {
            let e_y = self.resultant_field.value_at(self.electrons[i].position.x);
            let e = self.electrons.get_mut(i).unwrap();
            // set electron properties to those set in the UI
            e.mass = self.electron_mass;
            e.spring_constant = self.spring_constant;
            e.damping = self.damping;
            e.update(e_y, self.t);
            // combine this electron's contribution
            self.resultant_field.add(&e.field);
        }

        self.t += TIME_STEP;

        // returning true indiates the end and stops the simulation.
        //return self.t > (1.3 * self.size.span() / C); // terminate simulation after wave has cleared the screen
        return false;
    }

    pub fn max_electrons(&self) -> u32 {
        // maximum number of electrons that would be visible with current spacing
        (self.size.min.abs() / self.electron_spacing).floor() as u32
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
