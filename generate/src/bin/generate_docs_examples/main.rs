use std::path::PathBuf;

use layouts::generate_layouts_guide;
use quick_start::generate_quick_start_guide;
use transition::generate_tile_transition_video;
use view_transitions::generate_view_transition_guide;

mod layouts;
mod quick_start;
mod transition;
mod view_transitions;

fn main() {
    generate_quick_start_guide().unwrap();
    generate_layouts_guide().unwrap();
    generate_tile_transition_video().unwrap();
    generate_view_transition_guide().unwrap();
}

fn workingdir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("workingdir")
        .join("inputs")
}

fn pages_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("docs")
        .join("pages")
}
