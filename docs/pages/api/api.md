# API

Compositor exposes HTTP API. After spawning a compositor process the first request has to be [an init request](https://github.com/membraneframework/video_compositor/wiki/API-%E2%80%90-general#init). After the compositor is initialized you can configure the processing pipeline using `RegisterInputStream`, `RegisterOutputStre`, `UpdateScene`, and other requests. When you are ready to start receiving the output stream from the compositor you can send a [`Start`](https://github.com/membraneframework/video_compositor/wiki/API-%E2%80%90-general#start) request.

API is served by default on port `8001`, but it can be configured via `MEMBRANE_VIDEO_COMPOSITOR_API_PORT` environment variable.

After the compositor is started with a [`Start`](https://github.com/membraneframework/video_compositor/wiki/API-%E2%80%90-general#start) request you can keep using `RegisterInputStream`, `RegisterOutputStream`, `UpdateScene` and others to modify the pipeline in real-time.


