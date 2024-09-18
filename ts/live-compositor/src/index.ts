import View, { ViewProps } from './components/View';
import Image, { ImageProps } from './components/Image';
import Text, { TextProps } from './components/Text';
import InputStream, { InputStreamProps } from './components/InputStream';
import Rescaler, { RescalerProps } from './components/Rescaler';
import WebView, { WebViewProps } from './components/WebView';
import Shader, { ShaderParam, ShaderParamStructField, ShaderProps } from './components/Shader';
import Tiles, { TilesProps } from './components/Tiles';
import { EasingFunction, Transition } from './components/common';
import { useAudioInput, useInputStreams } from './hooks';
import { CompositorEvent, CompositorEventType } from './types/events';

export { RegisterInput } from './types/registerInput';
export { RegisterOutput, OutputByteFormat } from './types/registerOutput';

export * as Inputs from './types/registerInput';
export * as Outputs from './types/registerOutput';
export * as Renderers from './types/registerRenderer';
export * as Api from './api';
export * as _liveCompositorInternals from './internal';

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
