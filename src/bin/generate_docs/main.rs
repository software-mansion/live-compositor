use parsing::generate_component_docs;
use schemars::gen::{SchemaGenerator, SchemaSettings};
use std::{fs, path::PathBuf};
use video_compositor::types::Component;

mod parsing;
mod type_definition;

fn main() {
    let docs_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/pages/api/components");
    if !docs_path.exists() {
        fs::create_dir(&docs_path).unwrap();
    }

    let mut settings = SchemaSettings::default();
    // Remove not needed prefix from references
    settings.definitions_path.clear();
    let schema_generator = SchemaGenerator::new(settings);

    let root_schema = schema_generator.into_root_schema_for::<Component>();
    let pages = [
        generate_component_docs(&root_schema, "InputStream", "input_stream"),
        generate_component_docs(&root_schema, "View", "view"),
        generate_component_docs(&root_schema, "WebView", "web_view"),
        generate_component_docs(&root_schema, "Shader", "shader"),
        generate_component_docs(&root_schema, "Image", "image"),
        generate_component_docs(&root_schema, "Text", "text"),
        generate_component_docs(&root_schema, "Tiles", "tiles"),
        generate_component_docs(&root_schema, "Rescaler", "rescaler"),
    ];

    for page in pages {
        fs::write(
            docs_path.join(format!("{}.md", page.title)),
            page.to_markdown(),
        )
        .unwrap();
    }
}
