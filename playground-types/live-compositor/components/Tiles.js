"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
Object.defineProperty(exports, "__esModule", { value: true });
const common_1 = require("./common");
const component_1 = __importStar(require("../component"));
class Tiles extends component_1.default {
    builder = sceneBuilder;
}
function sceneBuilder(props, children) {
    return {
        type: 'tiles',
        id: props.id,
        children: children.map(component_1.sceneComponentIntoApi),
        width: props.width,
        height: props.height,
        background_color_rgba: props.backgroundColorRgba,
        tile_aspect_ratio: props.tileAspectRatio,
        margin: props.margin,
        padding: props.padding,
        horizontal_align: props.horizontalAlign,
        vertical_align: props.verticalAlign,
        transition: props.transition && (0, common_1.intoApiTransition)(props.transition),
    };
}
exports.default = Tiles;
