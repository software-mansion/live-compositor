import * as Api from '../api';
import { intoApiRgbaColor, intoApiTransition, Transition } from './common';
import { createCompositorComponent, SceneComponent, sceneComponentIntoApi } from '../component';

export type TilesProps = {
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
   * (**default=`"#00000000"`**) Background color in a `"#RRGGBBAA"` or `"#RRGGBB"` format.
   */
  backgroundColor?: string;
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

const Tiles = createCompositorComponent<TilesProps>(sceneBuilder);

function sceneBuilder(props: TilesProps, children: SceneComponent[]): Api.Component {
  return {
    type: 'tiles',
    id: props.id,
    children: children.map(sceneComponentIntoApi),
    width: props.width,
    height: props.height,
    background_color_rgba: props.backgroundColor && intoApiRgbaColor(props.backgroundColor),
    tile_aspect_ratio: props.tileAspectRatio,
    margin: props.margin,
    padding: props.padding,
    horizontal_align: props.horizontalAlign,
    vertical_align: props.verticalAlign,
    transition: props.transition && intoApiTransition(props.transition),
  };
}

export default Tiles;
