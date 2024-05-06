use std::{collections::VecDeque, marker::PhantomData};

use schemars::schema::{InstanceType, RootSchema, SchemaObject, SingleOrVec};

use crate::{
    definition::{Kind, TopLevelDefinition, TypeDefinition},
    docs_config::DocsConfig,
    generation_strategy::GenerationStrategy,
};

use self::utils::{inner_schema_object, SchemaObjectExt};

pub mod utils;

pub struct SchemaParser<'a, S> {
    config: &'a DocsConfig,
    _strategy: PhantomData<S>,
}

impl<'a, S: GenerationStrategy> SchemaParser<'a, S> {
    pub fn parse(
        name: String,
        mut root_schema: RootSchema,
        config: &DocsConfig,
    ) -> Vec<TopLevelDefinition> {
        let parser: SchemaParser<'_, S> = SchemaParser {
            config,
            _strategy: PhantomData,
        };
        let mut definitions = Vec::new();

        let root_definition = parser.parse_schema_object(root_schema.schema);
        let mut references = VecDeque::from(root_definition.references());

        definitions.push(TopLevelDefinition::new(name, root_definition));

        while let Some(reference) = references.pop_front() {
            // Ignore definitions that are explicitly ignored
            if parser
                .config
                .ignored_definitions
                .contains(&reference.as_str())
            {
                continue;
            }

            // Ignore definitions that are already parsed
            if definitions.iter().any(|def| def.name == reference) {
                continue;
            }

            let schema = root_schema
                .definitions
                .remove(&reference)
                .unwrap_or_else(|| panic!("Definition not found: {reference}"));

            let definition = parser.parse_schema_object(schema.into_object());
            references.extend(definition.references());
            definitions.push(TopLevelDefinition::new(reference, definition));
        }

        definitions
    }

    pub fn parse_schema_object(&self, schema: SchemaObject) -> TypeDefinition {
        match schema.is_union() {
            true => self.parse_union_type(schema),
            false => self.parse_single_type(schema),
        }
    }

    fn parse_union_type(&self, schema: SchemaObject) -> TypeDefinition {
        let description = schema.description();
        let is_optional = schema.is_optional();

        let mut variants = schema
            .union_variants()
            .into_iter()
            .map(|schema| self.parse_schema_object(schema))
            .collect::<Vec<_>>();

        // Sometimes unions have additional properties for object variants
        // We need to merge these properties with the object variants
        if let Some(fields) = schema.object {
            let fields = fields
                .properties
                .into_iter()
                .map(|(name, prop)| {
                    let field = self.parse_schema_object(prop.into_object());
                    (name, field)
                })
                .collect::<Vec<_>>();

            for variant in variants.iter_mut() {
                if let Kind::Object {
                    fields: variant_fields,
                } = &mut variant.kind
                {
                    *variant_fields = [fields.as_slice(), variant_fields.as_slice()].concat();
                }
            }
        }

        TypeDefinition {
            description,
            kind: Kind::Union(variants),
            is_optional,
        }
    }

    fn parse_single_type(&self, schema: SchemaObject) -> TypeDefinition {
        let description = schema.description();
        let is_optional = schema.is_optional();
        let schema = inner_schema_object(schema);

        if let Some(reference) = schema.reference {
            return TypeDefinition {
                description,
                kind: Kind::Ref(reference),
                is_optional,
            };
        }

        let instance_types = schema
            .instance_type
            .clone()
            .expect("Instance type not found");

        let instance_type = match instance_types {
            SingleOrVec::Single(ty) => *ty,
            SingleOrVec::Vec(types) => match types.as_slice() {
                [InstanceType::Null, ty] => *ty,
                [ty, InstanceType::Null] => *ty,
                _ => {
                    panic!("Unsupported instance type: {:?}", types);
                }
            },
        };

        let kind = S::parse_instance_type(self, schema, instance_type);

        TypeDefinition {
            description,
            kind,
            is_optional,
        }
    }
}
