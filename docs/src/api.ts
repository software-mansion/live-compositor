// The @generated/docusaurus.config file is built at runtime, not present during linting.
// eslint-disable-next-line import/no-unresolved
import config from '@generated/docusaurus.config';

const BACKEND_URL: string =
  config.customFields.environment === 'development'
    ? 'http://localhost:8081'
    : 'https://playground.compositor.live';

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
  const MAX_TIME_FOR_RETRIES_MS = 30000;
  const TIMEOUT_MS = 5000;
  const requestObject = buildRequestRenderImage(body);

  const response = await retryWithTimeout(
    async () => await fetchWithTimeout('/render_image', requestObject, TIMEOUT_MS),
    MAX_TIME_FOR_RETRIES_MS
  );
  if (response.status >= 400) {
    throw await createError(response);
  }
  return await response.blob();
}

async function fetchWithTimeout(
  path: string,
  requestObject: RequestObject,
  timeout: number
): Promise<Response> {
  const fullUrl = new URL(path, BACKEND_URL);

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

async function retryWithTimeout<T>(fn: () => Promise<T>, timeoutMs: number): Promise<T> {
  const DELAY_MS = 1000;
  const startTime = performance.now();

  while (true) {
    try {
      return await fn();
    } catch (error) {
      await sleep(DELAY_MS);
      if (performance.now() - startTime > timeoutMs) {
        throw error;
      }
    }
  }
}

async function sleep(timeoutMs: number): Promise<void> {
  await new Promise<void>(res => {
    setTimeout(() => {
      res();
    }, timeoutMs);
  });
}
