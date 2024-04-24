use schemars::schema::{InstanceType, SchemaObject};

use crate::{
    definition::{Kind, TopLevelDefinition},
    docs_config::DocsConfig,
    schema_parser::{utils::SchemaObjectExt, SchemaParser},
};

mod legacy_markdown_generation;
pub mod parsing_utils;

pub trait GenerationStrategy {
    fn parse_instance_type<S>(
        parser: &SchemaParser<'_, S>,
        schema: SchemaObject,
        ty: InstanceType,
    ) -> Kind
    where
        S: GenerationStrategy,
        Self: Sized,
    {
        use parsing_utils::*;

        match ty {
            InstanceType::Null => Kind::Null,
            InstanceType::Boolean => Kind::Bool,
            InstanceType::Number | InstanceType::Integer => parse_number(schema),
            InstanceType::String => parse_string(schema),
            InstanceType::Object if schema.is_map() => parse_map(parser, schema),
            InstanceType::Object => parse_object(parser, schema),
            InstanceType::Array if schema.is_tuple() => parse_tuple(parser, schema),
            InstanceType::Array => parse_array(parser, schema),
        }
    }

    // NOTE(noituri): This will be rewritten in the follow up PR
    fn to_markdown(definitions: Vec<TopLevelDefinition>, config: &DocsConfig) -> String {
        let mut markdown = String::new();
        for def in definitions {
            markdown += &def.to_markdown(config);
        }

        markdown
    }
}

pub struct DefaultGenerationStrategy;

impl GenerationStrategy for DefaultGenerationStrategy {}
