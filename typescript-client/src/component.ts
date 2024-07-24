import * as Api from './api';
import { RenderContext } from './context';

export interface ClassComponent<Props> {
  new (props: Props): Component<Props>;
}

export interface FunctionComponent<Props> {
  (props: Props): Component<Props>;
}

export abstract class Component<Props> {
  abstract scene(context: RenderContext): Api.Component;
  abstract update(props: Props): void;
}

export function isClassComponent<Props>(
  component: FunctionComponent<Props> | ClassComponent<Props>
): component is ClassComponent<Props> {
  return component.prototype instanceof Component;
}
