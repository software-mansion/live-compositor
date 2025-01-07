import { createElement, useContext, useEffect, useState } from 'react';
import type { ComponentBaseProps, SceneComponent } from '../component.js';
import { createCompositorComponent } from '../component.js';
import { View, type Api } from '../index.js';
import { imageRefIntoRawId } from '../types/refs/imageRef.js';
import { newInternalImageId } from '../context/internalImageIdManager.js';
import { newBlockingTask } from '../hooks.js';
import { LiveCompositorContext } from '../context/index.js';
import { isValidImageType } from '../types/utils.js';

export type ImageProps = Omit<ComponentBaseProps, 'children'> &
  (
    | {
        imageId: Api.RendererId;
        source?: never; // Ensuring 'source' cannot be used alongside 'imageId'
      }
    | {
        source: string;
        imageId?: never; // Ensuring 'imageId' cannot be used alongside 'source'
      }
  );

export const InnerImage = createCompositorComponent<ImageSceneBuliderProps>(sceneBuilder);

function Image(props: ImageProps) {
  const ctx = useContext(LiveCompositorContext);
  const [imageId, setImageId] = useState(0);
  const [isImageRegistered, setIsImageRegistered] = useState(false);

  useEffect(() => {
    const newImageId = newInternalImageId();
    setImageId(newImageId);
    const task = newBlockingTask(ctx);
    const pathOrUrl =
      props.source?.startsWith('http://') || props.source?.startsWith('https://')
        ? { url: props.source }
        : { path: props.source };
    const extension = props.source?.split('.').pop();
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
        setIsImageRegistered(true);
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

  if (!isImageRegistered) return createElement(View, {});
  else if (props.source)
    return createElement(InnerImage, {
      ...props,
      imageId: imageRefIntoRawId({ type: 'image-local', id: imageId, outputId: ctx.outputId }),
    });
  else if (props.imageId)
    return createElement(InnerImage, {
      ...props,
      imageId: imageRefIntoRawId({ type: 'global', id: props.imageId }),
    });

  return createElement(View, {});
}

type ImageSceneBuliderProps = Omit<ImageProps, 'imageId'> & { imageId: string };

function sceneBuilder(props: ImageSceneBuliderProps, _children: SceneComponent[]): Api.Component {
  return {
    type: 'image',
    id: props.id,
    image_id: props.imageId,
  };
}

export default Image;
