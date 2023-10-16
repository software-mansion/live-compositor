use std::{fs, io, path::PathBuf};

use schemars::{schema::RootSchema, schema_for};
use video_compositor::types;

const ROOT_DIR: &str = env!("CARGO_MANIFEST_DIR");

fn main() {
    let update_flag = std::env::args().any(|arg| &arg == "--update");

    generate_schema(schema_for!(types::Scene), "scene", update_flag);
}

fn generate_schema(current_schema: RootSchema, name: &'static str, update: bool) {
    let root_dir: PathBuf = ROOT_DIR.into();
    let schema_path = root_dir.join(format!("schemas/{}.schema.json", name));
    fs::create_dir_all(schema_path.parent().unwrap()).unwrap();

    let json_from_disk = match fs::read_to_string(&schema_path) {
        Ok(v) => Some(serde_json::from_str::<serde_json::Value>(&v).unwrap()),
        Err(err) if err.kind() == io::ErrorKind::NotFound => None,
        Err(err) => panic!("{}", err),
    };
    let json_current = serde_json::to_value(&current_schema).unwrap();

    if json_from_disk.is_none() || json_current != json_from_disk.unwrap() {
        if update {
            fs::write(
                schema_path,
                serde_json::to_string_pretty(&json_current).unwrap(),
            )
            .unwrap();
        } else {
            panic!("Schema changed. Rerun with --update to regenerate it.")
        }
    }
}
