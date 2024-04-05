import { sendAsync } from "../utils/api";
import { ffplayListenAsync } from "../utils/ffmpeg";
import { runCompositorExample } from "../utils/run";
import { Component, Resolution } from "../types/types";
import { gstStreamWebcam } from "../utils/gst";
import { sleepAsync } from "../utils/utils";

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

  await sendAsync({
    type: "register",
    entity_type: "rtp_input_stream",
    transport_protocol: "tcp_server",
    input_id: "input_1",
    port: INPUT_PORT,
    video: {
      codec: "h264"
    }
  });

  await sendAsync({
    "type": "register",
    "entity_type": "output_stream",
    "output_id": "output_1",
    "ip": IP,
    "port": OUTPUT_PORT,
    "video": {
      "resolution": OUTPUT_RESOLUTION,
      "encoder_preset": "ultrafast",
      "initial": sceneWithInputs(0)
    }
  });

  
  const inputs = [1, 2, 3, 4, 5, 6, 7, 8, 9, 8, 7, 6, 5, 4, 3, 2, 1];
  inputs.forEach(async (input, index) =>
    await sendAsync({
      type: "update_output",
      output_id: "output_1",
      video: sceneWithInputs(input),
      schedule_time_ms: 2000 * index
    })
  );

  await sendAsync({ type: "start" });
  
  await sleepAsync(2000);
  gstStreamWebcam(IP, INPUT_PORT);
}


function sceneWithInputs(n: number): Component {
  let children: Array<Component> = Array.from({ length: n }, () => {
    return { type: "input_stream", input_id: "input_1" }
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
