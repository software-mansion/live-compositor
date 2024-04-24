use docs_config::DocsConfig;
use document::Doc;
use std::{fs, path::PathBuf};
use video_compositor::types::{
    Image, ImageSpec, InputStream, Mp4, Rescaler, RtpInputStream, RtpOutputStream, Shader,
    ShaderSpec, Text, Tiles, View, WebRendererSpec, WebView,
};

mod definition;
mod docs_config;
mod document;
mod generation_strategy;
mod schema_parser;

fn main() {
    tracing_subscriber::fmt().init();
    let docs_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/pages/api/generated");
    if docs_path.exists() {
        fs::remove_dir_all(&docs_path).unwrap();
    }
    fs::create_dir(&docs_path).unwrap();

    let config = DocsConfig::default();

    let renderer_pages = [
        Doc::<ShaderSpec>::generate("Shader", &config),
        Doc::<ImageSpec>::generate("Image", &config),
        Doc::<WebRendererSpec>::generate("WebRenderer", &config),
        Doc::<RtpInputStream>::generate("RtpInputStream", &config),
        Doc::<Mp4>::generate("Mp4", &config),
    ];

    let component_pages = [
        Doc::<Shader>::generate("Shader", &config),
        Doc::<InputStream>::generate("InputStream", &config),
        Doc::<View>::generate("View", &config),
        Doc::<WebView>::generate("WebView", &config),
        Doc::<Image>::generate("Image", &config),
        Doc::<Text>::generate("Text", &config),
        Doc::<Tiles>::generate("Tiles", &config),
        Doc::<Rescaler>::generate("Rescaler", &config),
    ];

    let output_pages = [Doc::<RtpOutputStream>::generate("OutputStream", &config)];

    for page in renderer_pages {
        fs::write(
            docs_path.join(format!("renderer-{}.md", page.title)),
            page.markdown,
        )
        .unwrap();
    }
    for page in component_pages {
        fs::write(
            docs_path.join(format!("component-{}.md", page.title)),
            page.markdown,
        )
        .unwrap();
    }
    for page in output_pages {
        fs::write(
            docs_path.join(format!("output-{}.md", page.title)),
            page.markdown,
        )
        .unwrap();
    }
}
