import { BrowserWindow, screen } from "electron";
import { Url } from "./common";
import { Resolution } from "./schemas";

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
        this.window.webContents.on("paint", (_event, _dirty, img) => {
            this.last_frame = img.toJPEG(90);
        });
        this.window.webContents.setFrameRate(60);
    }

    public get frame(): Buffer {
        return this.last_frame;
    }
}
