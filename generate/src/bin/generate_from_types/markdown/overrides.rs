use crate::{
    definition::{Kind, TypeDefinition},
    markdown::MarkdownGenerator,
};

pub fn force_multiline(generator: &mut MarkdownGenerator<'_>, definition: TypeDefinition) {
    match definition.kind {
        Kind::Object { fields } => generator.write_multi_line_object(fields),
        Kind::Union(variants) => generator.write_multi_line_union(variants),
        _ => panic!("Expected object or union definition"),
    }
}
