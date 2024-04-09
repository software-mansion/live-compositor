import { registerInput, registerOutput, start, updateOutput, } from "../utils/api";
import { ffplayListenAsync } from "../utils/ffmpeg";
import { runCompositorExample } from "../utils/run";
import { gstStreamWebcam } from "../utils/gst";
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
    transport_protocol: "tcp_server",
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
      initial: sceneWithInputs(0)
    }
  });

  const inputs = [1, 2, 3, 4, 5, 6, 7, 8, 9, 8, 7, 6, 5, 4, 3, 2, 1];
  inputs.forEach(async (input, index) =>
    await updateOutput("output_1", {
      video: sceneWithInputs(input),
      schedule_time_ms: 2000 * index
    })
  );

  await sleepAsync(2000);

  await start();
  gstStreamWebcam(IP, INPUT_PORT);
}


function sceneWithInputs(n: number): Component {
  const input_stream: Component = {
    type: "rescaler",
    child: {
      type: "input_stream",
      input_id: "input_1",
    }
  }
  const children: Array<Component> = Array.from({ length: n }, (_, i) => {
    const text: Component = {
      type: "text",
      text: `InputStream ${i} 🚀`,
      font_size: 25,
      align: "center",
      color_rgba: "#FFFFFFFF",
      background_color_rgba: "#007BFFFF",
    };

    return {
      type: "view",
      background_color_rgba: "#007BFFFF",
      children: [
        input_stream,
        {
          type: "view",
          width: 300,
          height: 50,
          left: 0,
          bottom: 0,
          children: [text]
        }
      ]
    };
  });

  return {
    type: "tiles",
    id: "tile",
    padding: 5,
    background_color_rgba: "#444444FF",
    children: children,
    transition: {
      duration_ms: 700,
      easing_function: {
        function_name: "cubic_bezier",
        points: [0.35, 0.22, 0.1, 0.8]
      }
    }
  }
}

runCompositorExample(example);
