import { registerInput, registerOutput, start } from "../utils/api";
import { ffmpegStreamScreen, ffplayListenAsync } from "../utils/ffmpeg";
import { runCompositorExample } from "../utils/run";
import { sleepAsync } from "../utils/utils";
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

    ffmpegStreamScreen(IP, INPUT_PORT);
}

function initialScene(): Component {
    return {
        type: "input_stream",
        input_id: "input_1"
    }
}

runCompositorExample(example);
