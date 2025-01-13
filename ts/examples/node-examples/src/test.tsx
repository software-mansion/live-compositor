import LiveCompositor from '@live-compositor/node';
import { View, Text, InputStream, Rescaler, Image } from 'live-compositor';
import { ffplayStartPlayerAsync, sleep } from './utils';
import path from 'path';

function RnckStream() {
  return (
    <View>
      <View style={{ top: 0, left: 0 }}>
        <Image imageId="image_1" />
      </View>
      <View style={{ bottom: 70, right: 110, width: 280, height: 58 }}>
        <Text style={{ align: 'center', fontFamily: 'Futura', fontWeight: 'bold', fontSize: 32 }}>
          RNCK #14
        </Text>
      </View>
      <View style={{ bottom: 10, left: 10, width: 318, height: 357, borderRadius: 12 }}>
        <Rescaler>
          <InputStream inputId="input_1" />
        </Rescaler>
      </View>
      <View style={{ direction: 'column', bottom: 10, left: 338, height: 165 }}>
        <Text
          style={{
            align: 'center',
            fontFamily: 'DMSans',
            fontWeight: 'medium',
            fontSize: 32,
            backgroundColor: '#33539d',
            lineHeight: 15,
          }}>
          {`KACPER KAPUSCIAK\ntest123`}
        </Text>

        <View style={{ height: 10 }} />
        <Text
          style={{
            align: 'center',
            fontFamily: 'Futura',
            fontWeight: 'bold',
            fontSize: 32,
            backgroundColor: '#62b3de',
          }}>
          What's new in React Native
        </Text>
      </View>
      <View
        style={{
          top: 10,
          right: 10,
          width: 1570,
          height: 884,
          borderRadius: 12,
        }}>
        <Rescaler>
          <InputStream inputId="input_2" />
        </Rescaler>
      </View>
    </View>
  );
}

async function run() {
  const compositor = new LiveCompositor();
  await compositor.init();

  await ffplayStartPlayerAsync('127.0.0.1', 8001);
  await sleep(2000);

  await compositor.registerImage('image_1', {
    assetType: 'png',
    serverPath: path.join(__dirname, '../.assets/rnck_background.png'),
  });

  await compositor.registerOutput('output_1', <RnckStream />, {
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

  await compositor.registerInput('input_1', {
    type: 'mp4',
    serverPath: path.join(__dirname, '../.assets/speaker.mp4'),
    loop: true,
  });

  await compositor.registerInput('input_2', {
    type: 'mp4',
    serverPath: path.join(__dirname, '../.assets/rnck.mp4'),
    loop: true,
  });

  await compositor.start();
}

void run();
