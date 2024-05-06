"use strict";(self.webpackChunkcompositor_live=self.webpackChunkcompositor_live||[]).push([[1872],{6914:(e,n,t)=>{t.r(n),t.d(n,{assets:()=>l,contentTitle:()=>r,default:()=>u,frontMatter:()=>o,metadata:()=>d,toc:()=>c});var i=t(5893),s=t(1151);const o={},r=void 0,d={id:"api/generated/output-OutputStream",title:"output-OutputStream",description:"OutputStream",source:"@site/pages/api/generated/output-OutputStream.md",sourceDirName:"api/generated",slug:"/api/generated/output-OutputStream",permalink:"/docs/api/generated/output-OutputStream",draft:!1,unlisted:!1,tags:[],version:"current",frontMatter:{}},l={},c=[{value:"OutputStream",id:"outputstream",level:2},{value:"Properties",id:"properties",level:4},{value:"OutputRtpVideoOptions",id:"outputrtpvideooptions",level:2},{value:"Properties",id:"properties-1",level:4},{value:"OutputRtpAudioOptions",id:"outputrtpaudiooptions",level:2},{value:"Properties",id:"properties-2",level:4},{value:"OutputEndCondition",id:"outputendcondition",level:2},{value:"Properties",id:"properties-3",level:4},{value:"VideoEncoderOptions",id:"videoencoderoptions",level:2},{value:"Properties",id:"properties-4",level:4},{value:"AudioEncoderOptions",id:"audioencoderoptions",level:2},{value:"Properties (<code>type: &quot;opus&quot;</code>)",id:"properties-type-opus",level:4},{value:"InputAudio",id:"inputaudio",level:2},{value:"Properties",id:"properties-5",level:4}];function p(e){const n={a:"a",code:"code",h2:"h2",h4:"h4",li:"li",p:"p",pre:"pre",strong:"strong",ul:"ul",...(0,s.a)(),...e.components};return(0,i.jsxs)(i.Fragment,{children:[(0,i.jsx)(n.h2,{id:"outputstream",children:"OutputStream"}),"\n",(0,i.jsx)(n.pre,{children:(0,i.jsx)(n.code,{className:"language-typescript",children:'type OutputStream = {\n  port: string | u16;\n  ip?: string;\n  transport_protocol?: "udp" | "tcp_server";\n  video?: OutputRtpVideoOptions;\n  audio?: OutputRtpAudioOptions;\n}\n'})}),"\n",(0,i.jsx)(n.h4,{id:"properties",children:"Properties"}),"\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"port"})," - Depends on the value of the ",(0,i.jsx)(n.code,{children:"transport_protocol"})," field:","\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"udp"})," - An UDP port number that RTP packets will be sent to."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"tcp_server"})," - A local TCP port number or a port range that LiveCompositor will listen for incoming connections."]}),"\n"]}),"\n"]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"ip"})," - Only valid if ",(0,i.jsx)(n.code,{children:'transport_protocol="udp"'}),". IP address where RTP packets should be sent to."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"transport_protocol"})," - (",(0,i.jsxs)(n.strong,{children:["default=",(0,i.jsx)(n.code,{children:'"udp"'})]}),") Transport layer protocol that will be used to send RTP packets.","\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:'"udp"'})," - UDP protocol."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:'"tcp_server"'})," - TCP protocol where LiveCompositor is the server side of the connection."]}),"\n"]}),"\n"]}),"\n"]}),"\n",(0,i.jsx)(n.h2,{id:"outputrtpvideooptions",children:"OutputRtpVideoOptions"}),"\n",(0,i.jsx)(n.pre,{children:(0,i.jsx)(n.code,{className:"language-typescript",children:"type OutputRtpVideoOptions = {\n  resolution: {\n    width: u32;\n    height: u32;\n  };\n  send_eos_when?: OutputEndCondition;\n  encoder: VideoEncoderOptions;\n  initial: {\n    root: Component;\n  };\n}\n"})}),"\n",(0,i.jsx)(n.h4,{id:"properties-1",children:"Properties"}),"\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"resolution"})," - Output resolution in pixels."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"send_eos_when"})," - Defines when output stream should end if some of the input streams are finished. If output includes both audio and video streams, then EOS needs to be sent on both."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"encoder"})," - Video encoder options."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"initial"})," - Root of a component tree/scene that should be rendered for the output. Use ",(0,i.jsxs)(n.a,{href:"/docs/api/routes#update-output",children:[(0,i.jsx)(n.code,{children:"update_output"})," request"]})," to update this value after registration. ",(0,i.jsx)(n.a,{href:"/docs/concept/component",children:"Learn more"}),"."]}),"\n"]}),"\n",(0,i.jsx)(n.h2,{id:"outputrtpaudiooptions",children:"OutputRtpAudioOptions"}),"\n",(0,i.jsx)(n.pre,{children:(0,i.jsx)(n.code,{className:"language-typescript",children:'type OutputRtpAudioOptions = {\n  mixing_strategy?: "sum_clip" | "sum_scale";\n  send_eos_when?: OutputEndCondition;\n  encoder: AudioEncoderOptions;\n  initial: {\n    inputs: InputAudio[];\n  };\n}\n'})}),"\n",(0,i.jsx)(n.h4,{id:"properties-2",children:"Properties"}),"\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"mixing_strategy"})," - (",(0,i.jsx)(n.strong,{children:'default="sum_clip"'}),") Specifies how audio should be mixed.","\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:'"sum_clip"'})," - Firstly, input samples are summed. If the result is outside the i16 PCM range, it gets clipped."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:'"sum_scale"'})," - Firstly, input samples are summed. If the result is outside the i16 PCM range,\nnearby summed samples are scaled down by factor, such that the summed wave is in the i16 PCM range."]}),"\n"]}),"\n"]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"send_eos_when"})," - Condition for termination of output stream based on the input streams states."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"encoder"})," - Audio encoder options."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"initial"})," - Initial audio mixer configuration for output."]}),"\n"]}),"\n",(0,i.jsx)(n.h2,{id:"outputendcondition",children:"OutputEndCondition"}),"\n",(0,i.jsx)(n.pre,{children:(0,i.jsx)(n.code,{className:"language-typescript",children:"type OutputEndCondition = {\n  any_of?: string[];\n  all_of?: string[];\n  any_input?: bool;\n  all_inputs?: bool;\n}\n"})}),"\n",(0,i.jsx)(n.p,{children:"This type defines when end of an input stream should trigger end of the output stream. Only one of those fields can be set at the time.\nUnless specified otherwise the input stream is considered finished/ended when:"}),"\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsx)(n.li,{children:"TCP connection was dropped/closed."}),"\n",(0,i.jsxs)(n.li,{children:["RTCP Goodbye packet (",(0,i.jsx)(n.code,{children:"BYE"}),") was received."]}),"\n",(0,i.jsx)(n.li,{children:"Mp4 track has ended."}),"\n",(0,i.jsx)(n.li,{children:"Input was unregistered already (or never registered)."}),"\n"]}),"\n",(0,i.jsx)(n.h4,{id:"properties-3",children:"Properties"}),"\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"any_of"})," - Terminate output stream if any of the input streams from the list are finished."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"all_of"})," - Terminate output stream if all the input streams from the list are finished."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"any_input"})," - Terminate output stream if any of the input streams ends. This includes streams added after the output was registered. In particular, output stream will ",(0,i.jsx)(n.strong,{children:"not be"})," terminated if no inputs were ever connected."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"all_inputs"})," - Terminate output stream if all the input streams finish. In particular, output stream will ",(0,i.jsx)(n.strong,{children:"be"})," terminated if no inputs were ever connected."]}),"\n"]}),"\n",(0,i.jsx)(n.h2,{id:"videoencoderoptions",children:"VideoEncoderOptions"}),"\n",(0,i.jsx)(n.pre,{children:(0,i.jsx)(n.code,{className:"language-typescript",children:'type VideoEncoderOptions = \n  | {\n      type: "ffmpeg_h264";\n      preset: \n        | "ultrafast"\n        | "superfast"\n        | "veryfast"\n        | "faster"\n        | "fast"\n        | "medium"\n        | "slow"\n        | "slower"\n        | "veryslow"\n        | "placebo";\n      ffmpeg_options?: Map<string, string>;\n    }\n'})}),"\n",(0,i.jsx)(n.h4,{id:"properties-4",children:"Properties"}),"\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"preset"})," - (",(0,i.jsxs)(n.strong,{children:["default=",(0,i.jsx)(n.code,{children:'"fast"'})]}),") Preset for an encoder. See ",(0,i.jsx)(n.code,{children:"FFmpeg"})," ",(0,i.jsx)(n.a,{href:"https://trac.ffmpeg.org/wiki/Encode/H.264#Preset",children:"docs"})," to learn more."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"ffmpeg_options"})," - Raw FFmpeg encoder options. See ",(0,i.jsx)(n.a,{href:"https://ffmpeg.org/ffmpeg-codecs.html",children:"docs"})," for more."]}),"\n"]}),"\n",(0,i.jsx)(n.h2,{id:"audioencoderoptions",children:"AudioEncoderOptions"}),"\n",(0,i.jsx)(n.pre,{children:(0,i.jsx)(n.code,{className:"language-typescript",children:'type AudioEncoderOptions = \n  | {\n      type: "opus";\n      channels: "mono" | "stereo";\n      preset?: "quality" | "voip" | "lowest_latency";\n      forward_error_correction?: bool;\n    }\n'})}),"\n",(0,i.jsxs)(n.h4,{id:"properties-type-opus",children:["Properties (",(0,i.jsx)(n.code,{children:'type: "opus"'}),")"]}),"\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"channels"}),"\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:'"mono"'})," - Mono audio (single channel)."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:'"stereo"'})," - Stereo audio (two channels)."]}),"\n"]}),"\n"]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"preset"})," - (",(0,i.jsx)(n.strong,{children:'default="voip"'}),") Specifies preset for audio output encoder.","\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:'"quality"'})," - Best for broadcast/high-fidelity application where the decoded audio\nshould be as close as possible to the input."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:'"voip"'})," - Best for most VoIP/videoconference applications where listening quality\nand intelligibility matter most."]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:'"lowest_latency"'})," - Only use when lowest-achievable latency is what matters most."]}),"\n"]}),"\n"]}),"\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"forward_error_correction"})," - (",(0,i.jsxs)(n.strong,{children:["default=",(0,i.jsx)(n.code,{children:"false"})]}),") Specifies whether the stream use forward error correction.\nIt's specific for Opus codec.\nFor more information, check out ",(0,i.jsx)(n.a,{href:"https://datatracker.ietf.org/doc/html/rfc6716#section-2.1.7",children:"RFC"}),"."]}),"\n"]}),"\n",(0,i.jsx)(n.h2,{id:"inputaudio",children:"InputAudio"}),"\n",(0,i.jsx)(n.pre,{children:(0,i.jsx)(n.code,{className:"language-typescript",children:"type InputAudio = {\n  input_id: string;\n  volume?: f32;\n}\n"})}),"\n",(0,i.jsx)(n.h4,{id:"properties-5",children:"Properties"}),"\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsxs)(n.li,{children:[(0,i.jsx)(n.code,{children:"volume"})," - (",(0,i.jsxs)(n.strong,{children:["default=",(0,i.jsx)(n.code,{children:"1.0"})]}),") float in ",(0,i.jsx)(n.code,{children:"[0, 1]"})," range representing input volume"]}),"\n"]})]})}function u(e={}){const{wrapper:n}={...(0,s.a)(),...e.components};return n?(0,i.jsx)(n,{...e,children:(0,i.jsx)(p,{...e})}):p(e)}},1151:(e,n,t)=>{t.d(n,{Z:()=>d,a:()=>r});var i=t(7294);const s={},o=i.createContext(s);function r(e){const n=i.useContext(o);return i.useMemo((function(){return"function"==typeof e?e(n):{...n,...e}}),[n,e])}function d(e){let n;return n=e.disableParentContext?"function"==typeof e.components?e.components(s):e.components||s:r(e.components),i.createElement(o.Provider,{value:n},e.children)}}}]);