import * as Api from '../api';
import { Component } from '../component';
import { intoComponent } from '../element';
import { RenderContext } from '../context';
import { intoApiTransition, Transition } from './common';

type RescalerProps = {
  children?: Component<any>[];

  /**
   * Id of a component.
   */
  id?: Api.ComponentId;
  /**
   * (**default=`"fit"`**) Resize mode:
   */
  mode?: Api.RescaleMode;
  /**
   * (**default=`"center"`**) Horizontal alignment.
   */
  horizontalAlign?: Api.HorizontalAlign;
  /**
   * (**default=`"center"`**) Vertical alignment.
   */
  verticalAlign?: Api.VerticalAlign;
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
   * Distance in pixels between this component's top edge and its parent's top edge.
   * If this field is defined, then the component will ignore a layout defined by its parent.
   */
  top?: number;
  /**
   * Distance in pixels between this component's left edge and its parent's left edge.
   * If this field is defined, this element will be absolutely positioned, instead of being
   * laid out by its parent.
   */
  left?: number;
  /**
   * Distance in pixels between this component's bottom edge and its parent's bottom edge.
   * If this field is defined, this element will be absolutely positioned, instead of being
   * laid out by its parent.
   */
  bottom?: number;
  /**
   * Distance in pixels between this component's right edge and its parent's right edge.
   * If this field is defined, this element will be absolutely positioned, instead of being
   * laid out by its parent.
   */
  right?: number;
  /**
   * Rotation of a component in degrees. If this field is defined, this element will be
   * absolutely positioned, instead of being laid out by its parent.
   */
  rotation?: number;
  /**
   * Defines how this component will behave during a scene update. This will only have an
   * effect if the previous scene already contained a View component with the same id.
   */
  transition?: Transition;
};

class Rescaler extends Component<RescalerProps> {
  props: RescalerProps;

  constructor(props: RescalerProps) {
    super();
    this.props = props;
  }

  scene(ctx: RenderContext): Api.Component {
    return {
      type: 'rescaler',
      id: this.props.id,
      child: this.child(ctx),
      mode: this.props.mode,
      horizontal_align: this.props.horizontalAlign,
      vertical_align: this.props.verticalAlign,
      width: this.props.width,
      height: this.props.height,
      top: this.props.top,
      bottom: this.props.bottom,
      left: this.props.left,
      right: this.props.right,
      rotation: this.props.rotation,
      transition: this.props.transition && intoApiTransition(this.props.transition),
    };
  }

  private child(ctx: RenderContext): Api.Component {
    const children = this.props?.children?.map(intoComponent);
    if (!children || children.length === 0) {
      return { type: 'view' };
    } else {
      if (children.length > 1) {
        console.error('Rescaler component can only have one child.');
      }
      return children[0].scene(ctx);
    }
  }

  update(props: RescalerProps): void {
    this.props = props;
  }
}

export default Rescaler;
