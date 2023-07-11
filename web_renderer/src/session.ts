import { BrowserWindow, screen } from "electron";
import { Url } from "./common";

export class Session {
    public url: Url;
    public width: number;
    public height: number;
    public command_buffer: string;
    private last_frame: Buffer;
    private window: BrowserWindow;

    public constructor(url: Url, width: number, height: number) {
        this.url = url;
        this.width = width;
        this.height = height;
        this.last_frame = Buffer.from([]);
    }

    public run(): void {
        const factor = screen.getPrimaryDisplay().scaleFactor;
        this.window = new BrowserWindow({
            width: this.width / factor,
            height: this.height / factor,
            show: false,
            webPreferences: {
                offscreen: true,
                zoomFactor: 1 / factor
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
        
        const factor = screen.getPrimaryDisplay().scaleFactor;
        this.window.setSize(this.width / factor, this.height / factor);
    }

    public get frame(): Buffer {
        return this.last_frame;
    }
}