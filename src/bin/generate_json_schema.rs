use std::{fs, io, path::PathBuf};

use schemars::{
    schema::{RootSchema, Schema, SchemaObject},
    schema_for,
};
use video_compositor::types;

const ROOT_DIR: &str = env!("CARGO_MANIFEST_DIR");

fn main() {
    let update_flag = std::env::args().any(|arg| &arg == "--update");

    generate_schema(schema_for!(types::Scene), "scene", update_flag);
    generate_schema(schema_for!(types::RegisterRequest), "register", update_flag);
}

/// When variant inside oneOf has a schema additionalProperties set to false then
/// all the values outside of the variant are not allowed.
///
/// This function copies all the entries from `properties` to `oneOf[variant].properties`.
fn flatten_definitions_with_one_of(schema: &mut RootSchema) {
    for (_, schema) in schema.definitions.iter_mut() {
        match schema {
            Schema::Bool(_) => (),
            Schema::Object(definition) => flatten_definition_with_one_of(definition),
        }
    }
}

fn flatten_definition_with_one_of(definition: &mut SchemaObject) {
    let Some(ref properties) = definition.object.clone() else {
        return;
    };

    let Some(ref mut one_of) = definition.subschemas().one_of else {
        return;
    };

    for variant in one_of.iter_mut() {
        match variant {
            Schema::Bool(_) => (),
            Schema::Object(ref mut variant) => {
                for (prop_name, prop) in properties.properties.iter() {
                    variant
                        .object()
                        .properties
                        .insert(prop_name.clone(), prop.clone());
                }
            }
        }
    }
}

fn generate_schema(mut current_schema: RootSchema, name: &'static str, update: bool) {
    flatten_definitions_with_one_of(&mut current_schema);

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
