import * as ApiTypes from './api';
import Api from './api';
import { ServerManager } from './severInstance';
import ManagedInstance from './severInstance/managed';
import { Context } from './types';

class LiveCompositor<Props> {
  private ctx: Context = {
    inputs: [],
  };
  private lastScene: ApiTypes.Component | null = null;
  private serverManager: ServerManager;
  private apiInstance: Api;

  constructor(root: Element<Props>, serverManager?: ServerManager) {
    if (typeof root === 'string') {
      throw new Error("root component can't be a string");
    }
    this.serverManager = serverManager ?? new ManagedInstance(8000, '/tmp/compositor_tmp');
    this.apiInstance = new Api(this.serverManager);
  }

  public api(): Api {
    return this.apiInstance;
  }

  public async start() {
    await this.serverManager.sendRequest({
      method: 'POST',
      route: `/api/output/output_1/unregister`,
      body: {},
    });
  }
}

export default LiveCompositor;
