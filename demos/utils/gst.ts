import { exec } from "child_process";
import { SpawnPromise, spawn } from "./utils";

export async function gstStartPlayerAsync(ip: string, port: number): Promise<void> {
    const gstPipeline = [
        "-v",
        "tcpclientsrc",
        `host=${ip}`,
        `port=${port}`,
        "!",
        "\"application/x-rtp-stream\"",
        "!",
        "rtpstreamdepay",
        "!",
        "rtph264depay",
        "!",
        "decodebin",
        "!",
        "videoconvert",
        "!",
        "autovideosink"
    ];

    const plugins = [
        "tcpclientsrc",
        "rtpstreamdepay",
        "rtph264depay",
        "decodebin",
        "videoconvert",
        "autovideosink"
    ];
    checkGstPlugins(plugins);

    await spawn("gst-launch-1.0", gstPipeline, {
        stdio: "inherit",
        cwd: process.cwd()
    });
}

export function gstStreamWebcam(ip: string, port: number): SpawnPromise {
    const isMacOS = process.platform === 'darwin';

    const [gstWebcamSource, gstEncoder, gstEncoderOptions] = isMacOS ?
        ["avfvideosrc", "vtenc_h264", "bitrate=2000"] :
        ["v4l2src", "x264enc", "tune=zerolatency bitrate=2000 speed-preset=superfast"];

    const plugins = [
        gstWebcamSource,
        "videoconvert",
        gstEncoder,
        "rtph264pay",
        "udpsink",
    ];

   checkGstPlugins(plugins);

    const gstPipeline = [
        gstWebcamSource,
        "!",
        "videoconvert",
        "!",
        gstEncoder,
        gstEncoderOptions,
        "!",
        "rtph264pay",
        "config-interval=1",
        "pt=96",
        "!",
        "rtpstreampay",
        "!",
        "tcpclientsink",
        `host=${ip}`,
        `port=${port}`
    ];

    return spawn("gst-launch-1.0", gstPipeline, {
        stdio: "inherit",
        cwd: process.cwd()
    });
}

function checkGstPlugins(plugins: string[]) {
    plugins.forEach(plugin => {
        isGstPluginAvailable(plugin).then(isAvailable => {
            if (!isAvailable) {
                throw Error(`Gstreamer plugin: ${plugin} is not available.`);
            };
        });
    });
}

function isGstPluginAvailable(pluginName: string): Promise<boolean> {
    const command = `gst-inspect-1.0 ${pluginName}`;
    return new Promise((resolve, _reject) => {
        exec(command, (error, _stdout, _stderr) => {
            if (error) {
                resolve(false);
            } else {
                resolve(true);
            }
        });
    });
}
