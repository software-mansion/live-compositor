import type { Api } from 'live-compositor';
import type { RegisterOutput } from '../../workerApi';
import type { Output, RegisterOutputResult, RegisterWasmWhipOutput } from '../output';
import type { Logger } from 'pino';
import type { Framerate } from '../../compositor/compositor';
import type { AudioMixer } from '../AudioMixer';
import { MediaStreamAudioMixer } from '../AudioMixer';

type PeerConnectionOptions = {
  logger: Logger;
  endpointUrl: RegisterWasmWhipOutput['endpointUrl'];
  bearerToken?: RegisterWasmWhipOutput['bearerToken'];
  iceServers?: RegisterWasmWhipOutput['iceServers'];
  video?: {
    stream: MediaStream;
    track: MediaStreamTrack;
    resolution: Api.Resolution;
    maxBitrate?: number;
  };
  audio: boolean;
};

export class WhipOutput implements Output {
  private mixer?: MediaStreamAudioMixer = undefined;
  private options: PeerConnectionOptions;
  private pc?: RTCPeerConnection;
  private location?: string;

  constructor(options: PeerConnectionOptions) {
    this.options = options;
  }

  public get audioMixer(): AudioMixer | undefined {
    return this.mixer;
  }

  async init(): Promise<MediaStream> {
    const pc = new RTCPeerConnection({
      iceServers: this.options.iceServers || [{ urls: 'stun:stun.l.google.com:19302' }],
      bundlePolicy: 'max-bundle',
    });
    const negotiationNeededPromise = new Promise<void>(res => {
      pc.addEventListener('negotiationneeded', () => {
        res();
      });
    });
    const mediaStream = new MediaStream();

    if (this.options.video) {
      mediaStream.addTrack(this.options.video.track);
      const videoSender = pc.addTransceiver(this.options.video.track, {
        direction: 'sendonly',
        sendEncodings: [
          {
            maxBitrate: this.options.video.maxBitrate,
            priority: 'high',
            networkPriority: 'high',
            scaleResolutionDownBy: 1.0,
          },
        ],
      });

      const params = videoSender.sender.getParameters();
      params.degradationPreference = 'maintain-resolution';
      await videoSender.sender.setParameters(params);
    }

    if (this.options.audio) {
      this.mixer = new MediaStreamAudioMixer();
      const track = this.mixer.outputMediaStreamTrack();
      mediaStream.addTrack(track);
      pc.addTransceiver(track, { direction: 'sendonly' });
    }

    await negotiationNeededPromise;
    this.location = await establishWhipConnection(
      pc,
      this.options.endpointUrl,
      this.options.bearerToken
    );

    this.pc = pc;
    return mediaStream;
  }

  public async terminate(): Promise<void> {
    this.options.logger.debug('terminate WHIP connection');
    try {
      await fetch(this.location ?? this.options.endpointUrl, {
        method: 'DELETE',
        mode: 'cors',
        headers: {
          ...(this.options.bearerToken
            ? { authorization: `Bearer ${this.options.bearerToken}` }
            : {}),
        },
      });
    } catch (err: any) {
      // Some services like Twitch do not implement DELETE endpoint
      this.options.logger.debug({ err });
    }
    this.pc?.close();
    this.options.video?.stream.getTracks().forEach(track => track.stop());
    await this.audioMixer?.close();
  }
}

export async function handleRegisterWhipOutput(
  outputId: string,
  request: RegisterWasmWhipOutput,
  logger: Logger,
  framerate: Framerate
): Promise<RegisterOutputResult> {
  let video: RegisterOutput['video'] = undefined;
  let videoPeerConnection: PeerConnectionOptions['video'];
  let transferable: Transferable[] = [];

  if (request.video && request.initial.video) {
    const canvas = document.createElement('canvas');
    canvas.width = request.video.resolution.width;
    canvas.height = request.video.resolution.height;
    const stream = canvas.captureStream(framerate.num / framerate.den);
    const track = stream.getVideoTracks()[0];
    const offscreen = canvas.transferControlToOffscreen();

    await track.applyConstraints({
      width: { exact: request.video.resolution.width },
      height: { exact: request.video.resolution.height },
      frameRate: { ideal: framerate.num / framerate.den },
    });

    videoPeerConnection = {
      track,
      stream,
      resolution: request.video.resolution,
      maxBitrate: request.video.maxBitrate,
    };
    transferable.push(offscreen);
    video = {
      resolution: request.video.resolution,
      initial: request.initial.video,
      canvas: offscreen,
    };
  }

  const output = new WhipOutput({
    logger,
    iceServers: request.iceServers,
    bearerToken: request.bearerToken,
    endpointUrl: request.endpointUrl,
    video: videoPeerConnection,
    audio: !!request.audio,
  });
  const outputStream = await output.init();

  return {
    output,
    result: {
      type: 'web-wasm-whip',
      stream: outputStream,
    },
    workerMessage: [
      {
        type: 'registerOutput',
        outputId,
        output: {
          type: 'stream',
          video,
        },
      },
      transferable,
    ],
  };
}

async function establishWhipConnection(
  pc: RTCPeerConnection,
  endpoint: string,
  token?: string
): Promise<string> {
  await pc.setLocalDescription(await pc.createOffer());

  const offer = await gatherICECandidates(pc);
  if (!offer) {
    throw Error('failed to gather ICE candidates for offer');
  }

  /**
   * This response contains the server's SDP offer.
   * This specifies how the client should communicate,
   * and what kind of media client and server have negotiated to exchange.
   */
  let { sdp: sdpAnswer, location } = await postSdpOffer(endpoint, offer.sdp, token);

  await pc.setRemoteDescription(new RTCSessionDescription({ type: 'answer', sdp: sdpAnswer }));
  return location;
}

async function gatherICECandidates(
  peerConnection: RTCPeerConnection
): Promise<RTCSessionDescription | null> {
  return new Promise<RTCSessionDescription | null>(res => {
    setTimeout(function () {
      res(peerConnection.localDescription);
    }, 2000);

    peerConnection.onicegatheringstatechange = (_ev: Event) => {
      if (peerConnection.iceGatheringState === 'complete') {
        res(peerConnection.localDescription);
      }
    };
  });
}

async function postSdpOffer(
  endpoint: string,
  sdpOffer: string,
  token?: string
): Promise<{ sdp: string; location: string }> {
  const response = await fetch(endpoint, {
    method: 'POST',
    mode: 'cors',
    headers: {
      'content-type': 'application/sdp',
      ...(token ? { authorization: `Bearer ${token}` } : {}),
    },
    body: sdpOffer,
  });

  if (response.status === 201) {
    return {
      sdp: await response.text(),
      location: getLocationFromHeader(response.headers, endpoint),
    };
  } else {
    const errorMessage = await response.text();
    throw new Error(errorMessage);
  }
}

function getLocationFromHeader(headers: Headers, endpoint: string): string {
  const locationHeader = headers.get('Location');
  if (!locationHeader) {
    // e.g. Twitch CORS blocks access to Location header, so in this case let's assume that
    // location is under the same URL.
    return endpoint;
  }

  return new URL(locationHeader, endpoint).toString();
}
