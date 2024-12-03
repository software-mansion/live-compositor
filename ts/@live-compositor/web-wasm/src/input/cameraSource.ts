import { Framerate } from "../compositor";
import InputSource, { InputSourceCallbacks, VideoChunk } from "./source";

export default class CameraSource implements InputSource {
  private callbacks?: InputSourceCallbacks;

  public constructor() {

  }

  public init(): Promise<void> {
    throw new Error("Method not implemented.");
  }

  public start(): void {
    throw new Error("Method not implemented.");
  }

  public registerCallbacks(callbacks: InputSourceCallbacks): void {
    this.callbacks = callbacks;
  }

  public isFinished(): boolean {
    throw new Error("Method not implemented.");
  }

  public getFramerate(): Framerate | undefined {
    throw new Error("Method not implemented.");
  }

  public nextChunk(): VideoChunk | undefined {
    throw new Error("Method not implemented.");
  }

  public peekChunk(): VideoChunk | undefined {
    throw new Error("Method not implemented.");
  }
}
