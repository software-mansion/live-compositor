use compositor_common::{
    scene::builtin_transformations::{BuiltinSpec, FixedPositionLayoutSpec},
    util::InterpolationState,
};

mod box_layout;
pub mod error;
mod node;
pub mod params;
mod shader_params;
pub mod transformations;
pub mod utils;

pub use node::BuiltinNode;

#[derive(Debug)]
pub enum BuiltinState {
    Static(BuiltinSpec),
    Interpolated {
        transition: BuiltinTransition,
        state: InterpolationState,
    },
}

#[derive(Debug)]
pub enum BuiltinTransition {
    FixedPositionLayout(FixedPositionLayoutSpec, FixedPositionLayoutSpec),
}

impl BuiltinState {
    pub fn final_state(&self) -> BuiltinSpec {
        match self {
            BuiltinState::Static(spec) => spec.clone(),
            BuiltinState::Interpolated { transition, .. } => transition.final_state(),
        }
    }
}
impl BuiltinTransition {
    pub fn final_state(&self) -> BuiltinSpec {
        match self {
            BuiltinTransition::FixedPositionLayout(_, end) => {
                BuiltinSpec::FixedPositionLayout(end.clone())
            }
        }
    }
}
