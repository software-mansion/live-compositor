import LiveCompositor from '@live-compositor/node';
import { useInputStreams, Text, InputStream, Tiles, Rescaler, View } from 'live-compositor';
import { useId } from 'react';
import express from 'express';

function ExampleApp() {
  const inputs = useInputStreams();
  const id = useId();
  return (
    <Tiles id={id} transition={{ durationMs: 2000 }}>
      {inputs
        .filter(input => input.videoState === 'playing')
        .map(input => (
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

let compositor: LiveCompositor;
const server = express();

server.get('/add-participant', async (req, res) => {
  await compositor.registerInput(req.body.inputId, {
    type: 'rtp_stream',
    transportProtocol: 'tcp_server',
    port: req.body.port,
    video: {
      decoder: 'ffmpeg_h264',
    },
  });
  // Update some state managment e.g. redux store to e.g. specify data needed for scene
  // e.g. for video call this could be text label for the participant
  res.send({ status: 'ok' });
});

async function start() {
  const compositor = await LiveCompositor.create();
  await compositor.registerOutput('output_1', {
    type: 'rtp_stream',
    port: 8001,
    transportProtocol: 'tcp_server',
    video: {
      encoder: {
        type: 'ffmpeg_h264',
        preset: 'ultrafast',
      },
      resolution: {
        width: 1920,
        height: 1080,
      },
      root: <ExampleApp />,
    },
  });
  // For RTP you will need to notify here a destination server to start receiving the stream.
  await compositor.start();
  server.listen(3000);
}
start();
