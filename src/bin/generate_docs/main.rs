use parsing::{generate_component_docs, generate_renderer_docs};
use schemars::{
    gen::{SchemaGenerator, SchemaSettings},
    schema::RootSchema,
    JsonSchema,
};
use std::{fs, path::PathBuf};
use video_compositor::types::{Component, RegisterRequest};

mod parsing;
mod type_definition;

fn main() {
    let docs_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/pages/api/generated");
    if docs_path.exists() {
        fs::remove_dir_all(&docs_path).unwrap();
    }
    fs::create_dir(&docs_path).unwrap();

    let register_schema = generate_schema::<RegisterRequest>();
    let renderer_pages = [
        generate_renderer_docs(&register_schema, "InputStream", "input_stream"),
        generate_renderer_docs(&register_schema, "OutputStream", "output_stream"),
        generate_renderer_docs(&register_schema, "Shader", "shader"),
        generate_renderer_docs(&register_schema, "WebRenderer", "web_renderer"),
        generate_renderer_docs(&register_schema, "Image", "image"),
    ];

    let component_schema = generate_schema::<Component>();
    let component_pages = [
        generate_component_docs(&component_schema, "InputStream", "input_stream"),
        generate_component_docs(&component_schema, "View", "view"),
        generate_component_docs(&component_schema, "WebView", "web_view"),
        generate_component_docs(&component_schema, "Shader", "shader"),
        generate_component_docs(&component_schema, "Image", "image"),
        generate_component_docs(&component_schema, "Text", "text"),
        generate_component_docs(&component_schema, "Tiles", "tiles"),
        generate_component_docs(&component_schema, "Rescaler", "rescaler"),
    ];

    for page in renderer_pages {
        fs::write(
            docs_path.join(format!("renderer-{}.md", page.title)),
            page.to_markdown(),
        )
        .unwrap();
    }
    for page in component_pages {
        fs::write(
            docs_path.join(format!("component-{}.md", page.title)),
            page.to_markdown(),
        )
        .unwrap();
    }
}

fn generate_schema<T: JsonSchema>() -> RootSchema {
    let mut settings = SchemaSettings::default();
    // Remove not needed prefix from references
    settings.definitions_path.clear();
    let schema_generator = SchemaGenerator::new(settings);
    schema_generator.into_root_schema_for::<T>()
}
