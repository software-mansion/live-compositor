#[derive(Debug, Clone)]
pub struct TypeDefinition {
    pub name: Option<String>,
    pub description: String,
    pub kind: Kind,
    pub is_optional: bool,
}

impl TypeDefinition {
    pub fn references(&self) -> Vec<String> {
        fn find_references(ty: &Kind, references: &mut Vec<String>) {
            match ty {
                Kind::Ref(reference) => {
                    if !references.contains(reference) {
                        references.push(reference.clone());
                    }
                }
                Kind::Tuple(defs) | Kind::Union(defs) => defs
                    .iter()
                    .for_each(|variant| find_references(&variant.kind, references)),
                Kind::Array { array_type } => find_references(&array_type.kind, references),
                Kind::Object { fields } => fields
                    .iter()
                    .for_each(|(_, field)| find_references(&field.kind, references)),
                Kind::Map { value_type } => find_references(&value_type.kind, references),
                _ => {}
            }
        }

        let mut references = Vec::new();
        find_references(&self.kind, &mut references);

        references
    }

    /// Merges two definitions into one. The other definition's kind takes precedence.
    pub fn merge_into(&self, other: &Self) -> Self {
        let desc1 = self.description.clone();
        let desc2 = other.description.clone();

        let name = if other.name.is_some() {
            other.name.clone()
        } else {
            self.name.clone()
        };

        let description = if desc1.is_empty() {
            desc2
        } else if desc2.is_empty() {
            desc1
        } else {
            format!("{}\n\n{}", desc1, desc2)
        };

        Self {
            name,
            description,
            kind: other.kind.clone(),
            is_optional: self.is_optional || other.is_optional,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Kind {
    Null,
    I32,
    F32,
    F64,
    U32,
    U16,
    U8,
    Bool,
    String {
        specific_value: Option<String>,
    },
    Ref(String),
    Tuple(Vec<TypeDefinition>),
    Union(Vec<TypeDefinition>),
    Array {
        array_type: Box<TypeDefinition>,
    },
    Object {
        fields: Vec<(String, TypeDefinition)>,
    },
    Map {
        value_type: Box<TypeDefinition>,
    },
}

impl Kind {
    pub fn inlineable_by_default(&self) -> bool {
        match self {
            Kind::Null
            | Kind::I32
            | Kind::F32
            | Kind::F64
            | Kind::U32
            | Kind::U16
            | Kind::U8
            | Kind::String { .. } => true,
            Kind::Union(variants) => variants.iter().all(|v| v.kind.inlineable_by_default()),
            _ => false,
        }
    }
}
