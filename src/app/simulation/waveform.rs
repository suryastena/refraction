use strum_macros::EnumIter;

use crate::app::simulation::variables::{C, WORLD_SIZE};

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

impl Waveform {
    pub fn properties(&self) -> WaveformProperties {
        // function separated out to allow for future flexibility
        self.retrieve_properties()
    }

    fn retrieve_properties(&self) -> WaveformProperties {
        match self {
            Waveform::Gaussian => WaveformProperties {
                name: "Gaussian",
                function: wavefunctions::gaussian_wave,
                colour: (255, 50, 50),
            },
            Waveform::GaussianPacket => WaveformProperties {
                name: "Gaussian Packet",
                function: wavefunctions::gaussian_packet_wave,
                colour: (50, 255, 50),
            },
            Waveform::PlaneWave => WaveformProperties {
                name: "Plane Wave",
                function: wavefunctions::plane_wave,
                colour: (255, 50, 50),
            },
        }
    }
}

pub struct WaveformProperties {
    pub name: &'static str,
    pub function: fn(f32, f32) -> f32,
    pub colour: (u8, u8, u8), // RGB - default should be (255, 50, 50)
}

mod wavefunctions {
    use super::*;
    // definitions for waveforms
    pub fn gaussian_wave(x: f32, t: f32) -> f32 {
        let xp = x + C * t - WORLD_SIZE.max;
        (-4.0 * xp * xp).exp()
    }
    pub fn gaussian_packet_wave(x: f32, t: f32) -> f32 {
        let xp = x + C * t - WORLD_SIZE.max;
        (-xp * xp).exp() * (5.0 * xp).sin()
    }
    pub fn plane_wave(x: f32, t: f32) -> f32 {
        let xp = x + C * t - WORLD_SIZE.max;
        (1.0 * xp).sin()
    }
}
