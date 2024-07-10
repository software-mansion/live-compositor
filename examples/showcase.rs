use anyhow::Result;
use common::download_file;
use live_compositor::{server, types::Resolution};
use log::{error, info};
use serde_json::json;
use std::{
    f32::consts::PI,
    thread::{self},
    time::Duration,
};

use crate::common::{start_ffplay, start_websocket_thread};

#[path = "./common/common.rs"]
mod common;

const BUNNY_URL: &str =
    "https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4";
const TV_PATH: &str = "./examples/assets/tv.mp4";
const CODE_VIDEO_PATH: &str = "./examples/assets/code.mp4";
const MEMBRANE_PATH: &str = "./examples/assets/membrane.mp4";
const JELLY_PATH: &str = "./examples/assets/jelly.mp4";
const SQUIRTLE_PATH: &str = "./examples/assets/squirtle.mp4";
const PAWEL_PATH: &str = "./examples/assets/pawel.mp4";
const GAMEPLAY_PATH: &str = "./examples/assets/gameplay.mp4";

const BG_PATH: &str = "./examples/assets/bg.png";
const NEWS_PATH: &str = "./examples/assets/news_room.jpg";
const CODE_BG_PATH: &str = "./examples/assets/code_bg.png";
const BROADCAST_BG_PATH: &str = "./examples/assets/broadcast_bg.png";
const VC_BG_PATH: &str = "./examples/assets/vc_bg.png";
const LS_BG_PATH: &str = "./examples/assets/ls_bg.png";
const ENDING_BG_PATH: &str = "./examples/assets/ending.png";

const VIDEO_RESOLUTION: Resolution = Resolution {
    width: 1920,
    height: 1080,
};

const IP: &str = "127.0.0.1";
const OUTPUT_VIDEO_PORT: u16 = 8002;
const OUTPUT_AUDIO_PORT: u16 = 8004;

fn main() {
    ffmpeg_next::format::network::init();

    let a = glam::Mat4::perspective_rh(PI * 0.25, 16.0 / 9.0, 0.1, 1.0);
    println!("{:#?}", a);

    thread::spawn(|| {
        if let Err(err) = start_example_client_code() {
            error!("{err}")
        }
    });

    server::run();
}

