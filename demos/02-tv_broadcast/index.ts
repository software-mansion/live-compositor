import { registerImage, registerInput, registerOutput, registerShader, start } from "../utils/api";
import { ffmpegSendVideoFromMp4, ffplayListenAsync } from "../utils/ffmpeg";
import { runCompositorExample } from "../utils/run";
import { sleepAsync } from "../utils/utils";
import fs from "fs-extra";
import path from "path";
import { Component, Resolution } from "../types/api";

const OUTPUT_RESOLUTION: Resolution = {
    width: 1920,
    height: 1080,
};

const INPUT_PORT = 8002;
const OUTPUT_PORT = 8004;
const IP = "127.0.0.1";

async function example() {
    // starts ffplay that will listen for streams on port 8002 and display them.
    await ffplayListenAsync(OUTPUT_PORT);

    await registerInput("input_1", {
        type: "rtp_stream",
        port: INPUT_PORT,
        video: {
            codec: "h264"
        }
    });

    await registerShader("shader_1", {
        source: await fs.readFile(path.join(__dirname, "green_screen.wgsl"), "utf-8")
    })

    await registerImage("background", {
        asset_type: "jpeg",
        path: path.join(__dirname, "../assets/news_room.jpg")
    });

    await registerOutput("output_1", {
        type: "rtp_stream",
        ip: IP,
        port: OUTPUT_PORT,
        video: {
            resolution: OUTPUT_RESOLUTION,
            encoder_preset: "ultrafast",
            initial: initialScene()
        }
    });


    await sleepAsync(2000);
    await start();

    ffmpegSendVideoFromMp4(INPUT_PORT, path.join(__dirname, "../assets/green_screen_example.mp4"));
}

function initialScene(): Component {
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

    const shader: Component = {
        type: "shader",
        shader_id: "shader_1",
        children: [rescaledInputStream, rescaledImage],
        resolution: OUTPUT_RESOLUTION
    };

    return {
        type: "view",
        children: [
            rescaledImage,
            {
                type: "view",
                width: OUTPUT_RESOLUTION.width,
                height: OUTPUT_RESOLUTION.height,
                left: 0,
                top: 0,
                children: [
                    shader
                ],
            }
        ],
    };
}

runCompositorExample(example);
