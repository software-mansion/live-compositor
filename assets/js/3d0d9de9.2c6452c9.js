"use strict";(self.webpackChunkcompositor_live=self.webpackChunkcompositor_live||[]).push([[7180],{7346:(e,i,o)=>{o.r(i),o.d(i,{assets:()=>t,contentTitle:()=>l,default:()=>_,frontMatter:()=>s,metadata:()=>d,toc:()=>c});var n=o(5893),r=o(1151);const s={},l="Configuration",d={id:"deployment/configuration",title:"Configuration",description:"Environment variables",source:"@site/pages/deployment/configuration.md",sourceDirName:"deployment",slug:"/deployment/configuration",permalink:"/docs/deployment/configuration",draft:!1,unlisted:!1,tags:[],version:"current",frontMatter:{},sidebar:"sidebar",previous:{title:"Requirements",permalink:"/docs/deployment/requirements"},next:{title:"Example: AWS EC2",permalink:"/docs/deployment/aws-ec2"}},t={},c=[{value:"Environment variables",id:"environment-variables",level:2},{value:"<code>LIVE_COMPOSITOR_API_PORT</code>",id:"live_compositor_api_port",level:3},{value:"<code>LIVE_COMPOSITOR_INSTANCE_ID</code>",id:"live_compositor_instance_id",level:3},{value:"<code>LIVE_COMPOSITOR_OUTPUT_FRAMERATE</code>",id:"live_compositor_output_framerate",level:3},{value:"<code>LIVE_COMPOSITOR_OUTPUT_SAMPLE_RATE</code>",id:"live_compositor_output_sample_rate",level:3},{value:"<code>LIVE_COMPOSITOR_FORCE_GPU</code>",id:"live_compositor_force_gpu",level:3},{value:"<code>LIVE_COMPOSITOR_STREAM_FALLBACK_TIMEOUT_MS</code>",id:"live_compositor_stream_fallback_timeout_ms",level:3},{value:"<code>LIVE_COMPOSITOR_LOGGER_LEVEL</code>",id:"live_compositor_logger_level",level:3},{value:"<code>LIVE_COMPOSITOR_LOGGER_FORMAT</code>",id:"live_compositor_logger_format",level:3},{value:"<code>LIVE_COMPOSITOR_FFMPEG_LOGGER_LEVEL</code>",id:"live_compositor_ffmpeg_logger_level",level:3},{value:"<code>LIVE_COMPOSITOR_DOWNLOAD_DIR</code>",id:"live_compositor_download_dir",level:3},{value:"<code>LIVE_COMPOSITOR_WEB_RENDERER_ENABLE</code>",id:"live_compositor_web_renderer_enable",level:3},{value:"<code>LIVE_COMPOSITOR_WEB_RENDERER_GPU_ENABLE</code>",id:"live_compositor_web_renderer_gpu_enable",level:3},{value:"<code>LIVE_COMPOSITOR_OFFLINE_PROCESSING_ENABLE</code>",id:"live_compositor_offline_processing_enable",level:3},{value:"<code>LIVE_COMPOSITOR_AHEAD_OF_TIME_PROCESSING_ENABLE</code>",id:"live_compositor_ahead_of_time_processing_enable",level:3},{value:"<code>LIVE_COMPOSITOR_NEVER_DROP_OUTPUT_FRAMES</code>",id:"live_compositor_never_drop_output_frames",level:3},{value:"<code>LIVE_COMPOSITOR_RUN_LATE_SCHEDULED_EVENTS</code>",id:"live_compositor_run_late_scheduled_events",level:3},{value:"<code>LIVE_COMPOSITOR_REQUIRED_WGPU_FEATURES</code>",id:"live_compositor_required_wgpu_features",level:3},{value:"<code>LIVE_COMPOSITOR_INPUT_BUFFER_DURATION_MS</code>",id:"live_compositor_input_buffer_duration_ms",level:3}];function a(e){const i={a:"a",admonition:"admonition",code:"code",h1:"h1",h2:"h2",h3:"h3",li:"li",p:"p",ul:"ul",...(0,r.a)(),...e.components};return(0,n.jsxs)(n.Fragment,{children:[(0,n.jsx)(i.h1,{id:"configuration",children:"Configuration"}),"\n",(0,n.jsx)(i.h2,{id:"environment-variables",children:"Environment variables"}),"\n",(0,n.jsx)(i.h3,{id:"live_compositor_api_port",children:(0,n.jsx)(i.code,{children:"LIVE_COMPOSITOR_API_PORT"})}),"\n",(0,n.jsx)(i.p,{children:"API port. Defaults to 8081."}),"\n",(0,n.jsx)(i.h3,{id:"live_compositor_instance_id",children:(0,n.jsx)(i.code,{children:"LIVE_COMPOSITOR_INSTANCE_ID"})}),"\n",(0,n.jsxs)(i.p,{children:["ID that will be returned in ",(0,n.jsx)(i.code,{children:"GET /status"})," request. Can be used to identify if we are connecting to the correct compositor instance."]}),"\n",(0,n.jsx)(i.h3,{id:"live_compositor_output_framerate",children:(0,n.jsx)(i.code,{children:"LIVE_COMPOSITOR_OUTPUT_FRAMERATE"})}),"\n",(0,n.jsxs)(i.p,{children:["Output framerate for all output streams. This value can be a number or string in the ",(0,n.jsx)(i.code,{children:"NUM/DEN"})," format, where both ",(0,n.jsx)(i.code,{children:"NUM"})," and ",(0,n.jsx)(i.code,{children:"DEN"})," are unsigned integers. Defaults to ",(0,n.jsx)(i.code,{children:"30"})]}),"\n",(0,n.jsx)(i.h3,{id:"live_compositor_output_sample_rate",children:(0,n.jsx)(i.code,{children:"LIVE_COMPOSITOR_OUTPUT_SAMPLE_RATE"})}),"\n",(0,n.jsx)(i.p,{children:"Output sample rate for all output streams. This value has to be a number or string representing supported sample rate. Defaults to 48000."}),"\n",(0,n.jsx)(i.p,{children:"Supported sample rates are: 8000, 12000, 16000, 24000, 48000"}),"\n",(0,n.jsx)(i.h3,{id:"live_compositor_force_gpu",children:(0,n.jsx)(i.code,{children:"LIVE_COMPOSITOR_FORCE_GPU"})}),"\n",(0,n.jsxs)(i.p,{children:["If enabled, GPU will be required for rendering. If only CPU based adapters will be found then process will exit with an error. Defaults to ",(0,n.jsx)(i.code,{children:"false"}),"."]}),"\n",(0,n.jsx)(i.h3,{id:"live_compositor_stream_fallback_timeout_ms",children:(0,n.jsx)(i.code,{children:"LIVE_COMPOSITOR_STREAM_FALLBACK_TIMEOUT_MS"})}),"\n",(0,n.jsx)(i.p,{children:"A timeout that defines when the compositor should switch to fallback on the input stream that stopped sending frames. Defaults to 500."}),"\n",(0,n.jsx)(i.h3,{id:"live_compositor_logger_level",children:(0,n.jsx)(i.code,{children:"LIVE_COMPOSITOR_LOGGER_LEVEL"})}),"\n",(0,n.jsxs)(i.p,{children:["Logger level. Value can be defined as ",(0,n.jsx)(i.code,{children:"error"}),"/",(0,n.jsx)(i.code,{children:"warn"}),"/",(0,n.jsx)(i.code,{children:"info"}),"/",(0,n.jsx)(i.code,{children:"debug"}),"/",(0,n.jsx)(i.code,{children:"trace"}),"."]}),"\n",(0,n.jsxs)(i.p,{children:["This value also supports syntax for more detailed configuration. See ",(0,n.jsxs)(i.a,{href:"https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#example-syntax",children:[(0,n.jsx)(i.code,{children:"tracing-subscriber"})," crate documentation"]})," for more info."]}),"\n",(0,n.jsxs)(i.p,{children:["Defaults to ",(0,n.jsx)(i.code,{children:"info,wgpu_hal=warn,wgpu_core=warn"}),"."]}),"\n",(0,n.jsx)(i.h3,{id:"live_compositor_logger_format",children:(0,n.jsx)(i.code,{children:"LIVE_COMPOSITOR_LOGGER_FORMAT"})}),"\n",(0,n.jsx)(i.p,{children:"Logger format. Supported options:"}),"\n",(0,n.jsxs)(i.ul,{children:["\n",(0,n.jsx)(i.li,{children:(0,n.jsx)(i.code,{children:"json"})}),"\n",(0,n.jsx)(i.li,{children:(0,n.jsx)(i.code,{children:"compact"})}),"\n",(0,n.jsx)(i.li,{children:(0,n.jsx)(i.code,{children:"pretty"})}),"\n"]}),"\n",(0,n.jsx)(i.admonition,{type:"warning",children:(0,n.jsxs)(i.p,{children:["This option does not apply to logs produced by ",(0,n.jsx)(i.code,{children:"FFmpeg"})," or the embedded Chromium instance used for web rendering. Defaults to ",(0,n.jsx)(i.code,{children:"json"}),"."]})}),"\n",(0,n.jsx)(i.h3,{id:"live_compositor_ffmpeg_logger_level",children:(0,n.jsx)(i.code,{children:"LIVE_COMPOSITOR_FFMPEG_LOGGER_LEVEL"})}),"\n",(0,n.jsx)(i.p,{children:"Minimal log level that should be logged. Supported options:"}),"\n",(0,n.jsxs)(i.ul,{children:["\n",(0,n.jsxs)(i.li,{children:[(0,n.jsx)(i.code,{children:"error"})," - equivalent to FFmpeg's ",(0,n.jsx)(i.code,{children:"error, 16"})]}),"\n",(0,n.jsxs)(i.li,{children:[(0,n.jsx)(i.code,{children:"warn"})," - equivalent to FFmpeg's ",(0,n.jsx)(i.code,{children:"warning, 24"})]}),"\n",(0,n.jsxs)(i.li,{children:[(0,n.jsx)(i.code,{children:"info"})," - equivalent to FFmpeg's ",(0,n.jsx)(i.code,{children:"info, 32"})]}),"\n",(0,n.jsxs)(i.li,{children:[(0,n.jsx)(i.code,{children:"debug"})," - equivalent to FFmpeg's ",(0,n.jsx)(i.code,{children:"debug, 48"})]}),"\n"]}),"\n",(0,n.jsxs)(i.p,{children:["See ",(0,n.jsx)(i.code,{children:"-loglevel"})," option in ",(0,n.jsx)(i.a,{href:"https://ffmpeg.org/ffmpeg.html",children:"FFmpeg documentation"}),". Defaults to ",(0,n.jsx)(i.code,{children:"warn"}),"."]}),"\n",(0,n.jsx)(i.h3,{id:"live_compositor_download_dir",children:(0,n.jsx)(i.code,{children:"LIVE_COMPOSITOR_DOWNLOAD_DIR"})}),"\n",(0,n.jsx)(i.p,{children:"A path to a directory in which downloaded files will be stored. Uses the location provided by the OS if not set."}),"\n",(0,n.jsxs)(i.p,{children:["In this directory, an instance of the compositor will create a subdirectory named ",(0,n.jsx)(i.code,{children:"live-compositor-<random number>"}),". Downloaded temporary files will be stored there."]}),"\n",(0,n.jsx)(i.h3,{id:"live_compositor_web_renderer_enable",children:(0,n.jsx)(i.code,{children:"LIVE_COMPOSITOR_WEB_RENDERER_ENABLE"})}),"\n",(0,n.jsxs)(i.p,{children:["Enable web rendering capabilities. With this option disabled, you can not use ",(0,n.jsxs)(i.a,{href:"../api/components/WebView",children:[(0,n.jsx)(i.code,{children:"WebView"})," components"]})," or register ",(0,n.jsxs)(i.a,{href:"../api/renderers/web",children:[(0,n.jsx)(i.code,{children:"WebRenderer"})," instances"]}),"."]}),"\n",(0,n.jsxs)(i.p,{children:["Defaults to ",(0,n.jsx)(i.code,{children:"false"}),". Valid values: ",(0,n.jsx)(i.code,{children:"true"}),", ",(0,n.jsx)(i.code,{children:"false"}),", ",(0,n.jsx)(i.code,{children:"1"}),", ",(0,n.jsx)(i.code,{children:"0"}),"."]}),"\n",(0,n.jsx)(i.h3,{id:"live_compositor_web_renderer_gpu_enable",children:(0,n.jsx)(i.code,{children:"LIVE_COMPOSITOR_WEB_RENDERER_GPU_ENABLE"})}),"\n",(0,n.jsx)(i.p,{children:"Enable GPU support inside the embedded Chromium instance."}),"\n",(0,n.jsxs)(i.p,{children:["Defaults to ",(0,n.jsx)(i.code,{children:"true"}),". Valid values: ",(0,n.jsx)(i.code,{children:"true"}),", ",(0,n.jsx)(i.code,{children:"false"}),", ",(0,n.jsx)(i.code,{children:"1"}),", ",(0,n.jsx)(i.code,{children:"0"}),"."]}),"\n",(0,n.jsx)(i.h3,{id:"live_compositor_offline_processing_enable",children:(0,n.jsx)(i.code,{children:"LIVE_COMPOSITOR_OFFLINE_PROCESSING_ENABLE"})}),"\n",(0,n.jsxs)(i.p,{children:["If enabled, sets ",(0,n.jsx)(i.code,{children:"LIVE_COMPOSITOR_AHEAD_OF_TIME_PROCESSING_ENABLE"})," and ",(0,n.jsx)(i.code,{children:"LIVE_COMPOSITOR_NEVER_DROP_OUTPUT_FRAMES"})," options to ",(0,n.jsx)(i.code,{children:"true"}),". If those values are also defined then they take priority over this value."]}),"\n",(0,n.jsxs)(i.p,{children:["Defaults to ",(0,n.jsx)(i.code,{children:"false"}),". Valid values: ",(0,n.jsx)(i.code,{children:"true"}),", ",(0,n.jsx)(i.code,{children:"false"}),", ",(0,n.jsx)(i.code,{children:"1"}),", ",(0,n.jsx)(i.code,{children:"0"}),"."]}),"\n",(0,n.jsx)(i.h3,{id:"live_compositor_ahead_of_time_processing_enable",children:(0,n.jsx)(i.code,{children:"LIVE_COMPOSITOR_AHEAD_OF_TIME_PROCESSING_ENABLE"})}),"\n",(0,n.jsx)(i.p,{children:"If enabled, the LiveCompositor server will try to generate output frames/samples ahead of time if all inputs are available."}),"\n",(0,n.jsx)(i.p,{children:"When to enable this option:"}),"\n",(0,n.jsxs)(i.ul,{children:["\n",(0,n.jsx)(i.li,{children:"If you want to process input streams faster than in real time."}),"\n"]}),"\n",(0,n.jsxs)(i.p,{children:["Defaults to ",(0,n.jsx)(i.code,{children:"false"}),". Valid values: ",(0,n.jsx)(i.code,{children:"true"}),", ",(0,n.jsx)(i.code,{children:"false"}),", ",(0,n.jsx)(i.code,{children:"1"}),", ",(0,n.jsx)(i.code,{children:"0"}),"."]}),"\n",(0,n.jsx)(i.h3,{id:"live_compositor_never_drop_output_frames",children:(0,n.jsx)(i.code,{children:"LIVE_COMPOSITOR_NEVER_DROP_OUTPUT_FRAMES"})}),"\n",(0,n.jsx)(i.p,{children:"If enabled, the LiveCompositor server will not drop frames/samples from output stream even if rendering or encoding is not fast enough to process it in real time."}),"\n",(0,n.jsxs)(i.p,{children:["Defaults to ",(0,n.jsx)(i.code,{children:"false"}),". Valid values: ",(0,n.jsx)(i.code,{children:"true"}),", ",(0,n.jsx)(i.code,{children:"false"}),", ",(0,n.jsx)(i.code,{children:"1"}),", ",(0,n.jsx)(i.code,{children:"0"}),"."]}),"\n",(0,n.jsx)(i.h3,{id:"live_compositor_run_late_scheduled_events",children:(0,n.jsx)(i.code,{children:"LIVE_COMPOSITOR_RUN_LATE_SCHEDULED_EVENTS"})}),"\n",(0,n.jsxs)(i.p,{children:["Parts of the compositor API support a ",(0,n.jsx)(i.code,{children:"schedule_time_ms"})," field to apply certain actions at a specific time. If enabled, the event will still be executed, even if it was scheduled too late. Otherwise, it will be discarded."]}),"\n",(0,n.jsxs)(i.p,{children:["Defaults to ",(0,n.jsx)(i.code,{children:"false"}),". Valid values: ",(0,n.jsx)(i.code,{children:"true"}),", ",(0,n.jsx)(i.code,{children:"false"}),", ",(0,n.jsx)(i.code,{children:"1"}),", ",(0,n.jsx)(i.code,{children:"0"}),"."]}),"\n",(0,n.jsx)(i.h3,{id:"live_compositor_required_wgpu_features",children:(0,n.jsx)(i.code,{children:"LIVE_COMPOSITOR_REQUIRED_WGPU_FEATURES"})}),"\n",(0,n.jsxs)(i.p,{children:["Comma separated list of WebGPU features that need to be enabled. See ",(0,n.jsx)(i.a,{href:"https://docs.rs/wgpu/0.20.0/wgpu/struct.Features.html",children:"https://docs.rs/wgpu/0.20.0/wgpu/struct.Features.html"})," for a list of available options."]}),"\n",(0,n.jsxs)(i.p,{children:["Defaults to ",(0,n.jsx)(i.code,{children:"UNIFORM_BUFFER_AND_STORAGE_TEXTURE_ARRAY_NON_UNIFORM_INDEXING,SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING"}),"."]}),"\n",(0,n.jsxs)(i.p,{children:["Additionally, ",(0,n.jsx)(i.code,{children:"TEXTURE_BINDING_ARRAY"})," and ",(0,n.jsx)(i.code,{children:"PUSH_CONSTANTS"})," are also required, but this requirement can not be overwritten by changing this\nenvironment variable."]}),"\n",(0,n.jsx)(i.h3,{id:"live_compositor_input_buffer_duration_ms",children:(0,n.jsx)(i.code,{children:"LIVE_COMPOSITOR_INPUT_BUFFER_DURATION_MS"})}),"\n",(0,n.jsx)(i.p,{children:"Duration of an input buffer in milliseconds. New stream will not be processed until this buffer is filled, so this value controls the trade-off between\nlatency and resilience to stream delays."}),"\n",(0,n.jsx)(i.p,{children:"This value can be safely set to 0 if either:"}),"\n",(0,n.jsxs)(i.ul,{children:["\n",(0,n.jsxs)(i.li,{children:["All input streams are ",(0,n.jsx)(i.code,{children:"required"})]}),"\n",(0,n.jsxs)(i.li,{children:["All input streams are started with a specific ",(0,n.jsx)(i.code,{children:"offset_ms"})," and you are delivering them early enough for decoding to finish."]}),"\n"]}),"\n",(0,n.jsx)(i.admonition,{type:"warning",children:(0,n.jsx)(i.p,{children:"Increasing this value always increases the latency of the stream by the same amount."})}),"\n",(0,n.jsxs)(i.p,{children:["Defaults to ",(0,n.jsx)(i.code,{children:"80ms"})," (about 5 frames in 60 fps)."]})]})}function _(e={}){const{wrapper:i}={...(0,r.a)(),...e.components};return i?(0,n.jsx)(i,{...e,children:(0,n.jsx)(a,{...e})}):a(e)}},1151:(e,i,o)=>{o.d(i,{Z:()=>d,a:()=>l});var n=o(7294);const r={},s=n.createContext(r);function l(e){const i=n.useContext(s);return n.useMemo((function(){return"function"==typeof e?e(i):{...i,...e}}),[i,e])}function d(e){let i;return i=e.disableParentContext?"function"==typeof e.components?e.components(r):e.components||r:l(e.components),n.createElement(s.Provider,{value:i},e.children)}}}]);