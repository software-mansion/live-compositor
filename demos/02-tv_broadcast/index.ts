import { registerImage, registerInput, registerOutput, registerShader, start, updateOutput } from "../utils/api";
import { ffmpegSendVideoFromMp4, ffplayListenAudioAsync, ffplayListenVideoAsync } from "../utils/ffmpeg";
import { runCompositorExample } from "../utils/run";
import { downloadAsync, sleepAsync } from "../utils/utils";
import fs from "fs-extra";
import path from "path";
import { Component, Resolution } from "../types/api";

const OUTPUT_RESOLUTION: Resolution = {
    width: 1920,
    height: 1080,
};

const INPUT_PORT = 8002;
const VIDEO_OUTPUT_PORT = 8004;
const AUDIO_OUTPUT_PORT = 8006;
const IP = "127.0.0.1";

const DISPLAY_LOGS = true;
const BUNNY_PATH = path.join(__dirname, "../assets/bunny.mp4");
const TV_PATH = path.join(__dirname, "../assets/green_screen_example.mp4");

async function example() {
    process.env.LIVE_COMPOSITOR_LOGGER_LEVEL = "debug";
    await downloadAsync(
        "https://assets.mixkit.co/videos/preview/mixkit-female-reporter-reporting-with-microphone-in-hand-on-a-chroma-28293-large.mp4",
        TV_PATH
    );

    await downloadAsync(
        "https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4",
        BUNNY_PATH
    );

    // starts ffplay that will listen for streams on port 8002 and display them.
    await ffplayListenVideoAsync(IP, VIDEO_OUTPUT_PORT, DISPLAY_LOGS);
    await ffplayListenAudioAsync(IP, AUDIO_OUTPUT_PORT, DISPLAY_LOGS);

    // sleep to make sure ffplay have a chance to start before compositor starts sending packets
    await sleepAsync(2000);

    await registerInput("input_1", {
        type: "rtp_stream",
        port: INPUT_PORT,
        video: {
            decoder: "ffmpeg_h264"
        }
    });

    await registerShader("remove_green_screen", {
        source: await fs.readFile(path.join(__dirname, "remove_green_screen.wgsl"), "utf-8")
    })

    await registerImage("background", {
        asset_type: "jpeg",
        path: path.join(__dirname, "../assets/news_room.jpg")
    });

    await registerImage("logo", {
        asset_type: "png",
        path: path.join(__dirname, "../assets/logo.png")
    })

    await registerOutput("output_video", {
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
                root: initialScene()
            }
        }
    });


    await sleepAsync(2000);
    ffmpegSendVideoFromMp4(INPUT_PORT, TV_PATH, DISPLAY_LOGS);
    await start();

    await registerInput("bunny", {
        type: "mp4",
        path: BUNNY_PATH,
    });

    await sleepAsync(10_000);
    // First update to set start position of the bunny for transition
    await registerOutput("output_audio", {
        type: "rtp_stream",
        ip: IP,
        port: AUDIO_OUTPUT_PORT,
        audio: {
            encoder: {
                type: "opus",
                channels: "stereo",
            },
            initial: {
                inputs: [{ input_id: "bunny" }]
            }
        }
    });

    await updateOutput("output_video", {
        video: {
            root: bunnyOutsideScene()
        }
    });
    await updateOutput("output_video", {
        video: {
            root: bunnyInsideScene()
        }
    });
    await sleepAsync(5_000);
    await updateOutput("output_video", {
        video: {
            root: finalScene()
        }
    });
}

function finalScene(): Component {
    return {
        type: "view",
        children: [
            news_report(),
            {
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
            },
            logo()
        ]

    }
}

function initialScene(): Component {
    return {
        type: "view",
        children: [
            news_report(),
            logo()
        ]
    }
}

function bunnyOutsideScene(): Component {
    return {
        type: "view",
        children: [
            news_report(),
            {
                type: "view",
                id: "bunny_view",
                width: OUTPUT_RESOLUTION.width,
                height: OUTPUT_RESOLUTION.height,
                top: 0,
                left: OUTPUT_RESOLUTION.width,
                children: [bunny_input()]
            },
            logo()
        ]
    }
}

function bunnyInsideScene(): Component {
    return {
        type: "view",
        children: [
            news_report(),
            {
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
            },
            logo()
        ]
    }

}

function news_report(): Component {
    const rescaledInputStream: Component = {
        type: "rescaler",
        width: OUTPUT_RESOLUTION.width,
        height: OUTPUT_RESOLUTION.height,
        child: {
            type: "input_stream",
            input_id: "input_1",
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

    const reportWithBackground: Component = {
        type: "shader",
        shader_id: "remove_green_screen",
        children: [rescaledInputStream, rescaledImage],
        resolution: OUTPUT_RESOLUTION
    }

    return {
        type: "view",
        width: OUTPUT_RESOLUTION.width,
        height: OUTPUT_RESOLUTION.height,
        top: 0,
        left: 0,
        children: [
            reportWithBackground,
            breakingNewsText(),
        ]
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
                        text: "21:37",
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
