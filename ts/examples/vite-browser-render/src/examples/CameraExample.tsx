import { useCallback } from 'react';
import type { LiveCompositor } from '@live-compositor/web-wasm';
import { InputStream, Rescaler, Text, View } from 'live-compositor';
import CompositorCanvas from '../components/CompositorCanvas';
import NotoSansFont from '../../assets/NotoSans.ttf';

function ScreenCapture() {
  const onCanvasCreate = useCallback(async (compositor: LiveCompositor) => {
    await compositor.registerFont(NotoSansFont);
    try {
      await compositor.registerInput('camera', { type: 'camera' });
    } catch (err: any) {
      console.warn('Failed to register camera input', err);
    }
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
  return (
    <View>
      <Rescaler>
        <InputStream inputId="camera" />
      </Rescaler>
      <View style={{ width: 300, height: 40, backgroundColor: '#000000', bottom: 20, left: 520 }}>
        <Text style={{ fontSize: 30, fontFamily: 'Noto Sans' }}>Camera input</Text>
      </View>
    </View>
  );
}

export default ScreenCapture;