fn start_example_client_code() -> Result<()> {
    info!("[example] Start listening on output port.");
    start_ffplay(IP, OUTPUT_VIDEO_PORT, Some(OUTPUT_VIDEO_PORT))?;
    start_websocket_thread();

    const BUNNY_PATH: &str = "./examples/assets/bunny_out.mp4";
    download_file(BUNNY_URL, BUNNY_PATH)?;

    info!("[example] Send register input request.");
    common::post(
        "input/bunny/register",
        &json!({
            "type": "mp4",
            "path": BUNNY_PATH
        }),
    )?;

    info!("[example] Send register input request.");
    common::post(
        "input/tv/register",
        &json!({
            "type": "mp4",
            "path": TV_PATH,
            "required": true
        }),
    )?;

    info!("[example] Send register input request.");
    common::post(
        "input/code/register",
        &json!({
            "type": "mp4",
            "path": CODE_VIDEO_PATH,
            "offset_ms": 9000,
            "required": true
        }),
    )?;

    info!("[example] Send register input request.");
    common::post(
        "input/membrane/register",
        &json!({
            "type": "mp4",
            "path": MEMBRANE_PATH,
            "offset_ms": 45000,
            "required": true
        }),
    )?;

    info!("[example] Send register input request.");
    common::post(
        "input/jelly/register",
        &json!({
            "type": "mp4",
            "path": JELLY_PATH,
            "offset_ms": 45000,
            "required": true
        }),
    )?;

    info!("[example] Send register input request.");
    common::post(
        "input/squirtle/register",
        &json!({
            "type": "mp4",
            "path": SQUIRTLE_PATH,
            "offset_ms": 45000,
            "required": true
        }),
    )?;

    info!("[example] Send register input request.");
    common::post(
        "input/pawel/register",
        &json!({
            "type": "mp4",
            "path": PAWEL_PATH,
            "offset_ms": 50000,
            "required": true
        }),
    )?;

    info!("[example] Send register input request.");
    common::post(
        "input/gameplay/register",
        &json!({
            "type": "mp4",
            "path": GAMEPLAY_PATH,
            "offset_ms": 50000,
            "required": true
        }),
    )?;

    info!("[example] Send register shader request.");
    common::post(
        "shader/showcase_3d/register",
        &json!({
            "source": include_str!("./showcase_3d.wgsl")
        }),
    )?;

    info!("[example] Send register shader request.");
    common::post(
        "shader/showcase_code/register",
        &json!({
            "source": include_str!("./showcase_code.wgsl")
        }),
    )?;

    info!("[example] Send register shader request.");
    common::post(
        "shader/round_corners_20/register",
        &json!({
            "source": include_str!("./round_corners_20.wgsl")
        }),
    )?;

    info!("[example] Send register shader request.");
    common::post(
        "shader/round_corners_50/register",
        &json!({
            "source": include_str!("./round_corners_50.wgsl")
        }),
    )?;

    info!("[example] Send register shader request.");
    common::post(
        "shader/round_corners_100/register",
        &json!({
            "source": include_str!("./round_corners_100.wgsl")
        }),
    )?;

    info!("[example] Send register shader request.");
    common::post(
        "shader/round_corners_dynamic/register",
        &json!({
            "source": include_str!("./round_corners_dynamic.wgsl")
        }),
    )?;

    info!("[example] Send register shader request.");
    common::post(
        "shader/code_bg/register",
        &json!({
            "source": include_str!("./code_bg.wgsl")
        }),
    )?;


    info!("[example] Send register image request.");
    common::post(
        "image/background/register",
        &json!({
            "path": BG_PATH,
            "asset_type": "png"
        }),
    )?;

    info!("[example] Send register image request.");
    common::post(
        "image/news_room/register",
        &json!({
            "path": NEWS_PATH,
            "asset_type": "jpeg"
        }),
    )?;

    info!("[example] Send register image request.");
    common::post(
        "image/code_bg/register",
        &json!({
            "path": CODE_BG_PATH,
            "asset_type": "png"
        }),
    )?;

    info!("[example] Send register image request.");
    common::post(
        "image/broadcast_bg/register",
        &json!({
            "path": BROADCAST_BG_PATH,
            "asset_type": "png"
        }),
    )?;

    info!("[example] Send register image request.");
    common::post(
        "image/vc_bg/register",
        &json!({
            "path": VC_BG_PATH,
            "asset_type": "png"
        }),
    )?;

    info!("[example] Send register image request.");
    common::post(
        "image/ls_bg/register",
        &json!({
            "path": LS_BG_PATH,
            "asset_type": "png"
        }),
    )?;

    info!("[example] Send register image request.");
    common::post(
        "image/ending/register",
        &json!({
            "path": ENDING_BG_PATH,
            "asset_type": "png"
        }),
    )?;

    info!("[example] Send register output video request.");
    common::post(
        "output/output_video/register",
        &json!({
            "type": "rtp_stream",
            "ip": IP,
            "port": OUTPUT_VIDEO_PORT,
            "video": {
                "resolution": {
                    "width": VIDEO_RESOLUTION.width,
                    "height": VIDEO_RESOLUTION.height,
                },
                "encoder": {
                    "type": "ffmpeg_h264",
                    "preset": "veryslow"
                    // "preset": "ultrafast"
                },
                "initial": {
                    "root":
                        {
                            "type": "view",
                            "id": "animation",
                            "width": VIDEO_RESOLUTION.width,
                            "height": VIDEO_RESOLUTION.height,
                            "top": 0,
                            "left": 0,
                            "id": "animation",
                            "children": [
                                shader_3d()
                            ]
                        }

                }
            }
        }),
    )?;

    info!("[example] Send register output audio request.");
    common::post(
        "output/output_audio/register",
        &json!({
            "type": "rtp_stream",
            "ip": IP,
            "port": OUTPUT_AUDIO_PORT,
            "audio": {
                "encoder": {
                    "type": "opus",
                    "channels": "stereo"
                },
                "initial": {
                    "inputs": []
                }
            }
        }),
    )?;

    info!("[example] Send update output video request.");
    common::post(
        "output/output_video/update",
        &json!({
            "video": {
                "root": {
                    "type": "view",
                    "direction": "row",
                    "children": [
                        {
                            "type": "view",
                            "top": 0,
                            "left": 0,
                            "children": [code_bg()]
                        },
                        {
                            "type": "view",
                            "width": 928,
                            "height": 1080,
                            "top": 152,
                            "left": 992,
                            "children": [{
                                "type": "input_stream",
                                "input_id": "code"
                            }]
                        },
                        {
                            "type": "view",
                            "width": 992,
                            "height": 1080,
                            "children": [
                                {
                                    "type": "view",
                                    "id": "animation",
                                    "width": 992,
                                    "height": 558,
                                    "top": 337,
                                    "left": 0,
                                    "children": [
                                        {
                                            "type": "rescaler",
                                            "child": shader_3d()
                                        }
                                    ],
                                    "transition": {
                                        "duration_ms": 1000,
                                    }
                                },
                            ],
                        },
                    ]
                }
            },
            "schedule_time_ms": 9000
        }),
    )?;

    info!("[example] Send update output video request.");
    common::post(
        "output/output_video/update",
        &json!({
            "video": {
                "root": {
                    "type": "view",
                    "direction": "row",
                    "children": [
                        {
                            "type": "view",
                            "top": 0,
                            "left": 0,
                            "children": [code_bg()]
                        },
                        {
                            "type": "view",
                            "width": 928,
                            "height": 1080,
                            "top": 152,
                            "left": 992,
                            "children": [{
                                "type": "input_stream",
                                "input_id": "code"
                            }]
                        },
                        {
                            "type": "view",
                            "width": 992,
                            "height": 1080,
                            "children": [
                                {
                                    "type": "view",
                                    "id": "animation",
                                    "width": 992,
                                    "height": 558,
                                    "top": 337,
                                    "left": 0,
                                    "children": [{
                                        "type": "rescaler",
                                        "child": shader_3d()
                                    }],
                                    "transition": {
                                        "duration_ms": 1000,
                                    }
                                },
                            ],
                        },
                    ]
                }
            },
            "schedule_time_ms": 11000
        }),
    )?;

    let coding = json!(
        [
            {
                "type": "view",
                "id": "code_bg",
                "top": 0,
                "left": 0,
                "children": [code_bg()]
            },
            {
                "type": "view",
                "width": 928,
                "height": 1080,
                "top": 152,
                "left": 992,
                "children": [{
                    "type": "input_stream",
                    "input_id": "code"
                }]
            },
            {
                "type": "view",
                "width": 992,
                "height": 1080,
                "children": [{
                    "type": "view",
                    "id": "animation",
                    "width": 992,
                    "height": 558,
                    "top": 337,
                    "left": 0,
                    "children": [{
                        "type": "rescaler",
                        "child": shader_code()
                    }],
                }],
            }
        ]
    );

    info!("[example] Send update output video request.");
    common::post(
        "output/output_video/update",
        &json!({
            "video": {
                "root": {
                    "type": "view",
                    "children": [
                        {
                            "type": "view",
                            "id": "broadcast_bg",
                            "top": 0,
                            "left": 1920,
                            "children": [
                                {
                                    "type": "image",
                                    "image_id": "broadcast_bg"
                                }
                            ],
                        },
                        {
                            "type": "view",
                            "id": "coding",
                            "top": 0,
                            "left": 0,
                            "direction": "row",
                            "children": coding
                        },
                    ]

                },
            },
            "schedule_time_ms": 15000
        }),
    )?;

    info!("[example] Send update output video request.");
    common::post(
        "output/output_video/update",
        &json!({
            "video": {
                "root": {
                    "type": "view",
                    "children": [
                        {
                            "type": "view",
                            "id": "code_bg",
                            "top": 0,
                            "left": -1920,
                            "children": [code_bg()],
                            "transition": {
                                "duration_ms": 1000,
                            }
                        },
                        {
                            "type": "view",
                            "id": "broadcast_bg",
                            "top": 0,
                            "left": 0,
                            "children": [{
                                "type": "image",
                                "image_id": "broadcast_bg"
                            }],
                            "transition": {
                                "duration_ms": 1000,
                            }
                        },
                        {
                            "type": "view",
                            "id": "vc_bg",
                            "top": 0,
                            "left": 1920,
                            "children": [{
                                "type": "image",
                                "image_id": "vc_bg"
                            }],
                        },
                        {
                            "type": "view",
                            "id": "animation",
                            "width": 1600,
                            "height": 900,
                            "top": 110,
                            "left": 160,
                            "children": [{
                                "type": "rescaler",
                                "child": shader_dynamic()
                            }],
                            "transition": {
                                "duration_ms": 1000,
                            }
                        },
                        {
                            "type": "view",
                            "id": "video_conferencing",
                            "width": 1600,
                            "height": 900,
                            "top": 110,
                            "left": 1920,
                            "children": video_conferencing(),
                        },
                    ]
                },
            },
            "schedule_time_ms": 43000
        }),
    )?;

    info!("[example] Send update output video request.");
    common::post(
        "output/output_video/update",
        &json!({
            "video": {
                "root": {
                    "type": "view",
                    "children": [
                        {
                            "type": "view",
                            "id": "broadcast_bg",
                            "top": 0,
                            "left": -1920,
                            "children": [
                                {
                                    "type": "image",
                                    "image_id": "broadcast_bg"
                                }
                            ],
                            "transition": {
                                "duration_ms": 1000,
                            }
                        },
                        {
                            "type": "view",
                            "id": "vc_bg",
                            "top": 0,
                            "left": 0,
                            "children": [
                                {
                                    "type": "image",
                                    "image_id": "vc_bg"
                                }
                            ],
                            "transition": {
                                "duration_ms": 1000,
                            }
                        },
                        {
                            "type": "view",
                            "id": "animation",
                            "width": 1600,
                            "height": 900,
                            "top": 110,
                            "left": -1920,
                            "children": [shader_dynamic()],
                            "transition": {
                                "duration_ms": 1000,
                            }
                        },
                        {
                            "type": "view",
                            "id": "video_conferencing",
                            "width": 1600,
                            "height": 900,
                            "top": 110,
                            "left": 160,
                            "children": video_conferencing(),
                            "transition": {
                                "duration_ms": 1000,
                            }
                        },
                    ]
                },
            },
            "schedule_time_ms": 49000
        }),
    )?;


    info!("[example] Send update output video request.");
    common::post(
        "output/output_video/update",
        &json!({
            "video": {
                "root": {
                    "type": "view",
                    "children": [
                        {
                            "type": "view",
                            "id": "vc_bg",
                            "top": 0,
                            "left": 0,
                            "children": [
                                {
                                    "type": "image",
                                    "image_id": "vc_bg"
                                }
                            ]
                        },
                        {
                            "type": "view",
                            "id": "video_conferencing",
                            "width": 1600,
                            "height": 900,
                            "top": 110,
                            "left": 160,
                            "children": video_conferencing2(),
                        },
                        {
                            "type": "view",
                            "id": "ls_bg",
                            "top": 0,
                            "left": 1920,
                            "children": [
                                {
                                    "type": "image",
                                    "image_id": "ls_bg"
                                }
                            ],
                        },
                        {
                            "type": "view",
                            "id": "ls",
                            "width": 1600,
                            "height": 900,
                            "top": 110,
                            "left": 1920,
                            "children": [live_stream()],
                        }
                    ]
                },
            },
            "schedule_time_ms": 51000
        }),
    )?;

    info!("[example] Send update output video request.");
    common::post(
        "output/output_video/update",
        &json!({
            "video": {
                "root": {
                    "type": "view",
                    "children": [
                        {
                            "type": "view",
                            "id": "vc_bg",
                            "top": 0,
                            "left": -1920,
                            "children": [
                                {
                                    "type": "image",
                                    "image_id": "vc_bg"
                                }
                            ],
                            "transition": {
                                "duration_ms": 1000,
                            }
                        },
                        {
                            "type": "view",
                            "id": "ending",
                            "top": 0,
                            "left": 1920,
                            "children": [
                                {
                                    "type": "image",
                                    "image_id": "ending"
                                }
                            ],
                            "transition": {
                                "duration_ms": 1000,
                            }
                        },
                        {
                            "type": "view",
                            "id": "video_conferencing",
                            "width": 1600,
                            "height": 900,
                            "top": 110,
                            "left": -1920,
                            "children": video_conferencing2(),
                            "transition": {
                                "duration_ms": 1000,
                            }
                        },
                        {
                            "type": "view",
                            "id": "ls_bg",
                            "top": 0,
                            "left": 0,
                            "children": [
                                {
                                    "type": "image",
                                    "image_id": "ls_bg"
                                }
                            ],
                            "transition": {
                                "duration_ms": 1000,
                            }
                        },
                        {
                            "type": "view",
                            "id": "ls",
                            "width": 1600,
                            "height": 900,
                            "top": 110,
                            "left": 160,
                            "children": [live_stream()],
                            "transition": {
                                "duration_ms": 1000,
                            }
                        }
                    ]
                },
            },
            "schedule_time_ms": 55000
        }),
    )?;

    info!("[example] Send update output video request.");
    common::post(
        "output/output_video/update",
        &json!({
            "video": {
                "root": {
                    "type": "view",
                    "children": [
                        {
                            "type": "view",
                            "id": "ending",
                            "top": 0,
                            "left": 0,
                            "children": [
                                {
                                    "type": "image",
                                    "image_id": "ending"
                                }
                            ],
                            "transition": {
                                "duration_ms": 1000,
                            }
                        },
                        {
                            "type": "view",
                            "id": "ls_bg",
                            "top": 0,
                            "left": -1920,
                            "children": [
                                {
                                    "type": "image",
                                    "image_id": "ls_bg"
                                }
                            ],
                            "transition": {
                                "duration_ms": 1000,
                            }
                        },
                        {
                            "type": "view",
                            "id": "ls",
                            "width": 1600,
                            "height": 900,
                            "top": 110,
                            "left": -1920,
                            "children": [live_stream()],
                            "transition": {
                                "duration_ms": 1000,
                            }
                        }
                    ]
                },
            },
            "schedule_time_ms": 61000
        }),
    )?;


    std::thread::sleep(Duration::from_millis(500));

    info!("[example] Start pipeline");
    common::post("start", &json!({}))?;

    Ok(())
}

