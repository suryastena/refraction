//! Contains all simulation logic

mod field;
pub mod particle;
pub mod variables;
pub mod waveform;

use field::Field;
use particle::ChargedParticleType;
use variables::{C, DIVISIONS, INV_C_2, PARTICLE_SPACING, TIME_STEP, WORLD_SIZE};
use waveform::*;

use egui::{Pos2, Rangef, pos2, vec2};

/*
== Logic relating to the particles =========================================================
*/

struct PointInTime {
    t: f32, // point in time
    y: f32, // y displacement as t
    v: f32, // y velocity at t
    a: f32, // y acceleration at t
}

pub struct ChargedParticle {
    particle_type: ChargedParticleType,
    mass: f32,
    position: Pos2,
    velocity: f32,             // enforce always in y direction
    acceleration: f32,         // enforce always in y direction
    spring_constant: f32,      // treat particle as SHO with this k
    damping: f32,              // SHO damping factor
    field: Field,              // induced electric field from acceleration
    history: Vec<PointInTime>, // for implementing retarded time
}

impl ChargedParticle {
    pub fn new(position: Pos2, field_size: Rangef, particle_type: ChargedParticleType) -> Self {
        ChargedParticle {
            particle_type,
            mass: particle_type.mass(),
            spring_constant: particle_type.default_spring_constant(),
            damping: particle_type.default_damping(),
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

    pub fn particle_type(&self) -> &ChargedParticleType {
        &self.particle_type
    }

    // based on motion of particle, calculate the field seen by all points on x axis
    fn update_induced_field(&mut self, t: f32) {
        let charge = self.particle_type.charge();
        for i in 0..DIVISIONS {
            let x = self.field.position_at(i);
            // past motion of particle as seen by point at (x, 0)
            let e_rva = self.retarded_rva(x, t);

            let r = vec2(self.position.x - x, e_rva.y);
            let mod_r = r.length();
            // get perpendicular components of motion
            let cos_theta = r.x.abs() / mod_r;
            let a_perp = e_rva.a * cos_theta;

            // prevent big spikes in field close to the particle, this factor isn't physical but spikes make it look bad and makes it hard to understand what's going on.
            //let w = 2.0 * mod_r;
            //let pretty_factor = 1.0 / (1.0 / (w * w * w.exp()) + 1.0);

            // derived from second time-derivative term of Heaviside-Feynman formula
            // include charge in the field calculation
            self.field[i] = match r.x.abs() < self.field.size() / (DIVISIONS - 1) as f32 {
                true => 0.0,
                false => INV_C_2 * (charge * a_perp / mod_r), // * pretty_factor,
            };
        }
    }

    // update motion of particle based on the field it is experiencing
    fn update_position(&mut self, applied_field_strength: f32, t: f32) {
        // simple harmonic motion
        let charge = self.particle_type.charge();
        let force = charge * applied_field_strength
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

        // calculate the retarded time that this point is 'seeing' the particle at
        let distance = (x - self.position.x).abs();
        let past_t = (t - distance / C).max(0.0);

        // get index of this time point in particle's history, possible because the simulation increments time by a constant amount
        let i = ((self.history.len() as f32) * past_t / t).floor() as usize;

        // closest time points recorded by particle
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
    resultant_field: Field, // applied wave plus all particle fields

    particles: Vec<ChargedParticle>,
    pub particle_count: usize, // used for updating self.particles
    pub particle_spacing: f32, // used for updating self.particles
    pub particle_type: ChargedParticleType, // type of particles in the simulation

    pub spring_constant: f32, // need to record this on simulation for slider, updates particles once per frame
    pub particle_mass: f32, // need to record this on simulation for slider, updates particles once per frame
    pub damping: f32, // need to record this on simulation for slider, updates particles once per frame
}

impl Simulation {
    pub fn new(waveform: Waveform) -> Self {
        let size = WORLD_SIZE;
        let particle_type = ChargedParticleType::default(); // Default to electron
        Simulation {
            t: 0.0,
            size,
            waveform,
            particle_count: 1,
            particle_type,
            damping: particle_type.default_damping(),
            spring_constant: particle_type.default_spring_constant(),
            particle_mass: particle_type.mass(),
            particle_spacing: PARTICLE_SPACING.initial,
            applied_field: Field::new(size),
            resultant_field: Field::new(size),
            particles: vec![ChargedParticle::new(pos2(0.0, 0.0), size, particle_type)],
        }
    }

    pub fn reset(&mut self) {
        self.t = 0.0;
        self.applied_field = Field::new(self.size);
        self.resultant_field = Field::new(self.size);
        self.particles.clear();
        for i in 0..self.particle_count {
            // space particles evenly starting from origin
            self.particles.push(ChargedParticle::new(
                pos2(-(i as f32) * self.particle_spacing, 0.0),
                self.size,
                self.particle_type,
            ));
        }
    }

    pub fn size(&self) -> &Rangef {
        &self.size
    }

    pub fn update_particles(&mut self, update_all: bool) {
        if update_all {
            // erase all but the origin particle, to be refilled in the next step
            self.particles.resize_with(1, || -> ChargedParticle {
                ChargedParticle::new(pos2(0.0, 0.0), self.size, self.particle_type)
            });
        }
        // update number of particles, keeping existing if possible
        if self.particle_count != self.particles.len() {
            let mut i = self.particles.len() - 1;
            self.particles
                .resize_with(self.particle_count, || -> ChargedParticle {
                    i += 1;
                    ChargedParticle::new(
                        pos2(-(i as f32) * self.particle_spacing, 0.0),
                        self.size,
                        self.particle_type,
                    )
                });
        }
    }

    // move simulation forward by one time interval
    pub fn update(&mut self) -> bool {
        // set applied and resultant fields from waveform
        self.applied_field
            .set_from_function(self.waveform.properties().function, self.t);
        self.resultant_field
            .set_from_function(self.waveform.properties().function, self.t);

        for i in 0..self.particles.len() {
            let e_y = self.resultant_field.value_at(self.particles[i].position.x);
            let p = self.particles.get_mut(i).unwrap();
            // set particle properties to those set in the UI
            p.mass = self.particle_mass;
            p.spring_constant = self.spring_constant;
            p.damping = self.damping;
            p.update(e_y, self.t);
            // combine this particle's contribution
            self.resultant_field.add(&p.field);
        }

        self.t += TIME_STEP;

        // returning true indiates the end and stops the simulation.
        //return self.t > (1.3 * self.size.span() / C); // terminate simulation after wave has cleared the screen
        return false;
    }

    pub fn max_particles(&self) -> u32 {
        // maximum number of particles that would be visible with current spacing
        (self.size.min.abs() / self.particle_spacing).floor() as u32
    }

    pub fn particles(&self) -> &[ChargedParticle] {
        &self.particles
    }

    pub fn set_particle_type(&mut self, particle_type: ChargedParticleType) {
        self.particle_type = particle_type;
        // Update default values for the new particle type
        self.particle_mass = particle_type.mass();
        self.spring_constant = particle_type.default_spring_constant();
        self.damping = particle_type.default_damping();
        // Recreate all particles with the new type
        self.reset();
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
