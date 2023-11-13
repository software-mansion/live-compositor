use std::time::Duration;

use compositor_common::scene::Resolution;

use crate::transformations::layout::{self, Layout, LayoutContent, NestedLayout};

use super::{
    view_component::ViewComponentState, ComponentId, ComponentState, HorizontalPosition, Position,
    RelativePosition, VerticalPosition,
};

#[derive(Debug, Clone)]
pub(super) enum LayoutComponentState {
    View(ViewComponentState),
}

#[derive(Debug)]
pub(crate) struct SizedLayoutComponent {
    component: LayoutComponentState,
    resolution: Resolution,
}

#[derive(Debug)]
pub(crate) struct LayoutNode {
    pub(crate) root: SizedLayoutComponent,
}

impl layout::LayoutProvider for LayoutNode {
    fn layouts(
        &mut self,
        pts: std::time::Duration,
        inputs: Vec<Option<Resolution>>,
    ) -> Vec<layout::Layout> {
        self.root.component.update_state(&inputs);

        self.root.layout(pts).flatten(0)
    }

    fn resolution(&self, pts: Duration) -> Resolution {
        self.root.resolution(pts)
    }
}

impl LayoutComponentState {
    pub(super) fn layout(&self, resolution: Resolution, pts: Duration) -> NestedLayout {
        match self {
            LayoutComponentState::View(view) => view.layout(resolution, pts),
        }
    }

    pub(super) fn position(&self, pts: Duration) -> Position {
        match self {
            LayoutComponentState::View(view) => view.position(pts), // TODO
        }
    }

    pub(crate) fn component_id(&self) -> Option<&ComponentId> {
        match self {
            LayoutComponentState::View(view) => view.component_id(),
        }
    }

    pub(crate) fn component_type(&self) -> &'static str {
        match self {
            LayoutComponentState::View(_) => "View",
        }
    }

    pub(super) fn children(&self) -> Vec<&ComponentState> {
        match self {
            LayoutComponentState::View(view) => view.children(),
        }
    }

    pub(super) fn children_mut(&mut self) -> Vec<&mut ComponentState> {
        match self {
            LayoutComponentState::View(view) => view.children_mut(),
        }
    }

    pub(super) fn node_children(&self) -> Vec<&ComponentState> {
        self.children()
            .into_iter()
            .flat_map(|child| match child {
                ComponentState::Layout(layout) => layout.node_children(),
                _ => vec![child],
            })
            .collect()
    }

    pub(super) fn update_state(&mut self, input_resolutions: &[Option<Resolution>]) {
        let mut child_index_offset = 0;
        for child in self.children_mut().iter_mut() {
            match child {
                ComponentState::InputStream(input) => {
                    input.size = input_resolutions[child_index_offset];
                    child_index_offset += 1;
                }
                ComponentState::Shader(_) => {
                    child_index_offset += 1; // no state
                }
                ComponentState::Layout(layout) => {
                    let node_children = layout.node_children().len();
                    layout.update_state(
                        &input_resolutions[child_index_offset..child_index_offset + node_children],
                    );
                    child_index_offset += node_children
                }
            }
        }
    }

    pub(super) fn layout_relative_child(
        child: &ComponentState,
        position: RelativePosition,
        parent_size: Resolution,
        pts: Duration,
    ) -> NestedLayout {
        let layout = Layout {
            top: match position.position_vertical {
                VerticalPosition::Top(top) => top as f32,
                VerticalPosition::Bottom(bottom) => {
                    parent_size.height as f32 - bottom as f32 - position.height as f32
                }
            },
            left: match position.position_horizontal {
                HorizontalPosition::Left(left) => left as f32,
                HorizontalPosition::Right(right) => {
                    parent_size.width as f32 - right as f32 - position.width as f32
                }
            },
            width: position.width as f32,
            height: position.height as f32,
            rotation_degrees: position.rotation_degrees,
            content: match child {
                ComponentState::Layout(_layout) => LayoutContent::None,
                _ => LayoutContent::ChildNode(0),
            },
        };

        match child {
            ComponentState::Layout(layout_component) => {
                let children_layouts = layout_component.layout(
                    Resolution {
                        width: layout.width as usize,
                        height: layout.height as usize,
                    },
                    pts,
                );
                let child_nodes_count = match layout.content {
                    LayoutContent::ChildNode(_) => children_layouts.child_nodes_count + 1,
                    _ => children_layouts.child_nodes_count,
                };
                NestedLayout {
                    child_nodes_count,
                    layout,
                    children: vec![children_layouts],
                }
            }
            _non_layout_components => NestedLayout {
                child_nodes_count: match layout.content {
                    LayoutContent::ChildNode(_) => 1,
                    _ => 0,
                },
                layout,
                children: vec![],
            },
        }
    }
}

impl SizedLayoutComponent {
    pub(super) fn new(component: LayoutComponentState, resolution: Resolution) -> Self {
        Self {
            component,
            resolution,
        }
    }

    fn resolution(&self, pts: Duration) -> Resolution {
        match self.component.position(pts) {
            Position::Static { width, height } => Resolution {
                width: width.unwrap_or(self.resolution.width),
                height: height.unwrap_or(self.resolution.height),
            },
            Position::Relative(position) => Resolution {
                width: position.width,
                height: position.height,
            },
        }
    }

    fn layout(&self, pts: Duration) -> NestedLayout {
        self.component.layout(self.resolution, pts)
    }
}
