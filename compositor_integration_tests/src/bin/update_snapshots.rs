use std::env;

use compositor_integration_tests::{integration_test_prerequisites, integration_tests};

fn main() {
    env::set_var("LIVE_COMPOSITOR_LOGGER_LEVEL", "warn");
    integration_test_prerequisites();

    for run_test in integration_tests() {
        run_test(true).unwrap();
    }
}
