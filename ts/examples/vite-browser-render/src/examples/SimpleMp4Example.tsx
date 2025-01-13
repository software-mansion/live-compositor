import { InputStream, Text, useInputStreams, View } from 'live-compositor';
import TestWorker from './worker?worker';
import { useEffect, useRef, useState } from 'react';
import { WHIPClient } from '@eyevinn/whip-web-client';

const MP4_URL =
  'https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/ForBiggerEscapes.mp4';

let initialized = false;
async function startWorker(canvasRef: HTMLCanvasElement) {
  if (initialized) {
    return;
  }
  initialized = true;
  const worker = new TestWorker();
  await new Promise<void>(res => {
    setTimeout(() => {
      res();
    }, 1000);
  });
  const stream = canvasRef.captureStream(30);
  const offscreen = canvasRef.transferControlToOffscreen();
  worker.postMessage({ canvas: offscreen }, [offscreen]);
  worker.onmessage = async msg => {
    console.log('from worker', msg);
    const client = new WHIPClient({
      endpoint: 'https://g.webrtc.live-video.net:4443/v2/offer',
      opts: {
        debug: true,
        iceServers: [{ urls: 'stun:stun.l.google.com:19320' }],
        authkey: '',
        noTrickleIce: true,
      },
    });
    await client.setIceServersFromEndpoint();

    const mediaStream = await navigator.mediaDevices.getUserMedia({
      audio: true,
    });

    var options = {};
    var recordedBlobs = [];
    var newStream = new MediaStream();
    newStream.addTrack(mediaStream.getAudioTracks()[0]);
    newStream.addTrack(stream.getVideoTracks()[0]);
    const mediaRecorder = new MediaRecorder(newStream, options);
    mediaRecorder.ondataavailable = function (event) {
      if (event.data && event.data.size > 0) {
        recordedBlobs.push(event.data);
      }
    };
    mediaRecorder.start(1000);

    await client.ingest(mediaRecorder.stream);
  };
}

function SimpleMp4Example() {
  const canvasRef = useRef();
  const [started, setStarted] = useState(false);
  useEffect(() => {
    if (canvasRef.current) {
      void (async () => {
        await startWorker(canvasRef.current!);
        setStarted(true);
      })();
    }
  }, []);

  return (
    <div className="card">
      <p> {started ? 'started' : 'waiting'}</p>
      <canvas ref={canvasRef} width={1280} height={720} />
    </div>
  );
}

function Scene() {
  const inputs = useInputStreams();
  const inputState = inputs['bunny_video']?.videoState;

  if (inputState === 'playing') {
    return (
      <View style={{ width: 1280, height: 720 }}>
        <InputStream inputId="bunny_video" />
        <View style={{ width: 230, height: 40, backgroundColor: '#000000', bottom: 20, left: 500 }}>
          <Text style={{ fontSize: 30, fontFamily: 'Noto Sans' }}>Playing MP4 file</Text>
        </View>
      </View>
    );
  }

  if (inputState === 'finished') {
    return (
      <View style={{ backgroundColor: '#000000' }}>
        <View style={{ width: 530, height: 40, bottom: 340, left: 500 }}>
          <Text style={{ fontSize: 30, fontFamily: 'Noto Sans' }}>Finished playing MP4 file</Text>
        </View>
      </View>
    );
  }

  return (
    <View style={{ backgroundColor: '#000000' }}>
      <View style={{ width: 530, height: 40, bottom: 340, left: 500 }}>
        <Text style={{ fontSize: 30, fontFamily: 'Noto Sans' }}>Loading MP4 file</Text>
      </View>
    </View>
  );
}

export default SimpleMp4Example;
