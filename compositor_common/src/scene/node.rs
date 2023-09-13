use std::rc::Rc;

use serde::{Deserialize, Serialize};

use crate::{error::NodeSpecValidationError, renderer_spec::RendererId};

use super::{
    builtin_transformations::BuiltinSpec, shader::ShaderParam, text_spec::TextSpec, NodeSpec,
    Resolution,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NodeParams {
    WebRenderer {
        instance_id: RendererId,
    },
    Shader {
        shader_id: RendererId,
        shader_params: Option<ShaderParam>,
        resolution: Resolution,
    },
    Text(TextSpec),
    Image {
        image_id: RendererId,
    },
    #[serde(rename = "built-in")]
    Builtin {
        #[serde(flatten)]
        transformation: BuiltinSpec,
    },
}

impl NodeSpec {
    pub fn validate_params(&self) -> Result<(), NodeSpecValidationError> {
        match &self.params {
            NodeParams::Builtin { transformation, .. } => Ok(transformation.validate_params(self)?),
            _ => Ok(()),
        }
    }

    pub fn identification_name(&self) -> Rc<str> {
        match &self.params {
            NodeParams::WebRenderer { instance_id } => {
                Rc::from(format!("\"{}\" web renderer", instance_id))
            }
            NodeParams::Shader { shader_id, .. } => Rc::from(format!("\"{}\" shader", shader_id)),
            NodeParams::Text(_) => Rc::from("Text renderer"),
            NodeParams::Image { .. } => Rc::from("Image"),
            NodeParams::Builtin { transformation } => Rc::from(format!(
                "\"{}\" builtin transformation",
                transformation.transformation_name()
            )),
        }
    }
}