fn shader_3d() -> serde_json::Value {
    json!({
        "type": "rescaler",
        "width": VIDEO_RESOLUTION.width,
        "height": VIDEO_RESOLUTION.height,
        "child": {
            "type": "shader",
            "shader_id": "showcase_3d",
            "resolution": {
                "width": VIDEO_RESOLUTION.width * 2,
                "height": VIDEO_RESOLUTION.height * 2,
            },
            "children": [
                {
                    "type": "image",
                    "image_id": "background",
                },
                bg(),
                {
                    "type": "input_stream",
                    "input_id": "tv",
                },
                {
                    "type": "input_stream",
                    "input_id": "bunny"
                },
            ]
        }
    })
}


fn shader_dynamic() -> serde_json::Value {
    json!({
        "type": "rescaler",
        "id": "animation_res",
        "width": 1600,
        "height": 900,
        "child": {
            "type": "shader",
            "shader_id": "round_corners_dynamic",
            "resolution": {
                "width": 1920,
                "height": 1080,
            },
            "children": [{
                "type": "shader",
                "shader_id": "showcase_code",
                "resolution": {
                    "width": VIDEO_RESOLUTION.width * 2,
                    "height": VIDEO_RESOLUTION.height * 2,
                },
                "children": [
                    {
                        "type": "image",
                        "image_id": "news_room",
                    },
                    {
                        "type": "input_stream",
                        "input_id": "tv",
                    },
                    {
                        "type": "view",
                        "width": 1920,
                        "height": 1080,
                        "children": [{
                            "type": "view",
                            "height": 120,
                            "left": 0,
                            "bottom": 0,
                            "background_color_rgba": "#B3B3B3FF",
                            "children": []
                        }]
                    },
                    {
                        "type": "view",
                        "width": 1920,
                        "height": 1080,
                        "children": [
                            {
                                "type": "view",
                                "height": 120,
                                "left": 0,
                                "bottom": 0,
                                "children": [
                                    {
                                        "type": "view"
                                    },
                                    {
                                        "type": "text",
                                        "text": "LiveCompositor",
                                        "font_size": 100,
                                        "weight": "bold",
                                        "color_rgba": "#000000FF"
                                    },
                                    {
                                        "type": "view"
                                    }
                                ]
                            }
                        ]
                    },
                    {
                        "type": "shader",
                        "shader_id": "round_corners_dynamic",
                        "resolution": {
                            "width": 1920,
                            "height": 1080,
                        },
                        "children": [{
                            "type": "input_stream",
                            "input_id": "bunny"
                        }]
                    },
                ]
            }]
        }   
    })
}

