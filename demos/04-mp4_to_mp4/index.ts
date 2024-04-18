import { registerInput, registerOutput, start } from "../utils/api";
import { runCompositorExample } from "../utils/run";
import { sleepAsync } from "../utils/utils";
import { Component, Resolution } from "../types/api";
import { gstStartPlayer } from "../utils/gst";

const OUTPUT_RESOLUTION: Resolution = {
    width: 1920,
    height: 1080,
};

const OUTPUT_PORT = 8000;
const IP = "127.0.0.1";

async function example() {
    await registerInput("bunny", {
        type: "mp4",
        url: "https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4"
    });

    // This is mock, since recording screen requires too many permissions and it's heavily OS specific.
    await registerInput("elephant", {
        type: "mp4",
        url: "https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/ElephantsDream.mp4"
    });

    await registerOutput("output_1", {
        type: "rtp_stream",
        transport_protocol: "tcp_server",
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
        },
        audio: {
            encoder: {
                type: "opus",
                channels: "stereo"
            },
            initial: {
                inputs: [{ input_id: "bunny" }, { input_id: "elephant", volume: 0.5 }]
            }
        }
    });


    await sleepAsync(2000);
    await start();
    gstStartPlayer(IP, OUTPUT_PORT);
}

function baseScene(): Component {
    return {
        type: "view",
        children: [
            {
                type: "view",
                children: [
                    {
                        type: "rescaler",
                        child: {
                            type: "input_stream",
                            input_id: "bunny"
                        }
                    }
                ]
            },
            {
                type: "view",
                children: [
                    {
                        type: "rescaler",
                        child: {
                            type: "input_stream",
                            input_id: "elephant"
                        }
                    }
                ]
            }
        ]
    };
}

runCompositorExample(example);
