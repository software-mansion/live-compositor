"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const component_1 = __importDefault(require("../component"));
class Text extends component_1.default {
    builder = sceneBuilder;
}
function sceneBuilder(props, children) {
    return {
        type: 'text',
        id: props.id,
        text: children.map(child => (typeof child === 'string' ? child : String(child))).join(''),
        width: props.width,
        height: props.height,
        max_width: props.maxWidth,
        max_height: props.maxHeight,
        font_size: props.fontSize,
        line_height: props.lineHeight,
        color_rgba: props.colorRgba,
        background_color_rgba: props.backgroundColorRgba,
        font_family: props.fontFamily,
        style: props.style,
        align: props.align,
        wrap: props.wrap,
        weight: props.weight,
    };
}
exports.default = Text;
