import { BrowserWindow, screen } from "electron";
import { Resolution, Url } from "./common";

export class Session {
    public url: Url;
    public resolution: Resolution;
    private last_frame: Buffer;
    private window: BrowserWindow;

    public constructor(url: Url, resolution: Resolution) {
        this.url = url;
        this.resolution = resolution;
        this.last_frame = Buffer.from([]);
    }

    public run(): void {
        const factor = screen.getPrimaryDisplay().scaleFactor;
        this.window = new BrowserWindow({
            width: this.resolution.width / factor,
            height: this.resolution.height / factor,
            show: false,
            frame: false,
            webPreferences: {
                offscreen: true,
                zoomFactor: 1 / factor,
                backgroundThrottling: false,
            }
        });

        this.window.loadURL(this.url);
        let start = Date.now();
        this.window.webContents.on("paint", (_event, _dirty, img) => {
            this.last_frame = img.toBitmap();
            // let now = Date.now();
            // console.log(now - start);
            // start = now;
        });
        this.window.webContents.setFrameRate(60);
    }

    public resize(resolution: Resolution): void {
        this.resolution = resolution;
        
        const factor = screen.getPrimaryDisplay().scaleFactor;
        this.window.setSize(this.resolution.width / factor, this.resolution.height / factor);
    }

    public get frame(): Buffer {
        // console.log(this.last_frame.length);
        return this.last_frame;
    }
}
