use parsing::generate_docs;
use std::{fs, path::PathBuf};
use video_compositor::types::{
    Image, ImageSpec, InputStream, Mp4, Rescaler, RtpInputStream, RtpOutputStream, Shader,
    ShaderSpec, Text, Tiles, View, WebRendererSpec, WebView,
};

mod parsing;
mod type_definition;

fn main() {
    let docs_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/pages/api/generated");
    if docs_path.exists() {
        fs::remove_dir_all(&docs_path).unwrap();
    }
    fs::create_dir(&docs_path).unwrap();

    let renderer_pages = [
        generate_docs::<ShaderSpec>("Shader"),
        generate_docs::<ImageSpec>("Image"),
        generate_docs::<WebRendererSpec>("WebRenderer"),
        generate_docs::<RtpInputStream>("RtpInputStream"),
        generate_docs::<Mp4>("Mp4"),
    ];

    let component_pages = [
        generate_docs::<Shader>("Shader"),
        generate_docs::<InputStream>("InputStream"),
        generate_docs::<View>("View"),
        generate_docs::<WebView>("WebView"),
        generate_docs::<Image>("Image"),
        generate_docs::<Text>("Text"),
        generate_docs::<Tiles>("Tiles"),
        generate_docs::<Rescaler>("Rescaler"),
    ];

    let output_pages = [generate_docs::<RtpOutputStream>("OutputStream")];

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
    for page in output_pages {
        fs::write(
            docs_path.join(format!("output-{}.md", page.title)),
            page.to_markdown(),
        )
        .unwrap();
    }
}
