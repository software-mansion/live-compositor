"use strict";(self.webpackChunkcompositor_live=self.webpackChunkcompositor_live||[]).push([[53],{1109:e=>{e.exports=JSON.parse('{"pluginId":"default","version":"current","label":"Next","banner":null,"badge":false,"noIndex":false,"className":"docs-version-current","isLast":true,"docsSidebars":{"sidebar":[{"type":"link","label":"Introduction","href":"/docs/intro","docId":"intro","unlisted":false},{"label":"Get started","type":"category","items":[{"type":"link","label":"Elixir","href":"/docs/get-started/elixir","docId":"get-started/elixir","unlisted":false},{"type":"link","label":"Node.js","href":"/docs/get-started/node","docId":"get-started/node","unlisted":false}],"collapsed":true,"collapsible":true,"href":"/docs/get-started"},{"type":"category","label":"API Reference","collapsible":false,"items":[{"type":"link","label":"HTTP Routes","href":"/docs/api/routes","docId":"api/routes","unlisted":false},{"type":"link","label":"Input/Output streams","href":"/docs/api/io","docId":"api/io","unlisted":false},{"type":"category","label":"Components","collapsible":false,"description":"Basic blocks used to define a scene.","items":[{"type":"link","label":"Image","href":"/docs/api/components/Image","docId":"api/components/Image","unlisted":false},{"type":"link","label":"InputStream","href":"/docs/api/components/InputStream","docId":"api/components/InputStream","unlisted":false},{"type":"link","label":"Shader","href":"/docs/api/components/Shader","docId":"api/components/Shader","unlisted":false},{"type":"link","label":"Text","href":"/docs/api/components/Text","docId":"api/components/Text","unlisted":false},{"type":"link","label":"Tiles","href":"/docs/api/components/Tiles","docId":"api/components/Tiles","unlisted":false},{"type":"link","label":"View","href":"/docs/api/components/View","docId":"api/components/View","unlisted":false},{"type":"link","label":"WebView","href":"/docs/api/components/WebView","docId":"api/components/WebView","unlisted":false}],"collapsed":false},{"type":"category","label":"Renderers","collapsible":false,"description":"Resources that need to be registered first before they can be used.","items":[{"type":"link","label":"Shader","href":"/docs/api/renderers/shader","docId":"api/renderers/shader","unlisted":false}],"collapsed":false}],"collapsed":false,"href":"/docs/category/api-reference"}]},"docs":{"api/api":{"id":"api/api","title":"API","description":"Compositor exposes HTTP API. After spawning a compositor process the first request has to be an init request. After the compositor is initialized you can configure the processing pipeline using RegisterInputStream, RegisterOutputStre, UpdateScene, and other requests. When you are ready to start receiving the output stream from the compositor you can send a Start request."},"api/components/Image":{"id":"api/components/Image","title":"Image","description":"Properties","sidebar":"sidebar"},"api/components/InputStream":{"id":"api/components/InputStream","title":"InputStream","description":"Component representing incoming RTP stream. Specific streams can be identified by an input_id that was part of a RegisterInputStream request.","sidebar":"sidebar"},"api/components/Shader":{"id":"api/components/Shader","title":"Shader","description":"Properties","sidebar":"sidebar"},"api/components/Text":{"id":"api/components/Text","title":"Text","description":"Properties","sidebar":"sidebar"},"api/components/Tiles":{"id":"api/components/Tiles","title":"Tiles","description":"Properties","sidebar":"sidebar"},"api/components/View":{"id":"api/components/View","title":"View","description":"Properties","sidebar":"sidebar"},"api/components/WebView":{"id":"api/components/WebView","title":"WebView","description":"WebView component renders a website using Chromium.","sidebar":"sidebar"},"api/io":{"id":"api/io","title":"Streams","description":"Configuration and delivery of input and output streams.","sidebar":"sidebar"},"api/renderers/shader":{"id":"api/renderers/shader","title":"Shader","description":"","sidebar":"sidebar"},"api/routes":{"id":"api/routes","title":"Routes","description":"API routes to configure the compositor.","sidebar":"sidebar"},"get-started":{"id":"get-started","title":"Get started","description":"To familiarize yourself with a compositor you can start with examples directory. It includes example applications that use ffmpeg and ffplay to simulate compositor inputs and outputs. For a more detailed explanation of some of the terms used in this documentation, you can check this page.","sidebar":"sidebar"},"get-started/elixir":{"id":"get-started/elixir","title":"Elixir","description":"See Membrane Live Compositor plugin for more.","sidebar":"sidebar"},"get-started/node":{"id":"get-started/node","title":"Node.js","description":"See github.com/membraneframework-labs/rtconvideocompositorworkshops for example usage.","sidebar":"sidebar"},"intro":{"id":"intro","title":"Introduction","description":"Live compositor is an application for real-time video processing/transforming/composing, providing simple, language-agnostic API for live video rendering. It targets real-time use cases, like video conferencing, live-streaming, or broadcasting (e.g. with WebRTC / HLS / RTMP).","sidebar":"sidebar"}}}')}}]);