fn shader_code() -> serde_json::Value {
    json!({
        "type": "rescaler",
        "id": "animation_res",
        "width": VIDEO_RESOLUTION.width,
        "height": VIDEO_RESOLUTION.height,
        "child": {
            "type": "shader",
            "shader_id": "showcase_code",
            "resolution": {
                "width": VIDEO_RESOLUTION.width * 2,
                "height": VIDEO_RESOLUTION.height * 2,
            },
            "children": [
                {
                    "type": "image",
                    "image_id": "news_room",
                },
                {
                    "type": "input_stream",
                    "input_id": "tv",
                },
                {
                    "type": "view",
                    "width": 1920,
                    "height": 1080,
                    "children": [{
                        "type": "view",
                        "height": 120,
                        "left": 0,
                        "bottom": 0,
                        "background_color_rgba": "#B3B3B3FF",
                        "children": []
                    }]
                },
                {
                    "type": "view",
                    "width": 1920,
                    "height": 1080,
                    "children": [
                        {
                            "type": "view",
                            "height": 120,
                            "left": 0,
                            "bottom": 0,
                            "children": [
                                {
                                    "type": "view"
                                },
                                {
                                    "type": "text",
                                    "text": "LiveCompositor",
                                    "font_size": 100,
                                    "weight": "bold",
                                    "color_rgba": "#000000FF"
                                },
                                {
                                    "type": "view"
                                }
                            ]
                        }
                    ]
                },
                {
                    "type": "input_stream",
                    "input_id": "bunny"
                },
            ]
        }   
    })
}

