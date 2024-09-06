import React from 'react';
import * as Api from '../api';
export type RegisterOutput = {
    type: 'rtp_stream';
} & RegisterRtpOutput;
export type RegisterRtpOutput = {
    /**
     * Depends on the value of the `transport_protocol` field:
     * - `udp` - An UDP port number that RTP packets will be sent to.
     * - `tcp_server` - A local TCP port number or a port range that LiveCompositor will listen for incoming connections.
     */
    port: Api.PortOrPortRange;
    /**
     * Only valid if `transport_protocol="udp"`. IP address where RTP packets should be sent to.
     */
    ip?: string | null;
    /**
     * (**default=`"udp"`**) Transport layer protocol that will be used to send RTP packets.
     */
    transportProtocol?: Api.TransportProtocol | null;
    video?: OutputRtpVideoOptions;
    audio?: OutputRtpAudioOptions;
};
export type OutputRtpVideoOptions = {
    /**
     * Output resolution in pixels.
     */
    resolution: Api.Resolution;
    /**
     * Defines when output stream should end if some of the input streams are finished. If output includes both audio and video streams, then EOS needs to be sent on both.
     */
    sendEosWhen?: OutputEndCondition;
    /**
     * Video encoder options.
     */
    encoder: VideoEncoderOptions;
    root: React.ReactElement;
};
export type VideoEncoderOptions = {
    type: 'ffmpeg_h264';
    /**
     * (**default=`"fast"`**) Preset for an encoder. See `FFmpeg` [docs](https://trac.ffmpeg.org/wiki/Encode/H.264#Preset) to learn more.
     */
    preset: Api.H264EncoderPreset;
    /**
     * Raw FFmpeg encoder options. See [docs](https://ffmpeg.org/ffmpeg-codecs.html) for more.
     */
    ffmpegOptions?: Api.VideoEncoderOptions['ffmpeg_options'];
};
export type OutputRtpAudioOptions = {
    /**
     * (**default="sum_clip"**) Specifies how audio should be mixed.
     */
    mixingStrategy?: Api.MixingStrategy | null;
    /**
     * Condition for termination of output stream based on the input streams states.
     */
    sendEosWhen?: OutputEndCondition | null;
    /**
     * Audio encoder options.
     */
    encoder: AudioEncoderOptions;
    /**
     * Initial audio mixer configuration for output.
     */
    initial: AudioInputsConfiguration;
};
export type AudioEncoderOptions = {
    type: 'opus';
    channels: Api.AudioChannels;
    /**
     * (**default="voip"**) Specifies preset for audio output encoder.
     */
    preset?: Api.OpusEncoderPreset;
    /**
     * (**default=`false`**) Specifies whether the stream use forward error correction.
     * It's specific for Opus codec.
     * For more information, check out [RFC](https://datatracker.ietf.org/doc/html/rfc6716#section-2.1.7).
     */
    forwardErrorCorrection?: boolean;
};
export type OutputEndCondition = {
    /**
     * Terminate output stream if any of the input streams from the list are finished.
     */
    anyOf: Api.InputId[];
} | {
    /**
     * Terminate output stream if all the input streams from the list are finished.
     */
    allOf: Api.InputId[];
} | {
    /**
     * Terminate output stream if any of the input streams ends. This includes streams added after the output was registered. In particular, output stream will **not be** terminated if no inputs were ever connected.
     */
    anyInput: boolean;
} | {
    /**
     * Terminate output stream if all the input streams finish. In particular, output stream will **be** terminated if no inputs were ever connected.
     */
    allInputs: boolean;
};
export interface AudioInputsConfiguration {
    inputs: InputAudio[];
}
export interface InputAudio {
    inputId: Api.InputId;
    /**
     * (**default=`1.0`**) float in `[0, 1]` range representing input volume
     */
    volume?: number | null;
}
