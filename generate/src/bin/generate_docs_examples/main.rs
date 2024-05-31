use std::path::PathBuf;

use simple_scene::generate_simple_scene_guide;
use transition::generate_tile_transition_video;
use view_transitions::generate_view_transition_guide;

mod simple_scene;
mod transition;
mod view_transitions;

fn main() {
    generate_tile_transition_video().unwrap();
    generate_simple_scene_guide().unwrap();
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
