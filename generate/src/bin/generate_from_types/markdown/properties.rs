use crate::definition::{Kind, TypeDefinition};

use super::MarkdownGenerator;

impl<'a> MarkdownGenerator<'a> {
    pub(super) fn write_properties(&mut self, definition: &TypeDefinition) {
        let name = definition.name.clone();

        match &definition.kind {
            Kind::Object { fields } => self.write_object_properties(fields.clone()),
            Kind::Union(variants) => self.write_union_properties(name, variants.clone()),
            _ => {}
        }
    }

    fn write_object_properties(&mut self, fields: Vec<(String, TypeDefinition)>) {
        self.add_header(4, "Properties");
        self.write_object_fields_descriptions(fields);
    }

    fn write_union_properties(
        &mut self,
        union_name: Option<String>,
        variants: Vec<TypeDefinition>,
    ) {
        for variant in variants {
            let description = variant.description.clone();

            match variant.kind.clone() {
                Kind::Object { fields } => {
                    self.write_object_variant_properties(union_name.as_deref(), description, fields)
                }
                _ => self.write_variant_description(variant),
            }
        }
    }

    /// Generates descriptions for each field in an object
    fn write_object_fields_descriptions(&mut self, fields: Vec<(String, TypeDefinition)>) {
        // "- `{name}` - {description}\n{union_variant_properties}\n"
        for (name, def) in fields {
            let union_properties_len = self.calculate_generation_length(|generator| {
                if let Kind::Union(variants) = def.kind.clone() {
                    generator.write_union_properties(def.name.clone(), variants);
                }
            });
            if def.description.is_empty() && union_properties_len == 0 {
                continue;
            }

            self.add_text(format!("- `{name}`"));

            // Indent is needed when the description is multiline
            self.inc_indent();
            {
                if !def.description.is_empty() {
                    self.add_text(" - ");
                    self.add_text(def.description);
                }
                self.add_text("\n");
                if let Kind::Union(variants) = def.kind {
                    self.write_union_properties(def.name, variants);
                }
            }
            self.dec_indent();
        }
    }

    /// Generates new `Properties` section for an union variant
    /// If variant discriminant is present, it will be displayed in `Properties` header
    fn write_object_variant_properties(
        &mut self,
        union_name: Option<&str>,
        variant_description: String,
        fields: Vec<(String, TypeDefinition)>,
    ) {
        let any_field_descriptions = fields.iter().any(|(_, def)| !def.description.is_empty());
        if variant_description.is_empty() && !any_field_descriptions {
            return;
        }

        let discriminant = union_name
            .and_then(|name| self.config.variant_discriminators.get(name))
            .map(|discriminator_field| extract_variant_discriminant(discriminator_field, &fields));

        match discriminant {
            Some(discriminant) => {
                self.add_header(4, format!("Properties (`type: {discriminant}`)"))
            }
            None => self.add_header(4, "Properties"),
        }

        if !variant_description.is_empty() {
            self.add_text(variant_description);
            self.add_text("\n");
        }

        self.write_object_fields_descriptions(fields);
    }

    fn write_variant_description(&mut self, variant: TypeDefinition) {
        let description = variant.description.clone();
        if description.is_empty() {
            return;
        }

        // Generates "- `{type}` - {description}\n"
        self.add_text("- ");
        self.add_text("`");
        self.write_type_definition(variant);
        self.add_text("`");

        // Indent is needed when the description is multiline
        self.inc_indent();
        {
            self.add_text(" - ");
            self.add_text(description);
            self.add_text("\n");
        }
        self.dec_indent();
    }
}

fn extract_variant_discriminant(
    discriminator_field: &str,
    variant_fields: &[(String, TypeDefinition)],
) -> String {
    variant_fields
        .iter()
        .find(|(name, _)| name == discriminator_field)
        .and_then(|(_, def)| match &def.kind {
            Kind::String { specific_value } => specific_value.clone(),
            _ => None,
        })
        .unwrap_or_else(|| panic!("No discriminator found"))
}
