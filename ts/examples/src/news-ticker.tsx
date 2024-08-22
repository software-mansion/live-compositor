import LiveCompositor from '@live-compositor/node';
import { View, Text } from 'live-compositor';
import { useEffect, useState, useId } from 'react';
import { ffplayStartPlayerAsync, sleep } from './utils';

const EXAMPLE_TEXTS = [
  'Example text',
  'Another example text',
  'Example text',
  'Another example text',
  'Example text',
  'Another example text',
  'Example text',
  'Another example text',
  'Example text',
  'Another example text',
  'Example text',
  'Another example text',
  'Example text',
  'Another example text',
  'Example text',
  'Another example text',
  'Example text',
  'Another example text',
];

const TEXT_CHUNK_SIZE = 400;

type NewsTickerProps = {
  text: string[];
  width: number;
  /** Transition time to move entire width */
  durationMs: number;
};

function NewsTicker(props: NewsTickerProps) {
  const [tickerState, setTickerState] = useState<{
    offset: number;
    lastTextIndex: number;
    chunks: { text: string; id: number }[];
  }>({
    offset: 0,
    lastTextIndex: 0,
    chunks: [],
  });

  useEffect(() => {
    const timeout = setTimeout(() => {
      const offsetForTimeout = (1000 / props.durationMs) * props.width;

      const chunksToRemove = Math.floor(tickerState.offset / TEXT_CHUNK_SIZE);
      const offset = tickerState.offset - chunksToRemove * TEXT_CHUNK_SIZE + offsetForTimeout;

      const chunks = tickerState.chunks.slice(chunksToRemove);

      const chunksToAdd = Math.ceil(
        (props.width * 2 - chunks.length * TEXT_CHUNK_SIZE) / TEXT_CHUNK_SIZE
      );

      let lastTextIndex = tickerState.lastTextIndex;
      for (let i = 0; i < chunksToAdd; i++) {
        chunks.push({
          text: props.text[lastTextIndex % props.text.length],
          id: lastTextIndex,
        });
        lastTextIndex = lastTextIndex + 1;
      }

      setTickerState({
        chunks,
        offset,
        lastTextIndex,
      });
    }, 1000);
    return () => {
      clearInterval(timeout);
    };
  }, [tickerState]);

  return (
    <View width={props.width}>
      {tickerState.chunks.map(({ text, id }, index) => {
        return (
          <NewsTickerItem
            key={id}
            text={text}
            offset={index * TEXT_CHUNK_SIZE - tickerState.offset}
          />
        );
      })}
    </View>
  );
}

function NewsTickerItem(props: { text: string; offset: number }) {
  const id = useId();
  return (
    <View
      id={id}
      width={TEXT_CHUNK_SIZE}
      left={props.offset}
      top={0}
      transition={{
        easingFunction: 'linear',
        // bit longer than how often we are sending updates
        durationMs: 1050,
      }}>
      <Text colorRgba="#FF0000FF" fontSize={30}>
        {props.text}
      </Text>
    </View>
  );
}

function ExampleApp() {
  return (
    <View direction="column">
      <View bottom={0} left={0} height={60}>
        <NewsTicker text={EXAMPLE_TEXTS} width={1920} durationMs={10_000} />
      </View>
    </View>
  );
}

async function run() {
  const compositor = await LiveCompositor.create();

  ffplayStartPlayerAsync('127.0.0.1', 8001);
  await sleep(2000);

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
run();
