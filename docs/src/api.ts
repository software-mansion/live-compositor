const BACKEND_URL: URL = new URL('http://localhost:8081');

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
  const renderImageUrl = new URL('/render_image', BACKEND_URL);
  let response;
  try {
    response = await fetch(renderImageUrl, requestObject);
  } catch (error) {
    throw new ApiError(error.message);
  }
  if (response.status >= 400) {
    throw await createError(response);
  }
  return await response.blob();
}
