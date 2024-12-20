import { type Logger } from 'pino';
import WebSocket from 'ws';

export class WebSocketConnection {
  private url: string;
  private listeners: Set<(event: object) => void>;
  // @ts-expect-error: unused if we don't send messages via WebSocket
  private ws: WebSocket | null = null;

  constructor(url: string) {
    this.url = url;
    this.listeners = new Set();
  }

  public async connect(logger: Logger): Promise<void> {
    const ws = new WebSocket(this.url);

    let connected = false;
    await new Promise<void>((res, rej) => {
      ws.on('error', (err: any) => {
        if (connected) {
          logger.error(err, 'WebSocket error');
        } else {
          rej(err);
        }
      });

      ws.on('open', () => {
        connected = true;
        res();
      });

      ws.on('message', data => {
        const event = parseEvent(data, logger);
        if (event) {
          for (const listener of this.listeners) {
            listener(event);
          }
        }
      });

      ws.on('close', () => {
        this.ws = null;
      });
    });
    this.ws = ws;
  }

  public registerEventListener(cb: (event: object) => void): void {
    this.listeners.add(cb);
  }
}

function parseEvent(data: WebSocket.RawData, logger: Logger): unknown {
  try {
    return JSON.parse(data.toString());
  } catch (err: any) {
    logger.warn(err, `Invalid event received ${data}`);
    return null;
  }
}
