import { CompositorEvent, CompositorEventType } from 'live-compositor';
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

  public async connect(): Promise<void> {
    const ws = new WebSocket(this.url);

    let connected = false;
    await new Promise<void>((res, rej) => {
      ws.on('error', (err: any) => {
        if (connected) {
          console.log('error', err);
        } else {
          rej(err);
        }
      });

      ws.on('open', () => {
        connected = true;
        res();
      });

      ws.on('message', data => {
        const event = parseEvent(data);
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

  public registerEventListener(cb: (event: CompositorEvent) => void): void {
    this.listeners.add(cb);
  }
}

function parseEvent(data: WebSocket.RawData): unknown {
  try {
    return JSON.parse(data.toString());
  } catch {
    console.error(`Invalid event received ${data}`);
    return null;
  }
}
