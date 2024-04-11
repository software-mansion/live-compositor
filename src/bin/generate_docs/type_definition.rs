use std::{ops::Deref, rc::Rc};

use crate::docs_config::DocsConfig;

const INDENT: &str = " ";

#[derive(Debug, Clone, PartialEq)]
pub struct TypeDefinition {
    pub name: Option<Rc<str>>,
    pub description: Option<Rc<str>>,
    pub kind: Kind,
    pub is_optional: bool,
    pub references: Vec<Rc<str>>,
}

impl TypeDefinition {
    pub fn complex(
        name: Option<String>,
        description: Option<String>,
        kind: Kind,
        is_optional: bool,
    ) -> Self {
        let mut references = Vec::new();
        find_references(&kind, &mut references);

        Self {
            name: name.map(Into::into),
            description: description.map(Into::into),
            kind,
            is_optional,
            references,
        }
    }

    pub fn simple(kind: Kind, is_optional: bool) -> Self {
        Self::complex(None, None, kind, is_optional)
    }

    pub fn to_markdown(&self, config: &DocsConfig) -> String {
        let name = self.name.as_ref().unwrap();
        let description = self
            .description
            .as_ref()
            .map(|desc| format!("{desc}\n"))
            .unwrap_or_default();

        format!(
            "## {}\n```typescript\ntype {} = {}\n```\n{}{}",
            name,
            name,
            self.to_pretty_string(0),
            description,
            self.properties_markdown(config),
        )
    }

    fn properties_markdown(&self, config: &DocsConfig) -> String {
        fn format_props(properties: &[ObjectProperty]) -> String {
            properties
                .iter()
                .map(|prop| prop.to_markdown())
                .collect::<String>()
        }

        fn format_object_variant(key: Option<&str>, mut properties: Vec<ObjectProperty>) -> String {
            let properties_key = match key {
                Some(key) => {
                    let key_value = properties
                        .iter()
                        .find(|prop| prop.name.deref() == key)
                        .cloned()
                        .unwrap();
                    properties.retain(|prop| prop.name.deref() != key);

                    format!("(`type: {}`)", key_value.type_def.to_pretty_string(0))
                }
                None => String::new(),
            };

            let properties = format_props(&properties);
            if properties.is_empty() {
                return String::new();
            }

            format!("#### Properties {properties_key}\n{properties}")
        }

        match &self.kind {
            Kind::Object(properties) => {
                let properties = format_props(properties);
                if properties.is_empty() {
                    return String::new();
                }

                format!("#### Properties\n{properties}")
            }
            Kind::Union(variants) => {
                let key = self
                    .name
                    .clone()
                    .and_then(|name| config.variant_discriminators.get(name.deref()).cloned());

                let mut properties_string = String::new();
                for variant in variants {
                    let variant_description = match variant.kind.clone() {
                        Kind::Object(props) => format_object_variant(key, props),
                        _ => match &variant.description {
                            Some(description) => {
                                format!("#### {}\n{}", variant.to_pretty_string(0), description)
                            }
                            None => String::new(),
                        },
                    };

                    properties_string += &variant_description;
                }

                properties_string
            }
            _ => String::new(),
        }
    }

    fn object_to_string(properties: &[ObjectProperty], base_indent: usize) -> String {
        let mut out = "{\n".to_owned();
        for prop in properties {
            let mut name = prop.name.clone();
            if prop.type_def.is_optional {
                name = format!("{name}?").into();
            }

            let indent = match &prop.type_def.kind {
                Kind::Union(_) => base_indent + 2,
                Kind::Object(_) => base_indent + 2,
                _ => base_indent,
            };
            out += &format!(
                "{}{}: {};\n",
                INDENT.repeat(base_indent + 2),
                name,
                prop.type_def.to_pretty_string(indent)
            );
        }
        format!("{}{}}}", out, INDENT.repeat(base_indent))
    }

    fn object_to_single_line(properties: &[ObjectProperty]) -> String {
        let properties = properties
            .iter()
            .map(|prop| {
                let mut name = prop.name.clone();
                if prop.type_def.is_optional {
                    name = format!("{name}?").into();
                }
                format!("{}: {}", name, prop.type_def.to_pretty_string(0))
            })
            .collect::<Vec<_>>()
            .join("; ");
        format!("{{ {} }}", properties)
    }

