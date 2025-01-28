import type { Logger } from 'pino';
import type { ApiRequest } from './api.js';

export interface SetupInstanceOptions {
  /**
   * sets LIVE_COMPOSITOR_AHEAD_OF_TIME_PROCESSING_ENABLE environment variable.
   */
  aheadOfTimeProcessing: boolean;

  logger: Logger;
}

export interface CompositorManager {
  setupInstance(opts: SetupInstanceOptions): Promise<void>;
  sendRequest(request: ApiRequest): Promise<object>;
  sendMultipartRequest(request: ApiRequest): Promise<object>;
  registerEventListener(cb: (event: unknown) => void): void;
  terminate(): Promise<void>;
}
