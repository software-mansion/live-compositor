import fs from 'fs';
import { Stream } from 'stream';
import { promisify } from 'util';

import fetch from 'node-fetch';
import { ApiRequest } from '@live-compositor/core';

const pipeline = promisify(Stream.pipeline);

export async function sendRequest(baseUrl: string, request: ApiRequest): Promise<object> {
  const response = await fetch(new URL(request.route, baseUrl), {
    method: request.method,
    body: request.body && JSON.stringify(request.body),
    headers: {
      'Content-Type': 'application/json',
    },
  });
  if (response.status >= 400) {
    const err: any = new Error(`Request to compositor failed.`);
    err.response = response;
    try {
      err.body = await response.json();
    } catch {
      err.body = await response.text();
    }
    throw err;
  }
  return (await response.json()) as object;
}

export async function download(url: string, destination: string): Promise<void> {
  const response = await fetch(url, { method: 'GET' });
  if (response.status >= 400) {
    const err: any = new Error(`Request to ${url} failed. \n${response.body}`);
    err.response = response;
    throw err;
  }
  if (response.body) {
    await pipeline(response.body, fs.createWriteStream(destination));
  } else {
    throw Error(`Response with empty body.`);
  }
}
