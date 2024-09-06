import React from 'react';
import * as Api from '../api';
import { Transition } from './common';
import LiveCompositorComponent, { SceneBuilder } from '../component';
export type RescalerProps = {
    children: React.ReactElement | string | number;
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
declare class Rescaler extends LiveCompositorComponent<RescalerProps> {
    builder: SceneBuilder<RescalerProps>;
}
export default Rescaler;
