"use strict";(self.webpackChunkcompositor_live=self.webpackChunkcompositor_live||[]).push([[821],{660:(e,n,i)=>{i.r(n),i.d(n,{assets:()=>o,contentTitle:()=>r,default:()=>a,frontMatter:()=>s,metadata:()=>d,toc:()=>c});var t=i(5893),l=i(1151);const s={},r=void 0,d={id:"api/components/Text",title:"Text",description:"Text",source:"@site/pages/api/components/Text.md",sourceDirName:"api/components",slug:"/api/components/Text",permalink:"/docs/api/components/Text",draft:!1,unlisted:!1,tags:[],version:"current",frontMatter:{},sidebar:"sidebar",previous:{title:"Shader",permalink:"/docs/api/components/Shader"},next:{title:"Tiles",permalink:"/docs/api/components/Tiles"}},o={},c=[{value:"Text",id:"text",level:2},{value:"Properties",id:"properties",level:4}];function h(e){const n={a:"a",code:"code",h2:"h2",h4:"h4",li:"li",pre:"pre",ul:"ul",...(0,l.a)(),...e.components};return(0,t.jsxs)(t.Fragment,{children:[(0,t.jsx)(n.h2,{id:"text",children:"Text"}),"\n",(0,t.jsx)(n.pre,{children:(0,t.jsx)(n.code,{className:"language-typescript",children:'type Text = {\n  type: "text",\n  id: string,\n  text: string,\n  width?: f32,\n  height?: f32,\n  max_width?: f32,\n  max_height?: f32,\n  font_size: f32,\n  line_height?: f32,\n  color_rgba: string,\n  background_color_rgba: string,\n  font_family?: string,\n  style: "normal" | "italic" | "oblique",\n  align: "left" | "right" | "justified" | "center",\n  wrap: "none" | "glyph" | "word",\n  weight: \n    | "thin"\n    | "extra_light"\n    | "light"\n    | "normal"\n    | "medium"\n    | "semi_bold"\n    | "bold"\n    | "extra_bold"\n    | "black",\n}\n'})}),"\n",(0,t.jsx)(n.h4,{id:"properties",children:"Properties"}),"\n",(0,t.jsxs)(n.ul,{children:["\n",(0,t.jsxs)(n.li,{children:[(0,t.jsx)(n.code,{children:"width"})," - Width of a texture that text will be rendered on. If not provided the resulting texture will be sized based on the defined text, but limited to ",(0,t.jsx)(n.code,{children:"max_width"})," value."]}),"\n",(0,t.jsxs)(n.li,{children:[(0,t.jsx)(n.code,{children:"height"})," - Height of a texture that text will be rendered on. If not provided the resulting texture will be sized based on the defined text, but limited to ",(0,t.jsx)(n.code,{children:"max_width"})," value.\nIt's an error to provide ",(0,t.jsx)(n.code,{children:"height"})," if width is not defined."]}),"\n",(0,t.jsxs)(n.li,{children:[(0,t.jsx)(n.code,{children:"max_width"})," - (default=7682) Maximal width. Limits the width of a texture that text will be rendered on. Value is ignored if width is defined."]}),"\n",(0,t.jsxs)(n.li,{children:[(0,t.jsx)(n.code,{children:"max_height"})," - (default=4320) Maximal height. Limits the height of a texture that text will be rendered on. Value is ignored if height is defined."]}),"\n",(0,t.jsxs)(n.li,{children:[(0,t.jsx)(n.code,{children:"font_size"})," - Font size in pixels."]}),"\n",(0,t.jsxs)(n.li,{children:[(0,t.jsx)(n.code,{children:"line_height"})," - Distance between lines in pixels. Defaults to the value of the ",(0,t.jsx)(n.code,{children:"font_size"})," property."]}),"\n",(0,t.jsxs)(n.li,{children:[(0,t.jsx)(n.code,{children:"color_rgba"}),' - (default="#FFFFFFFF") Font color in ',(0,t.jsx)(n.code,{children:"#RRGGBBAA"})," format."]}),"\n",(0,t.jsxs)(n.li,{children:[(0,t.jsx)(n.code,{children:"background_color_rgba"}),' - (default="#00000000") Background color in ',(0,t.jsx)(n.code,{children:"#RRGGBBAA"})," format."]}),"\n",(0,t.jsxs)(n.li,{children:[(0,t.jsx)(n.code,{children:"font_family"}),' - (default="Verdana") Font family.\nProvide family-name for specific font. "generic-family" values like e.g. "sans-serif" will not work. ',(0,t.jsx)(n.a,{href:"https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/#family-name-value",children:"https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/#family-name-value"})]}),"\n",(0,t.jsxs)(n.li,{children:[(0,t.jsx)(n.code,{children:"style"}),' - (default="normal") Font style. The selected font needs to support this specific style.']}),"\n",(0,t.jsxs)(n.li,{children:[(0,t.jsx)(n.code,{children:"align"}),' - (default="left") Text align.']}),"\n",(0,t.jsxs)(n.li,{children:[(0,t.jsx)(n.code,{children:"wrap"}),' - (default="none") Text wrapping options.',"\n",(0,t.jsxs)(n.ul,{children:["\n",(0,t.jsxs)(n.li,{children:[(0,t.jsx)(n.code,{children:'"none"'})," - Disable text wrapping. Text that does not fit inside the texture will be cut off."]}),"\n",(0,t.jsxs)(n.li,{children:[(0,t.jsx)(n.code,{children:'"glyph"'})," - Wraps at a glyph level."]}),"\n",(0,t.jsxs)(n.li,{children:[(0,t.jsx)(n.code,{children:'"word"'})," - Wraps at a word level. Prevent splitting words when wrapping."]}),"\n"]}),"\n"]}),"\n",(0,t.jsxs)(n.li,{children:[(0,t.jsx)(n.code,{children:"weight"}),' - (default="normal") Font weight. The selected font needs to support this specific weight. Font weight, based on ',(0,t.jsx)(n.a,{href:"https://learn.microsoft.com/en-gb/typography/opentype/spec/os2#usweightclass",children:"OpenType specification"}),".","\n",(0,t.jsxs)(n.ul,{children:["\n",(0,t.jsxs)(n.li,{children:[(0,t.jsx)(n.code,{children:'"thin"'})," - Weight 100."]}),"\n",(0,t.jsxs)(n.li,{children:[(0,t.jsx)(n.code,{children:'"extra_light"'})," - Weight 200."]}),"\n",(0,t.jsxs)(n.li,{children:[(0,t.jsx)(n.code,{children:'"light"'})," - Weight 300."]}),"\n",(0,t.jsxs)(n.li,{children:[(0,t.jsx)(n.code,{children:'"normal"'})," - Weight 400."]}),"\n",(0,t.jsxs)(n.li,{children:[(0,t.jsx)(n.code,{children:'"medium"'})," - Weight 500."]}),"\n",(0,t.jsxs)(n.li,{children:[(0,t.jsx)(n.code,{children:'"semi_bold"'})," - Weight 600."]}),"\n",(0,t.jsxs)(n.li,{children:[(0,t.jsx)(n.code,{children:'"bold"'})," - Weight 700."]}),"\n",(0,t.jsxs)(n.li,{children:[(0,t.jsx)(n.code,{children:'"extra_bold"'})," - Weight 800."]}),"\n",(0,t.jsxs)(n.li,{children:[(0,t.jsx)(n.code,{children:'"black"'})," - Weight 900."]}),"\n"]}),"\n"]}),"\n"]})]})}function a(e={}){const{wrapper:n}={...(0,l.a)(),...e.components};return n?(0,t.jsx)(n,{...e,children:(0,t.jsx)(h,{...e})}):h(e)}},1151:(e,n,i)=>{i.d(n,{Z:()=>d,a:()=>r});var t=i(7294);const l={},s=t.createContext(l);function r(e){const n=t.useContext(s);return t.useMemo((function(){return"function"==typeof e?e(n):{...n,...e}}),[n,e])}function d(e){let n;return n=e.disableParentContext?"function"==typeof e.components?e.components(l):e.components||l:r(e.components),t.createElement(s.Provider,{value:n},e.children)}}}]);