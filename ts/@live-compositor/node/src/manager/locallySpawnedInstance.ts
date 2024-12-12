import os from 'os';
import path from 'path';

import { v4 as uuidv4 } from 'uuid';
import * as fs from 'fs-extra';
import * as tar from 'tar';
import type { ApiRequest, CompositorManager, SetupInstanceOptions } from '@live-compositor/core';

import { download, sendRequest } from '../fetch';
import { retry, sleep } from '../utils';
import { spawn } from '../spawn';
import { WebSocketConnection } from '../ws';

const VERSION = `v0.3.0`;

type ManagedInstanceOptions = {
  port: number;
  workingdir?: string;
  executablePath?: string;
  enableWebRenderer?: boolean;
};

/**
 * CompositorManager that will download and spawn it's own LiveCompositor instance locally.
 */
class LocallySpawnedInstance implements CompositorManager {
  private port: number;
  private workingdir: string;
  private executablePath?: string;
  private wsConnection: WebSocketConnection;
  private enableWebRenderer?: boolean;

  constructor(opts: ManagedInstanceOptions) {
    this.port = opts.port;
    this.workingdir = opts.workingdir ?? path.join(os.tmpdir(), `live-compositor-${uuidv4()}`);
    this.executablePath = opts.executablePath;
    this.enableWebRenderer = opts.enableWebRenderer;
    this.wsConnection = new WebSocketConnection(`ws://127.0.0.1:${this.port}/ws`);
  }

  public static defaultManager(): LocallySpawnedInstance {
    const port = process.env.LIVE_COMPOSITOR_API_PORT
      ? Number(process.env.LIVE_COMPOSITOR_API_PORT)
      : 8000;
    return new LocallySpawnedInstance({
      port,
      executablePath: process.env.LIVE_COMPOSITOR_PATH,
    });
  }

  public async setupInstance(opts: SetupInstanceOptions): Promise<void> {
    const executablePath = this.executablePath ?? (await prepareExecutable(this.enableWebRenderer));

    spawn(executablePath, [], {
      env: {
        LIVE_COMPOSITOR_DOWNLOAD_DIR: path.join(this.workingdir, 'download'),
        LIVE_COMPOSITOR_API_PORT: this.port.toString(),
        LIVE_COMPOSITOR_WEB_RENDERER_ENABLE: this.enableWebRenderer ? 'true' : 'false',
        // silence scene updates logging
        LIVE_COMPOSITOR_LOGGER_FORMAT: 'compact',
        LIVE_COMPOSITOR_LOGGER_LEVEL:
          'info,wgpu_hal=warn,wgpu_core=warn,compositor_pipeline::pipeline=warn,live_compositor::log_request_body=debug',
        LIVE_COMPOSITOR_AHEAD_OF_TIME_PROCESSING_ENABLE: opts.aheadOfTimeProcessing
          ? 'true'
          : 'false',
        ...process.env,
      },
    }).catch(err => {
      console.error('LiveCompositor instance failed', err);
    });

    await retry(async () => {
      await sleep(500);
      return await this.sendRequest({
        method: 'GET',
        route: '/status',
      });
    }, 10);

    await this.wsConnection.connect();
  }

  public async sendRequest(request: ApiRequest): Promise<object> {
    return await sendRequest(`http://127.0.0.1:${this.port}`, request);
  }

  public registerEventListener(cb: (event: object) => void): void {
    this.wsConnection.registerEventListener(cb);
  }
}

async function prepareExecutable(enableWebRenderer?: boolean): Promise<string> {
  const version = enableWebRenderer ? `${VERSION}-web` : VERSION;
  const downloadDir = path.join(os.homedir(), '.live_compositor', version, architecture());
  const readyFilePath = path.join(downloadDir, '.ready');
  const executablePath = path.join(downloadDir, 'live_compositor/live_compositor');

  if (await fs.pathExists(readyFilePath)) {
    return executablePath;
  }
  await fs.mkdirp(downloadDir);

  const tarGzPath = path.join(downloadDir, 'live_compositor.tar.gz');
  if (await fs.pathExists(tarGzPath)) {
    await fs.remove(tarGzPath);
  }
  await download(compositorTarGzUrl(enableWebRenderer), tarGzPath);

  await tar.x({
    file: tarGzPath,
    cwd: downloadDir,
  });
  await fs.remove(tarGzPath);

  await fs.writeFile(readyFilePath, '\n', 'utf-8');
  return executablePath;
}

function architecture(): 'linux_aarch64' | 'linux_x86_64' | 'darwin_x86_64' | 'darwin_aarch64' {
  if (process.arch === 'x64' && process.platform === 'linux') {
    return 'linux_x86_64';
  } else if (process.arch === 'arm64' && process.platform === 'linux') {
    return 'linux_aarch64';
  } else if (process.arch === 'x64' && process.platform === 'darwin') {
    return 'darwin_x86_64';
  } else if (process.arch === 'arm64' && process.platform === 'darwin') {
    return 'darwin_aarch64';
  } else {
    throw new Error(`Unsupported platform ${process.platform} ${process.arch}`);
  }
}

function compositorTarGzUrl(withWebRenderer?: boolean): string {
  const archiveNameSuffix = withWebRenderer ? '_with_web_renderer' : '';
  const archiveName = `live_compositor${archiveNameSuffix}_${architecture()}.tar.gz`;
  return `https://github.com/software-mansion/live-compositor/releases/download/${VERSION}/${archiveName}`;
}

export default LocallySpawnedInstance;
