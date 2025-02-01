import Smelter from '@swmansion/smelter-node';
import App from './App';
import { ffplayStartPlayerAsync } from './smelterFfplayHelper';

export const SmelterInstance = new Smelter();

export async function initializeSmelterInstance() {
  await SmelterInstance.init();

  // Display output with `ffplay`.
  await ffplayStartPlayerAsync('127.0.0.0', 8001);

  await SmelterInstance.registerOutput('output_1', <App />, {
    type: 'rtp_stream',
    port: 8001,
    ip: '127.0.0.1',
    transportProtocol: 'udp',
    video: {
      encoder: {
        type: 'ffmpeg_h264',
        preset: 'ultrafast',
      },
      resolution: {
        width: 1920,
        height: 1080,
      },
    },
  });

  await SmelterInstance.start();
}
