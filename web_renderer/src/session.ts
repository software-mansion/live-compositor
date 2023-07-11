import { BrowserWindow } from "electron";
import { Url } from "./common";

export class Session {
    public last_frame: Buffer;
    public url: Url;
    public width: number;
    public height: number;
    public command_buffer: string;
    private window: BrowserWindow;

    public constructor(url: Url, width: number, height: number) {
        this.url = url;
        this.width = width;
        this.height = height;
        this.last_frame = Buffer.alloc(1);
    }

    public run(): void {
        this.window = new BrowserWindow({
            width: this.width,
            height: this.height,
            show: false,
            webPreferences: {
                offscreen: true
            }
        });

        this.window.loadURL(this.url);
        this.window.webContents.on("paint", (_event, _dirty, img) => {
            this.last_frame = img.toJPEG(100);
        });
        this.window.webContents.setFrameRate(60);
    }

    public resize(width: number, height: number): void {
        this.width = width;
        this.height = height;

        this.window.setSize(this.width, this.height);
    }
}