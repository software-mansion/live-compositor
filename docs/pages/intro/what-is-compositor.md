# What is Live Compositor?

Live Compositor is an engine for applying effects to videos and for combining multiple videos together.
On a very basic level, it is an application which exposes an HTTP API.
The API allows you to specify where to get input videos, and how to modify and compose them together.
The resulting outputs can then be written to a file or streamed to a separate service.

## The TypeScript SDK

The TypeScript SDK is the recommended way to start using the compositor right now.
It is a library which allows controlling how the compositor manipulates videos in a way which is simpler to wrap your head around than raw HTTP requests.
It allows writing React-based code, which then controlls how the videos are processed.

