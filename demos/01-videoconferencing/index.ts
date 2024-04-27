import { runCompositorExample } from "../utils/run";
import { gstStreamWebcam } from "../utils/gst";
import { downloadAsync, sleepAsync } from "../utils/utils";
import { Component, Resolution } from "../types/api";
import { ffmpegSendVideoFromMp4, ffplayStartPlayerAsync } from "../utils/ffmpeg";
import path from "path";
import { registerImageAsync, registerInputAsync, registerOutputAsync, startAsync, updateOutputAsync } from "../utils/api";

const OUTPUT_RESOLUTION: Resolution = {
    width: 1920,
    height: 1080,
};

const INPUT_PORT = 8000;
const OUTPUT_PORT = 8002;
const IP = "127.0.0.1";
const DISPLAY_LOGS = true;

async function example() {
    const useWebCam = process.env.LIVE_COMPOSITOR_WEBCAM !== "false";
    await ffplayStartPlayerAsync(IP, DISPLAY_LOGS, OUTPUT_PORT);

    // sleep to make sure ffplay have a chance to start before compositor starts sending packets
    await sleepAsync(2000);

    await registerImageAsync("background", {
        asset_type: "png",
        // url: "https://raw.githubusercontent.com/membraneframework-labs/video_compositor_snapshot_tests/main/demo_assets/triangles_background.png"
        path: path.join(__dirname, "../assets/triangles_background.png")
    })

    await registerInputAsync("input_1", {
        type: "rtp_stream",
        transport_protocol: useWebCam ? "tcp_server" : "udp",
        port: INPUT_PORT,
        video: {
            decoder: "ffmpeg_h264"
        }
    });

    await registerOutputAsync("output_1", {
        type: "rtp_stream",
        ip: IP,
        port: OUTPUT_PORT,
        video: {
            resolution: OUTPUT_RESOLUTION,
            encoder: {
                type: "ffmpeg_h264",
                preset: "medium"
            },
            initial: {
                root: sceneWithInputs(1)
            }
        }
    });

    if (useWebCam) {
        gstStreamWebcam(IP, INPUT_PORT, DISPLAY_LOGS);
    } else {
        const callPath = path.join(__dirname, "../assets/call.mp4");
        await downloadAsync("https://raw.githubusercontent.com/membraneframework-labs/video_compositor_snapshot_tests/main/demo_assets/call.mp4", path.join(__dirname, "../assets/call.mp4"));
        ffmpegSendVideoFromMp4(INPUT_PORT, callPath, DISPLAY_LOGS);
    }
    await startAsync();

    const inputs = [2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 13, 12, 11, 10., 9, 8, 7, 6, 5, 4, 3, 2, 1];
    inputs.forEach(async (input, index) => {
        await sleepAsync(2000 * index);
        await updateOutputAsync("output_1", {
            video: {
                root: sceneWithInputs(input)
            },
        })
    });
}


function sceneWithInputs(n: number): Component {
    const children: Array<Component> = Array.from({ length: n }, (_, i) => {
        const emptyView: Component = { type: "view" }
        const text: Component = {
            type: "text",
            text: `InputStream ${i} 🚀`,
            font_size: 25,
            align: "center",
            color_rgba: "#FFFFFFFF",
            background_color_rgba: "#FF0000FF",
            font_family: "Arial",
        };

        const inputStreamTile: Component = {
            type: "view",
            children: [
                {
                    type: "rescaler",
                    child: {
                        type: "input_stream",
                        input_id: "input_1"
                    }
                },
                {
                    type: "view",
                    height: 50,
                    bottom: 0,
                    left: 0,
                    children: [emptyView, text, emptyView]
                }
            ]
        };

        return inputStreamTile;
    });

    const tiles: Component = {
        type: "tiles",
        id: "tile",
        padding: 5,
        children: children,
        transition: {
            duration_ms: 700,
            easing_function: {
                function_name: "cubic_bezier",
                points: [0.35, 0.22, 0.1, 0.8]
            }
        }
    };

    const background: Component = {
        type: "image",
        image_id: "background"
    };

    return {
        type: "view",
        width: OUTPUT_RESOLUTION.width,
        height: OUTPUT_RESOLUTION.height,
        children: [
            {
                type: "view",
                width: OUTPUT_RESOLUTION.width,
                height: OUTPUT_RESOLUTION.height,
                top: 0,
                left: 0,
                children: [background]
            },
            {
                type: "view",
                width: OUTPUT_RESOLUTION.width,
                height: OUTPUT_RESOLUTION.height,
                top: 0,
                left: 0,
                children: [tiles]
            }
        ]
    }
}

runCompositorExample(example, DISPLAY_LOGS);
