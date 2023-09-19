use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct Degree(pub f64);

impl Eq for Degree {
    fn assert_receiver_is_total_eq(&self) {}
}
