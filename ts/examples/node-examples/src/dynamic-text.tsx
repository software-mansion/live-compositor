import LiveCompositor from '@live-compositor/node';
import { View, Text } from 'live-compositor';
import { useEffect, useState } from 'react';
import { ffplayStartPlayerAsync } from './utils';

type PartialTextProps = {
  text: string;
  transitionMs: number;
};

function PartialText(props: PartialTextProps) {
  const intervalMs = props.transitionMs / props.text.length;

  const [textPart, updateTextPart] = useState({
    characters: props.text.length,
    shrink: false,
  });

  useEffect(() => {
    const timeout = setTimeout(() => {
      if (textPart.characters === 1 && textPart.shrink) {
        updateTextPart({ characters: 1, shrink: false });
      } else if (textPart.characters === props.text.length && !textPart.shrink) {
        updateTextPart({ characters: props.text.length, shrink: true });
      } else {
        updateTextPart({
          characters: textPart.shrink ? textPart.characters - 1 : textPart.characters + 1,
          shrink: textPart.shrink,
        });
      }
    }, intervalMs);
    return () => {
      clearTimeout(timeout);
    };
  }, [textPart]);

  return (
    <View>
      <Text fontSize={40}>{props.text.substring(0, textPart.characters)}</Text>
    </View>
  );
}

function ExampleApp() {
  return (
    <View direction="column">
      <PartialText text="Example partial text that transition in 1 second" transitionMs={1_000} />
      <PartialText text="Example partial text that transition in 2 second" transitionMs={2_000} />
      <PartialText text="Example partial text that transition in 5 second" transitionMs={5_000} />
    </View>
  );
}

async function run() {
  const compositor = new LiveCompositor();
  await compositor.init();

  await ffplayStartPlayerAsync('127.0.0.1', 8001);

  await compositor.registerOutput('output_1', {
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
      root: <ExampleApp />,
    },
  });

  await compositor.start();
}

void run();
