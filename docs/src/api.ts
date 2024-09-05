// The @generated/docusaurus.config file is built at runtime, not present during linting.
// eslint-disable-next-line import/no-unresolved
import config from '@generated/docusaurus.config';

interface RequestObject {
  method: string;
  headers: {
    'Content-Type': string;
  };
  body: string;
}

export class ApiError extends Error {
  response: Response;
  body: string | object;
  constructor(message: string, response?: Response, body?: string | object) {
    super(message);
    this.response = response;
    this.body = body;
  }
}

function buildRequestRenderImage(body: object): RequestObject {
  return {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(body),
  };
}

async function createError(response: Response): Promise<ApiError> {
  const contentType = response.headers.get('Content-Type');
  const errorStatus = `Error status: ${response.status} ${response.statusText}`;
  let errorMessage = '';
  let body;
  if (contentType === 'application/json') {
    const apiError = await response.json();
    body = apiError;
    if (apiError.stack) {
      errorMessage = apiError.stack.map(String).join('\n');
    } else {
      errorMessage = apiError.message;
    }
  } else {
    errorMessage = await response.text();
    body = errorMessage;
  }
  console.log(`${errorStatus};\nError message: ${errorMessage}`);
  return new ApiError(errorMessage, response, body);
}

export async function renderImage(body: object): Promise<Blob> {
  const requestObject = buildRequestRenderImage(body);
  return retryRequest(fetchWithTimeout, '/render_image', requestObject);
}

async function fetchWithTimeout(
  path: string,
  requestObject: RequestObject,
  timeout: number
): Promise<Response> {
  const backendUrl: URL = new URL(
    config.customFields.environment === 'development'
      ? 'http://localhost:8081'
      : 'https://playground.compositor.live'
  );
  const fullUrl = new URL(path, backendUrl);

  const controller = new AbortController();
  const signal = controller.signal;

  return new Promise<Response>((res, rej) => {
    const timer = setTimeout(() => {
      controller.abort();
      rej(new ApiError(`Fetch not responding after ${timeout / 1000} s`));
    }, timeout);

    fetch(fullUrl, { ...requestObject, signal })
      .then(response => {
        clearTimeout(timer);
        res(response);
      })
      .catch(error => {
        clearTimeout(timer);
        rej(new ApiError(error.message));
      });
  });
}

async function retryRequest(func, path, requestObject) {
  const delay = 1000;
  const timeout = 5000;
  const maxTimeForRetries = 30000;
  const startTime = performance.now();

  while (performance.now() - startTime < maxTimeForRetries) {
    try {
      const response = await func(path, requestObject, timeout);

      if (response.status >= 400) {
        throw await createError(response);
      }

      return await response.blob();
    } catch (error) {
      if (error instanceof ApiError && !error.response) {
        await new Promise(res => setTimeout(res, delay));
      } else {
        throw error;
      }
    }
  }
  throw new ApiError('Maximum time exited!');
}
