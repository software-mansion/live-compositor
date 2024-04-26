# Introduction

Live compositor is an application for real-time video processing/transforming/composing, providing simple, language-agnostic API for live video rendering. It targets real-time use cases, like video conferencing, live-streaming, or broadcasting (e.g. with [WebRTC](https://en.wikipedia.org/wiki/WebRTC) / [HLS](https://en.wikipedia.org/wiki/HTTP_Live_Streaming) / [RTMP](https://en.wikipedia.org/wiki/Real-Time_Messaging_Protocol)).

Compositor is a standalone media server. API is language, and technology agnostic. Video input and output streams are sent via RTP and all the configuration is done over HTTP [API](https://github.com/membraneframework/live_compositor/wiki/API-%E2%80%90-general). At some point, we plan to provide SDKs for specific languages, but you should be able to do everything SDK provides just using API.
