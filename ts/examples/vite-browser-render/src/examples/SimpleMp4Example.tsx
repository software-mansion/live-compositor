import { useCallback } from 'react';
import type { LiveCompositor } from '@live-compositor/web-wasm';
import { InputStream, Text, useInputStreams, View } from 'live-compositor';
import CompositorCanvas from '../components/CompositorCanvas';
import comicSans from '../../dist/assets/Comic Sans MS.ttf';

const MP4_URL =
  'https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/ForBiggerEscapes.mp4';

function SimpleMp4Example() {
  const onCanvasCreate = useCallback(async (compositor: LiveCompositor) => {
    await compositor.registerFont(comicSans);

    // await compositor.registerFont('https://fonts.gstatic.com/s/notosans/v36/o-0mIpQlx3QUlC5A4PNB6Ryti20_6n1iPHjcz6L1SoM-jCpoiyD9A-9a6Vc.ttf')
    await compositor.registerInput('video', { type: 'mp4', url: MP4_URL });
  }, []);

  return (
    <div className="card">
      <CompositorCanvas onCanvasCreate={onCanvasCreate} width={1280} height={720}>
        <Scene />
      </CompositorCanvas>
    </div>
  );
}

function Scene() {
  const inputs = useInputStreams();
  const inputState = inputs['video']?.videoState;

  if (inputState === 'playing') {
    return (
      <View style={{ width: 1280, height: 720 }}>
        <InputStream inputId="video" />
        <View style={{ width: 230, height: 40, backgroundColor: '#000000', bottom: 20, left: 500 }}>
          <Text style={{ fontSize: 30, fontFamily: 'Comic Sans MS' }}>Playing MP4 file</Text>
        </View>
      </View>
    );
  }

  if (inputState === 'finished') {
    return (
      <View style={{ backgroundColor: '#000000' }}>
        <View style={{ width: 530, height: 40, bottom: 340, left: 500 }}>
          <Text style={{ fontSize: 30, fontFamily: 'Comic Sans MS' }}>Finished playing MP4 file</Text>
        </View>
      </View>
    );
  }

  return (
    <View style={{ backgroundColor: '#000000' }}>
      <View style={{ width: 530, height: 40, bottom: 340, left: 500 }}>
        <Text style={{ fontSize: 30, fontFamily: 'Comic Sans MS' }}>Loading MP4 file</Text>
      </View>
    </View>
  );
}

export default SimpleMp4Example;
