use docs_config::DocsConfig;
use document::generate;
use live_compositor::types::{
    DeckLink, Image, ImageSpec, InputStream, Mp4, Rescaler, RtpInputStream, RtpOutputStream,
    Shader, ShaderSpec, Text, Tiles, View, WebRendererSpec, WebView,
};
use markdown::overrides;
use std::{fs, path::PathBuf};

mod definition;
mod docs_config;
mod document;
mod markdown;
mod schema_parser;

fn main() {
    tracing_subscriber::fmt().init();
    let docs_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("docs/pages/api/generated");
    if docs_path.exists() {
        fs::remove_dir_all(&docs_path).unwrap();
    }
    fs::create_dir(&docs_path).unwrap();

    let config = DocsConfig::default();

    let mut img_component_config = config.clone();
    img_component_config
        .overrides
        .insert("Image", overrides::force_multiline);

    let renderer_pages = [
        generate::<ShaderSpec>("Shader", &config),
        generate::<ImageSpec>("Image", &config),
        generate::<WebRendererSpec>("WebRenderer", &config),
        generate::<RtpInputStream>("RtpInputStream", &config),
        generate::<Mp4>("Mp4", &config),
        generate::<DeckLink>("DeckLink", &config),
    ];

    let component_pages = [
        generate::<Shader>("Shader", &config),
        generate::<InputStream>("InputStream", &config),
        generate::<View>("View", &config),
        generate::<WebView>("WebView", &config),
        generate::<Image>("Image", &img_component_config),
        generate::<Text>("Text", &config),
        generate::<Tiles>("Tiles", &config),
        generate::<Rescaler>("Rescaler", &config),
    ];

    let output_pages = [generate::<RtpOutputStream>("OutputStream", &config)];

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
