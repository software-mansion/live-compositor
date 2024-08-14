use anyhow::Result;
use serde_json::json;

use crate::{generate_transition_example, guide_asset_path};

pub(super) fn generate_transitions_guide() -> Result<()> {
    generate_transition_example(
        guide_asset_path("transitions_1.webp"),
        json!({
            "type": "view",
            "background_color_rgba": "#4d4d4dff",
            "children": [
                {
                    "id": "rescaler_1",
                    "type": "rescaler",
                    "width": 480,
                    "child": { "type": "input_stream", "input_id": "input_1" },
                }
            ]
        }),
        json!({
            "type": "view",
            "background_color_rgba": "#4d4d4dff",
            "children": [
                {
                    "id": "rescaler_1",
                    "type": "rescaler",
                    "width": 1280,
                    "transition": {
                        "duration_ms": 2000,
                    },
                    "child": { "type": "input_stream", "input_id": "input_1" },
                },
            ]
        }),
    )?;

    generate_transition_example(
        guide_asset_path("transitions_2.webp"),
        json!({
            "type": "view",
            "background_color_rgba": "#4d4d4dff",
            "children": [
                {
                    "id": "rescaler_1",
                    "type": "rescaler",
                    "width": 480,
                    "child": { "type": "input_stream", "input_id": "input_1" },
                },
                {
                    "type": "rescaler",
                    "child": { "type": "input_stream", "input_id": "input_2" },
                }

            ]
        }),
        json!({
            "type": "view",
            "background_color_rgba": "#4d4d4dff",
            "children": [
                {
                    "id": "rescaler_1",
                    "type": "rescaler",
                    "width": 1280,
                    "transition": {
                        "duration_ms": 2000,
                    },
                    "child": { "type": "input_stream", "input_id": "input_1" },
                },
                {
                    "type": "rescaler",
                    "child": { "type": "input_stream", "input_id": "input_2" },
                }
            ]
        }),
    )?;

    generate_transition_example(
        guide_asset_path("transitions_3.webp"),
        json!({
            "type": "view",
            "background_color_rgba": "#4d4d4dff",
            "children": [
                {
                    "id": "rescaler_1",
                    "type": "rescaler",
                    "width": 480,
                    "child": { "type": "input_stream", "input_id": "input_1" },
                },
            ]
        }),
        json!({
            "type": "view",
            "background_color_rgba": "#4d4d4dff",
            "children": [
                {
                    "id": "rescaler_1",
                    "type": "rescaler",
                    "width": 1280,
                    "top": 0,
                    "left": 0,
                    "transition": { "duration_ms": 2000 },
                    "child": { "type": "input_stream", "input_id": "input_1" },
                },
            ]
        }),
    )?;

    generate_transition_example(
        guide_asset_path("transitions_4.webp"),
        json!({
            "type": "view",
            "background_color_rgba": "#4d4d4dff",
            "children": [
                {
                    "id": "rescaler_1",
                    "type": "rescaler",
                    "width": 320, "height": 180, "top": 0, "left": 0,
                    "child": { "type": "input_stream", "input_id": "input_1" },
                },
                {
                    "id": "rescaler_2",
                    "type": "rescaler",
                    "width": 320, "height": 180, "top": 0, "left": 320,
                    "child": { "type": "input_stream", "input_id": "input_2" },
                },
                {
                    "id": "rescaler_3",
                    "type": "rescaler",
                    "width": 320, "height": 180, "top": 0, "left": 640,
                    "child": { "type": "input_stream", "input_id": "input_3" },
                },
                {
                    "id": "rescaler_4",
                    "type": "rescaler",
                    "width": 320, "height": 180, "top": 0, "left": 960,
                    "child": { "type": "input_stream", "input_id": "input_4" },
                },
            ]
        }),
        json!({
            "type": "view",
            "background_color_rgba": "#4d4d4dff",
            "children": [
                {
                    "id": "rescaler_1",
                    "type": "rescaler",
                    "width": 320, "height": 180, "top": 540, "left": 0,
                    "transition": { "duration_ms": 2000 },
                    "child": { "type": "input_stream", "input_id": "input_1" },
                },
                {
                    "id": "rescaler_2",
                    "type": "rescaler",
                    "width": 320, "height": 180, "top": 540, "left": 320,
                    "transition": { "duration_ms": 2000, "easing_function": {"function_name": "bounce"} },
                    "child": { "type": "input_stream", "input_id": "input_2" },
                },
                {
                    "id": "rescaler_3",
                    "type": "rescaler",
                    "width": 320, "height": 180, "top": 540, "left": 640,
                    "child": { "type": "input_stream", "input_id": "input_3" },
                    "transition": {
                        "duration_ms": 2000,
                        "easing_function": {
                            "function_name": "cubic_bezier",
                            "points": [0.65, 0, 0.35, 1]
                        }
                    },
                },
                {
                    "id": "rescaler_4",
                    "type": "rescaler",
                    "width": 320, "height": 180, "top": 540, "left": 960,
                    "child": { "type": "input_stream", "input_id": "input_4" },
                    "transition": {
                        "duration_ms": 2000,
                        "easing_function": {
                            "function_name": "cubic_bezier",
                            "points": [0.33, 1, 0.68, 1]
                        }
                    },
                },
            ]
        }),
    )?;
    Ok(())
}
