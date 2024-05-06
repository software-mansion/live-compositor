use schemars::JsonSchema;

use crate::{
    docs_config::DocsConfig,
    generation_strategy::{
        parsing_utils::{flatten_unions, inline_definitions},
        DefaultGenerationStrategy, GenerationStrategy,
    },
    schema_parser::{utils::new_schema_generator, SchemaParser},
};

pub struct Doc<T: JsonSchema, S: GenerationStrategy = DefaultGenerationStrategy> {
    _phantom: std::marker::PhantomData<(T, S)>,
}

pub struct MarkdownDoc {
    pub title: String,
    pub markdown: String,
}

impl<T: JsonSchema, S: GenerationStrategy> Doc<T, S> {
    pub fn generate(name: &str, config: &DocsConfig) -> MarkdownDoc {
        let schema_generator = new_schema_generator();
        let schema = schema_generator.into_root_schema_for::<T>();
        let mut definitions = SchemaParser::<S>::parse(name.to_string(), schema, config);

        flatten_unions(&mut definitions);
        inline_definitions(&mut definitions, config);

        MarkdownDoc {
            title: name.to_string(),
            markdown: S::to_markdown(definitions, config),
        }
    }
}
