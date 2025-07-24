use egui::Rangef;

pub struct Variable {
    pub default: f32,
    pub min: f32,
    pub max: f32,
}

pub const WORLD_SIZE: Rangef = Rangef {
    min: -20.0,
    max: 4.0,
};

pub const SPRING_CONSTANT: Variable = Variable {
    default: 0.5,
    min: 0.0,
    max: 1.0,
};
pub const ELECTRON_MASS: Variable = Variable {
    default: 0.5,
    min: 0.1,
    max: 1.0,
};
pub const ELECTRON_DAMPING: Variable = Variable {
    default: 0.02,
    min: 0.0,
    max: 0.2,
};
pub const ELECTRON_SPACING: Variable = Variable {
    default: 2.0,
    min: 0.1,
    max: 4.0,
};
