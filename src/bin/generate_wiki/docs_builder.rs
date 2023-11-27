use std::{collections::HashMap, ops::Deref, rc::Rc};

use schemars::{
    gen::{SchemaGenerator, SchemaSettings},
    schema::{InstanceType, RootSchema, Schema, SchemaObject, SingleOrVec, SubschemaValidation},
    JsonSchema,
};

#[derive(Debug)]
pub struct DocPage {
    title: Rc<str>,
    // TODO: Consider using HashMap with ordering
    definitions: Vec<TypeDefinition>,
}

impl DocPage {
    pub fn new(title: Rc<str>) -> Self {
        Self {
            title,
            definitions: Vec::new(),
        }
    }

    pub fn add_definition(&mut self, def: TypeDefinition) {
        self.definitions.push(def);
    }

    pub fn remove_definition(&mut self, name: &Rc<str>) {
        self.definitions
            .retain(|def| def.name.as_ref() != Some(name))
    }

    pub fn definition(&self, name: &Rc<str>) -> Option<&TypeDefinition> {
        self.definitions
            .iter()
            .find(|def| def.name.as_ref() == Some(name))
    }
}

pub struct DocsBuilder {
    schema_generator: SchemaGenerator,
    pages: Vec<DocPage>,
}

impl DocsBuilder {
    pub fn new() -> Self {
        let mut settings = SchemaSettings::default();
        // Remove not needed prefix from references
        settings.definitions_path.clear();

        Self {
            schema_generator: SchemaGenerator::new(settings),
            pages: Vec::new(),
        }
    }

    pub fn build(mut self) -> HashMap<Rc<str>, String> {
        self.simplify_pages();

        self.pages
            .into_iter()
            .map(|page| {
                (
                    page.title,
                    page.definitions
                        .iter()
                        .enumerate()
                        .map(|(i, def)| def.to_markdown(i == 0))
                        .collect::<Vec<_>>()
                        .join("\n"),
                )
            })
            .collect()
    }

    pub fn add_definition<T: JsonSchema>(mut self, use_multiple_pages: bool) -> Self {
        let mut root_schema = self.schema_generator.root_schema_for::<T>();
        let name: Rc<str> = root_schema.schema.metadata().title.clone().unwrap().into();
        let schema = root_schema.schema.clone();
        let mut page = DocPage::new(name.clone());

        populate_page(&mut page, name, &schema, &mut root_schema);
        if use_multiple_pages {
            for page in Self::split_into_multiple_pages(page) {
                self.pages.push(page);
            }
        } else {
            self.pages.push(page);
        }

        self
    }

    fn split_into_multiple_pages(mut page: DocPage) -> Vec<DocPage> {
        // Checks if an union variant can exist on its own, as a separate type definition
        fn can_be_standalone(def: &TypeDefinition) -> bool {
            def.name.is_some() && matches!(def.ty, Type::Object(_))
        }

        let mut new_definitions = HashMap::new();
        let mut new_pages = Vec::new();

        for def in page.definitions.iter_mut() {
            let Type::Union(types) = def.ty.clone() else {
                continue;
            };

            if types.len() > 6 && types.iter().all(can_be_standalone) {
                def.ty = Type::Union(
                    types
                        .iter()
                        .map(|def| TypeDefinition::simple(Type::Ref(def.name.clone().unwrap())))
                        .collect(),
                );
                new_definitions.extend(
                    types
                        .iter()
                        .map(|def| (def.name.clone().unwrap(), def.clone())),
                )
            }
        }

        for (name, def) in new_definitions {
            let mut new_page = DocPage::new(name.clone());
            let references = def.references.clone();
            new_page.add_definition(def);

            for refer in references {
                if refer == page.title {
                    continue;
                }
                let Some(ref_def) = page.definition(&refer) else {
                    continue;
                };

                new_page.add_definition(ref_def.clone());
            }

            new_pages.push(new_page);
        }

        new_pages
    }

