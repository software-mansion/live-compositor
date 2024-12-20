import type React from 'react';
import { createElement, useId } from 'react';
import type * as Api from './api.js';

export const DEFAULT_FONT_SIZE = 50;

type ComponentProps<P> = { children?: React.ReactNode; id?: Api.ComponentId } & P;

export type SceneComponent = Api.Component | string;
export type SceneBuilder<P> = (props: P, children?: SceneComponent[]) => Api.Component;

export function createCompositorComponent<P>(
  sceneBuilder: SceneBuilder<P>
): (props: ComponentProps<P>) => React.ReactNode {
  return (props: ComponentProps<P>): React.ReactNode => {
    const { children, ...otherProps } = props;
    const autoId = useId();

    return createElement(
      'compositor',
      {
        sceneBuilder,
        props: { ...otherProps, id: otherProps.id ?? autoId },
      },
      ...(Array.isArray(children) ? children : [children])
    );
  };
}

export function sceneComponentIntoApi(component: SceneComponent): Api.Component {
  if (typeof component === 'string') {
    return {
      type: 'text',
      text: component,
      font_size: DEFAULT_FONT_SIZE,
    };
  }
  return component;
}
