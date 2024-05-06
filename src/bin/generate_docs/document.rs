use std::collections::HashMap;

use schemars::JsonSchema;

use crate::{
    definition::{Kind, TypeDefinition},
    docs_config::DocsConfig,
    generation::MarkdownGenerator,
    schema_parser::{parse_schema, utils::new_schema_generator},
};

pub struct MarkdownDoc {
    pub title: String,
    pub markdown: String,
}

pub fn generate<T: JsonSchema>(name: &str, config: &DocsConfig) -> MarkdownDoc {
    let schema_generator = new_schema_generator();
    let schema = schema_generator.into_root_schema_for::<T>();
    let mut definitions = parse_schema(name.to_string(), schema, config);

    flatten_unions(&mut definitions);
    inline_definitions(&mut definitions, config);

    let mut markdown = String::new();
    for definition in definitions {
        markdown += &MarkdownGenerator::generate(definition, config)
    }

    MarkdownDoc {
        title: name.to_string(),
        markdown,
    }
}

/// Flattens nested unions into a single union
pub fn flatten_unions(definitions: &mut [TypeDefinition]) {
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

    for definition in definitions.iter_mut() {
        flatten(definition);
    }
}

/// Inlines definitions that are defined in config as always inlined or are inlineable (simple types, such as strings, numbers, etc.)
pub fn inline_definitions(definitions: &mut Vec<TypeDefinition>, config: &DocsConfig) {
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

    for definition in definitions.iter() {
        let name = definition.name.as_deref().unwrap();

        if config.never_inlined_definitions.contains(&name) {
            continue;
        }

        let should_inline = definition.kind.inlineable_by_default()
            || config.always_inlined_definitions.contains(&name);

        if should_inline {
            definitions_to_inline.insert(name.to_owned(), definition.clone());
        }
    }

    // Remove top level definitions that are inlined
    definitions.retain(|def| !definitions_to_inline.contains_key(def.name.as_deref().unwrap()));

    for definition in definitions.iter_mut() {
        inline_for(definition, &definitions_to_inline);
    }
}
