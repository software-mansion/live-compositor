"use strict";(self.webpackChunkcompositor_live=self.webpackChunkcompositor_live||[]).push([[821],{660:(e,t,n)=>{n.r(t),n.d(t,{assets:()=>d,contentTitle:()=>l,default:()=>h,frontMatter:()=>r,metadata:()=>o,toc:()=>c});var i=n(5893),s=n(1151);const r={},l="Text",o={id:"api/components/Text",title:"Text",description:"Properties",source:"@site/pages/api/components/Text.md",sourceDirName:"api/components",slug:"/api/components/Text",permalink:"/docs/api/components/Text",draft:!1,unlisted:!1,tags:[],version:"current",frontMatter:{},sidebar:"sidebar",previous:{title:"Shader",permalink:"/docs/api/components/Shader"},next:{title:"Tiles",permalink:"/docs/api/components/Tiles"}},d={},c=[{value:"Properties",id:"properties",level:4}];function a(e){const t={a:"a",code:"code",h1:"h1",h4:"h4",li:"li",p:"p",pre:"pre",ul:"ul",...(0,s.a)(),...e.components};return(0,i.jsxs)(i.Fragment,{children:[(0,i.jsx)(t.h1,{id:"text",children:"Text"}),"\n",(0,i.jsx)(t.pre,{children:(0,i.jsx)(t.code,{className:"language-typescript",children:'type Text = {\n  align?: "left" | "right" | "justified" | "center",\n  background_color_rgba?: string,\n  color_rgba?: string,\n  font_family?: string,\n  font_size: f32,\n  height?: f32,\n  id?: string,\n  line_height?: f32,\n  max_height?: f32,\n  max_width?: f32,\n  style?: "normal" | "italic" | "oblique",\n  text: string,\n  type: "text",\n  weight?: \n    | "thin"\n    | "extra_light"\n    | "light"\n    | "normal"\n    | "medium"\n    | "semi_bold"\n    | "bold"\n    | "extra_bold"\n    | "black",\n  width?: f32,\n  wrap?: "none" | "glyph" | "word",\n}\n'})}),"\n",(0,i.jsx)(t.h4,{id:"properties",children:"Properties"}),"\n",(0,i.jsxs)(t.ul,{children:["\n",(0,i.jsxs)(t.li,{children:[(0,i.jsx)(t.code,{children:"align"}),' - (default="left") Text align.']}),"\n",(0,i.jsxs)(t.li,{children:[(0,i.jsx)(t.code,{children:"background_color_rgba"}),' - (default="#00000000") Background color in ',(0,i.jsx)(t.code,{children:"#RRGGBBAA"})," format."]}),"\n",(0,i.jsxs)(t.li,{children:[(0,i.jsx)(t.code,{children:"color_rgba"}),' - (default="#FFFFFFFF") Font color in ',(0,i.jsx)(t.code,{children:"#RRGGBBAA"})," format."]}),"\n",(0,i.jsxs)(t.li,{children:[(0,i.jsx)(t.code,{children:"font_family"}),' - (default="Verdana") Font family.']}),"\n"]}),"\n",(0,i.jsxs)(t.p,{children:['Provide family-name for specific font. "generic-family" values like e.g. "sans-serif" will not work. ',(0,i.jsx)(t.a,{href:"https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/#family-name-value",children:"https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/#family-name-value"})]}),"\n",(0,i.jsxs)(t.ul,{children:["\n",(0,i.jsxs)(t.li,{children:[(0,i.jsx)(t.code,{children:"font_size"})," - Font size in pixels."]}),"\n",(0,i.jsxs)(t.li,{children:[(0,i.jsx)(t.code,{children:"height"})," - Height of a texture that text will be rendered on. If not provided the resulting texture will be sized based on the defined text, but limited to ",(0,i.jsx)(t.code,{children:"max_width"})," value."]}),"\n"]}),"\n",(0,i.jsxs)(t.p,{children:["It's an error to provide ",(0,i.jsx)(t.code,{children:"height"})," if width is not defined."]}),"\n",(0,i.jsxs)(t.ul,{children:["\n",(0,i.jsxs)(t.li,{children:[(0,i.jsx)(t.code,{children:"line_height"})," - Distance between lines in pixels. Defaults to the value of the ",(0,i.jsx)(t.code,{children:"font_size"})," property."]}),"\n",(0,i.jsxs)(t.li,{children:[(0,i.jsx)(t.code,{children:"max_height"})," - (default=4320) Maximal height. Limits the height of a texture that text will be rendered on. Value is ignored if height is defined."]}),"\n",(0,i.jsxs)(t.li,{children:[(0,i.jsx)(t.code,{children:"max_width"})," - (default=7682) Maximal width. Limits the width of a texture that text will be rendered on. Value is ignored if width is defined."]}),"\n",(0,i.jsxs)(t.li,{children:[(0,i.jsx)(t.code,{children:"style"}),' - (default="normal") Font style. The selected font needs to support this specific style.']}),"\n",(0,i.jsxs)(t.li,{children:[(0,i.jsx)(t.code,{children:"weight"}),' - (default="normal") Font weight. The selected font needs to support this specific weight.']}),"\n",(0,i.jsxs)(t.li,{children:[(0,i.jsx)(t.code,{children:"width"})," - Width of a texture that text will be rendered on. If not provided the resulting texture will be sized based on the defined text, but limited to ",(0,i.jsx)(t.code,{children:"max_width"})," value."]}),"\n",(0,i.jsxs)(t.li,{children:[(0,i.jsx)(t.code,{children:"wrap"}),' - (default="none") Text wrapping options.']}),"\n"]})]})}function h(e={}){const{wrapper:t}={...(0,s.a)(),...e.components};return t?(0,i.jsx)(t,{...e,children:(0,i.jsx)(a,{...e})}):a(e)}},1151:(e,t,n)=>{n.d(t,{Z:()=>o,a:()=>l});var i=n(7294);const s={},r=i.createContext(s);function l(e){const t=i.useContext(r);return i.useMemo((function(){return"function"==typeof e?e(t):{...t,...e}}),[t,e])}function o(e){let t;return t=e.disableParentContext?"function"==typeof e.components?e.components(s):e.components||s:l(e.components),i.createElement(r.Provider,{value:t},e.children)}}}]);