fn video_conferencing() -> serde_json::Value {
    json!([{
        "type": "tiles",
        "id": "tiles",
        "margin": 10,
        "children": [
           tile("membrane".to_owned(), " Membrane 🪼 ".to_owned()),
           tile("jelly".to_owned(), " Jelly 🐙 ".to_owned()),
        ],
    }])
}

fn video_conferencing2() -> serde_json::Value {
    json!([{
        "type": "tiles",
        "id": "tiles",
        "margin": 10,
        "children": [
           tile("membrane".to_owned(), " Membrane 🪼 ".to_owned()),
           tile("jelly".to_owned(), " Jelly 🐙 ".to_owned()),
           tile("squirtle".to_owned(), " Squirtle 🐢 ".to_owned()),
        ],
        "transition": {
            "duration_ms": 500,
        }
    }])
}

fn tile(input_id: String, text: String) -> serde_json::Value {
    json!({
        "type": "shader",
        "shader_id": "round_corners_20",
        "resolution": {
            "width": 1920,
            "height": 1080,
        },
        "children": [{
            "type": "view",
            "width": 1920,
            "height": 1080,
            "children": [
                {
                    "type": "input_stream",
                    "input_id": input_id
                },
                {
                    "type": "view",
                    "width": 1920,
                    "height": 200,
                    "left": 0,
                    "bottom": 0,
                    "direction": "row",
                    "children": [
                        { "type": "view" },
                        {
                        "type": "text",
                        "text": text,
                        "font_size": 100,
                        "weight": "bold",
                        "color_rgba": "#FFFFFFFF",
                        "background_color_rgba": "#40E0D0FF",
                        "font_family": "Comic Sans MS",
                        "align": "center"
                        },
                        { "type": "view" },
                    ]
                }
            ]
        }]
    })
}

