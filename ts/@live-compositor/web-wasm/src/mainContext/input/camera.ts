import type { Input, RegisterInputResult } from '../input';

export class CameraInput implements Input {
  private mediaStream: MediaStream;

  constructor(mediaStream: MediaStream) {
    this.mediaStream = mediaStream;
  }

  public async terminate(): Promise<void> {
    this.mediaStream.getTracks().forEach(track => track.stop());
  }
}

export async function handleRegisterCameraInput(inputId: string): Promise<RegisterInputResult> {
  const mediaStream = await navigator.mediaDevices.getUserMedia({
    audio: false,
    video: true,
  });
  const videoTrack = mediaStream.getVideoTracks()[0];
  const audioTrack = mediaStream.getAudioTracks()[0];
  const transferable = [];

  // @ts-ignore
  let videoTrackProcessor: MediaStreamTrackProcessor | undefined;
  if (videoTrack) {
    // @ts-ignore
    videoTrackProcessor = new MediaStreamTrackProcessor({ track: videoTrack });
    transferable.push(videoTrackProcessor.readable);
  }

  // @ts-ignore
  let audioTrackProcessor: MediaStreamTrackProcessor | undefined;
  if (audioTrack) {
    // @ts-ignore
    audioTrackProcessor = new MediaStreamTrackProcessor({ track: audioTrack });
    transferable.push(audioTrackProcessor.readable);
  }

  return {
    input: new CameraInput(mediaStream),
    workerMessage: [
      {
        type: 'registerInput',
        inputId,
        input: {
          type: 'stream',
          videoStream: videoTrackProcessor.readable,
          audioStream: videoTrackProcessor.readable,
        },
      },
      transferable,
    ],
  };
}
