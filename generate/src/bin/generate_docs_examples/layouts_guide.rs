use crate::guide_asset_path;

use super::generate_scene_example;
use anyhow::Result;
use serde_json::json;

pub(super) fn generate_layouts_guide() -> Result<()> {
    generate_scene_example(
        guide_asset_path("layouts_1.webp"),
        json!({
            "type": "view",
            "background_color_rgba": "#4d4d4dff",
        }),
    )?;
    generate_scene_example(
        guide_asset_path("layouts_2.webp"),
        json!({
            "type": "view",
            "background_color_rgba": "#4d4d4dff",
            "children": [
                { "type": "input_stream", "input_id": "input_1" },
            ]
        }),
    )?;
    generate_scene_example(
        guide_asset_path("layouts_3.webp"),
        json!({
            "type": "view",
            "background_color_rgba": "#4d4d4dff",
            "children": [
                {
                    "type": "rescaler",
                    "child": { "type": "input_stream", "input_id": "input_1" },
                },
            ]
        }),
    )?;
    generate_scene_example(
        guide_asset_path("layouts_4.webp"),
        json!({
            "type": "view",
            "background_color_rgba": "#4d4d4dff",
            "children": [
                {
                    "type": "rescaler",
                    "child": { "type": "input_stream", "input_id": "input_1" },
                },
                {
                    "type": "rescaler",
                    "child": { "type": "input_stream", "input_id": "input_2" },
                }
            ]
        }),
    )?;
    generate_scene_example(
        guide_asset_path("layouts_5.webp"),
        json!({
            "type": "view",
            "background_color_rgba": "#4d4d4dff",
            "children": [
                {
                    "type": "rescaler",
                    "child": { "type": "input_stream", "input_id": "input_1" },
                },
                {
                    "type": "rescaler",
                    "width": 320,
                    "height": 180,
                    "top": 20,
                    "right": 20,
                    "child": { "type": "input_stream", "input_id": "input_2" },
                }
            ]
        }),
    )?;
    Ok(())
}
