import * as Api from '../api';
import { Component } from '../component';
import { intoComponent } from '../element';
import { RenderContext } from '../context';
import { intoApiTransition, Transition } from './common';

type TilesProps = {
  children?: Component<any>[];

  /**
   * Id of a component.
   */
  id?: Api.ComponentId;
  /**
   * Width of a component in pixels. Exact behavior might be different based on the parent
   * component:
   * - If the parent component is a layout, check sections "Absolute positioning" and "Static
   * positioning" of that component.
   * - If the parent component is not a layout, then this field is required.
   */
  width?: number;
  /**
   * Height of a component in pixels. Exact behavior might be different based on the parent
   * component:
   * - If the parent component is a layout, check sections "Absolute positioning" and "Static
   * positioning" of that component.
   * - If the parent component is not a layout, then this field is required.
   */
  height?: number;
  /**
   * (**default=`"#00000000"`**) Background color in a `"#RRGGBBAA"` format.
   */
  backgroundColorRgba?: Api.RGBAColor;
  /**
   * (**default=`"16:9"`**) Aspect ratio of a tile in `"W:H"` format, where W and H are integers.
   */
  tileAspectRatio?: Api.AspectRatio | null;
  /**
   * (**default=`0`**) Margin of each tile in pixels.
   */
  margin?: number;
  /**
   * (**default=`0`**) Padding on each tile in pixels.
   */
  padding?: number;
  /**
   * (**default=`"center"`**) Horizontal alignment of tiles.
   */
  horizontalAlign?: Api.HorizontalAlign;
  /**
   * (**default=`"center"`**) Vertical alignment of tiles.
   */
  verticalAlign?: Api.VerticalAlign;
  /**
   * Defines how this component will behave during a scene update. This will only have an
   * effect if the previous scene already contained a `Tiles` component with the same id.
   */
  transition?: Transition;
};

class Tiles extends Component<TilesProps> {
  props: TilesProps;

  constructor(props: TilesProps) {
    super();
    this.props = props;
  }

  scene(ctx: RenderContext): Api.Component {
    return {
      type: 'tiles',
      id: this.props.id,
      children: (this.props.children || []).map(child => intoComponent(child).scene(ctx)),
      width: this.props.width,
      height: this.props.height,
      background_color_rgba: this.props.backgroundColorRgba,
      tile_aspect_ratio: this.props.tileAspectRatio,
      margin: this.props.margin,
      padding: this.props.padding,
      horizontal_align: this.props.horizontalAlign,
      vertical_align: this.props.verticalAlign,
      transition: this.props.transition && intoApiTransition(this.props.transition),
    };
  }

  update(props: TilesProps): void {
    this.props = props;
  }
}

export default Tiles;
