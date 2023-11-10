use compositor_common::{scene::Resolution, util::colors::RGBAColor};

use crate::transformations::layout::{self, Layout, LayoutContent, NestedLayout};

use super::{
    Component, ComponentId, HorizontalPosition, LayoutComponent, LayoutNode, Position,
    RelativePosition, VerticalPosition,
};

#[derive(Debug)]
pub(crate) struct SizedLayoutComponent {
    component: LayoutComponent,
    resolution: Resolution,
}

impl layout::LayoutProvider for LayoutNode {
    fn layouts(
        &mut self,
        _pts: std::time::Duration,
        inputs: Vec<Option<Resolution>>,
    ) -> Vec<layout::Layout> {
        self.root.component.update_state(&inputs);

        self.root
            .layout()
            .into_iter()
            .flat_map(|layout| layout.flatten())
            .collect()
    }

    fn resolution(&self) -> Resolution {
        self.root.resolution()
    }
}

impl LayoutComponent {
    pub(super) fn layout(&self, resolution: Resolution) -> Vec<NestedLayout> {
        match self {
            LayoutComponent::View(view) => view.layout(resolution),
        }
    }

    pub(super) fn position(&self) -> Position {
        match self {
            LayoutComponent::View(view) => view.position,
        }
    }

    pub(crate) fn component_id(&self) -> Option<&ComponentId> {
        match self {
            LayoutComponent::View(view) => view.id.as_ref(),
        }
    }

    pub(crate) fn component_type(&self) -> &'static str {
        match self {
            LayoutComponent::View(_) => "View",
        }
    }

    pub(super) fn background_color(&self) -> Option<RGBAColor> {
        match self {
            LayoutComponent::View(view) => Some(view.background_color),
        }
    }

    pub(super) fn children(&self) -> Vec<&Component> {
        match self {
            LayoutComponent::View(view) => view.children(),
        }
    }

    pub(super) fn children_mut(&mut self) -> Vec<&mut Component> {
        match self {
            LayoutComponent::View(view) => view.children_mut(),
        }
    }

    pub(super) fn node_children(&self) -> Vec<&Component> {
        self.children()
            .into_iter()
            .flat_map(|child| match child {
                Component::Layout(layout) => layout.node_children(),
                _ => vec![child],
            })
            .collect()
    }

    pub(super) fn update_state(&mut self, input_resolutions: &[Option<Resolution>]) {
        let mut child_index_offset = 0;
        for child in self.children_mut().iter_mut() {
            match child {
                Component::InputStream(input) => {
                    input.size = input_resolutions[child_index_offset];
                    child_index_offset += 1;
                }
                Component::Shader(_) => {
                    child_index_offset += 1; // no state
                }
                Component::Layout(layout) => {
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
        child: &Component,
        position: RelativePosition,
        parent_size: Resolution,
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
                Component::Layout(layout) => {
                    LayoutContent::Color(layout.background_color().unwrap_or(RGBAColor(0, 0, 0, 0)))
                }
                _ => LayoutContent::ChildNode(0),
            },
        };

        match child {
            Component::Layout(layout_component) => {
                let children_layouts = layout_component.layout(Resolution {
                    width: layout.width as usize,
                    height: layout.height as usize,
                });
                NestedLayout {
                    layout,
                    children: children_layouts,
                }
            }
            _non_layout_components => NestedLayout {
                layout,
                children: vec![],
            },
        }
    }
}

impl SizedLayoutComponent {
    pub(super) fn new(component: LayoutComponent, resolution: Resolution) -> Self {
        Self {
            component,
            resolution,
        }
    }

    fn resolution(&self) -> Resolution {
        match self.component.position() {
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

    fn layout(&self) -> Vec<NestedLayout> {
        self.component.layout(self.resolution)
    }
}
