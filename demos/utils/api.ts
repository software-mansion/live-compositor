import fetch from 'node-fetch';
import {
  ImageSpec,
  RegisterInput,
  RegisterOutput,
  ShaderSpec,
  UpdateOutputRequest,
  WebRendererSpec,
} from '../types/api.d';

const COMPOSITOR_URL = 'http://127.0.0.1:8081';

type CompositorRequestBody =
  | RegisterInput
  | RegisterOutput
  | UpdateOutputRequest
  | ShaderSpec
  | WebRendererSpec
  | ImageSpec;

export async function registerInputAsync(inputId: string, body: RegisterInput): Promise<object> {
  return sendAsync(`/api/input/${inputId}/register`, body);
}

export async function unregisterInputAsync(inputId: string): Promise<object> {
  return sendAsync(`/api/input/${inputId}/unregister`, {});
}

export async function registerOutputAsync(outputId: string, body: RegisterOutput): Promise<object> {
  return sendAsync(`/api/output/${outputId}/register`, body);
}

export async function unregisterOutputAsync(outputId: string): Promise<object> {
  return sendAsync(`/api/output/${outputId}/unregister`, {});
}

export async function updateOutputAsync(
  outputId: string,
  body: UpdateOutputRequest
): Promise<object> {
  return sendAsync(`/api/output/${outputId}/update`, body);
}

export async function registerShaderAsync(shaderId: string, body: ShaderSpec): Promise<object> {
  return sendAsync(`/api/shader/${shaderId}/register`, body);
}

export async function unregisterShaderAsync(shaderId: string): Promise<object> {
  return sendAsync(`/api/shader/${shaderId}/unregister`, {});
}

export async function registerWebRendererAsync(
  rendererId: string,
  body: WebRendererSpec
): Promise<object> {
  return sendAsync(`/api/web_renderer/${rendererId}/register`, body);
}

export async function unregisterWebRendererAsync(rendererId: string): Promise<object> {
  return sendAsync(`/api/web_renderer/${rendererId}/unregister`, {});
}

export async function registerImageAsync(imageId: string, body: ImageSpec): Promise<object> {
  return sendAsync(`/api/image/${imageId}/register`, body);
}

export async function unregisterImageAsync(imageId: string): Promise<object> {
  return sendAsync(`/api/image/${imageId}/unregister`, {});
}

export async function startAsync(): Promise<object> {
  return sendAsync(`/api/start`, {});
}

async function sendAsync(endpoint: string, body: CompositorRequestBody): Promise<object> {
  let response;
  try {
    response = await fetch(COMPOSITOR_URL + endpoint, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(body),
    });
  } catch (err: any) {
    err.endpoint = endpoint;
    err.request = JSON.stringify(body);
    throw err;
  }

  if (response.status >= 400) {
    const err: any = new Error(`Request to compositor failed.`);
    err.endpoint = endpoint;
    err.request = JSON.stringify(body);
    err.status = await response.status;
    err.response = await response.json();
    throw err;
  }
  return await response.json();
}
