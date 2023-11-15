use std::time::Duration;

use compositor_common::scene::Resolution;

use crate::transformations::layout::{self, Layout, LayoutContent, NestedLayout};

use super::{
    view_component::ViewComponentState, AbsolutePosition, ComponentId, ComponentState,
    HorizontalPosition, Position, Size, VerticalPosition,
};

#[derive(Debug, Clone)]
pub(super) enum LayoutComponentState {
    View(ViewComponentState),
}

#[derive(Debug)]
pub(crate) struct SizedLayoutComponent {
    component: LayoutComponentState,
    size: Size,
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
    pub(super) fn layout(&self, size: Size, pts: Duration) -> NestedLayout {
        match self {
            LayoutComponentState::View(view) => view.layout(size, pts),
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
                    input.size = input_resolutions[child_index_offset].map(Into::into);
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

    pub(super) fn layout_absolute_position_child(
        child: &ComponentState,
        position: AbsolutePosition,
        parent_size: Size,
        pts: Duration,
    ) -> NestedLayout {
        let layout = Layout {
            top: match position.position_vertical {
                VerticalPosition::TopOffset(top) => top,
                VerticalPosition::BottomOffset(bottom) => {
                    parent_size.height - bottom - position.height
                }
            },
            left: match position.position_horizontal {
                HorizontalPosition::LeftOffset(left) => left,
                HorizontalPosition::RightOffset(right) => {
                    parent_size.width - right - position.width
                }
            },
            width: position.width,
            height: position.height,
            rotation_degrees: position.rotation_degrees,
            content: match child {
                ComponentState::Layout(_layout) => LayoutContent::None,
                _ => LayoutContent::ChildNode(0),
            },
        };

        match child {
            ComponentState::Layout(layout_component) => {
                let children_layouts = layout_component.layout(
                    Size {
                        width: layout.width,
                        height: layout.height,
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
    pub(super) fn new(component: LayoutComponentState, size: Size) -> Self {
        Self { component, size }
    }

    fn resolution(&self, pts: Duration) -> Resolution {
        match self.component.position(pts) {
            Position::Static { width, height } => Size {
                width: width.unwrap_or(self.size.width),
                height: height.unwrap_or(self.size.height),
            },
            Position::Absolute(position) => Size {
                width: position.width,
                height: position.height,
            },
        }
        .into()
    }

    fn layout(&self, pts: Duration) -> NestedLayout {
        self.component.layout(self.size, pts)
    }
}
