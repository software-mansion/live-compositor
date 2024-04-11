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

    await registerImage("logo", {
        asset_type: "png",
        url: "https://i.ibb.co/vHtG8Rr/logo.png"
    })

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

    const reportWithBackground: Component = {
        type: "shader",
        shader_id: "shader_1",
        children: [rescaledInputStream, rescaledImage],
        resolution: OUTPUT_RESOLUTION
    }

    return {
        type: "view",
        width: OUTPUT_RESOLUTION.width,
        height: OUTPUT_RESOLUTION.height,
        children: [
            reportWithBackground,
            breakingNewsText(),
            logo()
        ]
    };
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
        width: 160,
        height: 90,
        top: 50,
        left: 50,
        overflow: "fit",
        children: [{
            type: "image",
            image_id: "logo",
        }]
    };
}

runCompositorExample(example);
