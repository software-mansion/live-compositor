"use strict";(self.webpackChunkcompositor_live=self.webpackChunkcompositor_live||[]).push([[5695],{7698:(e,n,t)=>{t.r(n),t.d(n,{assets:()=>c,contentTitle:()=>d,default:()=>p,frontMatter:()=>r,metadata:()=>o,toc:()=>a});var s=t(5893),i=t(1151);const r={description:"WebSocket events"},d="Events",o={id:"api/events",title:"Events",description:"WebSocket events",source:"@site/pages/api/events.md",sourceDirName:"api",slug:"/api/events",permalink:"/docs/api/events",draft:!1,unlisted:!1,tags:[],version:"current",frontMatter:{description:"WebSocket events"},sidebar:"sidebar",previous:{title:"Routes",permalink:"/docs/api/routes"},next:{title:"InputStream",permalink:"/docs/api/components/InputStream"}},c={},a=[{value:"<code>VIDEO_INPUT_DELIVERED</code>",id:"video_input_delivered",level:3},{value:"<code>VIDEO_INPUT_PLAYING</code>",id:"video_input_playing",level:3},{value:"<code>VIDEO_INPUT_EOS</code>",id:"video_input_eos",level:3},{value:"<code>AUDIO_INPUT_DELIVERED</code>",id:"audio_input_delivered",level:3},{value:"<code>AUDIO_INPUT_PLAYING</code>",id:"audio_input_playing",level:3},{value:"<code>AUDIO_INPUT_EOS</code>",id:"audio_input_eos",level:3},{value:"<code>OUTPUT_DONE</code>",id:"output_done",level:3}];function l(e){const n={a:"a",code:"code",h1:"h1",h3:"h3",li:"li",p:"p",pre:"pre",ul:"ul",...(0,i.a)(),...e.components};return(0,s.jsxs)(s.Fragment,{children:[(0,s.jsx)(n.h1,{id:"events",children:"Events"}),"\n",(0,s.jsx)(n.p,{children:"LiveCompositor is using WebSocket connection to send events to the connected clients. Supported events are listed below."}),"\n",(0,s.jsx)(n.h3,{id:"video_input_delivered",children:(0,s.jsx)(n.code,{children:"VIDEO_INPUT_DELIVERED"})}),"\n",(0,s.jsx)(n.pre,{children:(0,s.jsx)(n.code,{className:"language-typescript",children:'type Event = {\n  type: "VIDEO_INPUT_DELIVERED";\n  input_id: string;\n}\n'})}),"\n",(0,s.jsxs)(n.p,{children:["The compositor received the input, and the first frames of that input are ready to be used. If you want to ensure that some inputs are ready before you send the ",(0,s.jsx)(n.a,{href:"/docs/api/routes#start-request",children:(0,s.jsx)(n.code,{children:"start"})})," request, you can wait for those events for specific inputs."]}),"\n",(0,s.jsx)(n.h3,{id:"video_input_playing",children:(0,s.jsx)(n.code,{children:"VIDEO_INPUT_PLAYING"})}),"\n",(0,s.jsx)(n.pre,{children:(0,s.jsx)(n.code,{className:"language-typescript",children:'type Event = {\n  type: "VIDEO_INPUT_PLAYING";\n  input_id: string;\n}\n'})}),"\n",(0,s.jsxs)(n.p,{children:["The compositor received the input and is using the first frame for rendering. This event will not be sent before the ",(0,s.jsx)(n.a,{href:"/docs/api/routes#start-request",children:(0,s.jsx)(n.code,{children:"start"})})," request."]}),"\n",(0,s.jsxs)(n.p,{children:["This event is usually sent at the same time as ",(0,s.jsx)(n.code,{children:"VIDEO_INPUT_DELIVERED"})," except for 2 cases:"]}),"\n",(0,s.jsxs)(n.ul,{children:["\n",(0,s.jsxs)(n.li,{children:["Before ",(0,s.jsx)(n.a,{href:"/docs/api/routes#start-request",children:(0,s.jsx)(n.code,{children:"start"})})," request."]}),"\n",(0,s.jsxs)(n.li,{children:["If input has the ",(0,s.jsx)(n.code,{children:"offset_ms"})," field defined."]}),"\n"]}),"\n",(0,s.jsx)(n.h3,{id:"video_input_eos",children:(0,s.jsx)(n.code,{children:"VIDEO_INPUT_EOS"})}),"\n",(0,s.jsx)(n.pre,{children:(0,s.jsx)(n.code,{className:"language-typescript",children:'type Event = {\n  type: "VIDEO_INPUT_EOS";\n  input_id: string;\n}\n'})}),"\n",(0,s.jsxs)(n.p,{children:["The input stream has ended and all the frames were already processed.\nIt's not emitted on ",(0,s.jsx)(n.a,{href:"/docs/api/routes#unregister-input",children:(0,s.jsx)(n.code,{children:"input unregister"})}),"."]}),"\n",(0,s.jsx)(n.h3,{id:"audio_input_delivered",children:(0,s.jsx)(n.code,{children:"AUDIO_INPUT_DELIVERED"})}),"\n",(0,s.jsx)(n.pre,{children:(0,s.jsx)(n.code,{className:"language-typescript",children:'type Event = {\n  type: "AUDIO_INPUT_DELIVERED";\n  input_id: string;\n}\n'})}),"\n",(0,s.jsxs)(n.p,{children:["The compositor received the input, and the first samples on that input are ready to be used. If you want to ensure that some inputs are ready before you send the ",(0,s.jsx)(n.a,{href:"/docs/api/routes#start-request",children:(0,s.jsx)(n.code,{children:"start"})})," request, you can wait for those events for specific inputs."]}),"\n",(0,s.jsx)(n.h3,{id:"audio_input_playing",children:(0,s.jsx)(n.code,{children:"AUDIO_INPUT_PLAYING"})}),"\n",(0,s.jsx)(n.pre,{children:(0,s.jsx)(n.code,{className:"language-typescript",children:'type Event = {\n  type: "AUDIO_INPUT_PLAYING";\n  input_id: string;\n}\n'})}),"\n",(0,s.jsxs)(n.p,{children:["The compositor received the input and is using the first samples for rendering. This event will not be sent before the ",(0,s.jsx)(n.a,{href:"/docs/api/routes#start-request",children:(0,s.jsx)(n.code,{children:"start"})})," request."]}),"\n",(0,s.jsxs)(n.p,{children:["This event is usually sent at the same time as ",(0,s.jsx)(n.code,{children:"AUDIO_INPUT_DELIVERED"})," except for 2 cases:"]}),"\n",(0,s.jsxs)(n.ul,{children:["\n",(0,s.jsxs)(n.li,{children:["Before ",(0,s.jsx)(n.a,{href:"/docs/api/routes#start-request",children:(0,s.jsx)(n.code,{children:"start"})})," request."]}),"\n",(0,s.jsxs)(n.li,{children:["If input has the ",(0,s.jsx)(n.code,{children:"offset_ms"})," field defined."]}),"\n"]}),"\n",(0,s.jsx)(n.h3,{id:"audio_input_eos",children:(0,s.jsx)(n.code,{children:"AUDIO_INPUT_EOS"})}),"\n",(0,s.jsx)(n.pre,{children:(0,s.jsx)(n.code,{className:"language-typescript",children:'type Event = {\n  type: "AUDIO_INPUT_EOS";\n  input_id: string;\n}\n'})}),"\n",(0,s.jsxs)(n.p,{children:["The input stream has ended and all the audio samples were already processed.\nIt's not emitted on ",(0,s.jsx)(n.a,{href:"/docs/api/routes#unregister-input",children:(0,s.jsx)(n.code,{children:"input unregister"})}),"."]}),"\n",(0,s.jsx)(n.h3,{id:"output_done",children:(0,s.jsx)(n.code,{children:"OUTPUT_DONE"})}),"\n",(0,s.jsx)(n.pre,{children:(0,s.jsx)(n.code,{className:"language-typescript",children:'type Event = {\n  type: "OUTPUT_DONE",\n  output_id: string\n}\n'})}),"\n",(0,s.jsx)(n.p,{children:"The output has ended. All video frames and audio samples were sent/written."})]})}function p(e={}){const{wrapper:n}={...(0,i.a)(),...e.components};return n?(0,s.jsx)(n,{...e,children:(0,s.jsx)(l,{...e})}):l(e)}},1151:(e,n,t)=>{t.d(n,{Z:()=>o,a:()=>d});var s=t(7294);const i={},r=s.createContext(i);function d(e){const n=s.useContext(r);return s.useMemo((function(){return"function"==typeof e?e(n):{...n,...e}}),[n,e])}function o(e){let n;return n=e.disableParentContext?"function"==typeof e.components?e.components(i):e.components||i:d(e.components),s.createElement(r.Provider,{value:n},e.children)}}}]);