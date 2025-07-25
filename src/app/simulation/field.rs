use egui::Rangef;
use ndarray::{Array, Array1, Ix1, s};
use std::ops::{Index, IndexMut};

use crate::app::simulation::variables::DIVISIONS;

// represents a 1D vector field
pub struct Field {
    field: Array1<f32>,  // values of field at each point
    extent: Rangef,      // dimensions of field
    step: f32,           // distance between divisions
    points: Array1<f32>, // x coordinates of each division
}

impl Field {
    pub fn new(extent: Rangef) -> Self {
        let f = Field {
            extent,
            step: extent.span() / ((DIVISIONS - 1) as f32),
            field: Array::zeros(Ix1(DIVISIONS)),
            points: Array::linspace(extent.min, extent.max, DIVISIONS),
        };
        f
    }

    // get fractional index of value at this x coordinate
    fn index_of(&self, x: f32) -> f32 {
        (x - self.extent.min) / self.step
    }

    pub fn values(&self) -> &[f32] {
        self.field.slice(s![..]).to_slice().unwrap()
    }

    pub fn values_mut(&mut self) -> &mut [f32] {
        self.field.slice_mut(s![..]).into_slice().unwrap()
    }

    // return interpolated value of field at a given x coordinate
    pub fn value_at(&self, x: f32) -> f32 {
        // closure returns value of field at a given index, or 0 if index is out of bounds
        let get_value = |i: usize| -> f32 { *self.values().get(i).unwrap_or(&0.0) };
        // fractional index of coordinate
        let idx = self.index_of(x);
        // closest true indices to the fractional index
        let (lower_idx, upper_idx) = (idx.floor(), idx.ceil());
        // values at the nearest indices
        let (lower, upper) = (get_value(lower_idx as usize), get_value(upper_idx as usize));
        // linearly interpolate to get most accurate field value
        lower * (1.0 - idx + lower_idx) + upper * (idx - lower_idx)
    }

    // given a function of x coordinate and time, fill this field with values at time t
    pub fn set_from_function(&mut self, f: impl Fn(f32, f32) -> f32, t: f32) {
        for i in 0..DIVISIONS {
            self.field[i] = f(self.points[i], t);
        }
    }

    // x coordinates of field divisions
    pub fn intervals(&self) -> &[f32] {
        self.points.slice(s![..]).to_slice().unwrap()
    }

    // add another field to this field
    pub fn add(&mut self, rhs: &Field) {
        self.field += &rhs.field;
    }

    // x coordinate of a division index
    pub fn position_at(&self, idx: usize) -> f32 {
        self.points[idx]
    }

    // dimensions in simulation space
    pub fn size(&self) -> f32 {
        self.extent.span()
    }
}

// required for [i] operations
impl Index<usize> for Field {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.values()[index]
    }
}
impl IndexMut<usize> for Field {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.values_mut()[index]
    }
}
