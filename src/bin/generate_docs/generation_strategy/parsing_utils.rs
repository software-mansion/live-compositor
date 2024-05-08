use std::collections::HashMap;

use schemars::schema::{SchemaObject, SingleOrVec};

use crate::{
    definition::{Kind, TopLevelDefinition, TypeDefinition},
    docs_config::DocsConfig,
    schema_parser::SchemaParser,
};

use super::GenerationStrategy;

pub fn parse_tuple<S: GenerationStrategy>(
    parser: &SchemaParser<'_, S>,
    schema: SchemaObject,
) -> Kind {
    let tuple = schema.array.unwrap();
    let types = match tuple.items.unwrap() {
        SingleOrVec::Single(item) => {
            let ty = parser.parse_schema_object(item.into_object());
            // Only one type was specified, so we need to duplicate it `min_items` times
            vec![ty; tuple.min_items.unwrap() as usize]
        }
        SingleOrVec::Vec(items) => items
            .into_iter()
            .map(|item| parser.parse_schema_object(item.into_object()))
            .collect(),
    };

    Kind::Tuple(types)
}

pub fn parse_array<S: GenerationStrategy>(
    parser: &SchemaParser<'_, S>,
    schema: SchemaObject,
) -> Kind {
    let array = schema.array.unwrap();
    let Some(SingleOrVec::Single(array_type)) = array.items else {
        unimplemented!("Arrays with only one type are supported");
    };

    let array_type = parser.parse_schema_object(array_type.into_object());
    Kind::Array {
        array_type: Box::new(array_type),
    }
}

pub fn parse_string(schema: SchemaObject) -> Kind {
    match schema.enum_values {
        Some(values) if values.is_empty() => Kind::String {
            specific_value: None,
        },
        // String has predefined possible values
        Some(values) if values.len() == 1 => Kind::String {
            specific_value: Some(values[0].to_string()),
        },
        Some(values) => Kind::Union(
            values
                .into_iter()
                .map(|v| TypeDefinition {
                    description: String::new(),
                    kind: Kind::String {
                        specific_value: Some(v.to_string()),
                    },
                    is_optional: false,
                })
                .collect(),
        ),
        None => Kind::String {
            specific_value: None,
        },
    }
}

pub fn parse_number(schema: SchemaObject) -> Kind {
    let number_format = schema.format.unwrap();
    match number_format.as_str() {
        "float" => Kind::F32,
        "double" => Kind::F64,
        "uint32" | "uint" => Kind::U32,
        "int32" | "int" => Kind::I32,
        "uint16" => Kind::U16,
        "uint8" => Kind::U8,
        format => unimplemented!("Unknown number format \"{format}\""),
    }
}

pub fn parse_map<S: GenerationStrategy>(
    parser: &SchemaParser<'_, S>,
    schema: SchemaObject,
) -> Kind {
    let map = schema.object.unwrap();
    let value_type = parser.parse_schema_object(map.additional_properties.unwrap().into_object());

    Kind::Map {
        value_type: Box::new(value_type),
    }
}

pub fn parse_object<S: GenerationStrategy>(
    parser: &SchemaParser<'_, S>,
    schema: SchemaObject,
) -> Kind {
    let object = schema.object.unwrap();
    let mut fields = Vec::new();

    for (name, prop) in object.properties {
        let field = parser.parse_schema_object(prop.into_object());
        fields.push((name, field));
    }

    Kind::Object { fields }
}

/// Flattens nested unions into a single union
pub fn flatten_unions(definitions: &mut [TopLevelDefinition]) {
    fn flatten(def: &mut TypeDefinition) {
        let Kind::Union(variants) = &mut def.kind else {
            return;
        };

        let mut variants_to_merge = Vec::new();
        let mut variants_to_remove = Vec::new();
        for (i, variant) in variants.iter_mut().enumerate() {
            flatten(variant);

            if let Kind::Union(mut subvariants) = variant.kind.clone() {
                variants_to_merge.append(&mut subvariants);
                variants_to_remove.push(i);
            }
        }

        // Remove variants that were merged
        for i in variants_to_remove.into_iter().rev() {
            variants.remove(i);
        }

        variants.append(&mut variants_to_merge);
    }

    for TopLevelDefinition { definition, .. } in definitions.iter_mut() {
        flatten(definition);
    }
}

/// Inlines definitions that are defined in config as always inlined or are inlineable (simple types, such as strings, numbers, etc.)
pub fn inline_definitions(definitions: &mut Vec<TopLevelDefinition>, config: &DocsConfig) {
    fn inline_for(
        definition: &mut TypeDefinition,
        definitions_to_inline: &HashMap<String, TypeDefinition>,
    ) {
        match &mut definition.kind {
            Kind::Ref(reference) => {
                if let Some(inline_def) = definitions_to_inline.get(reference) {
                    *definition = definition.merge_into(inline_def);
                }
            }
            Kind::Tuple(variants) | Kind::Union(variants) => {
                variants
                    .iter_mut()
                    .for_each(|variant| inline_for(variant, definitions_to_inline));
            }
            Kind::Array { array_type } => inline_for(array_type, definitions_to_inline),
            Kind::Object { fields } => {
                fields
                    .iter_mut()
                    .for_each(|(_, prop)| inline_for(prop, definitions_to_inline));
            }
            Kind::Map { value_type } => inline_for(value_type, definitions_to_inline),
            _ => {}
        }
    }

    let mut definitions_to_inline = HashMap::new();

    for TopLevelDefinition { name, definition } in definitions.iter() {
        if config.never_inlined_definitions.contains(&name.as_str()) {
            continue;
        }

        let should_inline = definition.kind.inlineable_by_default()
            || config.always_inlined_definitions.contains(&name.as_str());

        if should_inline {
            definitions_to_inline.insert(name.clone(), definition.clone());
        }
    }

    // Remove top level definitions that are inlined
    definitions.retain(|def| !definitions_to_inline.contains_key(&def.name));

    for TopLevelDefinition { definition, .. } in definitions.iter_mut() {
        inline_for(definition, &definitions_to_inline);
    }
}
