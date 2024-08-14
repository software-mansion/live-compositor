use crate::guide_asset_path;

use super::generate_scene_example;
use anyhow::Result;

use serde_json::json;

pub(super) fn generate_quick_start_guide() -> Result<()> {
    generate_scene_example(
        guide_asset_path("quick_start_1.webp"),
        json!({
            "type": "view",
            "background_color_rgba": "#4d4d4dff",
            "children": [
                {
                    "type": "view",
                    "background_color_rgba": "#4d4d4dff",
                }
            ]
        }),
    )?;
    generate_scene_example(
        guide_asset_path("quick_start_2.webp"),
        json!({
            "type": "view",
            "background_color_rgba": "#4d4d4dff",
            "children": [
                {
                    "type": "tiles",
                    "background_color_rgba": "#4d4d4dff",
                    "padding": 10,
                    "children": [
                      { "type": "input_stream", "input_id": "input_1" },
                      { "type": "input_stream", "input_id": "input_2" }
                    ]
                }
            ]
        }),
    )?;
    Ok(())
}
