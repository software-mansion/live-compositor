use std::path::PathBuf;

use simple_scene::generate_simple_scene_guide;
use transition::generate_tile_transition_video;

mod simple_scene;
mod transition;

fn main() {
    generate_tile_transition_video().unwrap();
    generate_simple_scene_guide().unwrap();
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
