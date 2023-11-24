use std::{fmt::Display, sync::Arc};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct InputId(pub Arc<str>);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct OutputId(pub Arc<str>);

impl Display for InputId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Display for OutputId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<Arc<str>> for InputId {
    fn from(value: Arc<str>) -> Self {
        Self(value)
    }
}

impl From<Arc<str>> for OutputId {
    fn from(value: Arc<str>) -> Self {
        Self(value)
    }
}
