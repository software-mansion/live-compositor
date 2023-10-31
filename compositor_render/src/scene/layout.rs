use compositor_common::{scene::Resolution, util::colors::RGBAColor};

use crate::transformations::layout::{self, NestedLayout};

use super::{Component, ComponentId, LayoutComponent, LayoutNode};

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

    pub(super) fn width(&self) -> Option<usize> {
        match self {
            LayoutComponent::View(view) => view.width,
        }
    }

    pub(super) fn height(&self) -> Option<usize> {
        match self {
            LayoutComponent::View(view) => view.height,
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
                Component::Layout(layout) => layout.children(),
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
}

impl SizedLayoutComponent {
    pub(super) fn new(component: LayoutComponent, resolution: Resolution) -> Self {
        Self {
            component,
            resolution,
        }
    }

    fn width(&self) -> usize {
        self.component.width().unwrap_or(self.resolution.width)
    }

    fn height(&self) -> usize {
        self.component.height().unwrap_or(self.resolution.height)
    }

    fn resolution(&self) -> Resolution {
        Resolution {
            width: self.width(),
            height: self.height(),
        }
    }

    fn layout(&self) -> Vec<NestedLayout> {
        self.component.layout(self.resolution)
    }
}
