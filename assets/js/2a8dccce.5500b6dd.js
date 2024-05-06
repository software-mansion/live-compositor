"use strict";(self.webpackChunkcompositor_live=self.webpackChunkcompositor_live||[]).push([[8260,8421],{1032:(e,t,n)=>{n.r(t),n.d(t,{assets:()=>a,contentTitle:()=>o,default:()=>l,frontMatter:()=>s,metadata:()=>d,toc:()=>c});var r=n(5893),i=n(1151);const s={},o=void 0,d={id:"api/generated/renderer-Mp4",title:"renderer-Mp4",description:"Mp4",source:"@site/pages/api/generated/renderer-Mp4.md",sourceDirName:"api/generated",slug:"/api/generated/renderer-Mp4",permalink:"/docs/api/generated/renderer-Mp4",draft:!1,unlisted:!1,tags:[],version:"current",frontMatter:{}},a={},c=[{value:"Mp4",id:"mp4",level:2},{value:"Properties",id:"properties",level:4}];function p(e){const t={code:"code",h2:"h2",h4:"h4",li:"li",p:"p",pre:"pre",strong:"strong",ul:"ul",...(0,i.a)(),...e.components};return(0,r.jsxs)(r.Fragment,{children:[(0,r.jsx)(t.h2,{id:"mp4",children:"Mp4"}),"\n",(0,r.jsx)(t.pre,{children:(0,r.jsx)(t.code,{className:"language-typescript",children:"type Mp4 = {\n  url?: string;\n  path?: string;\n  required?: bool;\n  offset_ms?: f64;\n}\n"})}),"\n",(0,r.jsxs)(t.p,{children:["Input stream from MP4 file.\nExactly one of ",(0,r.jsx)(t.code,{children:"url"})," and ",(0,r.jsx)(t.code,{children:"path"})," has to be defined."]}),"\n",(0,r.jsx)(t.h4,{id:"properties",children:"Properties"}),"\n",(0,r.jsxs)(t.ul,{children:["\n",(0,r.jsxs)(t.li,{children:[(0,r.jsx)(t.code,{children:"url"})," - URL of the MP4 file."]}),"\n",(0,r.jsxs)(t.li,{children:[(0,r.jsx)(t.code,{children:"path"})," - Path to the MP4 file."]}),"\n",(0,r.jsxs)(t.li,{children:[(0,r.jsx)(t.code,{children:"required"})," - (",(0,r.jsxs)(t.strong,{children:["default=",(0,r.jsx)(t.code,{children:"false"})]}),") If input is required and frames are not processed\non time, then LiveCompositor will delay producing output frames."]}),"\n",(0,r.jsxs)(t.li,{children:[(0,r.jsx)(t.code,{children:"offset_ms"})," - Offset in milliseconds relative to the pipeline start (start request). If offset is\nnot defined then stream is synchronized based on the first frames delivery time."]}),"\n"]})]})}function l(e={}){const{wrapper:t}={...(0,i.a)(),...e.components};return t?(0,r.jsx)(t,{...e,children:(0,r.jsx)(p,{...e})}):p(e)}},1932:(e,t,n)=>{n.r(t),n.d(t,{assets:()=>c,contentTitle:()=>d,default:()=>u,frontMatter:()=>o,metadata:()=>a,toc:()=>p});var r=n(5893),i=n(1151),s=n(1032);const o={},d="MP4",a={id:"api/inputs/mp4",title:"MP4",description:"An input type that allows the compositor to read static MP4 files.",source:"@site/pages/api/inputs/mp4.md",sourceDirName:"api/inputs",slug:"/api/inputs/mp4",permalink:"/docs/api/inputs/mp4",draft:!1,unlisted:!1,tags:[],version:"current",frontMatter:{},sidebar:"sidebar",previous:{title:"RTP",permalink:"/docs/api/inputs/rtp"}},c={},p=[];function l(e){const t={h1:"h1",p:"p",...(0,i.a)(),...e.components};return(0,r.jsxs)(r.Fragment,{children:[(0,r.jsx)(t.h1,{id:"mp4",children:"MP4"}),"\n",(0,r.jsx)(t.p,{children:"An input type that allows the compositor to read static MP4 files."}),"\n",(0,r.jsx)(t.p,{children:"Mp4 files can contain video and audio tracks encoded with various codecs.\nThis input type supports mp4 video tracks encoded with h264 and audio tracks encoded with AAC."}),"\n",(0,r.jsx)(t.p,{children:"If the file contains multiple video or audio tracks, the first audio track and the first video track will be used and the other ones will be ignored."}),"\n",(0,r.jsx)(s.default,{})]})}function u(e={}){const{wrapper:t}={...(0,i.a)(),...e.components};return t?(0,r.jsx)(t,{...e,children:(0,r.jsx)(l,{...e})}):l(e)}},1151:(e,t,n)=>{n.d(t,{Z:()=>d,a:()=>o});var r=n(7294);const i={},s=r.createContext(i);function o(e){const t=r.useContext(s);return r.useMemo((function(){return"function"==typeof e?e(t):{...t,...e}}),[t,e])}function d(e){let t;return t=e.disableParentContext?"function"==typeof e.components?e.components(i):e.components||i:o(e.components),r.createElement(s.Provider,{value:t},e.children)}}}]);