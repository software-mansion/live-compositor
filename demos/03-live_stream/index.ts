import { registerInput, registerOutput, start, updateOutput } from "../utils/api";
import { ffplayListenVideoAsync } from "../utils/ffmpeg";
import { runCompositorExample } from "../utils/run";
import { sleepAsync } from "../utils/utils";
import { Component, Resolution } from "../types/api";
import { gstStreamWebcam } from "../utils/gst";

const OUTPUT_RESOLUTION: Resolution = {
    width: 1920,
    height: 1080,
};

const WEBCAM_INPUT_PORT = 8002;
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

    // This is mock, since recording screen requires too many permissions and it's heavily OS specific.
    await registerInput("screen_input", {
        type: "mp4",
        url: "https://d3cgycspz6gssr.cloudfront.net/b1eej1%2Ffile%2Fc3c04e4e6e7aadc26b80e06008e397ca_a5ce5f1975f904e035b2de01e0952d6f.mp4?response-content-disposition=inline%3Bfilename%3D%22c3c04e4e6e7aadc26b80e06008e397ca_a5ce5f1975f904e035b2de01e0952d6f.mp4%22%3B&response-content-type=video%2Fmp4&Expires=1713301809&Signature=I-Vvr5YOOfCp2vjyuVIac~IumIrDgouFUxie03fFHBufbhGH84n0-vE8KuKl0YlKHlWhVkqS6Hw2J3HZvKDQzwJlVAB0bEXTcgl0BWtqnnL8yNE2kIn0cvN4JDtcM2Gs95jpjRAGNnvy7lG5IMQFcanm~atEmj238EqEfJczx1uYsuJqJQX9n7pQPWuoI5ovO9WY5KgvLLeohF~tR0YDrFFAlk7xEtT~ZoImqkieGg4LPcvT7C4tQYGeEY4d64Z1rRaysrJMxUkgzR~q9kwQD2zHnTjHkqTrnmXb1iU69yx71wQXAIYuJm21usrhkJdO-4R4Q9JKBM3M~ZxFTgfiyg__&Key-Pair-Id=APKAJT5WQLLEOADKLHBQ"
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

    gstStreamWebcam(IP, WEBCAM_INPUT_PORT);
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
        top: 0,
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

runCompositorExample(example);
