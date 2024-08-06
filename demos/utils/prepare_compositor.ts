import fs from 'fs-extra';
import path from 'path';
import { downloadAsync, spawn } from './utils';

export const COMPOSITOR_DIR = path.join(__dirname, '../.video_compositor');

const VERSION = 'v0.2.0-rc.5';

const COMPOSITOR_X86_64_LINUX_DOWNLOAD_URL = `https://github.com/software-mansion/live-compositor/releases/download/${VERSION}/video_compositor_linux_x86_64.tar.gz`;
const COMPOSITOR_ARM_LINUX_DOWNLOAD_URL = `https://github.com/software-mansion/live-compositor/releases/download/${VERSION}/video_compositor_linux_aarch64.tar.gz`;
const COMPOSITOR_X86_64_MAC_DOWNLOAD_URL = `https://github.com/software-mansion/live-compositor/releases/download/${VERSION}/video_compositor_darwin_x86_64.tar.gz`;
const COMPOSITOR_ARM_MAC_DOWNLOAD_URL = `https://github.com/software-mansion/live-compositor/releases/download/${VERSION}/video_compositor_darwin_aarch64.tar.gz`;

export async function ensureCompositorReadyAsync(): Promise<void> {
  const versionFile = path.join(COMPOSITOR_DIR, '.version');
  if (
    (await fs.pathExists(versionFile)) &&
    (await fs.readFile(versionFile, 'utf8')).trim() === VERSION
  ) {
    return;
  }
  try {
    await prepareCompositorAsync();
  } catch (err) {
    await fs.remove(COMPOSITOR_DIR);
    throw err;
  }
}

export async function prepareCompositorAsync() {
  await fs.remove(COMPOSITOR_DIR);
  await fs.mkdirp(COMPOSITOR_DIR);
  console.log('Downloading video_compositor.');
  await downloadAsync(
    getCompositorDownloadUrl(),
    path.join(COMPOSITOR_DIR, 'video_compositor.tar.gz')
  );
  console.log('Unpacking video_compositor.');
  await spawn('tar', ['-xvf', 'video_compositor.tar.gz'], {
    displayOutput: true,
    cwd: COMPOSITOR_DIR,
  });
  await fs.writeFile(path.join(COMPOSITOR_DIR, '.version'), VERSION);
}

function getCompositorDownloadUrl(): string {
  if (process.platform === 'linux') {
    if (process.arch === 'arm64') {
      return COMPOSITOR_ARM_LINUX_DOWNLOAD_URL;
    } else if (process.arch === 'x64') {
      return COMPOSITOR_X86_64_LINUX_DOWNLOAD_URL;
    }
  } else if (process.platform === 'darwin') {
    if (process.arch === 'x64') {
      return COMPOSITOR_X86_64_MAC_DOWNLOAD_URL;
    } else if (process.arch === 'arm64') {
      return COMPOSITOR_ARM_MAC_DOWNLOAD_URL;
    }
  }
  throw new Error('Unsupported platform.');
}
