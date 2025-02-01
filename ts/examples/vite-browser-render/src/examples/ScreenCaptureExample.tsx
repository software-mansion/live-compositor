import { useCallback } from 'react';
import type { LiveCompositor } from '@live-compositor/web-wasm';
import { InputStream, Rescaler, Text, View } from 'live-compositor';
import CompositorCanvas from '../components/CompositorCanvas';

function ScreenCaptureExample() {
  const onCanvasCreate = useCallback(async (compositor: LiveCompositor) => {
    await compositor.registerFont(
      'https://fonts.gstatic.com/s/notosans/v36/o-0mIpQlx3QUlC5A4PNB6Ryti20_6n1iPHjcz6L1SoM-jCpoiyD9A-9a6Vc.ttf'
    );
    try {
      await compositor.registerInput('screen', { type: 'screen_capture' });
    } catch (err: any) {
      console.warn('Failed to register screen capture input', err);
    }
  }, []);

  return (
    <div className="card">
      <CompositorCanvas onCanvasStarted={onCanvasCreate} width={1280} height={720}>
        <Scene />
      </CompositorCanvas>
    </div>
  );
}

function Scene() {
  return (
    <View>
      <Rescaler>
        <InputStream inputId="screen" />
      </Rescaler>
      <View style={{ width: 400, height: 40, backgroundColor: '#000000', bottom: 20, left: 520 }}>
        <Text style={{ fontSize: 30, fontFamily: 'Noto Sans' }}>Screen capture example</Text>
      </View>
    </View>
  );
}

export default ScreenCaptureExample;
