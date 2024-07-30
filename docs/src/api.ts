interface RequestObject {
  method: string;
  headers: {
    'Content-Type': string;
  };
  body: string;
}

export function buildRequestSceneObject(method: string, scene: object): RequestObject {
  return {
    method: method,
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ scene: scene }),
  };
}

export async function handleRenderImageRequest(
  backendUrl: URL,
  requestObject: RequestObject
): Promise<Blob> {
  const renderImageUrl = new URL('/render_image', backendUrl);
  const response = await fetch(renderImageUrl, requestObject);
  if (response.status >= 400) {
    const contentType = response.headers.get('Content-Type');
    const errorStatus = `Error status: ${response.status}`;
    let errorMsg = '';

    if (contentType === 'application/json') {
      const apiError = await response.json();
      if (apiError.stack) {
        errorMsg = `Error message: ${apiError.stack.map(String).join('\n')}`;
      } else {
        errorMsg = `Error message: ${apiError.message}`;
      }
    } else {
      const txt = await response.text();
      errorMsg = `Error message: ${txt}`;
    }

    throw new Error(`${errorStatus}\n${errorMsg}`);
  }

  return await response.blob();
}
