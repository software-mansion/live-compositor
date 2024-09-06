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
const component_1 = __importStar(require("../component"));
class Shader extends component_1.default {
    builder = sceneBuilder;
}
function sceneBuilder(props, children) {
    return {
        type: 'shader',
        children: children.map(component_1.sceneComponentIntoApi),
        id: props.id,
        shader_id: props.shaderId,
        shader_param: props.shaderParam && intoShaderParams(props.shaderParam),
        resolution: props.resolution,
    };
}
function intoShaderParams(param) {
    if (param.type === 'f32') {
        return {
            type: 'f32',
            value: param.value,
        };
    }
    else if (param.type === 'u32') {
        return {
            type: 'u32',
            value: param.value,
        };
    }
    else if (param.type === 'i32') {
        return {
            type: 'i32',
            value: param.value,
        };
    }
    else if (param.type === 'list') {
        return {
            type: 'list',
            value: param.value.map(intoShaderParams),
        };
    }
    else if (param.type === 'struct') {
        return {
            type: 'struct',
            value: param.value.map(intoShaderStructField),
        };
    }
    else {
        throw new Error('Invalid shader params');
    }
}
function intoShaderStructField(param) {
    if (param.type === 'f32') {
        return {
            type: 'f32',
            value: param.value,
            field_name: param.fieldName,
        };
    }
    else if (param.type === 'u32') {
        return {
            type: 'u32',
            value: param.value,
            field_name: param.fieldName,
        };
    }
    else if (param.type === 'i32') {
        return {
            type: 'i32',
            value: param.value,
            field_name: param.fieldName,
        };
    }
    else if (param.type === 'list') {
        return {
            type: 'list',
            value: param.value.map(intoShaderParams),
            field_name: param.fieldName,
        };
    }
    else if (param.type === 'struct') {
        return {
            type: 'struct',
            value: param.value.map(intoShaderStructField),
            field_name: param.fieldName,
        };
    }
    else {
        throw new Error('Invalid shader params');
    }
}
exports.default = Shader;
