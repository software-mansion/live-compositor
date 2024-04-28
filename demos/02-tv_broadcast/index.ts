import { ffmpegSendVideoFromMp4, ffplayStartPlayerAsync } from "../utils/ffmpeg";
import { runCompositorExample } from "../utils/run";
import { downloadAsync, sleepAsync } from "../utils/utils";
import fs from "fs-extra";
import path from "path";
import { Component, Resolution } from "../types/api";
import { registerImageAsync, registerInputAsync, registerOutputAsync, registerShaderAsync, startAsync, updateOutputAsync } from "../utils/api";

const OUTPUT_RESOLUTION: Resolution = {
    width: 1920,
    height: 1080,
};

const INPUT_PORT = 9002;
const VIDEO_OUTPUT_PORT = 9004;
const AUDIO_OUTPUT_PORT = 9006;
const IP = "127.0.0.1";

const DISPLAY_LOGS = true;
const BUNNY_PATH = path.join(__dirname, "../assets/bunny.mp4");
const TV_PATH = path.join(__dirname, "../assets/green_screen_example.mp4");

async function example() {
    ffplayStartPlayerAsync(IP, DISPLAY_LOGS, VIDEO_OUTPUT_PORT, AUDIO_OUTPUT_PORT);
    await sleepAsync(2000);

    process.env.LIVE_COMPOSITOR_LOGGER_LEVEL = "debug";
    await downloadAsync(
        "https://assets.mixkit.co/videos/preview/mixkit-female-reporter-reporting-with-microphone-in-hand-on-a-chroma-28293-large.mp4",
        TV_PATH
    );

    await downloadAsync(
        "https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4",
        BUNNY_PATH
    );

    await registerInputAsync("tv_input", {
        type: "rtp_stream",
        port: INPUT_PORT,
        video: {
            decoder: "ffmpeg_h264"
        }
    });

    await registerInputAsync("bunny", {
        type: "mp4",
        path: BUNNY_PATH,
    });

    await registerShaderAsync("remove_green_screen", {
        source: await fs.readFile(path.join(__dirname, "remove_green_screen.wgsl"), "utf-8")
    })

    await registerImageAsync("background", {
        asset_type: "jpeg",
        url: "https://raw.githubusercontent.com/membraneframework-labs/video_compositor_snapshot_tests/main/demo_assets/news_room.jpeg"
    });

    await registerImageAsync("logo", {
        asset_type: "png",
        url: "https://raw.githubusercontent.com/membraneframework-labs/video_compositor_snapshot_tests/main/demo_assets/logo.png"
    })

    await registerOutputAsync("output_video", {
        type: "rtp_stream",
        ip: IP,
        port: VIDEO_OUTPUT_PORT,
        video: {
            resolution: OUTPUT_RESOLUTION,
            encoder: {
                type: "ffmpeg_h264",
                preset: "ultrafast"
            },
            initial: {
                root: makeScene(undefined)
            }
        }
    });

    await registerOutputAsync("output_audio", {
        type: "rtp_stream",
        ip: IP,
        port: AUDIO_OUTPUT_PORT,
        audio: {
            encoder: {
                channels: "stereo",
                type: "opus",
            },
            initial: {
                inputs: []
            }
        }
    });

    ffmpegSendVideoFromMp4(INPUT_PORT, TV_PATH, DISPLAY_LOGS);
    await startAsync();

    // First update to set start position of the bunny for transition
    await updateOutputAsync("output_video", {
        video: {
            root: makeScene(bunnyOutside)
        },
        schedule_time_ms: 10_000
    });

    // Bunny transitions
    await updateOutputAsync("output_video", {
        video: {
            root: makeScene(bunnyInside)
        },
        schedule_time_ms: 10_001
    });

    await updateOutputAsync("output_video", {
        video: {
            root: makeScene(finalBunnyPosition)
        },
        schedule_time_ms: 15_000
    });

    await updateOutputAsync("output_audio", {
        audio: {
            inputs: [{ input_id: "bunny" }]
        },
        schedule_time_ms: 10_000
    });
}

