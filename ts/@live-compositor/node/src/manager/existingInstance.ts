import type { ApiRequest, CompositorManager, SetupInstanceOptions } from '@live-compositor/core';

import { sendRequest } from '../fetch';
import { retry, sleep } from '../utils';
import { WebSocketConnection } from '../ws';

type CreateInstanceOptions = {
  port: number;
  ip: string;
  protocol: 'http' | 'https';
};

/**
 * CompositorManager that will connect to existing instance
 */
class ExistingInstance implements CompositorManager {
  private ip: string;
  private port: number;
  private protocol: 'http' | 'https';
  private wsConnection: WebSocketConnection;

  constructor(opts: CreateInstanceOptions) {
    this.port = opts.port;
    this.ip = opts.ip;
    this.protocol = opts.protocol ?? 'http';

    const wsProtocol = this.protocol === 'https' ? 'wss' : 'ws';
    this.wsConnection = new WebSocketConnection(`${wsProtocol}://${this.ip}:${this.port}/ws`);
  }

  public async setupInstance(_opts: SetupInstanceOptions): Promise<void> {
    // TODO: verify if options match
    // https://github.com/software-mansion/live-compositor/issues/877
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
    return await sendRequest(`${this.protocol}://${this.ip}:${this.port}`, request);
  }

  public registerEventListener(cb: (event: object) => void): void {
    this.wsConnection.registerEventListener(cb);
  }
}

export default ExistingInstance;
