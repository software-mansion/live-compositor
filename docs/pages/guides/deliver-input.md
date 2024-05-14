# How to deliver input streams

Live Compositor currently supports 2 input types:
- RTP
- MP4 (not supported in MembraneFramework plugin)

MP4 support is useful if you want to add some prerecorded assets, but for most streaming use cases RTP protocol will be a primary choice.

Our RTP implementation supports following codecs:
- H264 for video
- AAC and Opus for audio (AAC is not supported via MembraneFramework plugin)

### RTP over TCP vs UDP

RTP streams can be delivered over TCP or UDP. Depending on your use case different choice might make more sense, but in general we recommend using TCP if possible.

What to choose?
- If you are using MembraneFramework plugin all communication already happens over TCP. Currently, we do not support any way to configure it.
- Some of the popular multimedia tools do not support RTP over TCP e.g. FFmpeg.
- UDP should only be used for communication on localhost. We do not support retransmission or packet reordering, so if you use it in unreliable network it might lead to unexpected behavior.
- UDP does not have a congestion control, so if you are using any non-real time sources for inputs (e.g. streaming file with FFmpeg over RTP) then if you don't throttle the input it might lead high memory usage.

### What to use to stream over RTP?

#### MembraneFramework

If you are using MembraneFramework plugin you do not need anything else. Just connect appropriate input pads to the `LiveCompositor` bin.

#### FFmpeg

FFmpeg does not support RTP over TCP, so you are limited to UDP only.

Stream a H264 video from MP4 file (without transcoding) over RTP to `127.0.0.1:9001`.

```bash
ffmpeg -re -i path_to_file.mp4 -an -c:v copy -f rtp -bsf:v h264_mp4toannexb rtp://127.0.0.1:9001?rtcpport=9001
```

- `-re` - Limits speed of transfer to send data in real-time. Without this option entire file would be sent very quickly.
- `-c:v copy` - Copy video without transcoding.
- `-an` - Ignore audio stream.
- `-bsf:v h264_mp4toannexb` - Convert H264 to AnnexB bitstream format (no transcoding is necessary)


Stream a video from supported file formats (potentially with transcoding) over RTP to `127.0.0.1:9001`

```bash
ffmpeg -re -i path_to_file -an -c:v libx264 -f rtp rtp://127.0.0.1:9001?rtcpport=9001
```

Stream OPUS audio from supported file formats (potentially with transcoding) over RTP to `127.0.0.1:9001`

```bash
ffmpeg -re -i path_to_file -vn -c:a libopus -f rtp rtp://127.0.0.1:9001?rtcpport=9001
```

#### GStreamer

Stream audio and video from a MP4 file over RTP TCP:
- video to `127.0.0.1:9001`
- audio to `127.0.0.1:9002`

```bash
gst-launch-1.0 filesrc location=path_to_file.mp4 ! qtdemux name=demux \
    ! demux.video_0 ! queue ! h264parse ! rtph264pay config-interval=1 \
    ! "application/x-rtp,payload=96" ! rtpstreampay ! tcpclientsink host=127.0.0.1 port=9001  \
    ! demux.audio_0 ! queue ! decodebin ! audioconvert ! audioresample ! opusenc ! rtpopuspay \
    ! "application/x-rtp,payload=97" ! rtpstreampay ! tcpclientsink host=127.0.0.1 port=9002
```

- `"application/x-rtp,payload=97"`/`"application/x-rtp,payload=97"` - Compositor detects audio stream based on payload type. It needs 
to be set to 96 for video and 97 for audio.
- `decodebin ! audioconvert ! audioresample ! opusenc` - Transcode audio (most likely from AAC) from MP4 file into Opus. If you know 
what format is inside you can simplify this part.
- Compositor supports multiplexing audio and video stream on the same port, but it is hard to create a GStreamer pipeline that demuxes 
audio and video tracks converts them to RTP and multiplex them on the same port.

To stream over UDP replace `rtpstreampay ! tcpclientsink host=127.0.0.1 port=9002` with <nobr>`udpsink host=127.0.0.1 port=9002`</nobr>.
Additionally, you can use the same port for video and audio.
