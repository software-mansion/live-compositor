use std::{collections::HashSet, fmt::Display, rc::Rc};

use log::error;

use crate::scene::{validation::constraints::input_count::InputsCountConstraint, NodeId, OutputId};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SceneSpecValidationError {
    #[error("Unknown node \"{missing_node}\" used as an input in the node \"{node}\". Node is not defined in the scene and it was not registered as an input.")]
    UnknownInputPadOnNode { missing_node: NodeId, node: NodeId },
    #[error("Unknown node \"{missing_node}\" is connected to the output stream \"{output}\".")]
    UnknownInputPadOnOutput {
        missing_node: NodeId,
        output: OutputId,
    },
    #[error(
        "Unknown output stream \"{0}\". Register it first before using it in the scene definition."
    )]
    UnknownOutput(NodeId),
    #[error("Invalid node id. There is more than one node with the \"{0}\" id.")]
    DuplicateNodeNames(NodeId),
    #[error("Invalid node id. There is already an input stream with the \"{0}\" id.")]
    DuplicateNodeAndInputNames(NodeId),
    #[error("Cycles between nodes are not allowed. Node \"{0}\" depends on itself via input_pads or fallback option.")]
    CycleDetected(NodeId),
    #[error(transparent)]
    UnusedNodes(#[from] UnusedNodesError),
    #[error("Invalid params for node \"{1}\".")]
    InvalidNodeSpec(#[source] NodeSpecValidationError, NodeId),
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub struct UnusedNodesError(pub HashSet<NodeId>);

impl Display for UnusedNodesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut unused_nodes: Vec<String> = self.0.iter().map(ToString::to_string).collect();
        unused_nodes.sort();
        write!(
            f,
            "There are unused nodes in the scene definition: {0}",
            unused_nodes.join(", ")
        )
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum UnsatisfiedConstraintsError {
    #[error(
        "Invalid input pads specification for node \"{}\". {} requires {}. {} input pads were specified.",
        node_id,
        identification_name,
        input_count_constrain.required_inputs_message(),
        defined_input_pads_count
    )]
    InvalidInputsCount {
        node_id: NodeId,
        identification_name: Rc<str>,
        input_count_constrain: InputsCountConstraint,
        defined_input_pads_count: u32,
    },
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum NodeSpecValidationError {
    #[error(transparent)]
    Builtin(#[from] BuiltinSpecValidationError),
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum BuiltinSpecValidationError {
    #[error("Transformation \"fixed_position_layout\" expect {input_count} texture layouts (the same as number of input pads), but {layout_count} layouts were specified.")]
    FixedLayoutInvalidLayoutCount { layout_count: u32, input_count: u32 },
    #[error("Each entry in texture_layouts in transformation \"fixed_position_layout\" requires either bottom or top coordinate.")]
    FixedLayoutTopBottomRequired,
    #[error("Each entry in texture_layouts in transformation \"fixed_position_layout\" requires either right or left coordinate.")]
    FixedLayoutLeftRightRequired,
    #[error("Fields \"top\" and \"bottom\" are mutually exclusive, you can only specify one in texture layout in \"fixed_position_layout\" transformation.")]
    FixedLayoutTopBottomOnlyOne,
    #[error("Fields \"left\" and \"right\" are mutually exclusive, you can only specify one in texture layout in \"fixed_position_layout\" transformation.")]
    FixedLayoutLeftRightOnlyOne,
}

pub struct ErrorStack<'a>(Option<&'a (dyn std::error::Error + 'static)>);

impl<'a> ErrorStack<'a> {
    pub fn new(value: &'a (dyn std::error::Error + 'static)) -> Self {
        ErrorStack(Some(value))
    }

    pub fn into_string(self) -> String {
        let stack: Vec<String> = self.map(ToString::to_string).collect();
        stack.join("\n")
    }
}

impl<'a> Iterator for ErrorStack<'a> {
    type Item = &'a (dyn std::error::Error + 'static);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.map(|err| {
            self.0 = err.source();
            err
        })
    }
}
