import React from 'react';
import * as Api from './api';

type ComponentProps<P> = { children?: React.ReactNode } & P;

export type SceneComponent = Api.Component | string;
export type SceneBuilder<P> = (props: P, children: SceneComponent[]) => Api.Component;

abstract class LiveCompositorComponent<P> extends React.Component<ComponentProps<P>> {
  abstract builder: SceneBuilder<P>;

  render(): React.ReactNode {
    const { children, ...props } = this.props;

    return React.createElement(
      'compositor',
      {
        sceneBuilder: this.builder,
        props,
      },
      ...(Array.isArray(children) ? children : [children])
    );
  }
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

export default LiveCompositorComponent;
