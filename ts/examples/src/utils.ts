import path from 'path';
import fs from 'fs-extra';
import { ChildProcess, spawn as nodeSpawn } from 'child_process';
import { promisify } from 'util';
import { Stream } from 'stream';
import fetch from 'node-fetch';

const pipeline = promisify(Stream.pipeline);

const TMP_SDP_DIR = '/tmp/live-compositor-examples';

export async function ffplayStartPlayerAsync(
  ip: string,
  video_port: number,
  audio_port: number | undefined = undefined
): Promise<{ spawn_promise: SpawnPromise }> {
  await fs.mkdirp(TMP_SDP_DIR);
  let sdpFilePath;
  if (audio_port === undefined) {
    sdpFilePath = path.join(TMP_SDP_DIR, `video_input_${video_port}.sdp`);
    await writeVideoSdpFile(ip, video_port, sdpFilePath);
  } else {
    sdpFilePath = path.join(TMP_SDP_DIR, `video_audio_input_${video_port}_${audio_port}.sdp`);
    await writeVideoAudioSdpFile(ip, video_port, audio_port, sdpFilePath);
  }

  const promise = spawn('ffplay', ['-protocol_whitelist', 'file,rtp,udp', sdpFilePath]);
  return { spawn_promise: promise };
}

export function ffmpegSendVideoFromMp4(port: number, mp4Path: string): SpawnPromise {
  return spawn('ffmpeg', [
    '-stream_loop',
    '-1',
    '-re',
    '-i',
    mp4Path,
    '-an',
    '-c:v',
    'libx264',
    '-f',
    'rtp',
    `rtp://127.0.0.1:${port}?rtcpport=${port}`,
  ]);
}

interface SpawnPromise extends Promise<void> {
  child: ChildProcess;
}

function spawn(command: string, args: string[]): SpawnPromise {
  console.log(`Spawning: ${command} ${args.join(' ')}`);
  const child = nodeSpawn(command, args, {
    stdio: 'ignore',
  });

  return new Promise<void>((resolve, reject) => {
    child.on('exit', code => {
      if (code === 0) {
        console.log(`Command "${command} ${args.join(' ')}" completed successfully.`);
        resolve();
      } else {
        const errorMessage = `Command "${command} ${args.join(' ')}" failed with exit code ${code}.`;
        console.error(errorMessage);
        reject(new Error(errorMessage));
      }
    });
  }) as SpawnPromise;
}

async function writeVideoAudioSdpFile(
  ip: string,
  video_port: number,
  audio_port: number,
  destination: string
): Promise<void> {
  fs.writeFile(
    destination,
    `
v=0
o=- 0 0 IN IP4 ${ip}
s=No Name
c=IN IP4 ${ip}
m=video ${video_port} RTP/AVP 96
a=rtpmap:96 H264/90000
a=fmtp:96 packetization-mode=1
a=rtcp-mux
m=audio ${audio_port} RTP/AVP 97
a=rtpmap:97 opus/48000/2
a=rtcp-mux
`
  );
}

async function writeVideoSdpFile(ip: string, port: number, destination: string): Promise<void> {
  fs.writeFile(
    destination,
    `
v=0
o=- 0 0 IN IP4 ${ip}
s=No Name
c=IN IP4 ${ip}
m=video ${port} RTP/AVP 96
a=rtpmap:96 H264/90000
a=fmtp:96 packetization-mode=1
a=rtcp-mux
`
  );
}

export async function sleep(timeout_ms: number): Promise<void> {
  await new Promise<void>(res => {
    setTimeout(() => {
      res();
    }, timeout_ms);
  });
}

const exampleAssets = [
  {
    path: 'BigBuckBunny.mp4',
    url: 'https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4',
  },
  {
    path: 'ElephantsDream.mp4',
    url: 'http://commondatastorage.googleapis.com/gtv-videos-bucket/sample/ElephantsDream.mp4',
  },
];

export async function downloadAllAssets(): Promise<void> {
  const downloadDir = path.join(__dirname, '../.assets');
  await fs.mkdirp(downloadDir);

  for (const asset of exampleAssets) {
    if (!(await fs.pathExists(path.join(downloadDir, asset.path)))) {
      await download(asset.url, path.join(downloadDir, asset.path));
    }
  }
}

async function download(url: string, destination: string): Promise<void> {
  const response = await fetch(url, { method: 'GET' });
  if (response.status >= 400) {
    const err: any = new Error(`Request to ${url} failed. \n${response.body}`);
    err.response = response;
    throw err;
  }
  if (response.body) {
    await pipeline(response.body, fs.createWriteStream(destination));
  } else {
    throw Error(`Response with empty body.`);
  }
}