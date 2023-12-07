"use strict";(self.webpackChunkcompositor_live=self.webpackChunkcompositor_live||[]).push([[538,930],{4100:(e,n,i)=>{i.r(n),i.d(n,{assets:()=>a,contentTitle:()=>c,default:()=>m,frontMatter:()=>o,metadata:()=>d,toc:()=>p});var t=i(5893),r=i(1151),s=i(4928);const o={sidebar_position:8},c="WebView",d={id:"api/components/WebView",title:"WebView",description:"WebView renders a website using Chromium engine embedded inside the compositor.",source:"@site/pages/api/components/WebView.md",sourceDirName:"api/components",slug:"/api/components/WebView",permalink:"/docs/api/components/WebView",draft:!1,unlisted:!1,tags:[],version:"current",sidebarPosition:8,frontMatter:{sidebar_position:8},sidebar:"sidebar",previous:{title:"Image",permalink:"/docs/api/components/Image"},next:{title:"Shader",permalink:"/docs/api/renderers/shader"}},a={},p=[];function l(e){const n={a:"a",admonition:"admonition",code:"code",h1:"h1",p:"p",...(0,r.a)(),...e.components};return(0,t.jsxs)(t.Fragment,{children:[(0,t.jsx)(n.h1,{id:"webview",children:"WebView"}),"\n",(0,t.jsxs)(n.p,{children:[(0,t.jsx)(n.code,{children:"WebView"})," renders a website using Chromium engine embedded inside the compositor."]}),"\n",(0,t.jsx)(n.admonition,{type:"note",children:(0,t.jsxs)(n.p,{children:["To use this component, you need to first register the web renderer instance with matching ",(0,t.jsx)(n.code,{children:"instance_id"})," using ",(0,t.jsx)(n.a,{href:"../routes#register-renderer",children:(0,t.jsx)(n.code,{children:"RegisterRenderer"})})," request."]})}),"\n",(0,t.jsx)(n.admonition,{type:"warning",children:(0,t.jsxs)(n.p,{children:["Only one component can use specific ",(0,t.jsx)(n.code,{children:"instance_id"})," at the time."]})}),"\n",(0,t.jsx)(s.default,{})]})}function m(e={}){const{wrapper:n}={...(0,r.a)(),...e.components};return n?(0,t.jsx)(n,{...e,children:(0,t.jsx)(l,{...e})}):l(e)}},4928:(e,n,i)=>{i.r(n),i.d(n,{assets:()=>d,contentTitle:()=>o,default:()=>l,frontMatter:()=>s,metadata:()=>c,toc:()=>a});var t=i(5893),r=i(1151);const s={},o=void 0,c={id:"api/generated/component-WebView",title:"component-WebView",description:"WebView",source:"@site/pages/api/generated/component-WebView.md",sourceDirName:"api/generated",slug:"/api/generated/component-WebView",permalink:"/docs/api/generated/component-WebView",draft:!1,unlisted:!1,tags:[],version:"current",frontMatter:{}},d={},a=[{value:"WebView",id:"webview",level:2},{value:"Properties",id:"properties",level:4}];function p(e){const n={a:"a",admonition:"admonition",code:"code",h2:"h2",h4:"h4",li:"li",p:"p",pre:"pre",ul:"ul",...(0,r.a)(),...e.components};return(0,t.jsxs)(t.Fragment,{children:[(0,t.jsx)(n.h2,{id:"webview",children:"WebView"}),"\n",(0,t.jsx)(n.pre,{children:(0,t.jsx)(n.code,{className:"language-typescript",children:'type WebView = {\n  type: "web_view",\n  id: string,\n  children?: Component[],\n  instance_id: string,\n}\n'})}),"\n",(0,t.jsx)(n.p,{children:"WebView component renders a website using Chromium."}),"\n",(0,t.jsx)(n.h4,{id:"properties",children:"Properties"}),"\n",(0,t.jsxs)(n.ul,{children:["\n",(0,t.jsxs)(n.li,{children:[(0,t.jsx)(n.code,{children:"id"})," - Id of a component."]}),"\n",(0,t.jsxs)(n.li,{children:[(0,t.jsx)(n.code,{children:"children"})," - List of component's children."]}),"\n",(0,t.jsxs)(n.li,{children:[(0,t.jsx)(n.code,{children:"instance_id"})," - Id of a web renderer instance. It identifies an instance registered using a ",(0,t.jsx)(n.a,{href:"../routes#register-renderer",children:(0,t.jsx)(n.code,{children:"RegisterRenderer"})})," request.","\n",(0,t.jsx)("br",{}),"\n",(0,t.jsx)("br",{}),"\n",(0,t.jsx)(n.admonition,{type:"warning",children:(0,t.jsx)(n.p,{children:"You can only refer to specific instances in one Component at a time."})}),"\n"]}),"\n"]})]})}function l(e={}){const{wrapper:n}={...(0,r.a)(),...e.components};return n?(0,t.jsx)(n,{...e,children:(0,t.jsx)(p,{...e})}):p(e)}},1151:(e,n,i)=>{i.d(n,{Z:()=>c,a:()=>o});var t=i(7294);const r={},s=t.createContext(r);function o(e){const n=t.useContext(s);return t.useMemo((function(){return"function"==typeof e?e(n):{...n,...e}}),[n,e])}function c(e){let n;return n=e.disableParentContext?"function"==typeof e.components?e.components(r):e.components||r:o(e.components),t.createElement(s.Provider,{value:n},e.children)}}}]);