mod test_runner;
mod tests;
mod utils;

use tests::run_tests;

fn main() {
    run_tests();
    println!("All snapshot tests finished successfully")
}