function makeScene(bunnyProducer: (() => Component) | undefined): Component {
    let components: Component[] = bunnyProducer ? [
        news_report(),
        bunnyProducer(),
        logo(),
        breakingNewsText(),
    ] : [
        news_report(),
        logo(),
        breakingNewsText(),
    ];

    return {
        type: "view",
        children: components
    };
}

function bunnyOutside(): Component {
    return {
        type: "view",
        id: "bunny_view",
        width: OUTPUT_RESOLUTION.width,
        height: OUTPUT_RESOLUTION.height,
        top: 0,
        left: OUTPUT_RESOLUTION.width,
        children: [bunny_input()]
    }
}

function bunnyInside(): Component {
    return {
        type: "view",
        id: "bunny_view",
        width: OUTPUT_RESOLUTION.width,
        height: OUTPUT_RESOLUTION.height,
        top: 0,
        left: 0,
        children: [bunny_input()],
        transition: {
            duration_ms: 1000,
            easing_function: {
                function_name: "bounce"
            }
        }
    };
}

function finalBunnyPosition(): Component {
    return {
        type: "view",
        id: "bunny_view",
        width: OUTPUT_RESOLUTION.width / 4,
        height: OUTPUT_RESOLUTION.height / 4,
        top: 20,
        right: 20,
        rotation: 360,
        children: [bunny_input()],
        transition: {
            duration_ms: 1000,
            easing_function: {
                function_name: "linear"
            }

        }
    };
}

function news_report(): Component {
    const rescaledInputStream: Component = {
        type: "rescaler",
        width: OUTPUT_RESOLUTION.width,
        height: OUTPUT_RESOLUTION.height,
        child: {
            type: "input_stream",
            input_id: "tv_input",
        }
    };

    const rescaledImage: Component = {
        type: "rescaler",
        width: OUTPUT_RESOLUTION.width,
        height: OUTPUT_RESOLUTION.height,
        child: {
            type: "image",
            image_id: "background"
        }
    };

    return {
        type: "shader",
        shader_id: "remove_green_screen",
        children: [rescaledInputStream, rescaledImage],
        resolution: OUTPUT_RESOLUTION
    };
}

function bunny_input(): Component {
    return {
        type: "rescaler",
        child: {
            type: "view",
            width: 1280,
            height: 720,
            children: [
                {
                    type: "input_stream",
                    input_id: "bunny"
                },
            ]
        }
    }
}

function breakingNewsText(): Component {
    return {
        type: "view",
        width: OUTPUT_RESOLUTION.width,
        height: 180,
        bottom: 0,
        left: 0,
        direction: "column",
        children: [
            {
                type: "text",
                text: "BREAKING NEWS",
                width: 600,
                height: 50,
                font_size: 50,
                weight: "bold",
                align: "center",
                color_rgba: "#FFFFFFFF",
                background_color_rgba: "#FF0000FF",
            },
            {
                type: "text",
                text: "LiveCompositor is rumored to allegedly compose video",
                font_size: 65,
                width: OUTPUT_RESOLUTION.width,
                height: 80,
                align: "center",
                color_rgba: "#FFFFFFFF",
                background_color_rgba: "#808080FF",
            },
            {
                type: "view",
                width: OUTPUT_RESOLUTION.width,
                height: 50,
                children: [
                    {
                        type: "text",
                        text: "88:29",
                        font_size: 40,
                        width: 200,
                        height: 50,
                        align: "center",
                        color_rgba: "#FFFFFFFF",
                        background_color_rgba: "#000000FF",
                    },
                    {
                        type: "text",
                        text: "Leak docs can be found at https://compositor.live/docs/intro",
                        font_size: 40,
                        width: OUTPUT_RESOLUTION.width - 200,
                        height: 50,
                        align: "center",
                        color_rgba: "#000000FF",
                        background_color_rgba: "#FFFF00FF",
                    }
                ]
            }
        ]
    }
}

function logo(): Component {
    return {
        type: "view",
        top: 50,
        left: 50,
        overflow: "fit",
        children: [{
            type: "image",
            image_id: "logo",
        }]
    }
}

runCompositorExample(example, DISPLAY_LOGS);