fn live_stream() -> serde_json::Value {
    json!({
        "type": "rescaler",
        "child": {
            "type": "shader",
            "shader_id": "round_corners_50",
            "resolution": {
                "width": 1920,
                "height": 1080
            },
            "children": [{
                "type": "view",
                "width": 1600,
                "height": 900,
                "children": [
                    {
                        "type": "rescaler",
                        "child": {
                            "type": "input_stream",
                            "input_id": "gameplay"
                        }
                    },
                    {
                        "type": "view",
                        "width": 400,
                        "height": 300,
                        "top": 30,
                        "left": 30,
                        "children": [{
                            "type": "rescaler",
                            "child": {
                                "type": "shader",
                                "shader_id": "round_corners_50",
                                "resolution": {
                                    "width": 1440,
                                    "height": 1080,
                                },
                                "children": [
                                    {
                                        "type": "input_stream",
                                        "input_id": "pawel"
                                    }
                                ]
                            }
                        }]
                    }
                ]
            }]
        }
    })
}

fn bg() -> serde_json::Value {
    json!({
        "type": "view",
        "width": 1920,
        "height": 1080,
        "children": [
             {
                 "type": "view",
                 "width": 1920,
                 "height": 1080,
                 "top": 0, "left": 0,
                 "children": [{
                    "type": "rescaler",
                    "mode": "fill",
                    "child": {
                        "type": "image",
                        "image_id": "news_room"
                    }
                 }]
             },
             {
                 "type": "view",
                 "height": 120,
                 "bottom": 0, "left": 0,
                 "background_color_rgba": "#B3B3B3FF",
                 "children": [
                   { "type": "view" },
                   {
                     "type": "text",
                     "text": "LiveCompositor",
                     "font_size": 100,
                     "weight": "bold",
                     "color_rgba": "#000000FF"
                   },
                   { "type": "view" }
                 ]
             },
        ]
     })
}

fn code_bg() -> serde_json::Value {
    json!({
        "type": "view",
        "width": 1920,
        "height": 1080,
        "top": 0,
        "left": 0,
        "children": [{
            "type": "shader",
            "shader_id": "code_bg",
            "resolution": {
                "width": 1920,
                "height": 1080,
            },
            "children": [
                {
                    "type": "image",
                    "image_id": "code_bg"
                },
                {
                    "type": "input_stream",
                    "input_id": "code"
                }
            ]
        }]
    })
}
