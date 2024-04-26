use docs_config::DocsConfig;
use live_compositor::types::{
    Image, ImageSpec, InputStream, Mp4, Rescaler, RtpInputStream, RtpOutputStream, Shader,
    ShaderSpec, Text, Tiles, View, WebRendererSpec, WebView,
};
use parsing::generate_docs;
use std::{fs, path::PathBuf};

mod docs_config;
mod parsing;
mod type_definition;

fn main() {
    tracing_subscriber::fmt().init();
    let docs_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/pages/api/generated");
    if docs_path.exists() {
        fs::remove_dir_all(&docs_path).unwrap();
    }
    fs::create_dir(&docs_path).unwrap();

    let config = DocsConfig::default();

    let renderer_pages = [
        generate_docs::<ShaderSpec>("Shader", &config),
        generate_docs::<ImageSpec>("Image", &config),
        generate_docs::<WebRendererSpec>("WebRenderer", &config),
        generate_docs::<RtpInputStream>("RtpInputStream", &config),
        generate_docs::<Mp4>("Mp4", &config),
    ];

    let component_pages = [
        generate_docs::<Shader>("Shader", &config),
        generate_docs::<InputStream>("InputStream", &config),
        generate_docs::<View>("View", &config),
        generate_docs::<WebView>("WebView", &config),
        generate_docs::<Image>("Image", &config),
        generate_docs::<Text>("Text", &config),
        generate_docs::<Tiles>("Tiles", &config),
        generate_docs::<Rescaler>("Rescaler", &config),
    ];

    let output_pages = [generate_docs::<RtpOutputStream>("OutputStream", &config)];

    for page in renderer_pages {
        fs::write(
            docs_path.join(format!("renderer-{}.md", page.title)),
            page.to_markdown(&config),
        )
        .unwrap();
    }
    for page in component_pages {
        fs::write(
            docs_path.join(format!("component-{}.md", page.title)),
            page.to_markdown(&config),
        )
        .unwrap();
    }
    for page in output_pages {
        fs::write(
            docs_path.join(format!("output-{}.md", page.title)),
            page.to_markdown(&config),
        )
        .unwrap();
    }
}
