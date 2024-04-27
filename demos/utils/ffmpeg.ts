import { COMPOSITOR_DIR } from "./prepare_compositor";
import { SpawnPromise, spawn } from "./utils";
import path from "path";
import fs from "fs-extra";

export async function ffplayStartPlayerAsync(ip: string, displayOutput: boolean, video_port: number, audio_port: number | undefined = undefined): Promise<{ spawn_promise: SpawnPromise }> {
  let sdpFilePath;
  if (audio_port === undefined) {
    sdpFilePath = path.join(COMPOSITOR_DIR, `video_input_${video_port}.sdp`);
    await writeVideoSdpFile(ip, video_port, sdpFilePath);
  } else {
    sdpFilePath = path.join(COMPOSITOR_DIR, `video_audio_input_${video_port}_${audio_port}.sdp`);
    await writeVideoAudioSdpFile(ip, video_port, audio_port, sdpFilePath);
  }

  const promise = spawn(
    "ffplay",
    ["-protocol_whitelist", "file,rtp,udp", sdpFilePath],
    { displayOutput }
  );
  return { spawn_promise: promise };
}

export function ffmpegSendVideoFromMp4(
  port: number,
  mp4Path: string,
  displayOutput: boolean
): SpawnPromise {
  return spawn(
    "ffmpeg",
    [
      "-stream_loop",
      "-1",
      "-re",
      "-i",
      mp4Path,
      "-an",
      "-c:v",
      "libx264",
      "-f",
      "rtp",
      `rtp://127.0.0.1:${port}?rtcpport=${port}`,
    ],
    { displayOutput }
  );
}

export function ffmpegStreamScreen(ip: string, port: number, displayOutput: boolean): SpawnPromise {
  const platform = process.platform;
  let inputOptions: string[];
  if (platform === "darwin") {
    inputOptions = ["-f", "avfoundation", "-i", "1"];
  } else if (platform === "linux") {
    inputOptions = ["-f", "x11grab", "-i", ":0.0"];
  } else {
    throw new Error("Unsupported platform");
  }

  return spawn(
    "ffmpeg",
    [
      ...inputOptions,
      "-vf",
      "format=yuv420p",
      "-c:v",
      "libx264",
      "-f",
      "rtp",
      `rtp://${ip}:${port}?rtcpport=${port}`,
    ],
    { displayOutput }
  );
}

async function writeVideoAudioSdpFile(ip: string, video_port: number, audio_port: number, destination: string): Promise<void> {
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
`,
  );
}

async function writeVideoSdpFile(
  ip: string,
  port: number,
  destination: string,
): Promise<void> {
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
`,
  );
}
