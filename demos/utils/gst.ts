import { exec } from 'child_process';
import { SpawnPromise, spawn } from './utils';

export function gstStartPlayer(ip: string, port: number, displayOutput: boolean): SpawnPromise {
  const gstCommand =
    `gst-launch-1.0 -v ` +
    `rtpptdemux name=demux ` +
    `tcpclientsrc host=${ip} port=${port} ! "application/x-rtp-stream" ! rtpstreamdepay ! demux. ` +
    `demux.src_96 ! "application/x-rtp,media=video,clock-rate=90000,encoding-name=H264" ! queue ! rtph264depay ! decodebin ! videoconvert ! autovideosink ` +
    `demux.src_97 ! "application/x-rtp,media=audio,clock-rate=48000,encoding-name=OPUS" ! queue ! rtpopusdepay ! decodebin ! audioconvert ! autoaudiosink `;

  return spawn('bash', ['-c', gstCommand], { displayOutput });
}

export function gstStreamWebcam(ip: string, port: number, displayOutput: boolean): SpawnPromise {
  const isMacOS = process.platform === 'darwin';

  const [gstWebcamSource, gstEncoder, gstEncoderOptions] = isMacOS
    ? ['avfvideosrc', 'vtenc_h264', 'bitrate=2000']
    : ['v4l2src', 'x264enc', 'tune=zerolatency bitrate=2000 speed-preset=superfast'];

  const plugins = [gstWebcamSource, 'videoconvert', gstEncoder, 'rtph264pay', 'udpsink'];
  checkGstPlugins(plugins);

  const gstCommand =
    `gst-launch-1.0 -v ` +
    `${gstWebcamSource} ! videoconvert ! ${gstEncoder} ${gstEncoderOptions} ! rtph264pay config-interval=1 pt=96 ! rtpstreampay ! queue ! tcpclientsink host=${ip} port=${port}`;

  return spawn('bash', ['-c', gstCommand], { displayOutput });
}

function checkGstPlugins(plugins: string[]) {
  plugins.forEach(plugin => {
    void isGstPluginAvailable(plugin).then(isAvailable => {
      if (!isAvailable) {
        throw Error(`Gstreamer plugin: ${plugin} is not available.`);
      }
    });
  });
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
