import { registerInput, registerOutput, start, updateOutput } from "../utils/api";
import { ffmpegSendVideoFromMp4, ffplayListenVideoAsync } from "../utils/ffmpeg";
import { runCompositorExample } from "../utils/run";
import { downloadAsync, sleepAsync } from "../utils/utils";
import { Component, Resolution } from "../types/api";
import { gstStreamWebcam } from "../utils/gst";
import path from "path";

const OUTPUT_RESOLUTION: Resolution = {
    width: 1920,
    height: 1080,
};

const WEBCAM_INPUT_PORT = 8000;
const GAMEPLAY_PORT = 8002;
const OUTPUT_PORT = 8006;
const IP = "127.0.0.1";
const DISPLAY_LOGS = false;

async function example() {
    await ffplayListenVideoAsync(IP, OUTPUT_PORT, DISPLAY_LOGS);

    // sleep to make sure ffplay have a chance to start before compositor starts sending packets
    await sleepAsync(2000);

    const useWebCam = process.env.LIVE_COMPOSITOR_WEBCAM !== "false";
    const gameplayPath = path.join(__dirname, "../assets/gameplay.mp4");
    await downloadAsync("https://raw.githubusercontent.com/membraneframework-labs/video_compositor_snapshot_tests/main/demo_assets/gameplay.mp4", gameplayPath);

    // This is mock, since recording screen requires too many permissions and it's heavily OS specific.
    await registerInput("screen_input", {
        type: "mp4",
        path: path.join(__dirname, "../assets/green_screen_example.mp4"),
    });

    await registerInput("webcam_input", {
        type: "rtp_stream",
        port: WEBCAM_INPUT_PORT,
        transport_protocol: useWebCam ? "tcp_server" : "udp",
        video: {
            decoder: "ffmpeg_h264"
        }
    })


    if (useWebCam) {
        gstStreamWebcam(IP, WEBCAM_INPUT_PORT, DISPLAY_LOGS);
    } else {
        const callPath = path.join(__dirname, "../assets/call.mp4");
        await downloadAsync("https://raw.githubusercontent.com/membraneframework-labs/video_compositor_snapshot_tests/main/demo_assets/call.mp4", path.join(__dirname, "../assets/call.mp4"));
        ffmpegSendVideoFromMp4(WEBCAM_INPUT_PORT, callPath, DISPLAY_LOGS);
    }

    await registerInput("screen_input", {
        type: "rtp_stream",
        port: GAMEPLAY_PORT,

    });

    await registerOutput("output_1", {
        type: "rtp_stream",
        ip: IP,
        port: OUTPUT_PORT,
        video: {
            resolution: OUTPUT_RESOLUTION,
            encoder: {
                type: "ffmpeg_h264",
                preset: "ultrafast"
            },
            initial: {
                root: baseScene()
            }
        }
    });

    await sleepAsync(2000);

    await start();
    await sleepAsync(5000);
    displayDonate("XDDD!");
}

function displayDonate(msg: string) {
    const newScene: Component = {
        type: "view",
        children: [
            baseScene(),
            donateCard(msg)
        ]
    };

    updateOutput("output_1", {
        video: {
            root: newScene
        }
    });
};

function donateCard(msg: string): Component {
    const emptyView: Component = { type: "view" };
    return {
        type: "view",
        width: OUTPUT_RESOLUTION.width,
        height: OUTPUT_RESOLUTION.height,
        top: 50,
        left: 0,
        children: [
            emptyView,
            {
                type: "view",
                width: 500,
                height: 300,
                background_color_rgba: "#87CEEBFF",
                children: [{
                    type: "text",
                    text: msg,
                    font_size: 30,
                    align: "center",
                    color_rgba: "#32CD32FF",
                    font_family: "Comic Sans MS",
                    width: 500,
                }]
            },
            emptyView
        ]
    };
}

function baseScene(): Component {
    return {
        type: "view",
        children: [
            {
                type: "rescaler",
                child: {
                    type: "input_stream",
                    input_id: "screen_input"
                }
            },
            {
                type: "view",
                width: 500,
                height: 300,
                top: 30,
                right: 30,
                children: [{
                    type: "rescaler",
                    vertical_align: "top",
                    horizontal_align: "right",
                    child: {
                        type: "input_stream",
                        input_id: "webcam_input"
                    }
                }]
            }
        ]
    };
}

runCompositorExample(example, DISPLAY_LOGS);
