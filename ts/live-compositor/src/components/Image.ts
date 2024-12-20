import { createElement, useContext, useEffect, useState } from 'react';
import type { SceneComponent } from '../component.js';
import { createCompositorComponent } from '../component.js';
import type { Api } from '../index.js';
import { imageRefIntoRawId } from '../types/refs/imageRef.js';
import { newInternalImageId } from '../context/internalImageIdManager.js';
import { newBlockingTask } from '../hooks.js';
import { LiveCompositorContext } from '../context/index.js';
import { isValidImageType } from '../types/utils.js';

export type ImageProps = {
  children?: undefined;

  /**
   * Id of a component.
   */
  id?: Api.ComponentId;
  /**
   * Id of an image. It identifies an image registered using `LiveCompositor.registerImage`.
   */
  imageId: Api.RendererId;
  /**
   *  Url or path to the image file. File path refers to the filesystem where LiveCompositor server is deployed.
   */
  source: string;
};

export const InnerImage = createCompositorComponent<ImageProps>(sceneBuilder);

function Image(props: ImageProps) {
  const ctx = useContext(LiveCompositorContext);
  const [imageId, setImageId] = useState(0);

  useEffect(() => {
    const newImageId = newInternalImageId();
    setImageId(newImageId);
    const task = newBlockingTask(ctx);
    const pathOrUrl =
      props.source.startsWith('http://') || props.source.startsWith('https://')
        ? { url: props.source }
        : { path: props.source };
    const extension = props.source.split('.').pop();
    const assetType = extension && isValidImageType(extension) ? extension : undefined;

    let registerPromise: Promise<any>;

    void (async () => {
      try {
        if (!assetType) throw new Error('Unsupported image type');

        registerPromise = ctx.registerImage(newImageId, {
          ...pathOrUrl,
          assetType,
        });
        await registerPromise;
      } finally {
        task.done();
      }
    })();

    return () => {
      task.done();
      void (async () => {
        await registerPromise.catch(() => {});
        await ctx.unregisterImage(newImageId);
      })();
    };
  }, [props.source]);

  if (props.source)
    return createElement(InnerImage, {
      ...props,
      imageId: imageRefIntoRawId({ type: 'output-local', id: imageId, outputId: ctx.outputId }),
    });

  return createElement(InnerImage, {
    ...props,
    imageId: imageRefIntoRawId({ type: 'global', id: props.imageId }),
  });
}

function sceneBuilder(props: ImageProps, _children: SceneComponent[]): Api.Component {
  return {
    type: 'image',
    id: props.id,
    image_id: props.imageId,
  };
}

export default Image;
