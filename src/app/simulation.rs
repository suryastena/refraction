//! Contains all simulation logic

use log::info;

pub struct Simulation {
    world_size: f32,
}

impl Simulation {
    pub fn new() -> Self {
        Simulation {
            world_size: 10.0,
        }
    }

    pub fn size(&self) -> f32 {
        self.world_size
    }

    pub fn update(&self, speed_factor: f32) {
        info!("{}", speed_factor)
    }
}
