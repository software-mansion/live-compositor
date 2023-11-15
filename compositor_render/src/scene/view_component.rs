use std::{ops::Add, time::Duration};

use compositor_common::util::{colors::RGBAColor, ContinuousValue, InterpolationState};

use crate::{scene::ViewChildrenDirection, transformations::layout::NestedLayout};

use super::{
    components::ViewComponent, layout::LayoutComponentState, scene_state::BuildStateTreeCtx,
    BuildSceneError, Component, ComponentId, ComponentState, IntermediateNode, Position, Size,
    Transition,
};

mod interpolation;
mod layout;

#[derive(Debug, Clone)]
pub(super) struct ViewComponentState {
    start: Option<ViewComponentParam>,
    end: ViewComponentParam,
    transition: Option<Transition>,
    children: Vec<ComponentState>,
    start_pts: Duration,
}

#[derive(Debug, Clone)]
struct ViewComponentParam {
    id: Option<ComponentId>,

    direction: ViewChildrenDirection,
    position: Position,

    background_color: RGBAColor,
}

impl ViewComponentState {
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
                None
            } else {
                self.start_pts.add(transition.duration).checked_sub(pts)
            }
        })
    }

    pub(super) fn children(&self) -> Vec<&ComponentState> {
        self.children.iter().collect()
    }

    pub(super) fn children_mut(&mut self) -> Vec<&mut ComponentState> {
        self.children.iter_mut().collect()
    }

    pub(super) fn position(&self, pts: Duration) -> Position {
        self.view(pts).position
    }

    pub(super) fn component_id(&self) -> Option<&ComponentId> {
        self.end.id.as_ref()
    }

    pub(super) fn base_node(&self) -> Result<IntermediateNode, BuildSceneError> {
        let children = self
            .children
            .iter()
            .map(|component| {
                let node = component.base_node()?;
                match node {
                    IntermediateNode::Layout { root: _, children } => Ok(children),
                    _ => Ok(vec![node]),
                }
            })
            .collect::<Result<Vec<_>, _>>()? // convert Vec<Result<Vec<_>>> into Result<Vec<Vec<_>>>
            .into_iter()
            .flatten()
            .collect();

        Ok(IntermediateNode::Layout {
            root: LayoutComponentState::View(self.clone()),
            children,
        })
    }

    pub(super) fn layout(&self, size: Size, pts: Duration) -> NestedLayout {
        self.view(pts).layout(size, &self.children, pts)
    }
}

impl ViewComponent {
    pub(super) fn state_component(self, ctx: &BuildStateTreeCtx) -> ComponentState {
        let previous_state = self
            .id
            .as_ref()
            .and_then(|id| ctx.prev_state.get(id))
            .and_then(|component| match component {
                ComponentState::Layout(LayoutComponentState::View(view_state)) => Some(view_state),
                _ => None,
            });

        // TODO: to handle cases like transition from top to bottom this view needs
        // to be further processed to use the same type of coordinates as end
        let start = previous_state.map(|state| state.view(ctx.last_render_pts));
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
        ComponentState::Layout(LayoutComponentState::View(ViewComponentState {
            start,
            end: ViewComponentParam {
                id: self.id,
                direction: self.direction,
                position: self.position,
                background_color: self.background_color,
            },
            transition,
            children: self
                .children
                .into_iter()
                .map(|c| Component::state_component(c, ctx))
                .collect(),
            start_pts: ctx.last_render_pts,
        }))
    }
}
