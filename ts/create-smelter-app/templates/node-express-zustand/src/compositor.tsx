import LiveCompositor from '@live-compositor/node';
import App from './App';
import { ffplayStartPlayerAsync } from './liveCompositorFfplayHelper';

export const Compositor = new LiveCompositor();

export async function initializeCompositor() {
  await Compositor.init();

  // Display output with `ffplay`.
  await ffplayStartPlayerAsync('127.0.0.0', 8001);

  await Compositor.registerOutput('output_1', <App />, {
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

  await Compositor.start();
}
