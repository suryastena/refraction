use egui::Rangef;
extern crate static_assertions as sa;

pub struct Variable {
    pub initial: f32,
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
    initial: 0.5,
    min: 0.0,
    max: 1.0,
};
// mass of bound electron modelled as SHO
pub const ELECTRON_MASS: Variable = Variable {
    initial: 0.5,
    min: 0.1,
    max: 1.0,
};
// spring constant of bound electron modelled as SHO
pub const ELECTRON_DAMPING: Variable = Variable {
    initial: 0.0,
    min: 0.0,
    max: 1.0,
};
// distance between neighbouring electrons
pub const ELECTRON_SPACING: Variable = Variable {
    initial: 3.0,
    min: 1.0,
    max: 8.0,
};

sa::const_assert!(DIVISIONS % 2 == 1);
sa::const_assert!(WORLD_SIZE.min < WORLD_SIZE.max);
sa::const_assert!(SPRING_CONSTANT.min < SPRING_CONSTANT.max);
sa::const_assert!(ELECTRON_MASS.min < ELECTRON_MASS.max);
sa::const_assert!(ELECTRON_DAMPING.min < ELECTRON_DAMPING.max);
sa::const_assert!(ELECTRON_SPACING.min < ELECTRON_SPACING.max);
sa::const_assert!(TIME_STEP > 0.0);
sa::const_assert!(C > 0.0);
