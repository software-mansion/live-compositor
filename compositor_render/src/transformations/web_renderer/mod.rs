mod electron_api;
pub mod transformation;

use serde::{Deserialize, Serialize};
pub use transformation::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionId(pub String);
