use std::{ops::Add, time::Duration};

use crate::{scene::ViewChildrenDirection, transformations::layout::NestedLayout};

use super::{
    components::ViewComponent,
    interpolation::{ContinuousValue, InterpolationState},
    layout::StatefulLayoutComponent,
    scene_state::BuildStateTreeCtx,
    Component, ComponentId, IntermediateNode, Overflow, Position, RGBAColor, SceneError, Size,
    StatefulComponent, Transition,
};

mod interpolation;
mod layout;

#[derive(Debug, Clone)]
pub(super) struct StatefulViewComponent {
    start: Option<ViewComponentParam>,
    end: ViewComponentParam,
    transition: Option<Transition>,
    children: Vec<StatefulComponent>,
    start_pts: Duration,
}

#[derive(Debug, Clone)]
struct ViewComponentParam {
    id: Option<ComponentId>,

    direction: ViewChildrenDirection,
    position: Position,
    overflow: Overflow,

    background_color: RGBAColor,
}

impl StatefulViewComponent {
    fn view(&self, pts: Duration) -> ViewComponentParam {
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
                self.start_pts.add(transition.duration).checked_sub(pts)
            } else {
                None
            }
        })
    }

    pub(super) fn children(&self) -> Vec<&StatefulComponent> {
        self.children.iter().collect()
    }

    pub(super) fn children_mut(&mut self) -> Vec<&mut StatefulComponent> {
        self.children.iter_mut().collect()
    }

    pub(super) fn position(&self, pts: Duration) -> Position {
        self.view(pts).position
    }

    pub(super) fn component_id(&self) -> Option<&ComponentId> {
        self.end.id.as_ref()
    }

    pub(super) fn intermediate_node(&self) -> IntermediateNode {
        let children = self
            .children
            .iter()
            .flat_map(|component| {
                let node = component.intermediate_node();
                match node {
                    IntermediateNode::Layout { root: _, children } => children,
                    _ => vec![node],
                }
            })
            .collect();

        IntermediateNode::Layout {
            root: StatefulLayoutComponent::View(self.clone()),
            children,
        }
    }

    pub(super) fn layout(&mut self, size: Size, pts: Duration) -> NestedLayout {
        self.view(pts).layout(size, &mut self.children, pts)
    }
}

impl ViewComponent {
    pub(super) fn stateful_component(
        self,
        ctx: &BuildStateTreeCtx,
    ) -> Result<StatefulComponent, SceneError> {
        let previous_state = self
            .id
            .as_ref()
            .and_then(|id| ctx.prev_state.get(id))
            .and_then(|component| match component {
                StatefulComponent::Layout(StatefulLayoutComponent::View(view_state)) => {
                    Some(view_state)
                }
                _ => None,
            });

        // TODO: to handle cases like transition from top to bottom this view needs
        // to be further processed to use the same type of coordinates as end
        let start = previous_state.map(|state| state.view(ctx.last_render_pts));
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
        let view = StatefulViewComponent {
            start,
            end: ViewComponentParam {
                id: self.id,
                direction: self.direction,
                position: self.position,
                background_color: self.background_color,
                overflow: self.overflow,
            },
            transition,
            children: self
                .children
                .into_iter()
                .map(|c| Component::stateful_component(c, ctx))
                .collect::<Result<_, _>>()?,
            start_pts: ctx.last_render_pts,
        };
        Ok(StatefulComponent::Layout(StatefulLayoutComponent::View(
            view,
        )))
    }
}
