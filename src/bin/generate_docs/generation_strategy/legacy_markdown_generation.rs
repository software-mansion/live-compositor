//
// NOTE(noituri): This whole is mostly copy pasted from the previous implementation.
// The file will be completely removed in the follow up PR
//

use crate::{
    definition::{Kind, TopLevelDefinition, TypeDefinition},
    docs_config::DocsConfig,
};

const INDENT: &str = " ";

impl TopLevelDefinition {
    pub fn to_markdown(&self, config: &DocsConfig) -> String {
        let name = &self.name;
        let definition = self.definition.to_pretty_string(0);
        let description = self.definition.description.clone();
        let properties = self.properties_markdown(config);

        format!(
            "## {name}\n```typescript\ntype {name} = {definition}\n```\n{description}\n{properties}\n",
        )
    }

    fn object_field_to_markdown(name: &str, field_def: &TypeDefinition) -> String {
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
                output_description += &format!("{}{}\n", INDENT.repeat(indent_size + 2), desc);
            }

            output_description
        }

        let description = &field_def.description;
        let output_markdown = format_description(name, description, 0);
        let variants = match &field_def.kind {
            Kind::Union(variants) => variants.clone(),
            _ => Vec::new(),
        };

        let mut variants_markdown = String::new();
        for variant in variants {
            let name = variant.to_pretty_string(0);
            if variant.description.is_empty() {
                continue;
            }

            variants_markdown += &format_description(&name, &variant.description, 2)
        }

        if description.trim().is_empty() && variants_markdown.is_empty() {
            return String::new();
        }

        output_markdown + &variants_markdown
    }

    fn properties_markdown(&self, config: &DocsConfig) -> String {
        fn format_props(properties: &[(String, TypeDefinition)]) -> String {
            properties
                .iter()
                .map(|(name, def)| TopLevelDefinition::object_field_to_markdown(name, def))
                .collect::<String>()
        }

        fn format_object_variant(
            key: Option<&str>,
            mut properties: Vec<(String, TypeDefinition)>,
        ) -> String {
            let properties_key = match key {
                Some(key) => {
                    let (_, value_def) = properties
                        .iter()
                        .find(|(name, _)| name == key)
                        .cloned()
                        .unwrap();
                    properties.retain(|(name, _)| name != key);

                    format!("(`type: {}`)", value_def.to_pretty_string(0))
                }
                None => String::new(),
            };

            let properties = format_props(&properties);
            if properties.is_empty() {
                return String::new();
            }

            format!("#### Properties {properties_key}\n{properties}")
        }

        match &self.definition.kind {
            Kind::Object { fields } => {
                let properties = format_props(fields);
                if properties.is_empty() {
                    return String::new();
                }

                format!("#### Properties\n{properties}")
            }
            Kind::Union(variants) => {
                let key = config
                    .variant_discriminators
                    .get(&self.name.as_str())
                    .cloned();

                let mut properties_string = String::new();
                for variant in variants {
                    let variant_description = match variant.kind.clone() {
                        Kind::Object { fields } => format_object_variant(key, fields),
                        _ => match variant.description.is_empty() {
                            false => format!(
                                "#### {}\n{}",
                                variant.to_pretty_string(0),
                                variant.description
                            ),
                            true => String::new(),
                        },
                    };

                    properties_string += &variant_description;
                }

                properties_string
            }
            _ => String::new(),
        }
    }
}

impl TypeDefinition {
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
            Kind::String { specific_value } => match specific_value {
                Some(value) => value.clone(),
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
            Kind::Array { array_type } => format!("{}[]", array_type.to_pretty_string(base_indent)),
            Kind::Union(variants) => Self::union_to_string(variants, base_indent),
            Kind::Object { fields } => Self::object_to_string(fields, base_indent),
            Kind::Map { value_type } => format!("Map<string, {}>", value_type.to_pretty_string(0)),
        }
    }

    fn union_to_string(variants: &[TypeDefinition], base_indent: usize) -> String {
        fn variant_to_string(variant: &TypeDefinition, indent: usize) -> String {
            match &variant.kind {
                Kind::Object { fields } => {
                    let single_line = TypeDefinition::object_to_single_line(fields);
                    if single_line.len() < 60 {
                        return single_line;
                    }
                    TypeDefinition::object_to_string(fields, indent)
                }
                _ => variant.to_pretty_string(indent),
            }
        }

        let variant_indent = match variants
            .iter()
            .any(|v| matches!(v.kind, Kind::Object { .. }))
        {
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

    fn object_to_string(properties: &[(String, TypeDefinition)], base_indent: usize) -> String {
        let mut out = "{\n".to_owned();
        for (name, def) in properties {
            let mut name = name.clone();
            if def.is_optional {
                name = format!("{name}?");
            }

            let indent = match &def.kind {
                Kind::Union(_) => base_indent + 2,
                Kind::Object { .. } => base_indent + 2,
                _ => base_indent,
            };
            out += &format!(
                "{}{}: {};\n",
                INDENT.repeat(base_indent + 2),
                name,
                def.to_pretty_string(indent)
            );
        }
        format!("{}{}}}", out, INDENT.repeat(base_indent))
    }

    fn object_to_single_line(properties: &[(String, TypeDefinition)]) -> String {
        let properties = properties
            .iter()
            .map(|(name, def)| {
                let mut name = name.clone();
                if def.is_optional {
                    name = format!("{name}?");
                }
                format!("{}: {}", name, def.to_pretty_string(0))
            })
            .collect::<Vec<_>>()
            .join("; ");
        format!("{{ {} }}", properties)
    }
}
