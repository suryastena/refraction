use egui::Rangef;

pub struct Variable {
    pub default: f32,
    pub min: f32,
    pub max: f32,
}

// number of x axis points to calculate the field for. Should be an odd number.
pub const DIVISIONS: usize = 1001;
// amount internal simulation time increments by each update
pub const TIME_STEP: f32 = 1.0 / (crate::app::SIMULATION_FPS as f32);
// speed of light
pub const C: f32 = 1.0;

// size of simulation
pub const WORLD_SIZE: Rangef = Rangef {
    min: -20.0,
    max: 4.0,
};
// spring constant of electron bond modelled as SHO
pub const SPRING_CONSTANT: Variable = Variable {
    default: 0.5,
    min: 0.0,
    max: 1.0,
};
// mass of bound electron modelled as SHO
pub const ELECTRON_MASS: Variable = Variable {
    default: 0.5,
    min: 0.1,
    max: 1.0,
};
// spring constant of bound electron modelled as SHO
pub const ELECTRON_DAMPING: Variable = Variable {
    default: 0.02,
    min: 0.0,
    max: 0.2,
};
// distance between neighbouring electrons
pub const ELECTRON_SPACING: Variable = Variable {
    default: 2.0,
    min: 0.1,
    max: 4.0,
};
