const BACKEND_URL: URL = new URL('http://localhost:8081');

interface RequestObject {
  method: string;
  headers: {
    'Content-Type': string;
  };
  body: string;
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

async function getErrorDescription(response: Response): Promise<string> {
  const contentType = response.headers.get('Content-Type');
  const errorStatus = `Error status: ${response.status}`;
  let errorMessage = '';

  if (contentType === 'application/json') {
    const apiError = await response.json();
    if (apiError.stack) {
      errorMessage = `Error message: ${apiError.stack.map(String).join('\n')}`;
    } else {
      errorMessage = `Error message: ${apiError.message}`;
    }
  } else {
    const txt = await response.text();
    errorMessage = `Error message: ${txt}`;
  }
  return `${errorStatus};\n${errorMessage}`;
}

export async function renderImage(body: object): Promise<Blob> {
  const requestObject = buildRequestRenderImage(body);
  const renderImageUrl = new URL('/render_image', BACKEND_URL);
  const response = await fetch(renderImageUrl, requestObject);

  if (response.status >= 400) {
    const errorDescription = await getErrorDescription(response);
    throw new Error(errorDescription);
  }
  return await response.blob();
}
