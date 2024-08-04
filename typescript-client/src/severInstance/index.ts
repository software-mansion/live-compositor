export type ApiRequest = {
  method: 'GET' | 'POST';
  route: string;
  body?: object;
};

export interface ServerManager {
  ensureStarted(): Promise<void>;
  sendRequest(request: ApiRequest): Promise<object>;
}
