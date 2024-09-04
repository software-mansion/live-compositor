import LiveCompositor from '@live-compositor/node';
import { Text, InputStream, Tiles, Rescaler, View, useInputStreams } from 'live-compositor';
import { downloadAllAssets, gstReceiveTcpStream, sleep } from './utils';
import path from 'path';
import fs from 'fs-extra';

function ExampleApp() {
  const inputs = useInputStreams();
  return (
    <Tiles transition={{ durationMs: 200 }}>
      {Object.values(inputs).map(input => (
        <InputTile key={input.inputId} inputId={input.inputId} />
      ))}
    </Tiles>
  );
}

function InputTile({ inputId }: { inputId: string }) {
  return (
    <View>
      <Rescaler>
        <InputStream inputId={inputId} />
      </Rescaler>
      <View bottom={10} left={10} height={40}>
        <Text fontSize={40}>Input ID: {inputId}</Text>
      </View>
    </View>
  );
}

async function run() {
  await fs.mkdirp(path.join(__dirname, '../.workingdir'));
  await downloadAllAssets();
  const compositor = await LiveCompositor.create();

  const RESOLUTION = {
    width: 1920,
    height: 1080,
  } as const;
  const VIDEO_ENCODER_OPTS = {
    type: 'ffmpeg_h264',
    preset: 'ultrafast',
  } as const;

  await compositor.registerOutput('output_stream', {
    type: 'rtp_stream',
    port: 8001,
    transportProtocol: 'tcp_server',
    video: {
      encoder: VIDEO_ENCODER_OPTS,
      resolution: RESOLUTION,
      root: <ExampleApp />,
    },
    audio: {
      encoder: {
        type: 'opus',
        channels: 'stereo',
      },
    },
  });
  gstReceiveTcpStream('127.0.0.1', 8001);
  await compositor.registerOutput('output_recording', {
    type: 'mp4',
    serverPath: path.join(__dirname, '../.workingdir/dynamic_outputs_recording.mp4'),
    video: {
      encoder: VIDEO_ENCODER_OPTS,
      resolution: RESOLUTION,
      root: <ExampleApp />,
    },
    audio: {
      encoder: {
        type: 'aac',
        channels: 'stereo',
      },
    },
  });

  await compositor.registerInput('input_1', {
    type: 'mp4',
    serverPath: path.join(__dirname, '../.assets/BigBuckBunny.mp4'),
  });
  console.log(
    'Start LiveCompositor pipeline with single input ("input_1") and two outputs (RTP "output_stream" and MP4 "output_recording").'
  );
  await compositor.start();

  await sleep(10_000);
  console.log('Connect new input ("input_2") and start new output to MP4 "output_recording_part2"');
  await compositor.registerInput('input_2', {
    type: 'mp4',
    serverPath: path.join(__dirname, '../.assets/ElephantsDream.mp4'),
  });
  await compositor.registerOutput('output_recording_part2', {
    type: 'mp4',
    serverPath: path.join(__dirname, '../.workingdir/dynamic_outputs_recording_10s.mp4'),
    video: {
      encoder: VIDEO_ENCODER_OPTS,
      resolution: RESOLUTION,
      root: <ExampleApp />,
    },
    audio: {
      encoder: {
        type: 'aac',
        channels: 'stereo',
      },
    },
  });

  await sleep(10_000);
  console.log('Stop output "output_recording"');
  await compositor.unregisterOutput('output_recording');

  await sleep(10_000);
  console.log('Stop all remaining outputs.');
  await compositor.unregisterOutput('output_recording_part2');
  await compositor.unregisterOutput('output_stream');
}
run();
