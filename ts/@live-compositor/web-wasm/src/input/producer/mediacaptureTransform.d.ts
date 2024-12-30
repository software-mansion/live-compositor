interface MediaStreamTrackProcessor<T extends AudioData | VideoFrame> {
    readonly readable: ReadableStream<T>;
    readonly writableControl: WritableStream<MediaStreamTrackSignal>;
}

declare var MediaStreamTrackProcessor: {
    prototype: MediaStreamTrackProcessor<any>;

    new<T extends AudioData | VideoFrame>(init: MediaStreamTrackProcessorInit & { track: MediaStreamTrack }): MediaStreamTrackProcessor<T>;
};

interface MediaStreamTrackProcessorInit {
    track: MediaStreamTrack;
    maxBufferSize?: number | undefined;
}

interface MediaStreamTrackGenerator<T extends AudioData | VideoFrame> extends MediaStreamTrack {
    readonly writable: WritableStream<T>;
    readonly readableControl: ReadableStream<MediaStreamTrackSignal>;
}


type MediaStreamTrackSignalType = "request-frame";

interface MediaStreamTrackSignal {
    signalType: MediaStreamTrackSignalType;
}
