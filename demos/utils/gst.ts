import { exec } from 'child_process';
import { spawn } from './utils';

export function gstStartPlayer(port: number) {
  const gstCommand =
    `gst-launch-1.0 -v ` +
    `rtpptdemux name=demux ` +
    `tcpclientsrc host=127.0.0.1 port=${port} ! "application/x-rtp-stream" ! rtpstreamdepay ! queue ! demux. ` +
    `demux.src_96 ! "application/x-rtp,media=video,clock-rate=90000,encoding-name=H264" ! queue ! rtph264depay ! decodebin ! videoconvert ! autovideosink ` +
    `demux.src_97 ! "application/x-rtp,media=audio,clock-rate=48000,encoding-name=OPUS" ! queue ! rtpopusdepay ! decodebin ! audioconvert ! autoaudiosink `;

  void spawn('bash', ['-c', gstCommand], {});
}

export async function gstStartWebcamStream(port: number): Promise<void> {
  const isMacOS = process.platform === 'darwin';

  const [gstWebcamSource, gstEncoder, gstEncoderOptions] = isMacOS
    ? ['avfvideosrc', 'vtenc_h264', 'bitrate=2000']
    : ['v4l2src', 'x264enc', 'tune=zerolatency bitrate=2000 speed-preset=superfast'];

  const plugins = [gstWebcamSource, 'videoconvert', gstEncoder, 'rtph264pay', 'udpsink'];
  await checkGstPlugins(plugins);

  const gstCommand =
    `gst-launch-1.0 -v ` +
    `${gstWebcamSource} ! videoconvert ! ${gstEncoder} ${gstEncoderOptions} ! rtph264pay config-interval=1 pt=96 ! rtpstreampay ! queue ! tcpclientsink host=127.0.0.1 port=${port}`;

  void spawn('bash', ['-c', gstCommand], {});
}

async function checkGstPlugins(plugins: string[]) {
  await Promise.all(
    plugins.map(async plugin => {
      const isAvailable = await isGstPluginAvailable(plugin);
      if (!isAvailable) {
        throw Error(`Gstreamer plugin: ${plugin} is not available.`);
      }
    })
  );
}

function isGstPluginAvailable(pluginName: string): Promise<boolean> {
  const command = `gst-inspect-1.0 ${pluginName}`;
  return new Promise(resolve => {
    exec(command, error => {
      if (error) {
        resolve(false);
      } else {
        resolve(true);
      }
    });
  });
}
