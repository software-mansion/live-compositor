import View, { ViewProps } from './components/View.js';
import Image, { ImageProps } from './components/Image.js';
import Text, { TextProps } from './components/Text.js';
import InputStream, { InputStreamProps } from './components/InputStream.js';
import Rescaler, { RescalerProps } from './components/Rescaler.js';
import WebView, { WebViewProps } from './components/WebView.js';
import Shader, { ShaderParam, ShaderParamStructField, ShaderProps } from './components/Shader.js';
import Tiles, { TilesProps } from './components/Tiles.js';
import { EasingFunction, Transition } from './components/common.js';
import { useAudioInput, useInputStreams } from './hooks.js';
import { CompositorEvent, CompositorEventType } from './types/events.js';

export { RegisterInput } from './types/registerInput.js';
export { RegisterOutput, OutputFrameFormat } from './types/registerOutput.js';

export * as Inputs from './types/registerInput.js';
export * as Outputs from './types/registerOutput.js';
export * as Renderers from './types/registerRenderer.js';
export * as Api from './api.js';
export * as _liveCompositorInternals from './internal.js';

export {
  View,
  ViewProps,
  Image,
  ImageProps,
  Text,
  TextProps,
  InputStream,
  InputStreamProps,
  Rescaler,
  RescalerProps,
  WebView,
  WebViewProps,
  Shader,
  ShaderProps,
  Tiles,
  TilesProps,
};

export { CompositorEvent, CompositorEventType };

export { useInputStreams, useAudioInput };

export { ShaderParam, ShaderParamStructField, EasingFunction, Transition };
