export type MediaStreamInitFn = () => Promise<MediaStream>;

export async function initCameraMediaStream(): Promise<MediaStream> {
  return await navigator.mediaDevices.getUserMedia({ video: true });
}
