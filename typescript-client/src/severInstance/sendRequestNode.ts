import fetch from 'node-fetch';
import { ApiRequest } from '.';

export async function sendRequest(baseUrl: string, request: ApiRequest): Promise<object> {
  const response = await fetch(new URL(request.route, baseUrl), {
    method: request.method,
    body: request.body && JSON.stringify(request.body),
  });
  if (response.status >= 400) {
    // TODO: better error printing
    const err: any = new Error(`Request to compositor failed.`);
    err.response = response;
    throw err;
  }
  return (await response.json()) as object;
}
