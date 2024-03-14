use anyhow::Result;

use crate::CompositorInstance;

pub use self::simple::run_simple_test;

mod simple;

pub struct IntegrationTest {
    pub name: &'static str,
    port: u16,
    run: fn(instance: CompositorInstance, update_dumps: bool) -> Result<()>,
}

impl IntegrationTest {
    pub fn run_test(&self) -> Result<()> {
        (self.run)(CompositorInstance::start(self.port), false)
    }

    pub fn run_update(&self) -> Result<()> {
        (self.run)(CompositorInstance::start(self.port), true)
    }
}

// All integration tests should be added here
pub fn integration_tests() -> Vec<IntegrationTest> {
    vec![IntegrationTest {
        name: "simple",
        port: 8000,
        run: run_simple_test,
    }]
}
