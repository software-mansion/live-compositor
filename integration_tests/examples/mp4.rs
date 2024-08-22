use anyhow::Result;
use compositor_api::types::Resolution;
use serde_json::json;
use std::time::Duration;

use integration_tests::examples::{self, run_example};

const BUNNY_PATH: &str = "./examples/assets/BigBuckBunny.mp4";
const SINTEL_PATH: &str = "./examples/assets/Sintel.mp4";

const OUTPUT_RESOLUTION: Resolution = Resolution {
    width: 1920,
    height: 1080,
};

fn main() {
    run_example(client_code);
}

fn client_code() -> Result<()> {
    examples::post(
        "input/bunny/register",
        &json!({
            "type": "mp4",
            "path": BUNNY_PATH,
            "offset_ms": 0,
            "required": true
        }),
    )?;

    examples::post(
        "input/sintel/register",
        &json!({
            "type": "mp4",
            "path": SINTEL_PATH,
            "offset_ms": 0,
            "required": true
        }),
    )?;

    examples::post(
        "shader/corners_rounding/register",
        &json!({
            "source": include_str!("./corners_rounding.wgsl"),
        }),
    )?;

    examples::post(
        "output/output_1/register",
        &json!({
            "type": "mp4",
            "path": "output.mp4",
            "video": {
                "resolution": {
                    "width": OUTPUT_RESOLUTION.width,
                    "height": OUTPUT_RESOLUTION.height,
                },
                "encoder": {
                    "type": "ffmpeg_h264",
                    "preset": "medium"
                },
                "initial": {
                    "root": {
                        "type": "view",
                        "children": [
                            {
                                "type": "view",
                                "id": "bunny_view",
                                "top": 0,
                                "left": 0,
                                "width": 1920,
                                "height": 1080,
                                "children": [
                                    {
                                        "type": "rescaler",
                                        "child": {
                                            "type": "shader",
                                            "shader_id": "corners_rounding",
                                            "resolution": {
                                                "width": 1920,
                                                "height": 1080
                                            },
                                            "children": [{
                                                "type": "input_stream",
                                                "input_id": "bunny"
                                            }]
                                        }
                                    }
                                ]
                            },
                            {
                                "type": "view",
                                "id": "sintel_view",
                                "top": 20,
                                "right": 20,
                                "width": 640,
                                "height": 273,
                                "children": [
                                    {
                                        "type": "rescaler",
                                        "child": {
                                            "type": "shader",
                                            "shader_id": "corners_rounding",
                                            "resolution": {
                                                "width": 2520,
                                                "height": 1080
                                            },
                                            "children": [{
                                                "type": "input_stream",
                                                "input_id": "sintel"
                                            }]
                                        }
                                    }
                                ]
                            },
                            {
                                "type": "view",
                                "bottom": 0,
                                "left": 0,
                                "width": 1920,
                                "height": 100,
                                "background_color_rgba": "#40E0D0FF",
                                "children": [
                                    {
                                        "type": "text",
                                        "width": 1920,
                                        "align": "center",
                                        "weight": "bold",
                                        "text": "LiveCompositor - mixing MP4s 🚀",
                                        "font_size": 80,
                                        "color_rgba": "#000000FF",
                                    }
                                ]
                            },
                        ]
                    }
                }
            },
            "audio": {
                "encoder": {
                    "type": "aac",
                    "channels": "stereo"
                },
                "initial": {
                    "inputs": [
                        {"input_id": "bunny"},
                        {"input_id": "sintel", "volume": 0.35}
                    ]
                }
            }
        }),
    )?;

    examples::post(
        "output/output_1/update",
        &json!({
            "video": {
                "root": {
                    "type": "view",
                    "children": [
                        {
                            "type": "view",
                            "top": 273,
                            "left": 0,
                            "width": 950,
                            "height": 534,
                            "id": "bunny_view",
                            "children": [
                                {
                                    "type": "rescaler",
                                    "child": {
                                        "type": "shader",
                                        "shader_id": "corners_rounding",
                                        "resolution": {
                                            "width": 1920,
                                            "height": 1080
                                        },
                                        "children": [{
                                            "type": "input_stream",
                                            "input_id": "bunny"
                                        }]
                                    }
                                }
                            ],
                            "transition": {
                                "duration_ms": 1000,
                                "easing_function": {
                                    "function_name": "cubic_bezier",
                                    "points": [0.33, 1, 0.68, 1]
                                }
                            }
                        },
                        {
                            "type": "view",
                            "id": "sintel_view",
                            "top": 273,
                            "right": 0,
                            "width": 950,
                            "height": 534,
                            "children": [
                                {
                                    "type": "rescaler",
                                    "child": {
                                        "type": "shader",
                                        "shader_id": "corners_rounding",
                                        "resolution": {
                                            "width": 2520,
                                            "height": 1080
                                        },
                                        "children": [{
                                            "type": "input_stream",
                                            "input_id": "sintel"
                                        }]
                                    }
                                }
                            ],
                            "transition": {
                                "duration_ms": 1000,
                                "easing_function": {
                                    "function_name": "cubic_bezier",
                                    "points": [0.33, 1, 0.68, 1]
                                }
                            }
                        },
                        {
                            "type": "view",
                            "id": "text_1",
                            "bottom": 0,
                            "left": 0,
                            "width": 1920,
                            "height": 100,
                            "background_color_rgba": "#40E0D0FF",
                            "children": [
                                {
                                    "type": "text",
                                    "width": 1920,
                                    "align": "center",
                                    "weight": "bold",
                                    "text": "LiveCompositor - mixing MP4s 🚀",
                                    "font_size": 80,
                                    "color_rgba": "#000000FF",
                                }
                            ]
                        },
                        {
                            "type": "view",
                            "id": "text_2",
                            "bottom": 0,
                            "left": 1920,
                            "width": 1920,
                            "height": 100,
                            "background_color_rgba": "#40E0D0FF",
                            "children": [
                                {
                                    "type": "text",
                                    "width": 1920,
                                    "align": "center",
                                    "weight": "bold",
                                    "text": "Learn more at: compositor.live",
                                    "font_size": 80,
                                    "color_rgba": "#000000FF",
                                }
                            ],
                            "transition": {
                                "duration_ms": 1000,
                                "easing_function": {
                                    "function_name": "linear"
                                }
                            }
                        }
                    ]
                }
            },
            "audio": {
                "inputs": [
                    {"input_id": "bunny"},
                    {"input_id": "sintel", "volume": 0.35}
                ]
            },
            "schedule_time_ms": 5000
        }),
    )?;

    examples::post(
        "output/output_1/update",
        &json!({
            "video": {
                "root": {
                    "type": "view",
                    "children": [
                        {
                            "type": "view",
                            "id": "sintel_view",
                            "top": 0,
                            "right": 0,
                            "width": 1920,
                            "height": 1080,
                            "children": [
                                {
                                    "type": "rescaler",
                                    "child": {
                                        "type": "shader",
                                        "shader_id": "corners_rounding",
                                        "resolution": {
                                            "width": 2520,
                                            "height": 1080
                                        },
                                        "children": [{
                                            "type": "input_stream",
                                            "input_id": "sintel"
                                        }]
                                    }
                                }
                            ],
                            "transition": {
                                "duration_ms": 1000,
                                "easing_function": {
                                    "function_name": "cubic_bezier",
                                    "points": [0.33, 1, 0.68, 1]
                                }
                            }
                        },
                        {
                            "type": "view",
                            "top": 100,
                            "left": 20,
                            "width": 480,
                            "height": 360,
                            "id": "bunny_view",
                            "children": [
                                {
                                    "type": "rescaler",
                                    "child": {
                                        "type": "shader",
                                        "shader_id": "corners_rounding",
                                        "resolution": {
                                            "width": 1920,
                                            "height": 1080
                                        },
                                        "children": [{
                                            "type": "input_stream",
                                            "input_id": "bunny"
                                        }]
                                    }
                                }
                            ],
                            "transition": {
                                "duration_ms": 1000,
                                "easing_function": {
                                    "function_name": "cubic_bezier",
                                    "points": [0.33, 1, 0.68, 1]
                                }
                            }
                        },
                        {
                            "type": "view",
                            "id": "text_1",
                            "bottom": 0,
                            "left": -1920,
                            "width": 1920,
                            "height": 100,
                            "background_color_rgba": "#40E0D0FF",
                            "children": [
                                {
                                    "type": "text",
                                    "width": 1920,
                                    "align": "center",
                                    "weight": "bold",
                                    "text": "LiveCompositor - mixing MP4s 🚀",
                                    "font_size": 80,
                                    "color_rgba": "#000000FF",
                                }
                            ],
                            "transition": {
                                "duration_ms": 1000,
                                "easing_function": {
                                    "function_name": "linear"
                                }
                            }
                        },
                        {
                            "type": "view",
                            "id": "text_2",
                            "bottom": 0,
                            "left": 0,
                            "width": 1920,
                            "height": 100,
                            "background_color_rgba": "#40E0D0FF",
                            "children": [
                                {
                                    "type": "text",
                                    "width": 1920,
                                    "align": "center",
                                    "weight": "bold",
                                    "text": "Learn more at: compositor.live",
                                    "font_size": 80,
                                    "color_rgba": "#000000FF",
                                }
                            ],
                            "transition": {
                                "duration_ms": 1000,
                                "easing_function": {
                                    "function_name": "linear"
                                }
                            }
                        }
                    ]
                }
            },
            "audio": {
                "inputs": [
                    {"input_id": "bunny"},
                    {"input_id": "sintel", "volume": 0.35}
                ]
            },
            "schedule_time_ms": 10000
        }),
    )?;

    std::thread::sleep(Duration::from_millis(500));

    examples::post("start", &json!({}))?;

    examples::post(
        "output/output_1/unregister",
        &json!({
            "schedule_time_ms": 28_000
        }),
    )?;

    Ok(())
}
