import { ClassComponent, Component, FunctionComponent, isClassComponent } from './component';
import Func from './components/Func';
import Text from './components/Text';
import View from './components/View';

export type Element<Props> = Component<Props> | string | number;

export function createElement<Props>(
  component: FunctionComponent<Props> | ClassComponent<Props>,
  props: Props,
  ...children: Element<any>[]
): Element<Props> {
  const propsWithChildren = { ...props, children: children.flat(Infinity) };
  if (isClassComponent(component)) {
    return new component(propsWithChildren);
  } else {
    return new Func<Props>(component, propsWithChildren);
  }
}

export function intoComponent<Props>(element: Element<Props>): Component<Props> {
  if (element instanceof Component) {
    return element;
  } else if (['string', 'number'].includes(typeof element)) {
    return createElement(Text, { fontSize: 20 }, element.toString()) as Component<Props>;
  } else {
    console.error('Invalid child element ${element}');
    return createElement(View, {}) as Component<Props>;
  }
}
