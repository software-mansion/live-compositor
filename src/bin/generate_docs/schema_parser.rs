use std::collections::VecDeque;

use schemars::schema::{InstanceType, RootSchema, SchemaObject, SingleOrVec};

use crate::{
    definition::{Kind, TypeDefinition},
    docs_config::DocsConfig,
};

use self::utils::{inner_schema_object, SchemaObjectExt};

pub mod utils;

pub fn parse_schema(
    name: String,
    mut root_schema: RootSchema,
    config: &DocsConfig,
) -> Vec<TypeDefinition> {
    let mut definitions = Vec::new();

    let mut root_definition = parse_schema_object(root_schema.schema);
    root_definition.name = Some(name);

    let mut references = VecDeque::from(root_definition.references());

    definitions.push(root_definition);

    while let Some(reference) = references.pop_front() {
        // Ignore definitions that are explicitly ignored
        if config.ignored_definitions.contains(&reference.as_str()) {
            continue;
        }

        // Ignore definitions that are already parsed
        if definitions
            .iter()
            .any(|def| def.name.as_ref() == Some(&reference))
        {
            continue;
        }

        let schema = root_schema
            .definitions
            .remove(&reference)
            .unwrap_or_else(|| panic!("Definition not found: {reference}"));

        let mut definition = parse_schema_object(schema.into_object());
        definition.name = Some(reference);
        references.extend(definition.references());

        definitions.push(definition);
    }

    definitions
}

pub fn parse_schema_object(schema: SchemaObject) -> TypeDefinition {
    match schema.is_union() {
        true => parse_union_type(schema),
        false => parse_single_type(schema),
    }
}

fn parse_union_type(schema: SchemaObject) -> TypeDefinition {
    let description = schema.description();
    let is_optional = schema.is_optional();

    let mut variants = schema
        .union_variants()
        .into_iter()
        .map(parse_schema_object)
        .collect::<Vec<_>>();

    // Sometimes unions have additional properties for object variants
    // We need to merge these properties with the object variants
    if let Some(fields) = schema.object {
        let fields = fields
            .properties
            .into_iter()
            .map(|(name, prop)| {
                let field = parse_schema_object(prop.into_object());
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
        name: None,
        description,
        kind: Kind::Union(variants),
        is_optional,
    }
}

fn parse_single_type(schema: SchemaObject) -> TypeDefinition {
    let description = schema.description();
    let is_optional = schema.is_optional();
    let schema = inner_schema_object(schema);

    if let Some(reference) = schema.reference {
        return TypeDefinition {
            name: None,
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

    let kind = parse_instance_type(schema, instance_type);

    TypeDefinition {
        name: None,
        description,
        kind,
        is_optional,
    }
}

pub fn parse_instance_type(schema: SchemaObject, ty: InstanceType) -> Kind {
    match ty {
        InstanceType::Null => Kind::Null,
        InstanceType::Boolean => Kind::Bool,
        InstanceType::Number | InstanceType::Integer => parse_number(schema),
        InstanceType::String => parse_string(schema),
        InstanceType::Object if schema.is_map() => parse_map(schema),
        InstanceType::Object => parse_object(schema),
        InstanceType::Array if schema.is_tuple() => parse_tuple(schema),
        InstanceType::Array => parse_array(schema),
    }
}

pub fn parse_tuple(schema: SchemaObject) -> Kind {
    let tuple = schema.array.unwrap();
    let types = match tuple.items.unwrap() {
        SingleOrVec::Single(item) => {
            let ty = parse_schema_object(item.into_object());
            // Only one type was specified, so we need to duplicate it `min_items` times
            vec![ty; tuple.min_items.unwrap() as usize]
        }
        SingleOrVec::Vec(items) => items
            .into_iter()
            .map(|item| parse_schema_object(item.into_object()))
            .collect(),
    };

    Kind::Tuple(types)
}

pub fn parse_array(schema: SchemaObject) -> Kind {
    let array = schema.array.unwrap();
    let Some(SingleOrVec::Single(array_type)) = array.items else {
        unimplemented!("Arrays with only one type are supported");
    };

    let array_type = parse_schema_object(array_type.into_object());
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
                    name: None,
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

pub fn parse_map(schema: SchemaObject) -> Kind {
    let map = schema.object.unwrap();
    let value_type = parse_schema_object(map.additional_properties.unwrap().into_object());

    Kind::Map {
        value_type: Box::new(value_type),
    }
}

pub fn parse_object(schema: SchemaObject) -> Kind {
    let object = schema.object.unwrap();
    let mut fields = Vec::new();

    for (name, prop) in object.properties {
        let field = parse_schema_object(prop.into_object());
        fields.push((name, field));
    }

    Kind::Object { fields }
}
