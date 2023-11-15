use std::{fmt::Display, sync::Arc};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct NodeId(pub Arc<str>);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct InputId(pub NodeId);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct OutputId(pub NodeId);

impl Display for InputId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0 .0.fmt(f)
    }
}

impl Display for OutputId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0 .0.fmt(f)
    }
}

impl Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<NodeId> for InputId {
    fn from(value: NodeId) -> Self {
        Self(value)
    }
}

impl From<NodeId> for OutputId {
    fn from(value: NodeId) -> Self {
        Self(value)
    }
}

impl From<Arc<str>> for InputId {
    fn from(value: Arc<str>) -> Self {
        Self(NodeId(value))
    }
}

impl From<Arc<str>> for OutputId {
    fn from(value: Arc<str>) -> Self {
        Self(NodeId(value))
    }
}
