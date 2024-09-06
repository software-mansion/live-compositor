"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.sceneComponentIntoApi = sceneComponentIntoApi;
const react_1 = __importDefault(require("react"));
class LiveCompositorComponent extends react_1.default.Component {
    render() {
        const { children, ...props } = this.props;
        return react_1.default.createElement('compositor', {
            sceneBuilder: this.builder,
            props,
        }, ...(Array.isArray(children) ? children : [children]));
    }
}
function sceneComponentIntoApi(component) {
    if (typeof component === 'string') {
        return {
            type: 'text',
            text: component,
            font_size: 50,
        };
    }
    return component;
}
exports.default = LiveCompositorComponent;
