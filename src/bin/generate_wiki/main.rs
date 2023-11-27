use docs_builder::DocsBuilder;
use std::{fs, path::PathBuf};
use types::Component;

mod docs_builder;
mod types;

fn main() {
    let docs_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs");
    if !docs_path.exists() {
        fs::create_dir(&docs_path).unwrap();
    }

    let pages = DocsBuilder::new().add_definition::<Component>(true).build();
    for (name, content) in pages {
        fs::write(docs_path.join(format!("{name}.md")), &content).unwrap();
    }
}
