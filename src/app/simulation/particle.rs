//! Charged particle types and derived properties for the simulation
//! To add a new particle, simply add it to the ChargedParticleType enum,
//! then enter its properties in to a corresponding branch of the match statement in ChargedParticleType::retrieve_properties()

use strum_macros::EnumIter;

use super::variables::{ELECTRON_DAMPING, ELECTRON_MASS, SPRING_CONSTANT};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, EnumIter)]
pub enum ChargedParticleType {
    Electron,
    Proton,
}

impl ChargedParticleType {
    pub fn properties(&self) -> ParticleProperties {
        let props = self.retrieve_properties();
        ParticleProperties {
            mass: props.mass,
            charge: props.charge,
            default_spring_constant: props.default_spring_constant,
            default_damping: props.default_damping,
            name: props.name,
            colour: props.colour,
        }
    }

    fn retrieve_properties(&self) -> ParticleProperties {
        match self {
            ChargedParticleType::Electron => ParticleProperties {
                mass: ELECTRON_MASS.initial,
                charge: -1.0,
                default_spring_constant: SPRING_CONSTANT.initial,
                default_damping: ELECTRON_DAMPING.initial,
                name: "Electron",
                colour: (0, 0, 255),
            },
            ChargedParticleType::Proton => ParticleProperties {
                mass: 1836.0,
                charge: 1.0,
                default_spring_constant: 2.0,
                default_damping: 0.8,
                name: "Proton",
                colour: (255, 0, 0),
            },
        }
    }

    pub fn mass(&self) -> f32 {
        self.properties().mass
    }

    pub fn charge(&self) -> f32 {
        self.properties().charge
    }

    pub fn default_spring_constant(&self) -> f32 {
        self.properties().default_spring_constant
    }

    pub fn default_damping(&self) -> f32 {
        self.properties().default_damping
    }

    pub fn name(&self) -> &'static str {
        self.properties().name
    }

    pub fn colour(&self) -> (u8, u8, u8) {
        self.properties().colour
    }
}

impl Default for ChargedParticleType {
    fn default() -> Self {
        ChargedParticleType::Electron
    }
}

impl fmt::Display for ChargedParticleType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ParticleProperties {
    pub mass: f32,
    pub charge: f32,
    pub default_spring_constant: f32,
    pub default_damping: f32,
    pub name: &'static str,
    pub colour: (u8, u8, u8), //RGB
}
