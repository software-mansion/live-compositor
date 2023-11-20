use std::time::Duration;

use compositor_common::scene::Resolution;

use crate::transformations::layout::{self, LayoutContent, NestedLayout};

use super::{
    view_component::StatefulViewComponent, AbsolutePosition, ComponentId, HorizontalPosition,
    Position, Size, StatefulComponent, VerticalPosition,
};

#[derive(Debug, Clone)]
pub(super) enum StatefulLayoutComponent {
    View(StatefulViewComponent),
}

#[derive(Debug)]
pub(crate) struct SizedLayoutComponent {
    component: StatefulLayoutComponent,
    size: Size,
}

#[derive(Debug)]
pub(crate) struct LayoutNode {
    pub(crate) root: SizedLayoutComponent,
}

impl layout::LayoutProvider for LayoutNode {
    fn layouts(&mut self, pts: std::time::Duration, inputs: &[Option<Resolution>]) -> NestedLayout {
        self.root.component.update_state(inputs);
        self.root.layout(pts)
    }

    fn resolution(&self, pts: Duration) -> Resolution {
        self.root.resolution(pts)
    }
}

impl StatefulLayoutComponent {
    pub(super) fn layout(&self, size: Size, pts: Duration) -> NestedLayout {
        match self {
            StatefulLayoutComponent::View(view) => view.layout(size, pts),
        }
    }

    pub(super) fn position(&self, pts: Duration) -> Position {
        match self {
            StatefulLayoutComponent::View(view) => view.position(pts), // TODO
        }
    }

    pub(crate) fn component_id(&self) -> Option<&ComponentId> {
        match self {
            StatefulLayoutComponent::View(view) => view.component_id(),
        }
    }

    pub(crate) fn component_type(&self) -> &'static str {
        match self {
            StatefulLayoutComponent::View(_) => "View",
        }
    }

    pub(super) fn children(&self) -> Vec<&StatefulComponent> {
        match self {
            StatefulLayoutComponent::View(view) => view.children(),
        }
    }

    pub(super) fn children_mut(&mut self) -> Vec<&mut StatefulComponent> {
        match self {
            StatefulLayoutComponent::View(view) => view.children_mut(),
        }
    }

    pub(super) fn node_children(&self) -> Vec<&StatefulComponent> {
        self.children()
            .into_iter()
            .flat_map(|child| match child {
                StatefulComponent::Layout(layout) => layout.node_children(),
                _ => vec![child],
            })
            .collect()
    }

    pub(super) fn update_state(&mut self, input_resolutions: &[Option<Resolution>]) {
        let mut child_index_offset = 0;
        for child in self.children_mut().iter_mut() {
            match child {
                StatefulComponent::InputStream(input) => {
                    // TODO
                    input.size = input_resolutions[child_index_offset]
                        .map(Into::into)
                        .unwrap_or(Size {
                            width: 0.0,
                            height: 0.0,
                        });
                    child_index_offset += 1;
                }
                StatefulComponent::Shader(_)
                | StatefulComponent::Image(_)
                | StatefulComponent::WebView(_) => {
                    child_index_offset += 1; // no state
                }
                StatefulComponent::Layout(layout) => {
                    let node_children = layout.node_children().len();
                    layout.update_state(
                        &input_resolutions[child_index_offset..child_index_offset + node_children],
                    );
                    child_index_offset += node_children
                }
            }
        }
    }

    pub(super) fn layout_content(component: &StatefulComponent, index: usize) -> LayoutContent {
        match component {
            StatefulComponent::Layout(_layout) => LayoutContent::None,
            StatefulComponent::InputStream(input) => LayoutContent::ChildNode {
                index,
                size: input.size,
            },
            StatefulComponent::Shader(shader) => LayoutContent::ChildNode {
                index,
                size: shader.component.size,
            },
            StatefulComponent::WebView(web) => LayoutContent::ChildNode {
                index,
                size: web.size(),
            },
            StatefulComponent::Image(image) => LayoutContent::ChildNode {
                index,
                size: image.size(),
            },
        }
    }

    pub(super) fn layout_absolute_position_child(
        child: &StatefulComponent,
        position: AbsolutePosition,
        parent_size: Size,
        pts: Duration,
    ) -> NestedLayout {
        let top = match position.position_vertical {
            VerticalPosition::TopOffset(top) => top,
            VerticalPosition::BottomOffset(bottom) => parent_size.height - bottom - position.height,
        };
        let left = match position.position_horizontal {
            HorizontalPosition::LeftOffset(left) => left,
            HorizontalPosition::RightOffset(right) => parent_size.width - right - position.width,
        };
        let width = position.width;
        let height = position.height;
        let rotation_degrees = position.rotation_degrees;
        let content = Self::layout_content(child, 0);
        let crop = None;

        match child {
            StatefulComponent::Layout(layout_component) => {
                let children_layouts = layout_component.layout(Size { width, height }, pts);
                let child_nodes_count = match content {
                    LayoutContent::ChildNode { .. } => children_layouts.child_nodes_count + 1,
                    _ => children_layouts.child_nodes_count,
                };
                NestedLayout {
                    top,
                    left,
                    width,
                    height,
                    rotation_degrees,
                    scale_x: 1.0,
                    scale_y: 1.0,
                    crop,

                    content,
                    child_nodes_count,
                    children: vec![children_layouts],
                }
            }
            _non_layout_components => {
                let child_nodes_count = match content {
                    LayoutContent::ChildNode { .. } => 1,
                    _ => 0,
                };

                NestedLayout {
                    top,
                    left,
                    width,
                    height,
                    rotation_degrees,
                    scale_x: 1.0,
                    scale_y: 1.0,
                    crop,

                    content,
                    child_nodes_count,
                    children: vec![],
                }
            }
        }
    }
}

impl SizedLayoutComponent {
    pub(super) fn new(component: StatefulLayoutComponent, size: Size) -> Self {
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
