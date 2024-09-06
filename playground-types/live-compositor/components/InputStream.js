"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const component_1 = __importDefault(require("../component"));
class InputStream extends component_1.default {
    builder = sceneBuilder;
}
function sceneBuilder(props, _children) {
    return {
        type: 'input_stream',
        id: props.id,
        input_id: props.inputId,
    };
}
exports.default = InputStream;
