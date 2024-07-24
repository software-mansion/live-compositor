import { ApiRequest, ServerManager } from '.';
import { sendRequest } from './sendRequestNode';

/**
 * ServerManager that will download and spawn it's own LiveCompositor instance locally.
 */
class ManagedInstance implements ServerManager {
  private port: number;
  private workingdir: string;

  constructor(port: number, workingdir: string) {
    this.port = port;
    this.workingdir = workingdir;
  }

  public ensureStarted(): void {
    // start process and wait for status endpoint
    throw new Error('Method not implemented.');
  }

  public async sendRequest(request: ApiRequest): Promise<object> {
    return sendRequest(`http://127.0.0.1:${this.port}`, request);
  }
}

export default ManagedInstance;
