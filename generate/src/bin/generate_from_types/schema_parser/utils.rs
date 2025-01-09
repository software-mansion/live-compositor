use schemars::{
    gen::{SchemaGenerator, SchemaSettings},
    schema::{InstanceType, Schema, SchemaObject},
};

pub trait SchemaExt {
    fn schema_object(&self) -> &SchemaObject;
}

impl SchemaExt for Schema {
    fn schema_object(&self) -> &SchemaObject {
        match self {
            Schema::Object(schema) => schema,
            _ => panic!("Expected object schema"),
        }
    }
}

pub trait SchemaObjectExt {
    fn description(&self) -> String;

    fn union_variants(&self) -> Vec<SchemaObject>;

    fn is_optional(&self) -> bool;

    fn is_union(&self) -> bool;

    fn is_tuple(&self) -> bool;

    fn is_map(&self) -> bool;
}

impl SchemaObjectExt for SchemaObject {
    fn description(&self) -> String {
        self.metadata
            .as_ref()
            .and_then(|m| m.description.clone())
            .unwrap_or_default()
    }

    fn union_variants(&self) -> Vec<SchemaObject> {
        let Some(subschemas) = &self.subschemas else {
            return Vec::new();
        };

        let variants = match (subschemas.one_of.as_ref(), subschemas.any_of.as_ref()) {
            (Some(one_of), None) => one_of.clone(),
            (None, Some(any_of)) => any_of.clone(),
            _ => panic!("Expected oneOf or anyOf"),
        };

        variants
            .into_iter()
            .map(|schema| schema.into_object())
            .collect()
    }

    fn is_optional(&self) -> bool {
        // `Null` can be found in 2 places: `instance_type` and `subschemas.any_of`

        if let Some(types) = &self.instance_type {
            return types.contains(&InstanceType::Null);
        }

        if let Some(subschemas) = &self.subschemas {
            let Some(any_of) = subschemas.any_of.as_ref() else {
                return false;
            };

            return any_of
                .iter()
                .any(|schema| schema.schema_object().is_optional());
        }

        false
    }

    fn is_union(&self) -> bool {
        // Schema is a union when:
        // - oneOf or
        // - anyOf is present and has more than one non-nullable schema

        let has_one_of = self.subschemas.as_ref().is_some_and(|s| s.one_of.is_some());

        let non_nullabe_any_of_size = self
            .subschemas
            .as_ref()
            .and_then(|s| s.any_of.as_ref())
            .map(|any_of| {
                any_of
                    .iter()
                    .filter(|schema| !schema.schema_object().is_optional())
                    .count()
            })
            .unwrap_or(0);

        has_one_of || non_nullabe_any_of_size > 1
    }

    /// Check if the schema represents a tuple
    ///
    /// Tuples are represented as arrays in JSON Schema. This helpes to distinguish them from normal arrays.
    fn is_tuple(&self) -> bool {
        let Some(array) = &self.array else {
            return false;
        };

        let (Some(min), Some(max)) = (array.min_items, array.max_items) else {
            return false;
        };

        min == max
    }

    /// Check if the schema represents a map
    ///
    /// Maps are represented as objects in JSON Schema. This helpes to distinguish them from normal objects.
    fn is_map(&self) -> bool {
        let Some(object) = &self.object else {
            return false;
        };

        matches!(
            object.additional_properties.as_deref(),
            Some(Schema::Object(_))
        )
    }
}

/// Creates a new schema generator with removed reference prefix (definitions path)
pub fn new_schema_generator() -> SchemaGenerator {
    let mut settings = SchemaSettings::default();
    settings.definitions_path.clear();
    SchemaGenerator::new(settings)
}

/// Sometimes when schema is a reference or nullable value, its true definition is nested inside.
/// This function returns the innermost schema object representing the type
pub fn inner_schema_object(schema: SchemaObject) -> SchemaObject {
    fn is_null(schema: &SchemaObject) -> bool {
        let Some(instance_type) = &schema.instance_type else {
            return false;
        };

        instance_type.contains(&InstanceType::Null)
    }

    let Some(subschemas) = &schema.subschemas else {
        return schema;
    };

    if let Some(all_of) = &subschemas.all_of {
        if all_of.len() != 1 {
            panic!("Expected exactly one schema in allOf");
        }

        return all_of[0].clone().into_object();
    }

    if let Some(any_of) = &subschemas.any_of {
        let schemas = any_of
            .iter()
            .filter(|s| !is_null(s.schema_object()))
            .collect::<Vec<_>>();
        if schemas.len() != 1 {
            panic!("Expected exactly one non-null schema in anyOf");
        }

        return schemas[0].clone().into_object();
    }

    schema
}
