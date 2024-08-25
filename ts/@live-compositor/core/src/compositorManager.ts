import { ApiRequest } from './api';

export interface CompositorManager {
  setupInstance(): Promise<void>;
  sendRequest(request: ApiRequest): Promise<object>;
  registerEventListener(cb: (event: unknown) => void): void;
}
