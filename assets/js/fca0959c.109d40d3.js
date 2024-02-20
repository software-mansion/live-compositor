"use strict";(self.webpackChunkcompositor_live=self.webpackChunkcompositor_live||[]).push([[4771],{4625:(e,n,i)=>{i.r(n),i.d(n,{assets:()=>c,contentTitle:()=>d,default:()=>l,frontMatter:()=>s,metadata:()=>o,toc:()=>a});var t=i(5893),r=i(1151);const s={},d="Web Renderer",o={id:"concept/web",title:"Web Renderer",description:"Overview",source:"@site/pages/concept/web.md",sourceDirName:"concept",slug:"/concept/web",permalink:"/docs/concept/web",draft:!1,unlisted:!1,tags:[],version:"current",frontMatter:{},sidebar:"sidebar",previous:{title:"Shaders",permalink:"/docs/concept/shaders"},next:{title:"Deployment",permalink:"/docs/category/deployment"}},c={},a=[{value:"Overview",id:"overview",level:2},{value:"Embedding components",id:"embedding-components",level:2},{value:"Embedding methods",id:"embedding-methods",level:3},{value:"Example usage",id:"example-usage",level:2},{value:"Limitations",id:"limitations",level:2}];function h(e){const n={a:"a",admonition:"admonition",code:"code",h1:"h1",h2:"h2",h3:"h3",hr:"hr",li:"li",p:"p",pre:"pre",ul:"ul",...(0,r.a)(),...e.components};return(0,t.jsxs)(t.Fragment,{children:[(0,t.jsx)(n.h1,{id:"web-renderer",children:"Web Renderer"}),"\n",(0,t.jsx)(n.h2,{id:"overview",children:"Overview"}),"\n",(0,t.jsx)(n.p,{children:"Web rendering is an experimental feature that lets you render websites.\nFurthermore, you can place other components on the website. We refer to this process as embedding."}),"\n",(0,t.jsx)(n.p,{children:"Make sure you have a compositor version built with web renderer support. The web renderer introduces additional dependencies and significantly increases the size of the compositor binaries. To minimize that impact, we are supporting two versions of the compositor, one with web renderer support and one without it."}),"\n",(0,t.jsx)(n.admonition,{type:"tip",children:(0,t.jsxs)(n.p,{children:["You can view a working example ",(0,t.jsx)(n.a,{href:"https://github.com/membraneframework/video_compositor/blob/master/examples/web_view.rs",children:"here"})]})}),"\n",(0,t.jsx)(n.h2,{id:"embedding-components",children:"Embedding components"}),"\n",(0,t.jsxs)(n.p,{children:["Embedding is a process of displaying child components on a website. You can define the child components in the ",(0,t.jsx)(n.code,{children:"children"})," field of the web view.\nThe child component IDs have to correspond to the IDs of HTML elements.\nThe web renderer embeds the children's frames in the specified HTML elements."]}),"\n",(0,t.jsx)(n.h3,{id:"embedding-methods",children:"Embedding methods"}),"\n",(0,t.jsx)(n.p,{children:"There are 3 embedding methods available:"}),"\n",(0,t.jsxs)(n.ul,{children:["\n",(0,t.jsxs)(n.li,{children:[(0,t.jsx)(n.code,{children:"chromium_embedding"})," - Frames produced by child components are passed directly to a chromium instance and displayed on an HTML canvas. Passing frames to the chromium instance introduces one more copy operation on each input frame, which may cause performance problems for a large number of inputs. The HTML elements used for embedding have to be canvases."]}),"\n",(0,t.jsxs)(n.li,{children:[(0,t.jsx)(n.code,{children:"native_embedding_over_content"})," - Renders frames produced by child components on top of the website's content."]}),"\n",(0,t.jsxs)(n.li,{children:[(0,t.jsx)(n.code,{children:"native_embedding_under_content"})," - Renders frames produced by child components below the website's content. The website needs to have a transparent background. Otherwise, it will cover the frames underneath it."]}),"\n"]}),"\n",(0,t.jsxs)(n.p,{children:[(0,t.jsx)(n.code,{children:"native_embedding_over_content"})," is the default embedding method.\nYou can change it in the ",(0,t.jsx)(n.a,{href:"../api/routes#register-renderer",children:"register renderer request"}),". For example:"]}),"\n",(0,t.jsx)(n.pre,{children:(0,t.jsx)(n.code,{className:"language-typescript",children:'{\n    "type": "register",\n    "entity_type": "web_renderer",\n    "instance_id": "example_website",\n    "url": "https://example.com",\n    "resolution": {\n        "width": 1920,\n        "height": 1080\n    },\n    // highlight-next-line\n    "embedding_method": "chromium_embedding"\n}\n'})}),"\n",(0,t.jsx)(n.h2,{id:"example-usage",children:"Example usage"}),"\n",(0,t.jsx)(n.p,{children:"Firstly, the web renderer instance has to be registered:"}),"\n",(0,t.jsx)(n.pre,{children:(0,t.jsx)(n.code,{className:"language-typescript",children:'{\n    "type": "register",\n    "entity_type": "web_renderer",\n    "instance_id": "example_website",\n    "url": "https://example.com",\n    "resolution": {\n        "width": 1920,\n        "height": 1080\n    },\n    "embedding_method": "native_embedding_over_content"\n}\n'})}),"\n",(0,t.jsxs)(n.ul,{children:["\n",(0,t.jsxs)(n.li,{children:[(0,t.jsx)(n.code,{children:"instance_id"})," - unique renderer identifier. After the registration, the ",(0,t.jsx)(n.a,{href:"/docs/api/components/WebView",children:"web view"})," component references the web renderer using this identifier."]}),"\n",(0,t.jsxs)(n.li,{children:[(0,t.jsx)(n.code,{children:"url"})," - website url. All URL protocols supported by Chromium can be used here."]}),"\n"]}),"\n",(0,t.jsxs)(n.p,{children:["We can define a scene with a web view component that refers to the previously registered renderer instance using ",(0,t.jsx)(n.code,{children:"instance_id"})," field:"]}),"\n",(0,t.jsx)(n.pre,{children:(0,t.jsx)(n.code,{className:"language-typescript",children:'{\n    "type": "update_scene",\n    "outputs": [\n        {\n            "output_id": "output_1",\n            "scene": {\n                "id": "embed_input_on_website",\n                "type": "web_view",\n                "instance_id": "example_website",\n            }\n        }\n    ]\n}\n'})}),"\n",(0,t.jsxs)(n.p,{children:[(0,t.jsx)(n.code,{children:"instance_id"})," - the ID of previously registered web renderer."]}),"\n",(0,t.jsx)(n.admonition,{type:"warning",children:(0,t.jsx)(n.p,{children:"Only one web view component can use a specific web renderer instance at the same time."})}),"\n",(0,t.jsx)(n.hr,{}),"\n",(0,t.jsx)(n.p,{children:"The above request defines a simple scene which displays a website.\nNow, we can modify that request and embed an input stream into the website:"}),"\n",(0,t.jsx)(n.pre,{children:(0,t.jsx)(n.code,{className:"language-typescript",children:'{\n    "type": "update_scene",\n    "outputs": [\n        {\n            "output_id": "output_1",\n            "scene": {\n                "id": "embed_input_on_website",\n                "type": "web_view",\n                "instance_id": "example_website",\n                // highlight-start\n                "children": [\n                    {\n                        "id": "my_video",\n                        "type": "input_stream",\n                        "input_id": "input_1",\n                    }\n                ]\n                // highlight-end\n            }\n        }\n    ],\n}\n'})}),"\n",(0,t.jsxs)(n.ul,{children:["\n",(0,t.jsxs)(n.li,{children:[(0,t.jsx)(n.code,{children:"id"})," - the ID of an HTML element."]}),"\n"]}),"\n",(0,t.jsx)(n.admonition,{type:"note",children:(0,t.jsx)(n.p,{children:"The input stream has to be registered beforehand."})}),"\n",(0,t.jsxs)(n.p,{children:["Web renderer places frames in HTML elements that are inside the website. Each HTML element must have an ",(0,t.jsx)(n.code,{children:"id"})," attribute defined.\nHere's an example website:"]}),"\n",(0,t.jsx)(n.pre,{children:(0,t.jsx)(n.code,{className:"language-html",children:'<html>\n    <body>\n        <canvas id="my_video"></canvas>\n    </body>\n</html>\n'})}),"\n",(0,t.jsx)(n.h2,{id:"limitations",children:"Limitations"}),"\n",(0,t.jsxs)(n.p,{children:["Underneath, the web renderer uses Chromium Embedded Framework. To render a website, we have to make a lot of copies, which can become a bottleneck. That is especially true for ",(0,t.jsx)(n.code,{children:"chromium_embedding"})," since we have to copy frame data back and forth every frame."]})]})}function l(e={}){const{wrapper:n}={...(0,r.a)(),...e.components};return n?(0,t.jsx)(n,{...e,children:(0,t.jsx)(h,{...e})}):h(e)}},1151:(e,n,i)=>{i.d(n,{Z:()=>o,a:()=>d});var t=i(7294);const r={},s=t.createContext(r);function d(e){const n=t.useContext(s);return t.useMemo((function(){return"function"==typeof e?e(n):{...n,...e}}),[n,e])}function o(e){let n;return n=e.disableParentContext?"function"==typeof e.components?e.components(r):e.components||r:d(e.components),t.createElement(s.Provider,{value:n},e.children)}}}]);