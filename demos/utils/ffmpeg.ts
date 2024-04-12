import { COMPOSITOR_DIR } from "./prepare_compositor";
import { SpawnPromise, sleepAsync, spawn } from "./utils";
import path from "path";
import fs from "fs-extra";

const PIPE_OR_INHERIT = process.env.DEBUG ? "inherit" : "pipe";

export async function ffplayListenAsync(
  port: number,
): Promise<{ spawn_promise: SpawnPromise }> {
  const sdpFilePath = path.join(COMPOSITOR_DIR, `input_${port}.sdp`);
  await writeSdpFile("127.0.0.1", port, sdpFilePath);
  const promise = spawn(
    "ffplay",
    ["-protocol_whitelist", "file,rtp,udp", sdpFilePath],
    { stdio: [PIPE_OR_INHERIT, PIPE_OR_INHERIT, "ignore"] }, // no idea why stderr can't be set to "pipe"
  );
  // sleep to make sure ffplay have a chance to start before compositor starts sending frames
  await sleepAsync(2000);
  return { spawn_promise: promise };
}

export function ffmpegSendTestPattern(
  port: number,
  resolution: { width: number; height: number },
): SpawnPromise {
  const ffmpeg_source = `testsrc=s=${resolution.width}x${resolution.height}:r=30,format=yuv420p`;
  return spawn(
    "ffmpeg",
    [
      "-re",
      "-f",
      "lavfi",
      "-i",
      ffmpeg_source,
      "-c:v",
      "libx264",
      "-f",
      "rtp",
      `rtp://127.0.0.1:${port}?rtcpport=${port}`,
    ],
    { stdio: [PIPE_OR_INHERIT, PIPE_OR_INHERIT, "ignore"] },
  );
}

export function ffmpegSendVideoFromMp4(
  port: number,
  mp4Path: string,
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
    { stdio: [PIPE_OR_INHERIT, PIPE_OR_INHERIT, "ignore"] },
  );
}

export function ffmpegStreamScreen(ip: string, port: number): SpawnPromise {
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
    { stdio: [PIPE_OR_INHERIT, PIPE_OR_INHERIT, "ignore"] },
  );
}

async function writeSdpFile(
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
