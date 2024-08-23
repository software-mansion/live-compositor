import { ApiRequest, CompositorManager } from '@live-compositor/core';

import { sendRequest } from '../fetch';
import { retry, sleep } from '../utils';

type CreateInstanceOptions = {
  port: number;
  ip: string;
};

/**
 * CompositorManager that will connect to existing instance
 */
class ExistingInstance implements CompositorManager {
  private ip: string;
  private port: number;

  constructor(opts: CreateInstanceOptions) {
    this.port = opts.port;
    this.ip = opts.ip;
  }

  public async setupInstance(): Promise<void> {
    await retry(async () => {
      await sleep(500);
      return await this.sendRequest({
        method: 'GET',
        route: '/status',
      });
    }, 10);
  }

  public async sendRequest(request: ApiRequest): Promise<object> {
    return await sendRequest(`http://${this.ip}:${this.port}`, request);
  }
}

export default ExistingInstance;
