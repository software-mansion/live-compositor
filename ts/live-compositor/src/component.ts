import React, { useId } from 'react';
import * as Api from './api.js';

type ComponentProps<P> = { children?: React.ReactNode; id?: Api.ComponentId } & P;

export type SceneComponent = Api.Component | string;
export type SceneBuilder<P> = (props: P, children: SceneComponent[]) => Api.Component;

export function createCompositorComponent<P>(
  sceneBuilder: SceneBuilder<P>
): (props: ComponentProps<P>) => React.ReactNode {
  return (props: ComponentProps<P>): React.ReactNode => {
    const { children, ...otherProps } = props;
    const autoId = useId();

    return React.createElement(
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
      font_size: 50,
    };
  }
  return component;
}