    fn simplify_pages(&mut self) {
        fn inline_definition(
            def: &mut TypeDefinition,
            inline_definitions: &HashMap<Rc<str>, TypeDefinition>,
        ) {
            match &mut def.ty {
                Type::Optional(def) => inline_definition(def, inline_definitions),
                Type::Ref(reference) => {
                    if let Some(inline_def) = inline_definitions.get(reference) {
                        *def = inline_def.clone();
                    }
                }
                Type::Tuple(types) | Type::Union(types) => types
                    .iter_mut()
                    .for_each(|def| inline_definition(def, inline_definitions)),
                Type::Array(ty) => inline_definition(ty, inline_definitions),
                Type::Object(properties) => properties
                    .iter_mut()
                    .for_each(|prop| inline_definition(&mut prop.type_def, inline_definitions)),
                _ => {}
            }
        }

        for page in self.pages.iter_mut() {
            let mut inline_definitions = HashMap::new();
            for def in page.definitions.iter() {
                let should_inline = match &def.ty {
                    Type::Null
                    | Type::I32
                    | Type::F32
                    | Type::F64
                    | Type::U32
                    | Type::U16
                    | Type::Bool
                    | Type::String(_) => true,
                    Type::Union(types) => types
                        .iter()
                        .all(|def: &TypeDefinition| matches!(def.ty, Type::String(_))),
                    _ => false,
                };

                if should_inline {
                    inline_definitions.insert(def.name.clone().unwrap(), def.clone());
                }
            }

            for name in inline_definitions.keys() {
                page.remove_definition(name);
            }

            for def in page.definitions.iter_mut() {
                inline_definition(def, &inline_definitions);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct ObjectProperty {
    name: Rc<str>,
    type_def: TypeDefinition,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeDefinition {
    name: Option<Rc<str>>,
    description: Option<Rc<str>>,
    ty: Type,
    references: Vec<Rc<str>>,
}

impl TypeDefinition {
    fn complex(name: Option<String>, description: Option<String>, ty: Type) -> Self {
        let mut references = Vec::new();
        find_references(&ty, &mut references);

        Self {
            name: name.map(Into::into),
            description: description.map(Into::into),
            ty,
            references,
        }
    }

    fn simple(ty: Type) -> Self {
        Self::complex(None, None, ty)
    }

    // Move to utils
    fn to_markdown(&self, is_header: bool) -> String {
        let name = self.name.as_ref().unwrap();
        let description = self
            .description
            .as_ref()
            .map(|desc| format!("{desc}\n"))
            .unwrap_or_default();

        let header = match is_header {
            true => format!("# {name}"),
            false => format!("## {name}"),
        };

        format!(
            "{}\n```typescript\ntype {} = {}\n```\n{}{}",
            header,
            name,
            self.to_pretty_string(0),
            description,
            self.properties_markdown(),
        )
    }

    fn properties_markdown(&self) -> String {
        let Type::Object(properties) = &self.ty else {
            return String::new();
        };

        let properties = properties
            .iter()
            .flat_map(|prop| {
                prop.type_def
                    .description
                    .as_ref()
                    .map(|desc| format!("- `{}` - {}\n", prop.name, desc))
            })
            .collect::<String>();
        if properties.is_empty() {
            return String::new();
        }
        format!("#### Properties\n{properties}")
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Type {
    Null,
    Optional(Box<TypeDefinition>),
    I32,
    F32,
    F64,
    U32,
    U16,
    Bool,
    String(Option<Rc<str>>),
    Ref(Rc<str>),
    Tuple(Vec<TypeDefinition>),
    Array(Box<TypeDefinition>),
    Union(Vec<TypeDefinition>),
    Object(Vec<ObjectProperty>),
}

impl TypeDefinition {
    fn object_to_string(properties: &[ObjectProperty], base_indent: usize) -> String {
        let mut out = "{\n".to_owned();
        for prop in properties {
            let mut name = prop.name.clone();
            if prop.type_def.is_optional() {
                name = format!("{name}?").into();
            }
            out += &format!(
                "{}{}: {},\n",
                indent(base_indent + 1),
                name,
                prop.type_def.to_pretty_string(0)
            );
        }
        format!("{}{}}}", out, indent(base_indent))
    }

    fn union_to_string(variants: &[TypeDefinition], base_indent: usize) -> String {
        let variant_indent = match variants.iter().any(|v| matches!(v.ty, Type::Object(_))) {
            true => base_indent + 2,
            false => base_indent + 1,
        };

        let variants = variants
            .iter()
            .map(|ty| ty.to_pretty_string(variant_indent));
        if variants.len() > 4 {
            variants.into_iter().fold(String::new(), |acc, ty| {
                format!("{}\n{}| {}", acc, indent(base_indent + 1), ty)
            })
        } else {
            variants.collect::<Vec<_>>().join(" | ")
        }
    }

    fn to_pretty_string(&self, base_indent: usize) -> String {
        match &self.ty {
            Type::Null => "null".into(),
            Type::Optional(ty) => ty.to_pretty_string(base_indent),
            Type::I32 => "i32".into(),
            Type::F32 => "f32".into(),
            Type::F64 => "f64".into(),
            Type::U32 => "u32".into(),
            Type::U16 => "u16".into(),
            Type::Bool => "bool".into(),
            Type::String(value) => match value {
                Some(value) => format!("\"{value}\""),
                None => "string".into(),
            },
            Type::Ref(reference) => reference.to_string(),
            Type::Tuple(types) => format!(
                "[{}]",
                types
                    .iter()
                    .map(|ty| ty.to_pretty_string(base_indent))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Type::Array(ty) => format!("{}[]", ty.to_pretty_string(base_indent)),
            Type::Union(variants) => Self::union_to_string(variants, base_indent),
            Type::Object(properties) => Self::object_to_string(properties, base_indent),
        }
    }
}

fn parse_schema(value: &SchemaObject) -> TypeDefinition {
    let (name, description) = value
        .metadata
        .clone()
        .map(|metadata| (metadata.title, metadata.description))
        .unwrap_or_default();
    if let Some(reference) = &value.reference {
        return TypeDefinition::complex(name, description, Type::Ref(reference.as_str().into()));
    }

    let (ty, is_optional) = match &value.instance_type {
        Some(SingleOrVec::Single(ty)) => (ty.deref(), false),
        Some(SingleOrVec::Vec(types)) => match types.as_slice() {
            [ty, InstanceType::Null] => (ty, true),
            [InstanceType::Null, ty] => (ty, true),
            types => unimplemented!("Unsupported type: Vec({types:?})"),
        },
        None => {
            if let Some(subschemas) = &value.subschemas {
                let mut types = flatten_subschemas(subschemas)
                    .iter()
                    .map(parse_schema)
                    .collect::<Vec<_>>();
                let is_optional = types.iter().any(|def| def.ty == Type::Null);
                types.retain(|def| def.ty != Type::Null);

                if is_optional {
                    let definition = TypeDefinition::complex(
                        name.clone(),
                        description.clone(),
                        Type::Union(types),
                    );
                    return TypeDefinition::complex(
                        name,
                        description,
                        Type::Optional(Box::new(definition)),
                    );
                } else {
                    return TypeDefinition::complex(name, description, Type::Union(types));
                }
            }

            unimplemented!("Unsupported type");
        }
    };

    let ty = match ty {
        InstanceType::Boolean => Type::Bool,
        InstanceType::Array => {
            let array = value.array.as_ref().unwrap();
            match (array.min_items, array.max_items) {
                (Some(min), Some(max)) if min == max => {
                    let Some(SingleOrVec::Vec(items)) = &array.items else {
                        panic!("Expected typle types");
                    };
                    let tuple_ty = items
                        .iter()
                        .cloned()
                        .map(|schema| parse_schema(&schema.into_object()))
                        .collect();
                    Type::Tuple(tuple_ty)
                }
                _ => {
                    let Some(SingleOrVec::Single(items)) = &array.items else {
                        panic!("Arrays can accept only one type");
                    };
                    let array_ty = parse_schema(&items.clone().into_object());
                    Type::Array(Box::new(array_ty))
                }
            }
        }
        InstanceType::String => match &value.enum_values {
            Some(values) => match values.len() {
                0 => Type::String(None),
                1 => Type::String(Some(values[0].as_str().unwrap().into())),
                _ => {
                    let values = values
                        .iter()
                        .map(|v| TypeDefinition::simple(Type::String(v.as_str().map(Into::into))))
                        .collect::<Vec<_>>();
                    Type::Union(values)
                }
            },
            None => Type::String(None),
        },
        InstanceType::Number => match value.format.as_ref().unwrap().as_str() {
            "float" => Type::F32,
            "double" => Type::F64,
            format => unimplemented!("Unknown number format \"{format}\""),
        },
        InstanceType::Integer => match value.format.as_ref().unwrap().as_str() {
            "uint32" | "uint" => Type::U32,
            "int32" | "int" => Type::I32,
            "uint16" => Type::U16,
            format => unimplemented!("Unknown integer format \"{format}\""),
        },
        InstanceType::Object => parse_object(value),
        InstanceType::Null => Type::Null,
    };

    let definition = TypeDefinition::complex(name.clone(), description.clone(), ty);
    if is_optional {
        return TypeDefinition::complex(name, description, Type::Optional(Box::new(definition)));
    }

    definition
}

impl TypeDefinition {
    fn is_optional(&self) -> bool {
        matches!(self.ty, Type::Optional(_))
    }
}

fn parse_object(schema: &SchemaObject) -> Type {
    let mut properties = Vec::new();
    let object = schema.object.as_ref().unwrap();
    for (name, prop) in object.properties.clone() {
        properties.push(ObjectProperty {
            name: name.into(),
            type_def: parse_schema(&prop.into_object()),
        })
    }

    if let Some(subschemas) = &schema.subschemas {
        let objects = flatten_subschemas(subschemas)
            .iter()
            .map(parse_schema)
            .map(|mut def| {
                let ty = match def.ty {
                    Type::Object(sub_properties) => {
                        Type::Object([sub_properties, properties.clone()].concat())
                    }
                    _ => unreachable!("Expected object"),
                };

                def.ty = ty;
                def
            })
            .collect::<Vec<_>>();
        return Type::Union(objects);
    }

    Type::Object(properties)
}

fn flatten_subschemas(subschemas: &SubschemaValidation) -> Vec<SchemaObject> {
    let mut schemas = Vec::new();

    // These subschemas are represented by an enum in rust. Only one variant can be used at the time.
    if let Some(mut one_of) = subschemas.one_of.clone() {
        schemas.append(&mut one_of);
    }
    if let Some(mut any_of) = subschemas.any_of.clone() {
        schemas.append(&mut any_of);
    }
    if let Some(mut all_of) = subschemas.all_of.clone() {
        schemas.append(&mut all_of);
    }

    schemas.into_iter().map(Schema::into_object).collect()
}

fn find_references(ty: &Type, references: &mut Vec<Rc<str>>) {
    match &ty {
        Type::Optional(def) => find_references(&def.ty, references),
        Type::Ref(reference) => {
            if !references.contains(reference) {
                references.push(reference.clone());
            }
        }
        Type::Tuple(types) | Type::Union(types) => types
            .iter()
            .for_each(|def| find_references(&def.ty, references)),
        Type::Array(def) => find_references(&def.ty, references),
        Type::Object(properties) => properties
            .iter()
            .for_each(|prop| find_references(&prop.type_def.ty, references)),
        _ => {}
    }
}

fn indent(n: usize) -> String {
    "  ".repeat(n)
}

fn populate_page(
    page: &mut DocPage,
    name: Rc<str>,
    schema: &SchemaObject,
    root_schema: &mut RootSchema,
) {
    let mut definition = parse_schema(schema);
    definition.name = Some(name.clone());

    let references = definition.references.clone();
    page.add_definition(definition);

    // Parse every definition mentioned in `schema`
    for refer in references {
        let Some(schema) = root_schema.definitions.remove(refer.deref()) else {
            continue;
        };
        populate_page(page, refer, &schema.into_object(), root_schema);
    }
}
