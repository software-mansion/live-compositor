use std::time::Duration;

pub(super) trait DurationExt {
    fn chrono(self) -> chrono::Duration;
}

impl DurationExt for Duration {
    fn chrono(self) -> chrono::Duration {
        chrono::Duration::from_std(self).unwrap_or(chrono::Duration::max_value())
    }
}
