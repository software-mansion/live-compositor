use std::collections::HashSet;

use super::{Component, ComponentId, OutputScene, SceneError};

impl Component {
    fn component_id(&self) -> Option<&ComponentId> {
        match self {
            Component::InputStream(input) => input.id.as_ref(),
            Component::Shader(shader) => shader.id.as_ref(),
            Component::WebView(web) => web.id.as_ref(),
            Component::Image(image) => image.id.as_ref(),
            Component::Text(text) => text.id.as_ref(),
            Component::View(view) => view.id.as_ref(),
            Component::Tiles(tiles) => tiles.id.as_ref(),
            Component::Rescaler(rescaler) => rescaler.id.as_ref(),
        }
    }

    fn children(&self) -> Vec<&Component> {
        match self {
            Component::InputStream(_input) => vec![],
            Component::Shader(shader) => shader.children.iter().collect(),
            Component::WebView(view) => view.children.iter().collect(),
            Component::Image(_image) => vec![],
            Component::Text(_text) => vec![],
            Component::View(view) => view.children.iter().collect(),
            Component::Tiles(tiles) => tiles.children.iter().collect(),
            Component::Rescaler(rescaler) => vec![rescaler.child.as_ref()],
        }
    }
}

pub(super) fn validate_scene_update(outputs: &[OutputScene]) -> Result<(), SceneError> {
    validate_component_ids_uniqueness(outputs)?;
    Ok(())
}

fn validate_component_ids_uniqueness(outputs: &[OutputScene]) -> Result<(), SceneError> {
    let mut ids: HashSet<&ComponentId> = HashSet::new();

    fn visit<'a>(
        component: &'a Component,
        ids: &mut HashSet<&'a ComponentId>,
    ) -> Result<(), SceneError> {
        let id = component.component_id();
        if let Some(id) = id {
            if ids.contains(id) {
                return Err(SceneError::DuplicateComponentId(id.clone()));
            }

            ids.insert(id);
        }

        component
            .children()
            .into_iter()
            .try_for_each(|c| visit(c, ids))
    }

    outputs
        .iter()
        .try_for_each(|output| visit(&output.root, &mut ids))
}
