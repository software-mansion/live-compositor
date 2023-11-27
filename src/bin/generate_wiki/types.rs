use schemars::{schema::Schema, JsonSchema};
use serde_json::Value;
use video_compositor::types;

pub struct Component;

impl JsonSchema for Component {
    fn schema_name() -> String {
        "Component".into()
    }

    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> Schema {
        fn to_pascal_case(text: &str) -> String {
            text.split('_')
                .map(|v| {
                    let mut chars = v.chars().collect::<Vec<_>>();
                    chars[0] = chars[0].to_uppercase().next().unwrap();
                    chars.into_iter().collect::<String>()
                })
                .collect()
        }

        fn add_titles(schemas: Vec<Schema>) -> Vec<Schema> {
            schemas
                .into_iter()
                .map(|schema| {
                    let mut schema = schema.into_object();
                    let title = schema
                        .object()
                        .properties
                        .get("type")
                        .map(|ty| ty.clone().into_object())
                        .and_then(|s| s.enum_values)
                        .and_then(|v| match v.first() {
                            Some(Value::String(title)) => Some(to_pascal_case(title)),
                            _ => None,
                        });

                    schema.metadata().title = title;
                    Schema::Object(schema)
                })
                .collect()
        }

        let mut schema = gen.root_schema_for::<types::Component>().schema;
        gen.definitions_mut().remove("Component");

        let subschemas = schema.subschemas();

        if let Some(all_of) = &mut subschemas.all_of {
            *all_of = add_titles(all_of.clone());
        }
        if let Some(any_of) = &mut subschemas.any_of {
            *any_of = add_titles(any_of.clone());
        }
        if let Some(one_of) = &mut subschemas.one_of {
            *one_of = add_titles(one_of.clone());
        }

        Schema::Object(schema)
    }
}
