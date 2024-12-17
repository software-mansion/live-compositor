import type * as Api from '../api.js';
import type { SceneComponent } from '../component.js';
import { createCompositorComponent, DEFAULT_FONT_SIZE } from '../component.js';
import { intoApiRgbaColor } from './common.js';

export type TextStyle = {
  /**
   * Width of a texture that text will be rendered on. If not provided, the resulting texture
   * will be sized based on the defined text but limited to `max_width` value.
   */
  width?: number;
  /**
   * Height of a texture that text will be rendered on. If not provided, the resulting texture
   * will be sized based on the defined text but limited to `max_height` value.
   * It's an error to provide `height` if `width` is not defined.
   */
  height?: number;
  /**
   * (**default=`7682`**) Maximal `width`. Limits the width of the texture that the text will be rendered on.
   * Value is ignored if `width` is defined.
   */
  maxWidth?: number;
  /**
   * (**default=`4320`**) Maximal `height`. Limits the height of the texture that the text will be rendered on.
   * Value is ignored if height is defined.
   */
  maxHeight?: number;
  /**
   * Font size in pixels.
   */
  fontSize: number;
  /**
   * Distance between lines in pixels. Defaults to the value of the `font_size` property.
   */
  lineHeight?: number;
  /**
   * (**default=`"#FFFFFFFF"`**) Font color in `#RRGGBBAA` or `#RRGGBB` format.
   */
  color?: string;
  /**
   * (**default=`"#00000000"`**) Background color in `#RRGGBBAA` or `#RRGGBB` format.
   */
  backgroundColor?: string;
  /**
   * (**default=`"Verdana"`**) Font family. Provide [family-name](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/#family-name-value)
   * for a specific font. "generic-family" values like e.g. "sans-serif" will not work.
   */
  fontFamily?: string;
  /**
   * (**default=`"normal"`**) Font style. The selected font needs to support the specified style.
   */
  fontStyle?: Api.TextStyle;
  /**
   * (**default=`"left"`**) Text align.
   */
  align?: Api.HorizontalAlign;
  /**
   * (**default=`"none"`**) Text wrapping options.
   */
  wrap?: Api.TextWrapMode;
  /**
   * (**default=`"normal"`**) Font weight. The selected font needs to support the specified weight.
   */
  fontWeight?: Api.TextWeight;
};

export type TextProps = {
  children?: (string | number)[] | string | number;

  /**
   * Id of a component.
   */
  id?: Api.ComponentId;

  /**
   * Text styling properties
   */
  style?: TextStyle;
};

const Text = createCompositorComponent<TextProps>(sceneBuilder);

function sceneBuilder(props: TextProps, children: SceneComponent[]): Api.Component {
  const { id, style } = props;

  return {
    type: 'text',
    id: id,
    text: children.map(child => (typeof child === 'string' ? child : String(child))).join(''),
    width: style?.width,
    height: style?.height,
    max_width: style?.maxWidth,
    max_height: style?.maxHeight,
    font_size: style?.fontSize ?? DEFAULT_FONT_SIZE,
    line_height: style?.lineHeight,
    color_rgba: style?.color && intoApiRgbaColor(style?.color),
    background_color_rgba: style?.backgroundColor && intoApiRgbaColor(style?.backgroundColor),
    font_family: style?.fontFamily,
    style: style?.fontStyle,
    align: style?.align,
    wrap: style?.wrap,
    weight: style?.fontWeight,
  };
}

export default Text;
