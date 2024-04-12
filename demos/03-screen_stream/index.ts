import { registerInput, registerOutput, start } from "../utils/api";
import { ffmpegSendTestPattern, ffplayListenVideoAsync } from "../utils/ffmpeg";
import { runCompositorExample } from "../utils/run";
import { sleepAsync } from "../utils/utils";
import { Component, Resolution } from "../types/api";
import { gstStreamWebcam } from "../utils/gst";

const OUTPUT_RESOLUTION: Resolution = {
    width: 1920,
    height: 1080,
};

const WEBCAM_INPUT_PORT = 8002;
const SCREEN_CAPTURE_INPUT_PORT = 8004;
const OUTPUT_PORT = 8006;
const IP = "127.0.0.1";

async function example() {
    // starts ffplay that will listen for streams on port 8002 and display them.
    await ffplayListenVideoAsync(IP, OUTPUT_PORT);

    await registerInput("webcam_input", {
        type: "rtp_stream",
        port: WEBCAM_INPUT_PORT,
        transport_protocol: "tcp_server",
        video: {
            decoder: "ffmpeg_h264"
        }
    });

    await registerInput("screen_input", {
        type: "rtp_stream",
        port: SCREEN_CAPTURE_INPUT_PORT,
        video: {
            decoder: "ffmpeg_h264"
        }
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
                root: scene()
            }
        }
    });


    await sleepAsync(2000);
    await start();

    // TODO: replace with actual streams
    // ffmpegSendTestPattern(WEBCAM_INPUT_PORT, OUTPUT_RESOLUTION);
    gstStreamWebcam(IP, WEBCAM_INPUT_PORT);
    ffmpegSendTestPattern(SCREEN_CAPTURE_INPUT_PORT, OUTPUT_RESOLUTION);
}

function scene(): Component {
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

runCompositorExample(example);
