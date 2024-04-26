"use strict";(self.webpackChunkcompositor_live=self.webpackChunkcompositor_live||[]).push([[8421],{1032:(e,n,r)=>{r.r(n),r.d(n,{assets:()=>c,contentTitle:()=>o,default:()=>p,frontMatter:()=>i,metadata:()=>d,toc:()=>l});var t=r(5893),s=r(1151);const i={},o=void 0,d={id:"api/generated/renderer-Mp4",title:"renderer-Mp4",description:"Mp4",source:"@site/pages/api/generated/renderer-Mp4.md",sourceDirName:"api/generated",slug:"/api/generated/renderer-Mp4",permalink:"/docs/api/generated/renderer-Mp4",draft:!1,unlisted:!1,tags:[],version:"current",frontMatter:{}},c={},l=[{value:"Mp4",id:"mp4",level:2},{value:"Properties",id:"properties",level:4}];function a(e){const n={code:"code",h2:"h2",h4:"h4",li:"li",p:"p",pre:"pre",strong:"strong",ul:"ul",...(0,s.a)(),...e.components};return(0,t.jsxs)(t.Fragment,{children:[(0,t.jsx)(n.h2,{id:"mp4",children:"Mp4"}),"\n",(0,t.jsx)(n.pre,{children:(0,t.jsx)(n.code,{className:"language-typescript",children:"type Mp4 = {\n  url?: string;\n  path?: string;\n  required?: bool;\n  offset_ms?: f64;\n}\n"})}),"\n",(0,t.jsxs)(n.p,{children:["Input stream from MP4 file. Exactly one of ",(0,t.jsx)(n.code,{children:"url"})," and ",(0,t.jsx)(n.code,{children:"path"})," has to be defined."]}),"\n",(0,t.jsx)(n.h4,{id:"properties",children:"Properties"}),"\n",(0,t.jsxs)(n.ul,{children:["\n",(0,t.jsxs)(n.li,{children:[(0,t.jsx)(n.code,{children:"url"})," - URL of the MP4 file."]}),"\n",(0,t.jsxs)(n.li,{children:[(0,t.jsx)(n.code,{children:"path"})," - Path to the MP4 file."]}),"\n",(0,t.jsxs)(n.li,{children:[(0,t.jsx)(n.code,{children:"required"})," - (",(0,t.jsxs)(n.strong,{children:["default=",(0,t.jsx)(n.code,{children:"false"})]}),") If input is required and frames are not processed on time, then LiveCompositor will delay producing output frames."]}),"\n",(0,t.jsxs)(n.li,{children:[(0,t.jsx)(n.code,{children:"offset_ms"})," - Offset in milliseconds relative to the pipeline start (start request). If offset is not defined then stream is synchronized based on the first frames delivery time."]}),"\n"]})]})}function p(e={}){const{wrapper:n}={...(0,s.a)(),...e.components};return n?(0,t.jsx)(n,{...e,children:(0,t.jsx)(a,{...e})}):a(e)}},1151:(e,n,r)=>{r.d(n,{Z:()=>d,a:()=>o});var t=r(7294);const s={},i=t.createContext(s);function o(e){const n=t.useContext(i);return t.useMemo((function(){return"function"==typeof e?e(n):{...n,...e}}),[n,e])}function d(e){let n;return n=e.disableParentContext?"function"==typeof e.components?e.components(s):e.components||s:o(e.components),t.createElement(i.Provider,{value:n},e.children)}}}]);