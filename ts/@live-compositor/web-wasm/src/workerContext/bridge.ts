import type { Logger } from 'pino';

type RequestMessage<Request> = {
  id: string;
  request: Request;
};

type ResponseMessage<Response> = {
  type: 'workerResponse';
  id: string;
  response?: Response;
  error?: Error;
};

type EventMessage<Event> = {
  type: 'workerEvent';
  event: Event;
};

type PendingMessage<Response> = {
  res: (response: Response) => void;
  rej: (err: Error) => void;
};

let requestCounter = 1;

export function registerWorkerEntrypoint<Request, Response>(
  onMessage: (request: Request) => Promise<Response>
) {
  self.onmessage = async (event: MessageEvent<RequestMessage<Request>>) => {
    try {
      const response = await onMessage(event.data.request);
      self.postMessage({
        type: 'workerResponse',
        id: event.data.id,
        response,
      } as ResponseMessage<Response>);
    } catch (error: any) {
      self.postMessage({
        type: 'workerResponse',
        id: event.data.id,
        error,
      } as ResponseMessage<Response>);
    }
  };
}

export function workerPostEvent<Event>(event: Event) {
  self.postMessage({ type: 'workerEvent', event });
}

export class AsyncWorker<Request, Response, Event> {
  private worker: Worker;
  private pendingMessages: Record<string, PendingMessage<Response>> = {};
  private onEvent: (event: Event) => void;
  private logger: Logger;

  constructor(worker: Worker, onEvent: (event: Event) => void, logger: Logger) {
    this.logger = logger;
    this.worker = worker;
    this.worker.onmessage = (
      event: MessageEvent<ResponseMessage<Response> | EventMessage<Event>>
    ) => {
      if (event.data.type === 'workerEvent') {
        this.handleEvent(event.data.event);
      } else if (event.data.type === 'workerResponse') {
        this.handleResponse(event.data);
      }
    };
    this.onEvent = onEvent;
  }

  public async postMessage(request: Request, transferable?: Transferable[]): Promise<Response> {
    const requestId = String(requestCounter);
    requestCounter += 1;

    const pendingMessage: PendingMessage<Response> = {} as any;
    const responsePromise = new Promise<Response>((res, rej) => {
      pendingMessage.res = res;
      pendingMessage.rej = rej;
    });
    this.pendingMessages[requestId] = pendingMessage;

    if (transferable) {
      this.worker.postMessage({ id: requestId, request }, transferable);
    } else {
      this.worker.postMessage({ id: requestId, request });
    }
    return responsePromise;
  }

  public terminate() {
    this.worker.terminate();
  }

  private handleEvent(event: Event) {
    this.onEvent(event);
  }

  private handleResponse(msg: ResponseMessage<Response>) {
    const pendingMessage = this.pendingMessages[msg.id];
    if (!pendingMessage) {
      this.logger.error(`Unknown response from Web Worker received. ${JSON.stringify(msg)}`);
      return;
    }
    delete this.pendingMessages[msg.id];
    if (msg.error) {
      pendingMessage.rej(msg.error);
    } else {
      // Response will likely include just "void", so falsy value
      // still should mean that it is resolved
      pendingMessage.res(msg.response!);
    }
  }
}
