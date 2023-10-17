use std::time::Duration;

use compositor_common::{
    scene::{
        transition::{Interpolation, TransitionSpec, TransitionType},
        NodeId, Resolution,
    },
    util::InterpolationState,
};
use log::error;

use crate::{error::CreateNodeError, renderer::RenderCtx, wgpu::texture::NodeTexture};

use super::builtin::{BuiltinNode, BuiltinState, BuiltinTransition};

pub struct TransitionNode {
    builtin_node: BuiltinNode,
    start_pts: Option<Duration>,
    transition_duration: Duration,
    interpolation: Interpolation,
}

impl TransitionNode {
    pub fn new(
        ctx: &RenderCtx,
        spec: &TransitionSpec,
        input_count: usize,
    ) -> Result<Self, CreateNodeError> {
        let TransitionSpec {
            start,
            end,
            interpolation,
            transition_duration,
        } = spec;
        let transition: TransitionType = (start, end).try_into()?;

        let state = BuiltinState::Interpolated {
            // into converts from TransitionType -> BuiltinTransition
            // in the future not all transitions will be builtin transitions
            transition: transition.into(),
            state: InterpolationState(0.0),
        };
        let node = BuiltinNode::new(ctx, state, input_count);

        Ok(Self {
            builtin_node: node,
            start_pts: None,
            transition_duration: *transition_duration,
            interpolation: interpolation.clone(),
        })
    }

    pub fn render(
        &mut self,
        sources: &[(&NodeId, &NodeTexture)],
        target: &mut NodeTexture,
        pts: Duration,
    ) {
        let interpolation_state = self.next_interpolation_state(pts);
        match self.builtin_node.state_mut() {
            BuiltinState::Interpolated {
                transition: _,
                ref mut state,
            } => *state = interpolation_state,
            BuiltinState::Static(_) => error!("Static builtin inside transition node."),
        }
        self.builtin_node.render(sources, target, pts)
    }

    fn next_interpolation_state(&mut self, pts: Duration) -> InterpolationState {
        let start_pts = self.start_pts.get_or_insert(pts);
        let interpolation_state =
            (pts.as_secs_f64() - start_pts.as_secs_f64()) / self.transition_duration.as_secs_f64();

        // State before interpolate is applied needs to be clamped, but final value does not.
        // Interpolation result can be a value outside of 0..1 range e.g. when using spring
        // interpolation.
        let interpolation_state =
            InterpolationState(f64::max(0.0, f64::min(1.0, interpolation_state)));
        self.interpolation.interpolate(interpolation_state)
    }

    pub fn resolution(&self) -> Option<Resolution> {
        self.builtin_node.resolution_from_spec()
    }
}

impl From<TransitionType> for BuiltinTransition {
    fn from(value: TransitionType) -> Self {
        match value {
            TransitionType::FixedPositionLayout(start, end) => {
                Self::FixedPositionLayout(start, end)
            }
        }
    }
}
