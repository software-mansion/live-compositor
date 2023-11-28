use std::{ops::Add, time::Duration};

use compositor_common::util::{
    align::{HorizontalAlign, VerticalAlign},
    ContinuousValue, InterpolationState,
};

use crate::transformations::layout::NestedLayout;

use super::{
    components::RescalerComponent, layout::StatefulLayoutComponent, scene_state::BuildStateTreeCtx,
    Component, ComponentId, IntermediateNode, Position, ResizeMode, SceneError, Size,
    StatefulComponent, Transition,
};

mod interpolation;
mod layout;

#[derive(Debug, Clone)]
pub(super) struct StatefulRescalerComponent {
    start: Option<RescalerComponentParam>,
    end: RescalerComponentParam,
    transition: Option<Transition>,
    child: Box<StatefulComponent>,
    start_pts: Duration,
}

#[derive(Debug, Clone)]
struct RescalerComponentParam {
    id: Option<ComponentId>,

    position: Position,
    mode: ResizeMode,
    horizontal_align: HorizontalAlign,
    vertical_align: VerticalAlign,
}

impl StatefulRescalerComponent {
    /// Generate state of the component for particular pts value.
    fn transition_snapshot(&self, pts: Duration) -> RescalerComponentParam {
        let (Some(transition), Some(start)) = (self.transition, &self.start) else {
            return self.end.clone();
        };
        let interpolation_progress = InterpolationState(f64::min(
            1.0,
            (pts.as_secs_f64() - self.start_pts.as_secs_f64()) / transition.duration.as_secs_f64(),
        ));
        ContinuousValue::interpolate(start, &self.end, interpolation_progress)
    }

    fn remaining_transition_duration(&self, pts: Duration) -> Option<Duration> {
        self.transition.and_then(|transition| {
            if self.start_pts + transition.duration > pts {
                None
            } else {
                self.start_pts.add(transition.duration).checked_sub(pts)
            }
        })
    }

    pub(super) fn children(&self) -> Vec<&StatefulComponent> {
        vec![&self.child]
    }

    pub(super) fn children_mut(&mut self) -> Vec<&mut StatefulComponent> {
        vec![&mut self.child]
    }

    pub(super) fn position(&self, pts: Duration) -> Position {
        self.transition_snapshot(pts).position
    }

    pub(super) fn component_id(&self) -> Option<&ComponentId> {
        self.end.id.as_ref()
    }

    pub(super) fn intermediate_node(&self) -> IntermediateNode {
        let children = {
            let node = self.child.intermediate_node();
            match node {
                IntermediateNode::Layout { root: _, children } => children,
                _ => vec![node],
            }
        };

        IntermediateNode::Layout {
            root: StatefulLayoutComponent::Rescaler(self.clone()),
            children,
        }
    }

    pub(super) fn layout(&self, size: Size, pts: Duration) -> NestedLayout {
        self.transition_snapshot(pts).layout(size, &self.child, pts)
    }
}

impl RescalerComponent {
    pub(super) fn stateful_component(
        self,
        ctx: &BuildStateTreeCtx,
    ) -> Result<StatefulComponent, SceneError> {
        let previous_state = self
            .id
            .as_ref()
            .and_then(|id| ctx.prev_state.get(id))
            .and_then(|component| match component {
                StatefulComponent::Layout(StatefulLayoutComponent::Rescaler(view_state)) => {
                    Some(view_state)
                }
                _ => None,
            });

        // TODO: to handle cases like transition from top to bottom this view needs
        // to be further processed to use the same type of coordinates as end
        let start = previous_state.map(|state| state.transition_snapshot(ctx.last_render_pts));
        // TODO: this is incorrect for non linear transformations
        let transition = self.transition.or_else(|| {
            let Some(previous_state) = previous_state else {
                return None;
            };
            let Some(duration) = previous_state.remaining_transition_duration(ctx.last_render_pts)
            else {
                return None;
            };
            previous_state.transition.map(|_| Transition { duration })
        });
        let view = StatefulRescalerComponent {
            start,
            end: RescalerComponentParam {
                id: self.id,
                position: self.position,
                mode: self.mode,
                horizontal_align: self.horizontal_align,
                vertical_align: self.vertical_align,
            },
            transition,
            child: Box::new(Component::stateful_component(*self.child, ctx)?),
            start_pts: ctx.last_render_pts,
        };
        Ok(StatefulComponent::Layout(
            StatefulLayoutComponent::Rescaler(view),
        ))
    }
}
