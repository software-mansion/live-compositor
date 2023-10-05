use schemars::schema_for;
use video_compositor::types;

fn main() {
    let schema = schema_for!(types::Node);
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}
