export type ApiRequest = {
  method: 'GET' | 'POST';
  route: string;
  body?: object;
};

export interface ServerManager {
  ensureStarted(): void;
  sendRequest(request: ApiRequest): Promise<object>;
}
