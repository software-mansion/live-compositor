import React from 'react';
import * as Api from './api';
type ComponentProps<P> = {
    children?: React.ReactNode;
} & P;
export type SceneComponent = Api.Component | string;
export type SceneBuilder<P> = (props: P, children: SceneComponent[]) => Api.Component;
declare abstract class LiveCompositorComponent<P> extends React.Component<ComponentProps<P>> {
    abstract builder: SceneBuilder<P>;
    render(): React.ReactNode;
}
export declare function sceneComponentIntoApi(component: SceneComponent): Api.Component;
export default LiveCompositorComponent;
