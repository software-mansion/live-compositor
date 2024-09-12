import path from 'path';
import * as readline from 'readline';

import { ffmpegSendVideoFromMp4, ffplayStartPlayerAsync } from '../utils/ffmpeg';
import { runCompositorExample } from '../utils/run';
import { downloadAsync, sleepAsync } from '../utils/utils';
import { Component, Resolution } from '../types/api.d';
import { gstStreamWebcam } from '../utils/gst';
import {
  registerImageAsync,
  registerInputAsync,
  registerOutputAsync,
  startAsync,
  updateOutputAsync,
} from '../utils/api';

const OUTPUT_RESOLUTION: Resolution = {
  width: 1920,
  height: 1080,
};

const WEBCAM_INPUT_PORT = 10000;
const GAMEPLAY_PORT = 10002;
const VIDEO_OUTPUT_PORT = 10004;
const AUDIO_OUTPUT_PORT = 10006;
const IP = '127.0.0.1';
const DISPLAY_LOGS = false;

const GAMEPLAY_URL =
  'https://raw.githubusercontent.com/membraneframework-labs/video_compositor_snapshot_tests/main/demo_assets/gameplay.mp4';
const DONATE_URL =
  'https://raw.githubusercontent.com/membraneframework-labs/video_compositor_snapshot_tests/main/demo_assets/donate.gif';
const CALL_URL =
  'https://raw.githubusercontent.com/membraneframework-labs/video_compositor_snapshot_tests/main/demo_assets/call.mp4';

async function exampleAsync() {
  await ffplayStartPlayerAsync(IP, DISPLAY_LOGS, VIDEO_OUTPUT_PORT, AUDIO_OUTPUT_PORT);

  // sleep to make sure ffplay have a chance to start before compositor starts sending packets
  await sleepAsync(2000);

  const gameplayPath = path.join(__dirname, '../assets/gameplay.mp4');
  await downloadAsync(GAMEPLAY_URL, gameplayPath);

  await registerImageAsync('donate', {
    asset_type: 'gif',
    url: DONATE_URL,
  });

  const useWebCam = process.env.LIVE_COMPOSITOR_WEBCAM !== 'false';
  await registerInputAsync('webcam_input', {
    type: 'rtp_stream',
    port: WEBCAM_INPUT_PORT,
    transport_protocol: useWebCam ? 'tcp_server' : 'udp',
    video: {
      decoder: 'ffmpeg_h264',
    },
  });

  await registerInputAsync('gameplay', {
    type: 'rtp_stream',
    port: GAMEPLAY_PORT,
    video: {
      decoder: 'ffmpeg_h264',
    },
    audio: {
      decoder: 'opus',
    },
  });

  await registerOutputAsync('video_output', {
    type: 'rtp_stream',
    ip: IP,
    port: VIDEO_OUTPUT_PORT,
    video: {
      resolution: OUTPUT_RESOLUTION,
      encoder: {
        type: 'ffmpeg_h264',
        preset: 'ultrafast',
      },
      initial: {
        root: baseScene(),
      },
    },
  });

  await registerOutputAsync('audio_output', {
    type: 'rtp_stream',
    ip: IP,
    port: AUDIO_OUTPUT_PORT,
    audio: {
      encoder: {
        channels: 'stereo',
        type: 'opus',
      },
      initial: {
        inputs: [{ input_id: 'gameplay' }],
      },
    },
  });

  if (useWebCam) {
    void gstStreamWebcam(IP, WEBCAM_INPUT_PORT, DISPLAY_LOGS);
  } else {
    const callPath = path.join(__dirname, '../assets/call.mp4');
    await downloadAsync(CALL_URL, callPath);
    void ffmpegSendVideoFromMp4(WEBCAM_INPUT_PORT, callPath, DISPLAY_LOGS);
  }
  void ffmpegSendVideoFromMp4(GAMEPLAY_PORT, gameplayPath, DISPLAY_LOGS);

  await sleepAsync(2000);
  await startAsync();

  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
  });

  rl.question('Enter donate content: ', async donate_content => {
    console.log(`Donate content: ${donate_content}`);
    await displayDonateAsync(donate_content);
  });
}

async function displayDonateAsync(msg: string) {
  await updateOutputAsync('video_output', {
    video: {
      root: {
        type: 'view',
        children: [baseScene(), donateCard(msg, 'start')],
      },
    },
  });
  await updateOutputAsync('video_output', {
    video: {
      root: {
        type: 'view',
        children: [baseScene(), donateCard(msg, 'middle')],
      },
    },
  });
  await sleepAsync(4500);
  await updateOutputAsync('video_output', {
    video: {
      root: {
        type: 'view',
        children: [baseScene(), donateCard(msg, 'end')],
      },
    },
  });
}

function donateCard(msg: string, stage: 'start' | 'middle' | 'end'): Component {
  const width = 480;
  let top;
  if (stage === 'start' || stage === 'end') {
    top = -270;
  } else if (stage === 'middle') {
    top = 30;
  }

  return {
    type: 'view',
    id: 'donate_view',
    width,
    height: 270,
    top,
    left: OUTPUT_RESOLUTION.width / 2 - width / 2,
    direction: 'column',
    children: [
      {
        type: 'view',
        top: 0,
        left: 0,
        children: [
          {
            type: 'image',
            image_id: 'donate',
          },
        ],
      },
      {
        type: 'text',
        width,
        text: msg,
        weight: 'extra_bold',
        font_size: 50,
        align: 'center',
        color_rgba: '#FF0000FF',
        font_family: 'Comic Sans MS',
      },
    ],
    transition: {
      duration_ms: 1000,
      easing_function: {
        function_name: 'bounce',
      },
    },
  };
}

function baseScene(): Component {
  return {
    type: 'view',
    children: [
      {
        type: 'rescaler',
        child: {
          type: 'input_stream',
          input_id: 'gameplay',
        },
      },
      {
        type: 'view',
        width: 500,
        height: 300,
        top: 30,
        right: 30,
        children: [
          {
            type: 'rescaler',
            vertical_align: 'top',
            horizontal_align: 'right',
            child: {
              type: 'input_stream',
              input_id: 'webcam_input',
            },
          },
        ],
      },
    ],
  };
}

void runCompositorExample(exampleAsync, DISPLAY_LOGS);