    fn union_to_string(variants: &[TypeDefinition], base_indent: usize) -> String {
        fn variant_to_string(variant: &TypeDefinition, indent: usize) -> String {
            match &variant.kind {
                Kind::Object(properties) => {
                    let single_line = TypeDefinition::object_to_single_line(properties);
                    if single_line.len() < 60 {
                        return single_line;
                    }
                    TypeDefinition::object_to_string(properties, indent)
                }
                _ => variant.to_pretty_string(indent),
            }
        }

        let variant_indent = match variants.iter().any(|v| matches!(v.kind, Kind::Object(_))) {
            true => base_indent + 4,
            false => base_indent + 2,
        };

        let variants = variants
            .iter()
            .map(|ty: &TypeDefinition| variant_to_string(ty, variant_indent));
        let format_variants = |use_new_lines: bool| {
            let variants = variants.clone();
            if use_new_lines {
                variants.into_iter().fold(String::new(), |acc, ty| {
                    format!("{}\n{}| {}", acc, INDENT.repeat(base_indent + 2), ty)
                })
            } else {
                variants.collect::<Vec<_>>().join(" | ")
            }
        };

        let union_string = format_variants(variants.len() > 4);
        if union_string.len() > 80 {
            format_variants(true)
        } else {
            union_string
        }
    }

    fn to_pretty_string(&self, base_indent: usize) -> String {
        match &self.kind {
            Kind::Null => "null".into(),
            Kind::I32 => "i32".into(),
            Kind::F32 => "f32".into(),
            Kind::F64 => "f64".into(),
            Kind::U32 => "u32".into(),
            Kind::U16 => "u16".into(),
            Kind::U8 => "u8".into(),
            Kind::Bool => "bool".into(),
            Kind::String(value) => match value {
                Some(value) => format!("\"{value}\""),
                None => "string".into(),
            },
            Kind::Ref(reference) => reference.to_string(),
            Kind::Tuple(types) => format!(
                "[{}]",
                types
                    .iter()
                    .map(|ty| ty.to_pretty_string(base_indent))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Kind::Array(ty) => format!("{}[]", ty.to_pretty_string(base_indent)),
            Kind::Union(variants) => Self::union_to_string(variants, base_indent),
            Kind::Object(properties) => Self::object_to_string(properties, base_indent),
            Kind::Map { value_type } => format!("Map<string, {}>", value_type.to_pretty_string(0)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectProperty {
    pub name: Rc<str>,
    pub type_def: TypeDefinition,
}

impl ObjectProperty {
    fn to_markdown(&self) -> String {
        fn format_description(name: &str, description: &str, indent_size: usize) -> String {
            let mut description_parts = description.split('\n');
            let mut output_description = format!("{}- `{}` ", INDENT.repeat(indent_size), name);
            match description_parts.next() {
                Some(desc) if !desc.trim().is_empty() => {
                    output_description += &format!("- {desc}\n");
                }
                _ => output_description += "\n",
            }
            for desc in description_parts {
                if desc.trim().is_empty() {
                    continue;
                }
                output_description += &format!("{}{}\n", INDENT.repeat(indent_size + 2), desc);
            }

            output_description
        }
        let description = self
            .type_def
            .description
            .as_ref()
            .map(|desc| desc.to_string())
            .unwrap_or_default();

        let output_markdown = format_description(&self.name, &description, 0);
        let variants = match &self.type_def.kind {
            Kind::Union(variants) => variants.clone(),
            _ => Vec::new(),
        };

        let mut variants_markdown = String::new();
        for variant in variants {
            let name = variant.to_pretty_string(0);
            let Some(description) = &variant.description else {
                continue;
            };
            variants_markdown += &format_description(&name, description, 2)
        }

        if description.trim().is_empty() && variants_markdown.is_empty() {
            return String::new();
        }

        output_markdown + &variants_markdown
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    Null,
    I32,
    F32,
    F64,
    U32,
    U16,
    U8,
    Bool,
    String(Option<Rc<str>>),
    Ref(Rc<str>),
    Tuple(Vec<TypeDefinition>),
    Array(Box<TypeDefinition>),
    Union(Vec<TypeDefinition>),
    Object(Vec<ObjectProperty>),
    Map { value_type: Box<TypeDefinition> },
}

fn find_references(ty: &Kind, references: &mut Vec<Rc<str>>) {
    match &ty {
        Kind::Ref(reference) => {
            if !references.contains(reference) {
                references.push(reference.clone());
            }
        }
        Kind::Tuple(types) | Kind::Union(types) => types
            .iter()
            .for_each(|def| find_references(&def.kind, references)),
        Kind::Array(def) => find_references(&def.kind, references),
        Kind::Object(properties) => properties
            .iter()
            .for_each(|prop| find_references(&prop.type_def.kind, references)),
        Kind::Map { value_type } => find_references(&value_type.kind, references),
        _ => {}
    }
}
